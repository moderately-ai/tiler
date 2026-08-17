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
    use tiler_compiler::physical_provider::InstalledPhysicalProviders;
    use tiler_compiler::session::{
        BudgetRefusal, BudgetResource, CompileFailureClass, NumericalContract,
    };
    use tiler_ir::semantic::ProviderIdentity;

    use crate::profile::declared_workgroup_profile;
    use crate::program::five_op_program;
    use crate::providers::{Answer, as_dyn, flock, shared_tally};

    use super::compile_request_diagnostic;

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

    #[test]
    fn provider_attribution_is_test_only_and_absent_from_measurement_paths() {
        let providers = include_str!("providers.rs");
        for anchor in [
            "use std::collections::BTreeMap;",
            "provider_tallies: BTreeMap<ProviderIdentity, ProviderTally>,",
            "pub struct ProviderTally {",
            "impl ProviderTally {",
            "fn tally_identity(&self) -> ProviderIdentity {",
            "fn record_provider_invocation(&self, tally: &mut Tally, has_baseline: bool) {",
            "fn record_provider_proposal(&self, tally: &mut Tally) {",
            "fn record_provider_decline(&self, tally: &mut Tally) {",
            "self.record_provider_invocation(&mut tally, baseline.is_some());",
            "self.record_provider_proposal(&mut tally);",
            "self.record_provider_decline(&mut tally);",
        ] {
            assert_directly_cfg_test(providers, anchor);
        }

        let boundary = include_str!("boundary.rs");
        let (production_boundary, _) = boundary
            .split_once("#[cfg(test)]\nmod tests")
            .expect("the boundary test module remains cfg(test)");
        for anchor in ["provider_tallies", "ProviderTally", "tally_identity"] {
            assert!(
                !production_boundary.contains(anchor),
                "production boundary source contains test attribution anchor {anchor}",
            );
        }

        for (name, source) in [
            ("main.rs", include_str!("main.rs")),
            ("measure.rs", include_str!("measure.rs")),
        ] {
            for forbidden in [
                "provider_tallies",
                "ProviderTally",
                "tally_identity",
                "record_provider_",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{name} reached test-only attribution anchor {forbidden}",
                );
            }
        }
    }

    fn assert_directly_cfg_test(source: &str, anchor: &str) {
        let offsets = source
            .match_indices(anchor)
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert!(
            !offsets.is_empty(),
            "attribution anchor disappeared: {anchor}"
        );
        for offset in offsets {
            let line_start = source[..offset].rfind('\n').unwrap_or_default();
            let preceding = source[..line_start]
                .lines()
                .next_back()
                .map(str::trim)
                .unwrap_or_default();
            assert_eq!(
                preceding, "#[cfg(test)]",
                "attribution anchor is not directly cfg(test): {anchor}",
            );
        }
    }

    /// The accepted public refusal is reached by the complete seven-specialist
    /// population, not by constructing its expected class in a fixture.
    ///
    /// The provider counters prove every installed specialist saw every one of
    /// the seventeen region subjects before explain construction refused. The
    /// public compile result remains request-wide: no target success or plan
    /// alternative escapes beside the failure.
    #[test]
    fn seven_specialists_reach_the_public_explain_byte_refusal() {
        let program = five_op_program(4, 3);
        let tally = shared_tally();
        let providers = flock(
            "request-boundary",
            7,
            Answer::Specialize { threads: 32 },
            &tally,
        );
        let environment = InstalledPhysicalProviders::installed(as_dyn(&providers))
            .expect("all seven independently named specialists install");
        let profile = declared_workgroup_profile("test.request-boundary-7.v1", 64);

        let diagnostic = compile_request_diagnostic(
            &program,
            [NumericalContract::STRICT_F32],
            [profile],
            &environment,
        );
        let tally = tally.borrow();
        assert_eq!(
            providers.len(),
            7,
            "the exercised specialist population moved"
        );
        assert_eq!(
            (tally.invocations, tally.proposals, tally.declines),
            (119, 21, 98),
            "the complete seven-by-seventeen provider population moved",
        );
        assert_eq!(
            (tally.baseline_subjects, tally.coverless_or_unspellable),
            (21, 98),
            "the seven providers did not each reach three baselines and fourteen declines",
        );
        assert_eq!(
            tally.raw_outcomes(),
            119,
            "every provider invocation must emit one proposal or decline",
        );
        let expected_identities = (0..7)
            .map(|index| {
                ProviderIdentity::new("acme", format!("request-boundary-{index}"), 1)
                    .expect("the expected specialist identity is valid")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            tally.provider_tallies().keys().cloned().collect::<Vec<_>>(),
            expected_identities,
            "the exact seven attributed specialist identities moved",
        );
        for identity in &expected_identities {
            let provider = tally
                .provider_tallies()
                .get(identity)
                .expect("every expected specialist has an attributed tally");
            assert_eq!(
                (
                    provider.invocations,
                    provider.proposals,
                    provider.declines,
                    provider.baseline_subjects,
                    provider.coverless_or_unspellable,
                    provider.raw_outcomes(),
                ),
                (17, 3, 14, 3, 14, 17),
                "specialist {identity} did not reach every one of its seventeen opportunities",
            );
        }
        let attributed = tally.provider_tallies().values().fold(
            (0_u64, 0_u64, 0_u64, 0_u64, 0_u64),
            |(invocations, proposals, declines, baselines, coverless), provider| {
                (
                    invocations.saturating_add(provider.invocations),
                    proposals.saturating_add(provider.proposals),
                    declines.saturating_add(provider.declines),
                    baselines.saturating_add(provider.baseline_subjects),
                    coverless.saturating_add(provider.coverless_or_unspellable),
                )
            },
        );
        assert_eq!(
            attributed,
            (
                tally.invocations,
                tally.proposals,
                tally.declines,
                tally.baseline_subjects,
                tally.coverless_or_unspellable,
            ),
            "the attributed per-provider census did not sum to the aggregate tally",
        );
        assert_eq!(
            (diagnostic.successes, diagnostic.alternatives),
            (0, 0),
            "a request-wide capacity refusal returned partial target or plan output",
        );
        assert_eq!(
            diagnostic.failure,
            Some(CompileFailureClass::BudgetExhausted {
                resource: BudgetResource::ExplainDetailCanonicalBytes,
                limit: 1_048_576,
                reported: 1_048_698,
            }),
            "the seven-specialist public payload moved away from its exact attempted prefix",
        );
        assert_eq!(
            BudgetResource::ExplainDetailCanonicalBytes.refusal(),
            BudgetRefusal::ConstructionLowerBound,
            "the public byte resource lost its attempted-prefix provenance",
        );
        assert_eq!(diagnostic.explain_record_lines, 2_258);
        assert_eq!(diagnostic.explain_bytes, 643_313);
        assert_eq!(
            diagnostic.failure_explain_last_line.as_deref(),
            Some(
                "2257 target-feasibility compiler-failure rule=compile.failure@1 provider=compiler:tiler.compiler@1 subject=region:program-alternative:f10d1b8bfd323115/region:0 event=compiler-failure:explain-detail-capacity causes=2256",
            ),
            "the retained terminal compiler-failure record changed",
        );
    }
}
