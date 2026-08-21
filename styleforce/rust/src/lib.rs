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

#[derive(Serialize)]
struct PySampleResult {
    name: String,
    passed: bool,
    state: String,
    message: Option<String>,
    expected_output: Option<String>,
    actual_output: Option<String>,
}

#[derive(Serialize)]
struct PyPatternResult {
    name: String,
    passed: bool,
    samples: Vec<PySampleResult>,
}

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

#[pyfunction]
fn test_patterns(py: Python, cwd: &str) -> PyResult<PyObject> {
    let result = py.allow_threads(|| run_test_patterns(cwd));
    let test_result = result.map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("{e:#}"))
    })?;
    let json = serde_json::to_string(&test_result)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
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

        let grit_dir = find_grit_dir_from(cwd_path.clone())
            .await
            .context("no .grit directory found")?;

        let grit_parent = grit_dir
            .parent()
            .context(".grit directory has no parent")?
            .to_string_lossy()
            .to_string();
        let repo = ModuleRepo::from_dir(&grit_dir).await;
        let (patterns, errored) =
            resolve_patterns(&repo, &grit_parent, None).await?;
        if !errored.is_empty() {
            for (name, msg) in &errored {
                log::warn!("pattern {name} did not resolve cleanly: {msg}");
            }
        }

        let testable = collect_testable_patterns(patterns);
        if testable.is_empty() {
            anyhow::bail!(
                "No testable patterns found. To test a pattern, make sure \
                 it is defined in .grit/grit.yaml or a .md file in your \
                 .grit/patterns directory."
            );
        }

        let libs = find_and_resolve_grit_dir(Some(cwd_path), None).await?;

        let runtime = ExecutionContext::default();

        let mut pattern_results: Vec<PyPatternResult> = Vec::new();
        let mut total_samples = 0usize;
        let mut failed_samples = 0usize;

        for pattern in &testable {
            let pattern_name = pattern
                .local_name
                .clone()
                .unwrap_or_else(|| pattern.body.clone());

            let lang = PatternLanguage::get_language(&pattern.body);
            let chosen_lang = lang.unwrap_or_default();

            if let PatternLanguage::Universal = chosen_lang {
                continue;
            }

            let pattern_libs = libs.get_language_directory_or_default(lang)?;

            let compiled = src_to_problem_libs(
                pattern.body.clone(),
                &pattern_libs,
                chosen_lang.try_into()?,
                pattern.local_name.clone(),
                None,
                None,
                None,
            );

            let compiled = match compiled {
                Ok(c) => c,
                Err(e) => {
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

            let mut sample_results: Vec<PySampleResult> = Vec::new();
            if let Some(samples) = &pattern.config.samples {
                for sample in samples {
                    let result = test_pattern_sample(&problem, sample, runtime.clone());

                    // On output mismatch, re-run the language's formatter
                    // (ruff, gofmt, …) on both sides before re-checking —
                    // mirrors `grit patterns test`.
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

/// Re-runs the language formatter on the expected and actual outputs, then
/// re-checks. Must be `async` and awaited from the outer runtime — calling
/// `block_on` here panics on the already-running tokio current-thread runtime.
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

    Some(match mismatch {
        None => marzano_gritmodule::testing::SampleTestResult::new_passing(
            result.matches.clone(),
            true,
        ),
        Some(mismatch_info) => {
            use marzano_gritmodule::testing::{MismatchInfo, OutputInfo};
            let (expected_out, actual_out) = match mismatch_info {
                MismatchInfo::Path(OutputInfo { expected, actual })
                | MismatchInfo::Content(OutputInfo { expected, actual }) => (expected, actual),
            };
            marzano_gritmodule::testing::SampleTestResult {
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
    m.add("__version__", "0.0.3")?;
    Ok(())
}
