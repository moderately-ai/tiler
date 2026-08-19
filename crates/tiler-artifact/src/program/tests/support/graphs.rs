//! Verified semantic graphs and the executable coverage packaged plans claim.

use tiler_ir::index::{
    DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexInteger,
    IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
    IndexRegionBuilder, NumericalContractIdentity, ScalarAttributes, ScalarOpKey,
    TensorRole as IndexTensorRole, VerifiedIndexRegion, add_bf16_scalar_op, add_f32_scalar_op,
    constant_bf16_scalar_op, constant_f32_scalar_op, multiply_bf16_scalar_op,
    multiply_f32_scalar_op,
};
use tiler_ir::program::CoveredOccurrence;
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, F32NumericalContractKey,
    MaterializationRounding, NumericalPermission, NumericalRealization, SubnormalMode,
};
use tiler_ir::semantic::{
    AttributeFieldId, BF16_CONSTANT_BITS_ATTRIBUTE, CanonicalField, CanonicalValue, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant, F32Multiply, InputKey, OperationId,
    OutputKey, SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum, add_bf16_op,
    add_f32_op, constant_bf16_op, constant_f32_op, multiply_bf16_op, multiply_f32_op,
    strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

pub(crate) const SCALE_BITS: u32 = 0x4000_0000; // 2.0f32
pub(crate) const OTHER_SCALE_BITS: u32 = 0x4040_0000; // 3.0f32
pub(crate) const BIAS_BITS: u32 = 0x3f80_0000; // 1.0f32
pub(crate) const CANONICAL_NAN: u32 = 0x7fc0_0000;
pub(crate) const ELEMENT_BYTES: u64 = 4;

pub(crate) fn strict() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_NAN,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

pub(crate) fn input_shape() -> Shape {
    Shape::from_dims([2, 3])
}

pub(crate) fn output_shape() -> Shape {
    Shape::from_dims([2])
}

pub(crate) fn build_graph(draft: SemanticProgramBuilder) -> SemanticProgram {
    build_graph_scaled(draft, 2.0)
}

/// Builds the fixture graph, parameterized by the pointwise scale constant.
///
/// The scale is the cheapest way to obtain a genuinely different semantic graph
/// that keeps the same named interface: an unreached extra input would be
/// compacted away at commit (ADR 0064) and would not change graph identity.
pub(crate) fn build_graph_scaled(
    mut draft: SemanticProgramBuilder,
    scale_value: f32,
) -> SemanticProgram {
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, scale_value.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.build().unwrap()
}

pub(crate) fn semantic_program() -> SemanticProgram {
    build_graph(SemanticProgramBuilder::try_standard().unwrap())
}

/// Obtains proof-derived coverage over the governed standard scalar authority.
///
/// This crate cannot mint a refinement receipt, and the point of the coverage
/// binding is that it cannot: a `CoveredOccurrence` exists only where a
/// completed receipt does. So the fixtures walk the same sealed IR path a
/// lowering consumer walks — derive each occurrence's subject, admit an
/// authority, build a *candidate* index region here, and submit the pair to the
/// verifier, which mints a receipt only when the candidate's canonical identity
/// equals the registered law's.
///
/// Building the candidate here rather than asking the law for its own answer is
/// forced and is also the point: a caller that could obtain the expected region
/// and hand it straight back would turn the verifier into a rubber stamp.
pub(crate) fn checked_coverage(semantic: &SemanticProgram) -> Vec<CoveredOccurrence> {
    checked_coverage_under(semantic, &strict_contract())
}

pub(crate) fn checked_coverage_under(
    semantic: &SemanticProgram,
    contract: &NumericalContractIdentity,
) -> Vec<CoveredOccurrence> {
    let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority freezes");
    checked_coverage_over(semantic, &scalars, contract)
}

/// The same walk over a scalar authority composed for this exact graph.
///
/// The provider-provenance fixtures build semantic registries the standard
/// scalar profile is not composed with — that profile is pinned to
/// [`tiler_ir::semantic::FrozenSemanticRegistry::standard`], and a refinement
/// verifier refuses a scalar authority frozen over another semantic authority.
/// Those fixtures therefore pair their registry with [`scalars_over`].
pub(crate) fn checked_coverage_over(
    semantic: &SemanticProgram,
    scalars: &FrozenScalarRegistry,
    contract: &NumericalContractIdentity,
) -> Vec<CoveredOccurrence> {
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(
        semantic.semantic_registry().clone(),
        scalars.clone(),
    )
    .expect("the fixture's scalar and semantic authorities cohere");
    let mut coverage: Vec<CoveredOccurrence> = semantic
        .operations()
        .map(|operation| checked_occurrence(semantic, scalars, &laws, operation.id(), contract))
        .collect();
    coverage.sort_unstable_by_key(CoveredOccurrence::occurrence);
    coverage
}

pub(crate) fn checked_occurrence(
    semantic: &SemanticProgram,
    scalars: &FrozenScalarRegistry,
    laws: &FrozenIndexRealizationLawRegistry,
    operation: OperationId,
    contract: &NumericalContractIdentity,
) -> CoveredOccurrence {
    let subject = IndexRefinementSubject::derive(semantic, operation, contract.clone())
        .expect("every fixture operation derives a refinement subject");
    let (emitted, region) = if subject.operation() == &constant_f32_op() {
        (
            vec![constant_f32_scalar_op()],
            constant_region(
                &subject,
                scalars,
                F32_CONSTANT_BITS_ATTRIBUTE,
                constant_f32_scalar_op(),
            ),
        )
    } else if subject.operation() == &multiply_f32_op() {
        (
            vec![multiply_f32_scalar_op()],
            pointwise_region(&subject, scalars, multiply_f32_scalar_op()),
        )
    } else if subject.operation() == &add_f32_op() {
        (
            vec![add_f32_scalar_op()],
            pointwise_region(&subject, scalars, add_f32_scalar_op()),
        )
    } else if subject.operation() == &strict_serial_sum_f32_op() {
        (
            vec![add_f32_scalar_op()],
            serial_sum_region(&subject, scalars),
        )
    } else if subject.operation() == &constant_bf16_op() {
        (
            vec![constant_bf16_scalar_op()],
            constant_region(
                &subject,
                scalars,
                BF16_CONSTANT_BITS_ATTRIBUTE,
                constant_bf16_scalar_op(),
            ),
        )
    } else if subject.operation() == &multiply_bf16_op() {
        (
            vec![multiply_bf16_scalar_op()],
            pointwise_region(&subject, scalars, multiply_bf16_scalar_op()),
        )
    } else if subject.operation() == &add_bf16_op() {
        (
            vec![add_bf16_scalar_op()],
            pointwise_region(&subject, scalars, add_bf16_scalar_op()),
        )
    } else {
        panic!(
            "the fixture has no candidate region for {}",
            subject.operation()
        )
    };
    let authority = IndexRealizationAuthority::admit(
        semantic.semantic_registry(),
        scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &emitted,
    )
    .expect("the fixture's emission ceiling is admissible");
    let resolution = laws
        .resolve(&subject)
        .expect("the registered law resolves for this subject");
    match resolution
        .verify(&authority, &region)
        .expect("the fixture's candidate region realizes its operation")
    {
        IndexRefinementVerificationOutcome::Verified(receipt) => {
            CoveredOccurrence::from_receipt(&receipt)
        }
        IndexRefinementVerificationOutcome::Pending(_) => {
            panic!("the fixture's static regions retain no residual index-domain obligation")
        }
    }
}

/// The governed strict F32 contract the fixture kernels realize.
pub(crate) fn strict_contract() -> NumericalContractIdentity {
    F32NumericalContractKey::new(
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
        MaterializationRounding::NearestTiesToEven,
    )
    .expect("the fixture contract vector is coherent")
    .into()
}

/// Builds a rank-zero constant region from one width's bits attribute and scalar.
///
/// Both are parameters rather than the `f32` pair spelled inline, because the two
/// registered constant families carry *different* attribute identities and
/// different scalar operations while sharing one law template. A helper that
/// hardcoded either would build a region the `bf16` law's own realization does
/// not equal, and the verifier would refuse it — which is the check working, but
/// at the cost of a fixture that cannot express the second width at all.
pub(crate) fn constant_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    attribute: AttributeFieldId,
    scalar: ScalarOpKey,
) -> VerifiedIndexRegion {
    let [result] = subject.results() else {
        panic!("a constant has one result")
    };
    let bits = subject
        .attributes()
        .get(attribute)
        .expect("a constant carries its bits attribute")
        .clone();
    let attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(attribute, bits)])
            .expect("the scalar attribute record composes"),
    )
    .expect("scalar attributes are a record");
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the constant's output tensor");
    let value = region
        .apply(scalar, attributes, &[])
        .expect("the constant scalar applies")
        .get(0)
        .expect("one constant result");
    let write = region.write(output, &[], &[]).expect("the constant write");
    region.output(write, value).expect("the output root");
    region.build().expect("a verified constant region")
}

pub(crate) fn pointwise_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    operation: ScalarOpKey,
) -> VerifiedIndexRegion {
    let [result] = subject.results() else {
        panic!("a binary pointwise operation has one result")
    };
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let dimensions = result
        .shape()
        .extents()
        .iter()
        .copied()
        .map(|extent| {
            region
                .dimension(DomainRole::Parallel, extent)
                .expect("a parallel dimension")
        })
        .collect::<Vec<_>>();
    let coordinates = dimensions
        .iter()
        .copied()
        .map(|dimension| {
            region
                .dimension_expr(dimension)
                .expect("a dimension coordinate")
        })
        .collect::<Vec<_>>();
    let tensors = subject
        .inputs()
        .iter()
        .map(|input| {
            region
                .tensor(
                    IndexTensorRole::Input,
                    input.value_type().clone(),
                    input.shape().clone(),
                )
                .expect("a pointwise input tensor")
        })
        .collect::<Vec<_>>();
    let operands = subject
        .operands()
        .iter()
        .map(|position| {
            let input = &subject.inputs()[*position];
            if input.shape() == result.shape() {
                region
                    .read(tensors[*position], &dimensions, &coordinates)
                    .expect("an elementwise read")
            } else {
                region
                    .read(tensors[*position], &[], &[])
                    .expect("a rank-zero broadcast read")
            }
        })
        .collect::<Vec<_>>();
    let value = region
        .apply(operation, ScalarAttributes::empty(), &operands)
        .expect("the pointwise scalar applies")
        .get(0)
        .expect("one pointwise result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the pointwise output tensor");
    let write = region
        .write(output, &dimensions, &coordinates)
        .expect("the pointwise write");
    region.output(write, value).expect("the output root");
    region.build().expect("a verified pointwise region")
}

pub(crate) fn serial_sum_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
) -> VerifiedIndexRegion {
    let ([input], [result]) = (subject.inputs(), subject.results()) else {
        panic!("a serial sum has one input and one result")
    };
    let [rows, columns] = input.shape().extents() else {
        panic!("the fixture reduces a rank-two input")
    };
    let (rows, columns) = (*rows, columns.get());
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let row = region
        .dimension(DomainRole::Parallel, rows)
        .expect("the row dimension");
    let row_coordinate = region.dimension_expr(row).expect("the row coordinate");
    let zero = region
        .constant(IndexInteger::from_u64(0))
        .expect("the seed column");
    let input_tensor = region
        .tensor(
            IndexTensorRole::Input,
            input.value_type().clone(),
            input.shape().clone(),
        )
        .expect("the reduction input tensor");
    let seed = region
        .read(input_tensor, &[row], &[row_coordinate, zero])
        .expect("the first contributor");
    let tail = region
        .dimension(DomainRole::Reduction, Extent::new(columns - 1))
        .expect("the tail dimension");
    let tail_coordinate = region.dimension_expr(tail).expect("the tail coordinate");
    let one = IndexInteger::from_u64(1);
    let contributor_column = region
        .linear_combination(one.clone(), &[(one, tail_coordinate)])
        .expect("the tail contributor coordinate");
    let contributor = region
        .read(
            input_tensor,
            &[row, tail],
            &[row_coordinate, contributor_column],
        )
        .expect("a tail contributor");
    let total = region
        .reduce(&[tail], &[seed], &[contributor], |body| {
            let state = body.state(0).expect("one reduction state");
            let value = body.contributor(0).expect("one contributor");
            let accumulated = body
                .apply(
                    add_f32_scalar_op(),
                    ScalarAttributes::empty(),
                    &[state, value],
                )?
                .get(0)
                .expect("one accumulated result");
            body.yield_values(&[accumulated])
        })
        .expect("the serial reduction")
        .get(0)
        .expect("one reduction result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the reduction output tensor");
    let write = region
        .write(output, &[row], &[row_coordinate])
        .expect("the reduction write");
    region.output(write, total).expect("the output root");
    region.build().expect("a verified serial-sum region")
}

pub(crate) fn coverage_range(
    coverage: &[CoveredOccurrence],
    range: std::ops::Range<u32>,
) -> Vec<CoveredOccurrence> {
    coverage
        .iter()
        .filter(|covered| range.contains(&covered.occurrence().get()))
        .cloned()
        .collect()
}
