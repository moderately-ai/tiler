// `variant_count` is what pins `perturb::fused_operations_are_unexpressible`'s
// admitted-operation list to `corpus::Operation`. That perturbation reports that
// no fused multiply-add variant is expressible, and every other site that has to
// know about an operation is an exhaustive `match` — `Operation::apply` and
// `Operation::as_str` — which `rustc` already closes. A hand-written list of the
// admitted set has no such check, so a fused variant added to the enum would
// leave the perturbation reporting an absence that had stopped being true.
// Declaring that list at `variant_count` makes the omission an array-length
// build error at the claim instead. The use is in the binary itself rather than
// behind `cfg(test)`, so the gate is unconditional.
#![feature(variant_count)]
//! A bounded BF16 proof across the second-dtype seams.
//!
//! Run from this directory; see `README.md`. The binary's only product is a
//! verdict: every stage that fails exits non-zero with the stage named, and
//! there is no partial success.

mod bf16;
mod corpus;
mod format;
mod perturb;
mod promotion;
mod routing;
mod seams;

use tiler_compiler::target::{DTypeDispatchability, DTypeDispatchabilityResolution};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut failed = Vec::new();
    report_seams(&mut failed);
    report_oracle(&mut failed);
    report_promotion(&mut failed);
    report_routing(&mut failed)?;
    report_perturbations(&mut failed)?;

    println!();
    if failed.is_empty() {
        println!("VERDICT: every stage agreed and every perturbation was detected.");
        Ok(())
    } else {
        Err(format!("{} stage(s) failed: {}", failed.len(), failed.join(", ")).into())
    }
}

/// Reports which accepted boundaries admit BF16 and which refuse it.
fn report_seams(failed: &mut Vec<&'static str>) {
    println!("== seam probes ==");
    for verdict in [
        seams::descriptor_seam(),
        seams::descriptor_agreement_seam(),
        seams::ulp_metric_seam(),
        seams::reference_element_seam(),
        seams::dispatchability_seam(DTypeDispatchability::Dispatchable),
        seams::dispatchability_seam(DTypeDispatchability::Unsupported),
        seams::scalar_arithmetic_seam(),
    ] {
        println!(
            "  [{}] {} -- {}",
            if verdict.admitted {
                "ADMITS "
            } else {
                "REFUSES"
            },
            verdict.subject,
            verdict.detail
        );
        // The descriptor-agreement check is the one seam probe whose negative
        // answer is a defect rather than a finding: the other five report what
        // the boundary does, this one reports whether the spike is measuring the
        // format Tiler actually registers.
        if verdict.subject.starts_with("spike constants") && !verdict.admitted {
            failed.push("descriptor agreement");
        }
    }
}

/// Reports the exact-rational oracle's checks and the population they cover.
fn report_oracle(failed: &mut Vec<&'static str>) {
    println!("\n== reference oracle ==");
    stage(
        failed,
        "exhaustive round trip over all 65,536 encodings",
        &corpus::check_exhaustive_round_trip(),
    );
    stage(
        failed,
        "widening agreement over all 65,536 encodings",
        &corpus::check_widening_agrees(),
    );
    stage(
        failed,
        "named conformance witnesses",
        &corpus::check_witnesses(),
    );
    stage(
        failed,
        "overflow boundary, both sides",
        &corpus::check_overflow_boundary(),
    );
    println!(
        "  witness categories: {}",
        corpus::witness_categories()
            .iter()
            .map(|(name, cases)| format!("{} {name}", cases.len()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let census = corpus::census();
    println!(
        "  population: {}",
        census
            .iter()
            .map(|(name, count)| format!("{count} {name}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Reports the computation, accumulator, and conversion stages.
///
/// Ordered so the generic rounder is validated against the trusted BF16 one
/// before any stage that reads its answer, and so the two negative results —
/// the fused route and the accumulator width — follow the positive one they are
/// the exception to.
fn report_promotion(failed: &mut Vec<&'static str>) {
    println!("\n== computation, accumulator, and conversion ==");
    for stage in promotion::stages() {
        println!(
            "  [{}] {}\n         {}",
            if stage.passed { "OK  " } else { "FAIL" },
            stage.name,
            stage.detail
        );
        if !stage.passed {
            failed.push(stage.name);
        }
    }
}

/// Reports the three-family routing matrix and asserts its shape.
fn report_routing(failed: &mut Vec<&'static str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n== target routing matrix ==");
    let matrix = routing::routing_matrix()?;
    for route in &matrix {
        println!(
            "  {:<24} {:<5} -> {:?}",
            route.profile, route.dtype, route.resolution
        );
    }
    // Assert the matrix's shape, not six independent facts: the population is
    // named and counted, so a run where every answer collapsed to one value is a
    // failure rather than a uniform pass.
    let bf16_answers: Vec<_> = matrix
        .iter()
        .filter(|route| route.dtype == "bf16")
        .map(|route| &route.resolution)
        .collect();
    let shape_holds = matrix.len() == 6
        && matches!(
            bf16_answers.as_slice(),
            [
                DTypeDispatchabilityResolution::Dispatchable,
                DTypeDispatchabilityResolution::Unsupported,
                DTypeDispatchabilityResolution::Unknown,
            ]
        );
    if shape_holds {
        println!("  the three BF16 answers are distinct: Dispatchable, Unsupported, Unknown");
        println!(
            "  the simulator's Unsupported row is the measured refusal: {}",
            routing::SIMULATOR_REFUSAL_DIAGNOSTIC
        );
    } else {
        println!("  FAIL the routing matrix did not produce three distinct BF16 answers");
        failed.push("target routing matrix");
    }
    Ok(())
}

/// Reports each deliberate perturbation and whether its check noticed.
fn report_perturbations(failed: &mut Vec<&'static str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n== perturbations, each watched failing ==");
    let perturbations = [
        perturb::tie_rule_is_load_bearing(),
        perturb::widening_shift_is_load_bearing(),
        perturb::descriptor_lookup_can_refuse(),
        perturb::invalid_operations_are_decided(),
        perturb::fused_operations_are_unexpressible(),
        perturb::unmeasured_dtype_does_not_inherit()?,
        perturb::simulator_refusal_is_dtype_specific()?,
        perturb::promoted_route_depends_on_binary32_precision(),
        perturb::the_fused_witness_is_about_double_rounding(),
        perturb::the_accumulator_witness_needs_contributors(),
    ];
    for perturbation in &perturbations {
        println!(
            "  [{}] {} -- {}",
            if perturbation.detected {
                "DETECTED"
            } else {
                "MISSED  "
            },
            perturbation.subject,
            perturbation.detail
        );
        if !perturbation.detected {
            failed.push("perturbation");
        }
    }
    Ok(())
}

fn stage(failed: &mut Vec<&'static str>, name: &'static str, failures: &[corpus::Failure]) {
    if failures.is_empty() {
        println!("  [OK]   {name}");
        return;
    }
    println!("  [FAIL] {name}");
    for failure in failures.iter().take(10) {
        println!(
            "         {}: expected {:#06x}, got {:#06x}",
            failure.subject, failure.expected, failure.actual
        );
    }
    println!("         {} failure(s)", failures.len());
    failed.push(name);
}
