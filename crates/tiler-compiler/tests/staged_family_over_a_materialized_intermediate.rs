//! Where a staged family reading a materialized intermediate stops, measured.
//!
//! `admit-a-staged-family-that-reads-a-materialized-intermediate` moved one
//! wall and not the one below it, and this file is what keeps the pair honest.
//! **The recognizer admits `rms_norm(matmul(a, b), w)`**: the recognized staged
//! shape carries a boundary role per operand and the producer whose regions
//! write the edge, so the occurrence and the contraction are one output's
//! partition. **The scheduled-region vocabulary still cannot spell it**, and the
//! reason is one layer further down than this crate: the consuming stage would
//! read the occurrence's operand edge *and* the value the producing stage handed
//! it, and `tiler_ir::schedule`'s `reads_bind_boundary_tensors_in_order` admits
//! at most one `TensorRole::Intermediate` read because that role carries no
//! ordinal. `physical::staged_plan` declines the occurrence rather than
//! proposing a region the verifier would reject as invalid compiler output, and
//! [`admit-a-scheduled-region-that-reads-two-materialization-edges`] owns the
//! widening.
//!
//! **A ceiling stated only in prose drifts in both directions**, which is why
//! the refusal is asserted with a control beside it: without a program that
//! compiles under the identical request, "the chain refuses" is consistent with
//! a broken session boundary and this file would prove nothing. The control is
//! the same normalization over two *declared* inputs, which compiles end to end
//! and is bit-compared elsewhere
//! (`pipeline::tests::a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`),
//! so the two programs differ by exactly where the first operand comes from.
//!
//! [`admit-a-scheduled-region-that-reads-two-materialization-edges`]: ../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32RmsNorm, F32TensorContraction, InputKey,
    OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Shape};

#[path = "support/staged_rms_profile.rs"]
mod staged_rms_profile;
use staged_rms_profile::{RmsRealizationFixture, staged_rms_profile};

/// The five named F32 contract points this boundary suite exercises.
///
/// Named together rather than sampled at one preset: the two strict-order
/// points isolate the structural vocabulary wall, while the three
/// reassociation-permitting points make some multi-occurrence covers
/// fusion-legality `Unknown`. That mixed cause is deliberately not absorbed
/// into the pure vocabulary class. This is not the complete population of
/// caller-composable numerical contracts.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// A normalization whose value operand is a materialized contraction result.
///
/// `ab,bc->ac` over `a` and `b`, with a distinct third declared tensor `w` as
/// the normalization weight. This is the ordinary staged chain whose
/// contraction reads a subset of the program interface.
fn staged_over_an_edge() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let left = builder
        .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
        .unwrap();
    let right = builder
        .input::<F32>(InputKey::new("b").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), shape)
        .unwrap();
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product =
        F32TensorContraction::apply(&mut builder, &structure, left, right).expect("the product");
    let normalized = F32RmsNorm::apply(
        &mut builder,
        product,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .expect("the normalization");
    builder
        .output(OutputKey::new("result").unwrap(), normalized)
        .unwrap();
    builder.build().unwrap()
}

/// The control: the same normalization over two declared inputs.
fn staged_over_declared_inputs() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("b").unwrap(), shape)
        .unwrap();
    let normalized = F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .expect("the normalization");
    builder
        .output(OutputKey::new("result").unwrap(), normalized)
        .unwrap();
    builder.build().unwrap()
}

/// Compiles one program under one contract against one exact caller profile.
fn compile_under(
    program: &SemanticProgram,
    contract: NumericalContract,
    profile: &TargetProfile,
) -> Result<(), CompileFailureClass> {
    let targets = TargetRequest::new([profile.clone()]).unwrap();
    match compile(CompileRequest::new(program, contract, targets)) {
        Ok(batch) => {
            let outcome = batch.targets().next().expect("one requested profile");
            outcome
                .outcome()
                .map(|_| ())
                .map_err(TargetCompileFailure::class)
        }
        Err(failure) => Err(failure.class()),
    }
}

/// The chain is past the recognizer and stops at the region vocabulary.
///
/// **The class is what carries the claim.** A recognizer refusal is
/// `UnsupportedCapability` naming a `strategy` rule — `staged-operand` is what
/// this exact program reported before the widening. Planning now reports the
/// same class under `region-vocabulary`, but only after recognition and an
/// exhaustive non-empty cover search prove at least one complete cover stopped
/// at a non-partial `UnspellableRegion` wall and every other cover adds only
/// `UnspellableRegion` search noise. A bare `is_err` cannot distinguish those
/// phases, which is why the exact rule is asserted rather than only the class.
///
/// The contraction-permitting contract is the independent mixed-cause control:
/// fusion-legality `Unknown` keeps it on `NoFeasiblePlan` even though the same
/// non-partial vocabulary wall is present.
#[test]
fn a_staged_family_over_an_edge_is_recognized_and_stops_at_the_region_vocabulary() {
    let program = staged_over_an_edge();
    let profile = staged_rms_profile(RmsRealizationFixture::Discharging);
    let mut strict_walls = 0;
    let mut mixed_walls = 0;
    for contract in CONTRACTS {
        let expected = if matches!(
            contract,
            NumericalContract::STRICT_F32 | NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32
        ) {
            strict_walls += 1;
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "region-vocabulary",
            })
        } else {
            mixed_walls += 1;
            Err(CompileFailureClass::NoFeasiblePlan)
        };
        assert_eq!(compile_under(&program, contract, &profile), expected);
    }
    assert_eq!((strict_walls, mixed_walls), (2, 3));
}

/// The control compiles under the same request, so the refusal is the region
/// vocabulary's and not the session boundary's.
#[test]
fn the_same_normalization_over_declared_inputs_compiles_under_the_same_request() {
    let control = staged_over_declared_inputs();
    let profile = staged_rms_profile(RmsRealizationFixture::Discharging);
    let mut compiled = 0;
    for contract in CONTRACTS {
        compile_under(&control, contract, &profile).unwrap_or_else(|failure| {
            panic!("{contract:?} refused the declared-input control as {failure:?}")
        });
        compiled += 1;
    }
    assert_eq!(compiled, CONTRACTS.len());
}

/// Request accuracy and the structural wall refuse independently.
#[test]
fn elementary_accuracy_is_assessed_before_the_region_vocabulary() {
    let control = staged_over_declared_inputs();
    for (fixture, rule) in [
        (
            RmsRealizationFixture::Absent,
            "accuracy.elementary.no-installed-realization",
        ),
        (
            RmsRealizationFixture::Unrefined,
            "accuracy.elementary.unrefined-realization",
        ),
    ] {
        assert_eq!(
            compile_under(
                &control,
                NumericalContract::STRICT_F32,
                &staged_rms_profile(fixture),
            ),
            Err(CompileFailureClass::UnsupportedCapability { rule }),
        );
    }
    assert_eq!(
        compile_under(
            &control,
            NumericalContract::STRICT_F32,
            &TargetProfile::governed(),
        ),
        Err(CompileFailureClass::UnsupportedCapability {
            rule: "accuracy.elementary.no-installed-realization",
        }),
        "the governed profile remains independently silent",
    );
}
