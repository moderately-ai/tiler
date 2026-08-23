//! The squaring-prologue subject of `super::reduction`'s own
//! production module, split into its own file purely to keep both
//! under the size bound the split enforces; both map to
//! `schedule::builder::reduction`.

use super::super::{
    ReductionTopology, RegionProgram, ScalarProgram, ScheduledRegionDiagnostic, TensorRole,
};
use super::support::{SPLIT, final_pass_builder, set_scalar, squared_partial_pass_builder};
use crate::schedule::model::ContributorOrder;
use crate::schedule::numerics::ArithmeticType;
use crate::shape::Axis;

/// The squaring-prologue reduction verifies, reading the original input.
#[test]
fn the_squaring_prologue_reduction_verifies_as_a_partial_pass() {
    let region = squared_partial_pass_builder(SPLIT)
        .build()
        .expect("a squaring-prologue partial pass verifies");
    assert!(matches!(
        region.region().index.program,
        RegionProgram::Numerical {
            scalar: ScalarProgram::SquaredSerialSum { .. },
            ..
        }
    ));
}

/// An accumulation narrower than the declared width is rejected here too.
///
/// **This is `tiler::rms-norm-f32@1`'s accumulator refusal, fired.** The
/// operation declares `tiler::f32@1` in its definition facts and criterion 3
/// of `implement-parallel-reduction-strategies` requires a narrower strategy
/// to be rejected with a typed reason. The check is the schedule verifier's
/// single accumulation authority rather than a second copy beside it, and
/// this exercises it on the program the normalization actually schedules —
/// so a change that admitted a narrower accumulator for the squaring
/// prologue alone would fail here even while the bare sum's own test passed.
#[test]
fn a_narrowed_accumulation_width_is_rejected_for_the_squaring_prologue() {
    for narrower in [ArithmeticType::F16, ArithmeticType::Bf16] {
        let mut builder = squared_partial_pass_builder(SPLIT);
        let ReductionTopology::MultiPass { accumulation, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *accumulation = narrower;
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::AccumulationWidth {
                declared: narrower,
                required: ArithmeticType::F32,
            }],
            "{narrower:?} is narrower than the width tiler::rms-norm-f32@1 declares"
        );
    }
    // The control: the same region at the declared width verifies, so the
    // refusals above are about the accumulator rather than about the
    // program.
    assert!(squared_partial_pass_builder(SPLIT).build().is_ok());
}

/// The squaring prologue may not be applied in the final pass.
///
/// Squaring a partial sum would square an already-folded value, so the
/// prologue belongs to the pass that reads the original inputs. The refusal
/// is what stops a split from applying it twice.
#[test]
fn the_squaring_prologue_may_not_carry_the_final_pass() {
    let mut builder = final_pass_builder(SPLIT);
    let axes = match &builder.program {
        Some(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum { axes, .. },
            ..
        }) => axes.clone(),
        other => panic!("expected the final pass's serial sum, not {other:?}"),
    };
    set_scalar(
        &mut builder,
        ScalarProgram::SquaredSerialSum {
            axes,
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        },
    );
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

/// The squaring prologue does not share identity with the bare serial sum.
///
/// The two regions differ in nothing but their scalar program — same access
/// relation, same contributor order, same numerical realization — so an
/// appended scalar-program tag that had collided with an existing one would
/// make these equal. It is the check behind "the schedule domain did not
/// step": the new tag separates, and every earlier tag keeps its meaning.
#[test]
fn the_squaring_prologue_reduction_has_its_own_canonical_identity() {
    let squared = squared_partial_pass_builder(SPLIT)
        .build()
        .expect("the squaring-prologue pass verifies");
    let mut bare = squared_partial_pass_builder(SPLIT);
    bare.accesses[0].tensor = TensorRole::Intermediate;
    bare.bounds_proofs[0].tensor = TensorRole::Intermediate;
    set_scalar(
        &mut bare,
        ScalarProgram::StrictSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        },
    );
    let bare = bare.build().expect("the bare pass verifies");
    assert_ne!(squared.canonical_identity(), bare.canonical_identity());
}
