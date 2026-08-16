//! Native Python bindings for GritQL pattern testing.
//!
//! Replaces the previous approach of downloading a prebuilt `grit` CLI binary
//! and shelling out to `grit patterns test`. Instead we compile the relevant
//! GritQL crates directly into a PyO3 extension module and expose a single
//! `test_patterns` function that mirrors what `grit patterns test` does.
//!
//! # TODO (verification needed before merge)
//!
//! 1. **API signatures**: The imports below were written from reading the
//!    v0.0.3 source via the GitHub API. Verify each function signature
//!    matches at compile time — especially `src_to_problem_libs` (return type
//!    is `Result<CompilationResult>`, we access `.problem`),
//!    `resolve_patterns` (returns `(Vec<ResolvedGritDefinition>, HashMap)`),
//!    and `ModuleRepo::from_dir` (returns `Self`, not `Option<Self>`).
//! 2. **`PatternLanguage::get_language`**: verify this method exists and takes
//!    `&str`. It may be `from_body` or `infer` instead.
//! 3. **`chosen_lang.try_into()`**: `PatternLanguage` → `TargetLanguage`
//!    conversion. Verify the `TryFrom` impl exists.
//! 4. **`format_rich_files` signature**: takes `&PatternLanguage` and
//!    `Vec<RichFile>`, returns `Result<Vec<RichFile>>`. Verify.
//! 5. **`SampleTestResult` fields**: `expected_output`, `actual_output`,
//!    `message`, `expected_outputs`, `actual_outputs`, `matches` — verify all
//!    are `pub` and the types match (especially `matches: Vec<MatchResult>`).
//! 6. **PyO3 `py.detach`**: verify the API in pyo3 0.23. It may be
//!    `py.allow_threads` instead.
//! 7. **JSON → Python dict**: the current approach uses a temp module to call
//!    `json.loads`. Consider using `pyo3`'s native `PyDict` construction or
//!    the `pyo3-json` pattern instead for robustness.

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
        // TODO: verify `resolve_patterns` signature. It returns
        // `(Vec<ResolvedGritDefinition>, HashMap<String, String>)` where the
        // second element is errored patterns (name → error message).
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

            // TODO: verify `PatternLanguage::get_language` exists and takes
            // `&str`. It may be `from_body`, `infer`, or `from_pattern`.
            let lang = PatternLanguage::get_language(&pattern.body);
            let chosen_lang = lang.unwrap_or_default();

            // Universal patterns have no language to test against.
            if let PatternLanguage::Universal = chosen_lang {
                continue;
            }

            // TODO: verify `get_language_directory_or_default` takes `Option<PatternLanguage>`
            // (since `lang` is `Option<PatternLanguage>`). It may need `Some(lang)`
            // or `chosen_lang` directly.
            let pattern_libs = libs.get_language_directory_or_default(lang)?;

            // TODO: verify `src_to_problem_libs` returns `Result<CompilationResult>`
            // and that `CompilationResult` has a `pub problem: Problem` field.
            // The last three args (file_ranges, custom_built_ins, injected_limit)
            // are all `None` — verify that's the correct default for testing.
            let compiled = src_to_problem_libs(
                pattern.body.clone(),
                &pattern_libs,
                // TODO: verify `PatternLanguage` → `TargetLanguage` via `TryInto`.
                // It may need `.into()` or an explicit `TryFrom` call.
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

                    // If the result failed on output mismatch and has
                    // multi-file outputs, try formatting with ruff/etc.
                    // TODO: verify `should_try_formatting()` checks for
                    // `FailedOutput` state AND that expected/actual_outputs
                    // are `Some`. Verify `format_rich_files` is async and
                    // takes `(&PatternLanguage, Vec<RichFile>)`.
                    let result = if result.should_try_formatting() {
                        try_format_and_recheck(&chosen_lang, &result).unwrap_or(result)
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
/// TODO: verify `format_rich_files` is `pub async fn` and takes
/// `(&PatternLanguage, Vec<RichFile>)`. The `expected_outputs` and
/// `actual_outputs` fields on `SampleTestResult` are `Option<Vec<RichFile>>`.
fn try_format_and_recheck(
    lang: &PatternLanguage,
    result: &marzano_gritmodule::testing::SampleTestResult,
) -> Option<marzano_gritmodule::testing::SampleTestResult> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;

    rt.block_on(async {
        let expected = result.expected_outputs.clone()?;
        let actual = result.actual_outputs.clone()?;

        let formatted_expected = format_rich_files(lang, expected).await.ok()?;
        let formatted_actual = format_rich_files(lang, actual).await.ok()?;

        let mut exp = formatted_expected;
        let mut act = formatted_actual;
        let mismatch = has_output_mismatch(&mut exp, &mut act);

        // TODO: verify `SampleTestResult::new_passing` takes
        // `(Vec<MatchResult>, bool)` where the bool is `required_format`.
        Some(match mismatch {
            None => marzano_gritmodule::testing::SampleTestResult::new_passing(
                result.matches.clone(),
                true,
            ),
            Some(mismatch_info) => {
                // TODO: verify `MismatchInfo` and `OutputInfo` are `pub` and
                // that the fields `expected` and `actual` on `OutputInfo` are
                // `pub String`.
                use marzano_gritmodule::testing::{MismatchInfo, OutputInfo};
                let (expected_out, actual_out) = match mismatch_info {
                    MismatchInfo::Path(OutputInfo { expected, actual })
                    | MismatchInfo::Content(OutputInfo { expected, actual }) => (expected, actual),
                };
                marzano_gritmodule::testing::SampleTestResult {
                    // TODO: verify all fields are `pub` and that `matches`
                    // is `Vec<MatchResult>`. The `expected_outputs` and
                    // `actual_outputs` are set to `None` here since we've
                    // already consumed them for formatting.
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
    })
}

#[pymodule]
fn _native(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(test_patterns, m)?)?;
    // Expose the version for diagnostics.
    m.add("__version__", "0.0.3")?;
    Ok(())
}
