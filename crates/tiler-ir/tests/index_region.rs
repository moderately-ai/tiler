//! Public canonical-index construction and adversarial verifier coverage.

use tiler_ir::index::{
    BoundsProofView, DomainRole, IndexBuildError, IndexInteger, IndexRegionBuilder,
    IndexRegionDiagnostic, SemanticRegionIdentity, TensorRole, WriteOwnershipProofView,
};
use tiler_ir::semantic::{F32, InputKey, OutputKey, SemanticProgramBuilder};
use tiler_ir::shape::{Extent, Shape};

fn region_identity(name: &str) -> SemanticRegionIdentity {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new(name).unwrap(), Shape::from_dims([]))
        .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), input)
        .unwrap();
    SemanticRegionIdentity::for_program(&builder.build().unwrap())
}

#[test]
fn pointwise_relation_exposes_only_verified_cross_references() {
    let mut builder = IndexRegionBuilder::new(region_identity("pointwise")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([4]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([4]),
        )
        .unwrap();
    let i = builder
        .dimension(DomainRole::Parallel, Extent::new(4))
        .unwrap();
    let i_expr = builder.iteration(i).unwrap();
    let read = builder.read(input, [i], [i_expr]).unwrap();
    let loaded = builder.load(read).unwrap();
    let scale = builder.f32_constant([], 2.0_f32.to_bits()).unwrap();
    let product = builder.f32_multiply(loaded, scale).unwrap();
    let bias = builder.f32_constant([], 1.0_f32.to_bits()).unwrap();
    let value = builder.f32_add(product, bias).unwrap();
    let write = builder.write(output, [i_expr]).unwrap();
    builder.output(write, value).unwrap();

    let region = builder.build().unwrap();
    assert_eq!(region.tensors().len(), 2);
    assert_eq!(region.outputs().len(), 1);
    let output = region.outputs().next().unwrap();
    assert!(
        region
            .accesses()
            .any(|access| access.id() == output.access())
    );
    assert!(
        region
            .scalar_expressions()
            .any(|scalar| scalar.id() == output.value())
    );
    let write = region
        .accesses()
        .find(|access| access.id() == output.access())
        .unwrap();
    assert_eq!(write.bounds_proof(), BoundsProofView::Interval);
    assert_eq!(
        write.write_ownership_proof(),
        Some(WriteOwnershipProofView::CoordinatePermutation)
    );
}

#[test]
fn linear_normalization_is_independent_of_construction_form_and_unused_nodes() {
    fn build(scale_form: bool) -> Vec<u8> {
        let mut builder = IndexRegionBuilder::new(region_identity("canonical")).unwrap();
        let input = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([7]),
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([7]),
            )
            .unwrap();
        let i = builder
            .dimension(DomainRole::Parallel, Extent::new(7))
            .unwrap();
        let i_expr = builder.iteration(i).unwrap();
        let two_i = if scale_form {
            builder.scale(IndexInteger::from_i128(2), i_expr).unwrap()
        } else {
            builder.add([i_expr, i_expr]).unwrap()
        };
        let _unused = builder.floor_div(two_i, 3).unwrap();
        let read = builder.read(input, [i], [i_expr]).unwrap();
        let value = builder.load(read).unwrap();
        let write = builder.write(output, [i_expr]).unwrap();
        builder.output(write, value).unwrap();
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }

    assert_eq!(build(false), build(true));
}

#[test]
fn ordered_tensor_binding_participates_in_identity() {
    fn build(use_first: bool) -> Vec<u8> {
        let mut builder = IndexRegionBuilder::new(region_identity("bindings")).unwrap();
        let first = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([3]),
            )
            .unwrap();
        let second = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([3]),
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([3]),
            )
            .unwrap();
        let i = builder
            .dimension(DomainRole::Parallel, Extent::new(3))
            .unwrap();
        let i_expr = builder.iteration(i).unwrap();
        let read = builder
            .read(if use_first { first } else { second }, [i], [i_expr])
            .unwrap();
        let value = builder.load(read).unwrap();
        let write = builder.write(output, [i_expr]).unwrap();
        builder.output(write, value).unwrap();
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }

    assert_ne!(build(true), build(false));
}

#[test]
fn reachable_insertion_order_does_not_change_identity() {
    fn build(reverse_construction: bool) -> Vec<u8> {
        let mut builder = IndexRegionBuilder::new(region_identity("insertion-order")).unwrap();
        let left_tensor = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([2]),
            )
            .unwrap();
        let right_tensor = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([2]),
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([2]),
            )
            .unwrap();
        let i = builder
            .dimension(DomainRole::Parallel, Extent::new(2))
            .unwrap();
        let i_expr = builder.iteration(i).unwrap();
        let (left, right) = if reverse_construction {
            let right_access = builder.read(right_tensor, [i], [i_expr]).unwrap();
            let right = builder.load(right_access).unwrap();
            let left_access = builder.read(left_tensor, [i], [i_expr]).unwrap();
            let left = builder.load(left_access).unwrap();
            (left, right)
        } else {
            let left_access = builder.read(left_tensor, [i], [i_expr]).unwrap();
            let left = builder.load(left_access).unwrap();
            let right_access = builder.read(right_tensor, [i], [i_expr]).unwrap();
            let right = builder.load(right_access).unwrap();
            (left, right)
        };
        let value = builder.f32_add(left, right).unwrap();
        let write = builder.write(output, [i_expr]).unwrap();
        builder.output(write, value).unwrap();
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }

    assert_eq!(build(false), build(true));
}

#[test]
fn access_domain_is_explicit_and_rejects_unbound_coordinates() {
    let mut builder = IndexRegionBuilder::new(region_identity("domain")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([2]),
        )
        .unwrap();
    let i = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let i_expr = builder.iteration(i).unwrap();

    assert_eq!(
        builder.read(input, [], [i_expr]).unwrap_err(),
        IndexBuildError::CoordinateOutsideAccessDomain
    );
}

#[test]
fn conservative_interval_overlap_uses_finite_proof_instead_of_false_rejection() {
    let mut builder = IndexRegionBuilder::new(region_identity("interval-overlap")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([5]),
        )
        .unwrap();
    let i = builder
        .dimension(DomainRole::Parallel, Extent::new(5))
        .unwrap();
    let i_expr = builder.iteration(i).unwrap();
    let modulo = builder.modulo(i_expr, 2).unwrap();
    let quotient = builder.floor_div(i_expr, 2).unwrap();
    let coordinate = builder.add([modulo, quotient]).unwrap();
    let read = builder.read(input, [i], [coordinate]).unwrap();
    let value = builder.load(read).unwrap();
    let write = builder.write(output, [i_expr]).unwrap();
    builder.output(write, value).unwrap();

    let region = builder.build().unwrap();
    assert!(
        region
            .accesses()
            .any(|access| { access.bounds_proof() == BoundsProofView::Exhaustive { points: 5 } })
    );
}

#[test]
fn every_declared_output_tensor_requires_exactly_one_write_root() {
    let mut builder = IndexRegionBuilder::new(region_identity("missing-output")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let written = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let _missing = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let read = builder.read(input, [], []).unwrap();
    let value = builder.load(read).unwrap();
    let write = builder.write(written, []).unwrap();
    builder.output(write, value).unwrap();

    assert!(
        builder
            .build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                matches!(
                    diagnostic,
                    IndexRegionDiagnostic::MissingOutputTensor { .. }
                )
            })
    );
}

#[test]
fn duplicate_write_roots_for_one_output_tensor_are_rejected() {
    let mut builder = IndexRegionBuilder::new(region_identity("duplicate-output")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let read = builder.read(input, [], []).unwrap();
    let value = builder.load(read).unwrap();
    let first = builder.write(output, []).unwrap();
    let second = builder.write(output, []).unwrap();
    builder.output(first, value).unwrap();
    builder.output(second, value).unwrap();

    assert!(
        builder
            .build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                matches!(
                    diagnostic,
                    IndexRegionDiagnostic::DuplicateOutputTensor { .. }
                )
            })
    );
}

#[test]
fn boundary_tensor_rank_is_governed_transactionally() {
    let mut builder = IndexRegionBuilder::new(region_identity("rank-budget")).unwrap();
    let oversized = Shape::from_dims(std::iter::repeat_n(1, 1_025));
    assert!(matches!(
        builder.tensor(TensorRole::Input, F32::resolved_type(), oversized),
        Err(IndexBuildError::StructuralLimit { .. })
    ));

    assert!(
        builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([]),
            )
            .is_ok()
    );
}

#[test]
fn empty_reduction_read_is_vacuous_but_parallel_write_is_still_proved() {
    let mut builder = IndexRegionBuilder::new(region_identity("empty-reduction")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([2, 0]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([2]),
        )
        .unwrap();
    let i = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let k = builder
        .dimension(DomainRole::Reduction, Extent::new(0))
        .unwrap();
    let i_expr = builder.iteration(i).unwrap();
    let k_expr = builder.iteration(k).unwrap();
    let read = builder.read(input, [i, k], [i_expr, k_expr]).unwrap();
    let loaded = builder.load(read).unwrap();
    let sum = builder.strict_serial_f32_sum([k], loaded).unwrap();
    let write = builder.write(output, [i_expr]).unwrap();
    builder.output(write, sum).unwrap();

    let region = builder.build().unwrap();
    let mut accesses = region.accesses();
    assert_eq!(
        accesses.next().unwrap().bounds_proof(),
        BoundsProofView::VacuousEmptyDomain
    );
    let write = accesses.next().unwrap();
    assert_eq!(write.bounds_proof(), BoundsProofView::Interval);
    assert!(write.write_ownership_witness().is_some());
}

#[test]
fn constant_contributor_can_carry_a_reduction_evaluation_scope() {
    let mut builder = IndexRegionBuilder::new(region_identity("constant-reduction")).unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let k = builder
        .dimension(DomainRole::Reduction, Extent::new(3))
        .unwrap();
    let value = builder.f32_constant([k], 1.0_f32.to_bits()).unwrap();
    let sum = builder.strict_serial_f32_sum([k], value).unwrap();
    let write = builder.write(output, []).unwrap();
    builder.output(write, sum).unwrap();

    assert!(builder.build().is_ok());
}

#[test]
fn unused_and_free_reduction_dimensions_fail_closed() {
    let mut unused = IndexRegionBuilder::new(region_identity("unused-reduction")).unwrap();
    let input = unused
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([2]),
        )
        .unwrap();
    let output = unused
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([2]),
        )
        .unwrap();
    let i = unused
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let _k = unused
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let i_expr = unused.iteration(i).unwrap();
    let read = unused.read(input, [i], [i_expr]).unwrap();
    let value = unused.load(read).unwrap();
    let write = unused.write(output, [i_expr]).unwrap();
    unused.output(write, value).unwrap();
    assert!(
        unused
            .build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                matches!(
                    diagnostic,
                    IndexRegionDiagnostic::UnusedDomainDimension { .. }
                )
            })
    );

    let mut free = IndexRegionBuilder::new(region_identity("free-reduction")).unwrap();
    let input = free
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([2, 2]),
        )
        .unwrap();
    let output = free
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([2]),
        )
        .unwrap();
    let i = free
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let k = free
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let i_expr = free.iteration(i).unwrap();
    let k_expr = free.iteration(k).unwrap();
    let read = free.read(input, [i, k], [i_expr, k_expr]).unwrap();
    let value = free.load(read).unwrap();
    let write = free.write(output, [i_expr]).unwrap();
    free.output(write, value).unwrap();
    let diagnostics = free.build().unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic,
            IndexRegionDiagnostic::FreeReductionDimension { .. }
        )
    }));
}

#[test]
fn non_permutation_write_uses_bounded_exhaustive_evidence() {
    let mut builder = IndexRegionBuilder::new(region_identity("ownership")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([4]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([4]),
        )
        .unwrap();
    let i = builder
        .dimension(DomainRole::Parallel, Extent::new(4))
        .unwrap();
    let i_expr = builder.iteration(i).unwrap();
    let negated = builder.negate(i_expr).unwrap();
    let three = builder.constant(IndexInteger::from_i128(3)).unwrap();
    let reversed = builder.add([three, negated]).unwrap();
    let read = builder.read(input, [i], [i_expr]).unwrap();
    let value = builder.load(read).unwrap();
    let write = builder.write(output, [reversed]).unwrap();
    builder.output(write, value).unwrap();

    let region = builder.build().unwrap();
    let write = region
        .accesses()
        .find(|access| access.write_ownership_witness().is_some())
        .unwrap();
    assert_eq!(
        write.write_ownership_proof(),
        Some(WriteOwnershipProofView::Exhaustive { points: 4 })
    );
}

#[test]
fn exhaustive_ownership_obeys_coordinate_cell_budget() {
    let extent = 1_048_577_u64;
    let mut builder = IndexRegionBuilder::new(region_identity("proof-budget")).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([extent]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([extent]),
        )
        .unwrap();
    let i = builder
        .dimension(DomainRole::Parallel, Extent::new(extent))
        .unwrap();
    let i_expr = builder.iteration(i).unwrap();
    let negated = builder.negate(i_expr).unwrap();
    let last = builder
        .constant(IndexInteger::from_u64(extent - 1))
        .unwrap();
    let reversed = builder.add([last, negated]).unwrap();
    let read = builder.read(input, [i], [i_expr]).unwrap();
    let value = builder.load(read).unwrap();
    let write = builder.write(output, [reversed]).unwrap();
    builder.output(write, value).unwrap();

    assert!(
        builder
            .build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                matches!(diagnostic, IndexRegionDiagnostic::ProofResourceLimit { .. })
            })
    );
}

#[test]
fn failed_foreign_handle_insertion_leaves_builder_usable() {
    let mut first = IndexRegionBuilder::new(region_identity("first")).unwrap();
    let foreign = first
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let mut builder = IndexRegionBuilder::new(region_identity("second")).unwrap();
    assert!(matches!(
        builder.iteration(foreign),
        Err(IndexBuildError::ForeignHandle { .. })
    ));

    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::from_dims([]),
        )
        .unwrap();
    let read = builder.read(input, [], []).unwrap();
    let value = builder.load(read).unwrap();
    let write = builder.write(output, []).unwrap();
    builder.output(write, value).unwrap();
    assert!(builder.build().is_ok());
}
