//! Explain-capacity diagnostics isolated from every timed compile path.

use tiler_compiler::physical_provider::InstalledPhysicalProviders;
use tiler_compiler::session::{
    CompileFailure, CompileFailureClass, CompileRequest, NumericalContract, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::SemanticProgram;

/// Observations used only by the explicit `request-boundary` subcommand.
#[derive(Debug)]
pub struct BoundaryObservation {
    /// Successful target slots before any refusal.
    pub successes: usize,
    /// Retained complete-plan alternatives.
    pub alternatives: usize,
    /// Rendered explain bytes retained on the successful or failed path.
    pub explain_bytes: usize,
    /// Rendered trace lines that begin with a record ordinal.
    pub explain_record_lines: usize,
    /// Public failure class, when planning refused.
    pub failure: Option<CompileFailureClass>,
    /// Last rendered failure-trace line, when present.
    pub failure_explain_last_line: Option<String>,
}

/// Compiles one request while collecting the explain-capacity boundary control.
///
/// Record, census, and RSS-child paths cannot reach this module: `main.rs`
/// calls it only from the `request-boundary` command arm.
pub fn compile_request_diagnostic(
    program: &SemanticProgram,
    contracts: impl IntoIterator<Item = NumericalContract>,
    profiles: impl IntoIterator<Item = TargetProfile>,
    providers: &InstalledPhysicalProviders<'_>,
) -> BoundaryObservation {
    let targets = TargetRequest::new(profiles).expect("the diagnostic target request is valid");
    let request = match CompileRequest::preferring(program, contracts, targets) {
        Ok(request) => request.with_physical_providers(providers.clone()),
        Err(failure) => return summarize_request_failure(&failure),
    };
    let batch = match compile(request) {
        Ok(batch) => batch,
        Err(failure) => return summarize_request_failure(&failure),
    };

    let mut observation = BoundaryObservation {
        successes: 0,
        alternatives: 0,
        explain_bytes: 0,
        explain_record_lines: 0,
        failure: None,
        failure_explain_last_line: None,
    };
    for slot in batch.into_targets() {
        let (_, outcome) = slot.into_parts();
        match outcome {
            Ok(compilation) => {
                let rendered = compilation.explain().render();
                observation.successes += 1;
                observation.alternatives += compilation.alternatives().len();
                observation.explain_bytes += rendered.len();
                observation.explain_record_lines += rendered_record_lines(&rendered);
            }
            Err(failure) => {
                observation.failure.get_or_insert(failure.class());
                if let Some(report) = failure.explain() {
                    let rendered = report.render();
                    observation.explain_bytes += rendered.len();
                    observation.explain_record_lines += rendered_record_lines(&rendered);
                    observation.failure_explain_last_line =
                        rendered.lines().last().map(ToOwned::to_owned);
                }
            }
        }
    }
    observation
}

fn summarize_request_failure(failure: &CompileFailure) -> BoundaryObservation {
    let rendered = failure.explain().map(|report| report.render());
    BoundaryObservation {
        successes: 0,
        alternatives: 0,
        explain_bytes: rendered.as_ref().map_or(0, String::len),
        explain_record_lines: rendered.as_deref().map_or(0, rendered_record_lines),
        failure: Some(failure.class()),
        failure_explain_last_line: rendered
            .as_deref()
            .and_then(|rendered| rendered.lines().last())
            .map(ToOwned::to_owned),
    }
}

fn rendered_record_lines(rendered: &str) -> usize {
    rendered
        .lines()
        .filter(|line| {
            line.split_once(' ')
                .is_some_and(|(ordinal, _)| ordinal.parse::<u32>().is_ok())
        })
        .count()
}

#[cfg(test)]
mod tests {
    #[test]
    fn diagnostic_observer_has_only_the_boundary_command_caller() {
        let main = include_str!("main.rs");
        assert_eq!(
            main.matches("compile_request_diagnostic(").count(),
            1,
            "the diagnostic observer must have exactly one call site"
        );
        let boundary = main
            .split_once("fn request_boundary(")
            .and_then(|(_, tail)| tail.split_once("\n#[derive(Debug)]"))
            .map(|(body, _)| body)
            .expect("the boundary function remains independently delimited");
        assert!(
            boundary.contains("compile_request_diagnostic("),
            "only request_boundary may call the diagnostic observer"
        );
        for forbidden in [
            "fn record(",
            "fn measure_named(",
            "fn measure_request_named(",
        ] {
            let body = main
                .split_once(forbidden)
                .map(|(_, tail)| tail)
                .expect("the timed entry remains present");
            assert!(
                !body
                    .lines()
                    .take_while(|line| !line.starts_with("fn "))
                    .any(|line| line.contains("compile_request_diagnostic(")),
                "{forbidden} reached the diagnostic observer"
            );
        }
        let measure = include_str!("measure.rs");
        assert!(
            !measure.contains("compile_request_diagnostic"),
            "timing and RSS child helpers must not import the diagnostic observer"
        );
    }
}
