//! Native Python bindings for GritQL pattern testing.
//!
//! Compiles the relevant GritQL crates directly into a PyO3 extension module
//! (`styleforce._native`) and exposes a single `test_patterns` function that
//! mirrors what `grit patterns test` does. The `.grit` pattern data ships
//! inside the `styleforce` Python package, so `test_patterns(cwd)` is pointed
//! at the installed package dir by default (see `styleforce/__init__.py`).

use std::path::PathBuf;

use anyhow::{Context as _, Result};
use marzano_core::pattern_compiler::src_to_problem_libs;
use marzano_gritmodule::resolver::{find_and_resolve_grit_dir, resolve_patterns};
use marzano_gritmodule::searcher::find_grit_dir_from;
use marzano_gritmodule::testing::{
    collect_testable_patterns, get_sample_name, has_output_mismatch, test_pattern_sample,
    GritTestResultState,
};
use marzano_gritmodule::fetcher::ModuleRepo;
use marzano_gritmodule::formatting::format_rich_files;
use marzano_language::target_language::PatternLanguage;
use marzano_util::runtime::ExecutionContext;
use pyo3::prelude::*;
use serde::Serialize;

/// A single sample's test outcome, serialized to a Python dict.
#[derive(Serialize)]
struct PySampleResult {
    name: String,
    passed: bool,
    state: String,
    message: Option<String>,
    expected_output: Option<String>,
    actual_output: Option<String>,
}

/// One pattern's aggregate result (name + per-sample results).
#[derive(Serialize)]
struct PyPatternResult {
    name: String,
    passed: bool,
    samples: Vec<PySampleResult>,
}

/// The top-level result returned to Python.
#[derive(Serialize)]
struct PyTestResult {
    passed: bool,
    total_patterns: usize,
    total_samples: usize,
    failed_samples: usize,
    patterns: Vec<PyPatternResult>,
    summary: String,
}

fn state_name(state: &GritTestResultState) -> &'static str {
    match state {
        GritTestResultState::Pass => "Pass",
        GritTestResultState::PassWithFormat => "PassWithFormat",
        GritTestResultState::FailedOutput => "FailedOutput",
        GritTestResultState::FailedMatch => "FailedMatch",
        GritTestResultState::FailedPattern => "FailedPattern",
    }
}

/// Discover `.grit` patterns from `cwd`, compile each, and run its Markdown
/// samples through the GritQL test runner.
///
/// This is the native equivalent of `grit patterns test --verbose`.
#[pyfunction]
fn test_patterns(py: Python, cwd: &str) -> PyResult<PyObject> {
    let result = py.allow_threads(|| run_test_patterns(cwd));
    let test_result = result.map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("{e:#}"))
    })?;
    let json = serde_json::to_string(&test_result)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    // Use Python's built-in json module to parse the serialized result.
    let json_mod = py.import_bound("json")?;
    let obj: PyObject = json_mod
        .getattr("loads")?
        .call1((json,))?
        .into();
    Ok(obj)
}

fn run_test_patterns(cwd: &str) -> Result<PyTestResult> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let cwd_path = PathBuf::from(cwd);

        // 1. Find the .grit directory by walking up from cwd.
        let grit_dir = find_grit_dir_from(cwd_path.clone())
            .await
            .context("no .grit directory found")?;

        // 2. Resolve local patterns (from .grit/patterns/*.md).
        let grit_parent = grit_dir
            .parent()
            .context(".grit directory has no parent")?
            .to_string_lossy()
            .to_string();
        let repo = ModuleRepo::from_dir(&grit_dir).await;
        // `resolve_patterns` returns `(Vec<ResolvedGritDefinition>,
        // HashMap<String, String>)` where the second element is errored
        // patterns (name → error message).
        let (patterns, errored) =
            resolve_patterns(&repo, &grit_parent, None).await?;
        if !errored.is_empty() {
            for (name, msg) in &errored {
                log::warn!("pattern {name} did not resolve cleanly: {msg}");
            }
        }

        // 3. Collect testable patterns (those with samples).
        let testable = collect_testable_patterns(patterns);
        if testable.is_empty() {
            anyhow::bail!(
                "No testable patterns found. To test a pattern, make sure \
                 it is defined in .grit/grit.yaml or a .md file in your \
                 .grit/patterns directory."
            );
        }

        // 4. Build the PatternsDirectory (libs for compilation).
        let libs = find_and_resolve_grit_dir(Some(cwd_path), None).await?;

        // 5. ExecutionContext::default() — no auth needed for local testing.
        let runtime = ExecutionContext::default();

        let mut pattern_results: Vec<PyPatternResult> = Vec::new();
        let mut total_samples = 0usize;
        let mut failed_samples = 0usize;

        for pattern in &testable {
            let pattern_name = pattern
                .local_name
                .clone()
                .unwrap_or_else(|| pattern.body.clone());

            // `PatternLanguage::get_language` infers the language from the
            // pattern body. Falls back to the default (Universal) when it
            // cannot be determined.
            let lang = PatternLanguage::get_language(&pattern.body);
            let chosen_lang = lang.unwrap_or_default();

            // Universal patterns have no language to test against.
            if let PatternLanguage::Universal = chosen_lang {
                continue;
            }

            // `get_language_directory_or_default` accepts the `Option<
            // PatternLanguage>` directly, falling back to the default
            // directory when `None`.
            let pattern_libs = libs.get_language_directory_or_default(lang)?;

            // `src_to_problem_libs` compiles the pattern body into a
            // `CompilationResult` exposing a `pub problem: Problem` field.
            // The last three args (file_ranges, custom_built_ins,
            // injected_limit) are all `None` for standalone pattern testing.
            let compiled = src_to_problem_libs(
                pattern.body.clone(),
                &pattern_libs,
                // `PatternLanguage` converts to `TargetLanguage` via `TryInto`.
                chosen_lang.try_into()?,
                pattern.local_name.clone(),
                None,
                None,
                None,
            );

            let compiled = match compiled {
                Ok(c) => c,
                Err(e) => {
                    // Compilation failure — record as a failed pattern.
                    let samples: Vec<PySampleResult> = pattern
                        .config
                        .samples
                        .as_ref()
                        .map(|s| {
                            s.iter()
                                .map(|sample| PySampleResult {
                                    name: get_sample_name(sample),
                                    passed: false,
                                    state: "FailedPattern".to_string(),
                                    message: Some(format!(
                                        "Failed to compile pattern: {e}"
                                    )),
                                    expected_output: sample.output.clone(),
                                    actual_output: None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let count = samples.len();
                    failed_samples += count;
                    total_samples += count;
                    pattern_results.push(PyPatternResult {
                        name: pattern_name,
                        passed: false,
                        samples,
                    });
                    continue;
                }
            };

            let problem = compiled.problem;

            // Run each sample.
            let mut sample_results: Vec<PySampleResult> = Vec::new();
            if let Some(samples) = &pattern.config.samples {
                for sample in samples {
                    let result = test_pattern_sample(&problem, sample, runtime.clone());

                    // When a sample fails on output mismatch with
                    // multi-file outputs, re-run the language's formatter
                    // (ruff for Python, gofmt for Go, etc.) on both sides
                    // before re-checking.
                    let result = if result.should_try_formatting() {
                        try_format_and_recheck(&chosen_lang, &result).await.unwrap_or(result)
                    } else {
                        result
                    };

                    let passed = result.is_pass();
                    if !passed {
                        failed_samples += 1;
                    }
                    total_samples += 1;

                    sample_results.push(PySampleResult {
                        name: get_sample_name(sample),
                        passed,
                        state: state_name(&result.state).to_string(),
                        message: result.message.clone(),
                        expected_output: result.expected_output.clone(),
                        actual_output: result.actual_output.clone(),
                    });
                }
            }

            let all_passed = sample_results.iter().all(|s| s.passed);
            pattern_results.push(PyPatternResult {
                name: pattern_name,
                passed: all_passed,
                samples: sample_results,
            });
        }

        let passed = failed_samples == 0;
        let summary = if passed {
            format!(
                "All {total_samples} samples across {} patterns passed.",
                pattern_results.len()
            )
        } else {
            format!(
                "{failed_samples} out of {total_samples} samples failed."
            )
        };

        Ok(PyTestResult {
            passed,
            total_patterns: pattern_results.len(),
            total_samples,
            failed_samples,
            patterns: pattern_results,
            summary,
        })
    })
}

/// When a sample fails with an output mismatch, the CLI tries running the
/// language's formatter (ruff for Python, gofmt for Go, etc.) on both the
/// expected and actual output, then re-checks. We replicate that here.
///
/// This is async because `format_rich_files` is `pub async fn` taking
/// `(&PatternLanguage, Vec<RichFile>)`, and we're already running inside
/// the tokio runtime from `run_test_patterns`. The `expected_outputs` and
/// `actual_outputs` fields on `SampleTestResult` are `Option<Vec<RichFile>>`.
async fn try_format_and_recheck(
    lang: &PatternLanguage,
    result: &marzano_gritmodule::testing::SampleTestResult,
) -> Option<marzano_gritmodule::testing::SampleTestResult> {
    let expected = result.expected_outputs.clone()?;
    let actual = result.actual_outputs.clone()?;

    let formatted_expected = format_rich_files(lang, expected).await.ok()?;
    let formatted_actual = format_rich_files(lang, actual).await.ok()?;

    let mut exp = formatted_expected;
    let mut act = formatted_actual;
    let mismatch = has_output_mismatch(&mut exp, &mut act);

    // `SampleTestResult::new_passing` takes `(Vec<MatchResult>, bool)` where
    // the bool is `required_format`.
    Some(match mismatch {
        None => marzano_gritmodule::testing::SampleTestResult::new_passing(
            result.matches.clone(),
            true,
        ),
        Some(mismatch_info) => {
            // `MismatchInfo` and `OutputInfo` are `pub`, and `OutputInfo`'s
            // `expected` and `actual` fields are `pub String`.
            use marzano_gritmodule::testing::{MismatchInfo, OutputInfo};
            let (expected_out, actual_out) = match mismatch_info {
                MismatchInfo::Path(OutputInfo { expected, actual })
                | MismatchInfo::Content(OutputInfo { expected, actual }) => (expected, actual),
            };
            marzano_gritmodule::testing::SampleTestResult {
                // All fields are `pub`; `matches` is `Vec<MatchResult>`.
                // `expected_outputs`/`actual_outputs` are `None` here since
                // we've already consumed them for formatting.
                matches: result.matches.clone(),
                state: GritTestResultState::FailedOutput,
                message: Some(
                    "Actual output doesn't match expected output, even after formatting"
                        .to_string(),
                ),
                expected_output: Some(expected_out),
                actual_output: Some(actual_out),
                expected_outputs: None,
                actual_outputs: None,
            }
        }
    })
}

#[pymodule]
fn _native(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(test_patterns, m)?)?;
    // Expose the version for diagnostics.
    m.add("__version__", "0.0.3")?;
    Ok(())
}
