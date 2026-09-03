use std::collections::BTreeMap;

use anyhow::Result;
use marzano_core::api::{FileMatchResult, MatchResult};
use marzano_core::pattern_compiler::src_to_problem_libs;
use marzano_language::target_language::PatternLanguage;
use marzano_util::rich_path::RichFile;
use marzano_util::runtime::ExecutionContext;
use pyo3::prelude::*;

/// Rewrite `source` by the GritQL `pattern` body, returning the new source or
/// `source` unchanged when nothing matches. The pattern's own `language` line
/// selects the grammar; `filename` only names the snippet for the engine.
#[pyfunction]
fn apply(py: Python, pattern: &str, source: &str, filename: &str) -> PyResult<String> {
    py.allow_threads(|| run_apply(pattern, source, filename))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:#}")))
}

fn run_apply(pattern: &str, source: &str, filename: &str) -> Result<String> {
    let language = PatternLanguage::get_language(pattern).unwrap_or_default();
    let problem = src_to_problem_libs(
        pattern.to_string(),
        &BTreeMap::new(),
        language.try_into()?,
        None,
        None,
        None,
        None,
    )?
    .problem;
    let files = vec![RichFile::new(filename.to_string(), source.to_string())];
    for result in problem.execute_files(files, &ExecutionContext::default()) {
        if let MatchResult::Rewrite(rewrite) = result {
            return Ok(rewrite.content()?.to_string());
        }
    }
    Ok(source.to_string())
}

#[pymodule]
fn _native(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(apply, m)?)?;
    m.add("__version__", "0.0.3")?;
    Ok(())
}
