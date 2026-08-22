#![feature(variant_count)]
//! Admission, refusal, proof, and identity tests for the index-layer gather.
//!
//! Every perturbation here breaks the *subject* — the tag, a field order, a
//! threshold, a shape, a coordinate — rather than the assertion, and each one
//! is driven separately so that a reddening perturbation names which property
//! is load-bearing.

use std::mem::variant_count;
use std::sync::Arc;

use tiler_ir::index::FrozenScalarRegistry;
use tiler_ir::index::{
    DomainRole, GatherAccessRule, GatherIndexBoundsProofKind, IndexBuildError,
    IndexDomainFactSource, IndexInteger, IndexRegionBuilder, TensorAccessView, TensorRole,
    VerifiedIndexRegion,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{F32, gather_index_resolved_type};
use tiler_ir::shape::{
    Axis, BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, InterfaceParameterKey,
    RootBinding, SemanticInputConstraint, Shape, ShapeEnv, ShapeEnvBuilder, ShapeSymbol,
    SourcedExtent, SymbolScope,
};

fn registry() -> FrozenScalarRegistry {
    FrozenScalarRegistry::standard().expect("the governed scalar profile composes")
}

/// A region whose one output stores the result of one gather.
///
/// `source_dims`, `index_dims`, and `axis` are the whole of what varies between
/// the cases below, so each test perturbs exactly one of them.
struct GatherCase {
    source: Vec<u64>,
    index: Vec<u64>,
    axis: u32,
}

impl GatherCase {
    fn new(source: &[u64], index: &[u64], axis: u32) -> Self {
        Self {
            source: source.to_vec(),
            index: index.to_vec(),
            axis,
        }
    }

    /// Builds and verifies the region, with optional symbolic source coordinate.
    fn build(&self, symbolic_coordinate: bool) -> Result<VerifiedIndexRegion, String> {
        let environment = symbolic_coordinate.then(scale_environment);
        let mut builder = match environment {
            Some(environment) => {
                IndexRegionBuilder::new_with_shape_environment(registry(), environment)
                    .map_err(|error| format!("{error:?}"))?
            }
            None => IndexRegionBuilder::new(registry()).map_err(|error| format!("{error:?}"))?,
        };
        let result = gather_result(&self.source, &self.index, self.axis);
        let dimensions: Vec<_> = result
            .iter()
            .map(|extent| {
                builder
                    .dimension(DomainRole::Parallel, Extent::new(*extent))
                    .expect("a parallel dimension is admitted")
            })
            .collect();
        let source = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type().clone(),
                Shape::try_new(self.source.iter().copied().map(Extent::new)).unwrap(),
            )
            .map_err(|error| format!("{error:?}"))?;
        let index = builder
            .tensor(
                TensorRole::Input,
                gather_index_resolved_type(),
                Shape::try_new(self.index.iter().copied().map(Extent::new)).unwrap(),
            )
            .map_err(|error| format!("{error:?}"))?;
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type().clone(),
                Shape::try_new(result.iter().copied().map(Extent::new)).unwrap(),
            )
            .map_err(|error| format!("{error:?}"))?;

        // The result axes are laid out [source before axis | index | source
        // after axis], so the source coordinates are the outer and inner runs
        // and the index coordinates are the middle run.
        let axis = self.axis as usize;
        let outer = axis;
        let index_span = self.index.len();
        // The source coordinates are the outer and inner runs, in that order;
        // the symbol goes on whichever of them comes first, because a gather on
        // axis 0 has no outer run at all and would otherwise carry no symbol.
        let source_positions: Vec<_> = (0..outer)
            .chain((outer + index_span)..result.len())
            .collect();
        let source_coordinates: Vec<_> = source_positions
            .iter()
            .enumerate()
            .map(|(ordinal, position)| {
                coordinate(
                    &mut builder,
                    dimensions[*position],
                    symbolic_coordinate && ordinal == 0,
                )
            })
            .collect();
        let index_coordinates: Vec<_> = (outer..outer + index_span)
            .map(|position| coordinate(&mut builder, dimensions[position], false))
            .collect();
        let value = builder
            .gather_read(
                source,
                index,
                &dimensions,
                &source_coordinates,
                &index_coordinates,
                Axis::new(self.axis),
            )
            .map_err(|error| format!("{error:?}"))?;
        let write_coordinates: Vec<_> = dimensions
            .iter()
            .map(|dimension| builder.dimension_expr(*dimension).unwrap())
            .collect();
        let write = builder
            .write(output, &dimensions, &write_coordinates)
            .map_err(|error| format!("{error:?}"))?;
        builder
            .output(write, value)
            .map_err(|error| format!("{error:?}"))?;
        builder.build().map_err(|error| format!("{error:?}"))
    }
}

/// One coordinate over `dimension`, optionally spelled `S * d` rather than `d`.
///
/// `S` is bound to 1 by the environment, so the two spellings select the same
/// element and differ *only* in whether the proof subject names a declared
/// symbol. That is what isolates fact provenance from proof kind.
fn coordinate(
    builder: &mut IndexRegionBuilder,
    dimension: tiler_ir::index::DimensionId,
    symbolic: bool,
) -> tiler_ir::index::IndexExprId {
    let base = builder.dimension_expr(dimension).unwrap();
    if !symbolic {
        return base;
    }
    let scale = ShapeSymbol::new(SymbolScope::new("region/0").unwrap(), "s").unwrap();
    builder
        .sourced_linear_combination(
            IndexInteger::from_i128(0).into(),
            &[(tiler_ir::index::SourcedIndexInteger::Symbol(scale), base)],
        )
        .expect("a symbolic coefficient the environment declares is admitted")
}

fn scale_environment() -> Arc<ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let scale = ShapeSymbol::new(SymbolScope::new("region/0").unwrap(), "s").unwrap();
    draft.declare(scale.clone()).unwrap();
    draft
        .bind(
            &scale,
            RootBinding::new(
                BindingSource::InterfaceParameter {
                    key: InterfaceParameterKey::new("s").unwrap(),
                },
                // The one phase an interface parameter admits that is also no
                // later than the extent-source ceiling: earlier is refused as
                // `PhaseTooEarly` for this source class, later as
                // `SourceTooLate` for an index extent.
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    // Constrained to the single value one, not merely declared and bound. Every
    // caller of this environment depends on it: the symbolic coordinate below
    // reads `S * d` as selecting the same element as `d`, and the three sourced
    // refusal controls are only *controls* if the environment they run against
    // genuinely determines its symbol. An unconstrained symbol carries the
    // default interval, `determined_extent` answers `None`, and a rule wrongly
    // consulting `determined()` instead of `as_static()` would refuse for that
    // reason and leave the controls green without ever discriminating.
    draft
        .require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(scale), 1, 1).unwrap(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
    Arc::new(draft.build().unwrap())
}

fn gather_result(source: &[u64], index: &[u64], axis: u32) -> Vec<u64> {
    let axis = axis as usize;
    let mut result = source[..axis].to_vec();
    result.extend_from_slice(index);
    result.extend_from_slice(&source[axis + 1..]);
    result
}

fn only_gather(region: &VerifiedIndexRegion) -> tiler_ir::index::GatherReadAccessRef<'_> {
    region
        .accesses()
        .find_map(|access| access.view().gather_read())
        .expect("the fixture authors exactly one gather")
}

// ---------------------------------------------------------------------------
// Admission and the closed proof precedence
// ---------------------------------------------------------------------------

/// An ordinary gather owes invocation validation, not a proof.
///
/// The negative control for both static arguments: `[4, 5]` gathered on axis 1
/// by `[3]` has neither a zero result extent nor a source extent reaching the
/// U32 universe, so a proof here would mean one of the two arguments concluded
/// from a premise it does not have.
#[test]
fn an_ordinary_gather_requires_invocation_validation() {
    let region = GatherCase::new(&[4, 5], &[3], 1).build(false).unwrap();
    let gather = only_gather(&region);
    let resolution = gather.bounds_resolution();
    assert!(
        resolution.statically_proved().is_none(),
        "no closed argument reaches an ordinary gather",
    );
    let requirement = resolution
        .invocation_validation_required()
        .expect("the obligation is total, so it must be one or the other");
    assert_eq!(requirement.axis(), Axis::new(1));
    assert_eq!(requirement.source_extent(), Extent::new(5));
    assert_eq!(*requirement.result_shape(), Shape::from_dims([4, 3]));
}

/// An empty **result** extent proves vacuity even though the index is inhabited.
///
/// This is the repaired rule. Inspecting the index shape alone would find `[3]`
/// inhabited and mint a requirement, which is the false narrowing the packet's
/// own audit records twice.
#[test]
fn an_empty_result_extent_proves_vacuity_though_the_index_is_inhabited() {
    let region = GatherCase::new(&[0, 5], &[3], 1).build(false).unwrap();
    let proof = only_gather(&region)
        .bounds_resolution()
        .statically_proved()
        .expect("an empty result domain discharges the obligation vacuously");
    assert_eq!(
        proof.kind(),
        GatherIndexBoundsProofKind::VacuousEmptyResultDomain
    );
    assert_eq!(proof.facts(), IndexDomainFactSource::Program);
    assert_eq!(*proof.index_shape(), Shape::from_dims([3]));
}

/// A source axis of at least `2^32` contains every exact U32 value.
#[test]
fn a_source_axis_reaching_the_u32_universe_is_proved() {
    let region = GatherCase::new(&[1 << 32, 4], &[], 0).build(false).unwrap();
    let proof = only_gather(&region)
        .bounds_resolution()
        .statically_proved()
        .expect("the gathered axis contains every U32 value");
    assert_eq!(
        proof.kind(),
        GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent
    );
}

/// One below the threshold is **not** proved.
///
/// Perturbs the subject rather than the assertion: `u32::MAX` is the largest
/// value the index can hold, and an axis of exactly that extent leaves the
/// single value `u32::MAX` out of range. A threshold written `> u32::MAX as
/// u64` or narrowed into U32 would admit this case.
#[test]
fn a_source_axis_one_below_the_u32_universe_is_not_proved() {
    let region = GatherCase::new(&[u64::from(u32::MAX), 4], &[], 0)
        .build(false)
        .unwrap();
    assert!(
        only_gather(&region)
            .bounds_resolution()
            .statically_proved()
            .is_none(),
        "an axis of 2^32 - 1 leaves u32::MAX itself out of range",
    );
}

/// The empty-result argument wins over the U32-universe argument.
#[test]
fn empty_result_precedence_beats_the_u32_universe_argument() {
    // Source `[2^32, 0]`: axis 0 reaches the universe *and* the result carries
    // the zero extent from axis 1, so both arguments hold at once.
    let region = GatherCase::new(&[1 << 32, 0], &[], 0).build(false).unwrap();
    let proof = only_gather(&region)
        .bounds_resolution()
        .statically_proved()
        .unwrap();
    assert_eq!(
        proof.kind(),
        GatherIndexBoundsProofKind::VacuousEmptyResultDomain,
        "a domain that visits no point places no obligation on any value, so \
         attributing the conclusion to the source axis would name the wrong premise",
    );
}

// ---------------------------------------------------------------------------
// Fact provenance is independent of the proof-kind short circuit
// ---------------------------------------------------------------------------

/// A declared symbol in a coordinate moves `facts()` without moving the kind.
///
/// Driven separately for each proof kind, because a fact source derived from
/// whichever short circuit concluded would move only for whichever case the
/// derivation happened to reach.
#[test]
fn a_symbolic_coordinate_moves_facts_but_never_the_proof_kind() {
    for (case, expected) in [
        (
            GatherCase::new(&[0, 5], &[3], 1),
            GatherIndexBoundsProofKind::VacuousEmptyResultDomain,
        ),
        (
            GatherCase::new(&[1 << 32, 4], &[], 0),
            GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent,
        ),
    ] {
        // The U32-universe control gathers axis 0 of a rank-2 source, so its one
        // source coordinate covers axis 1 and can carry the symbol.
        let literal = case.build(false).unwrap();
        let symbolic = case.build(true).unwrap();
        let literal = only_gather(&literal)
            .bounds_resolution()
            .statically_proved()
            .unwrap();
        let symbolic = only_gather(&symbolic)
            .bounds_resolution()
            .statically_proved()
            .unwrap();
        assert_eq!(literal.kind(), expected);
        assert_eq!(symbolic.kind(), expected, "the kind is unchanged");
        assert_eq!(literal.facts(), IndexDomainFactSource::Program);
        assert_eq!(
            symbolic.facts(),
            IndexDomainFactSource::ShapeEnvironment,
            "a declared symbol participated in the subject even though the \
             argument did not need it",
        );
        assert_ne!(
            literal.identity().as_bytes(),
            symbolic.identity().as_bytes(),
            "the fact source is written into the proof identity",
        );
    }
}

// ---------------------------------------------------------------------------
// Identity: the fresh tag, field order, and injectivity
// ---------------------------------------------------------------------------

/// Swapping the two operands changes the region bytes.
///
/// The two tensor ordinals occupy fixed, distinct positions in the framed
/// access, so a gather of A by B is not a gather of B by A. A shared slot, or
/// an encoder that sorted the two ordinals, would make these identical.
#[test]
fn the_source_and_index_bindings_occupy_distinct_identity_positions() {
    // `[4, 4]` gathered by `[4]` on axis 0 and on axis 1 differ only in which
    // source axis the loaded value addresses.
    let first = GatherCase::new(&[4, 4], &[4], 0).build(false).unwrap();
    let second = GatherCase::new(&[4, 4], &[4], 1).build(false).unwrap();
    assert_ne!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes(),
        "the axis frame is part of the access encoding",
    );
}

/// A gather access and a direct read of the same source do not intern together.
#[test]
fn a_gather_does_not_intern_against_a_direct_read_of_its_source() {
    let mut builder = IndexRegionBuilder::new(registry()).unwrap();
    let d0 = builder
        .dimension(DomainRole::Parallel, Extent::new(3))
        .unwrap();
    let c0 = builder.dimension_expr(d0).unwrap();
    let source = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type().clone(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let index = builder
        .tensor(
            TensorRole::Input,
            gather_index_resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let direct = builder.read(source, &[d0], &[c0]).unwrap();
    let gathered = builder
        .gather_read(source, index, &[d0], &[], &[c0], Axis::new(0))
        .unwrap();
    assert_ne!(
        direct, gathered,
        "a separately authored direct read is a distinct access and does not \
         satisfy, merge with, or share the gather's address read",
    );
}

/// Two identical `gather_read` calls intern to one access and one value.
#[test]
fn an_identical_gather_interns_atomically() {
    let mut builder = IndexRegionBuilder::new(registry()).unwrap();
    let d0 = builder
        .dimension(DomainRole::Parallel, Extent::new(3))
        .unwrap();
    let c0 = builder.dimension_expr(d0).unwrap();
    let source = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type().clone(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let index = builder
        .tensor(
            TensorRole::Input,
            gather_index_resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let first = builder
        .gather_read(source, index, &[d0], &[], &[c0], Axis::new(0))
        .unwrap();
    let second = builder
        .gather_read(source, index, &[d0], &[], &[c0], Axis::new(0))
        .unwrap();
    assert_eq!(first, second);
}

// ---------------------------------------------------------------------------
// The exact refusal precedence
// ---------------------------------------------------------------------------

/// The one thing wrong with an otherwise well-formed gather.
///
/// An enum rather than a record of flags because these are not independent
/// switches: each test authors a region that would be admitted but for exactly
/// one defect, so a case that could set two at once would stop being a
/// one-variable perturbation of the admitted region.
#[derive(Clone, Copy)]
enum Defect {
    /// The source carries the index's type instead of F32.
    SourceIsU32,
    /// The index carries the source's type instead of U32.
    IndexIsF32,
    /// The source boundary is authored with a sourced extent.
    SourcedSource,
    /// The index boundary is authored with a sourced extent.
    SourcedIndex,
    /// The result-domain dimension is authored with a sourced extent.
    SourcedDomain,
    /// One handle is passed for both operands.
    Aliased,
    /// An axis the source does not have.
    Axis(u32),
    /// The source boundary is an output rather than an input.
    SourceIsOutput,
    /// The index boundary is an output rather than an input.
    IndexIsOutput,
    /// The source has no axes at all, so no axis can be gathered.
    SourceIsScalar,
    /// One source coordinate too many for the source's rank.
    ExtraSourceCoordinate,
    /// One index coordinate too few for the index's rank.
    MissingIndexCoordinate,
}

/// Authors a gather carrying exactly `defect` and returns the refusal it hits.
///
/// The environment here *determines* every symbol, so the three sourced cases
/// are the controls that distinguish "authored literal" from "resolves to a
/// literal": a rule that consulted the environment would admit them.
fn refusal(defect: Defect) -> IndexBuildError {
    let mut builder =
        IndexRegionBuilder::new_with_shape_environment(registry(), scale_environment()).unwrap();
    let symbol = ShapeSymbol::new(SymbolScope::new("region/0").unwrap(), "s").unwrap();
    let d0 = if matches!(defect, Defect::SourcedDomain) {
        builder
            .symbolic_dimension(DomainRole::Parallel, symbol.clone())
            .unwrap()
    } else {
        builder
            .dimension(DomainRole::Parallel, Extent::new(3))
            .unwrap()
    };
    let c0 = builder.dimension_expr(d0).unwrap();
    let source = if matches!(defect, Defect::SourcedSource) {
        builder
            .sourced_tensor(
                TensorRole::Input,
                F32::resolved_type().clone(),
                vec![SourcedExtent::Symbol(symbol.clone())],
            )
            .unwrap()
    } else {
        builder
            .tensor(
                if matches!(defect, Defect::SourceIsOutput) {
                    TensorRole::Output
                } else {
                    TensorRole::Input
                },
                if matches!(defect, Defect::SourceIsU32) {
                    gather_index_resolved_type()
                } else {
                    F32::resolved_type().clone()
                },
                if matches!(defect, Defect::SourceIsScalar) {
                    Shape::try_new([]).unwrap()
                } else {
                    Shape::from_dims([3])
                },
            )
            .unwrap()
    };
    let index = match defect {
        Defect::Aliased => source,
        Defect::SourcedIndex => builder
            .sourced_tensor(
                TensorRole::Input,
                gather_index_resolved_type(),
                vec![SourcedExtent::Symbol(symbol)],
            )
            .unwrap(),
        _ => builder
            .tensor(
                if matches!(defect, Defect::IndexIsOutput) {
                    TensorRole::Output
                } else {
                    TensorRole::Input
                },
                if matches!(defect, Defect::IndexIsF32) {
                    F32::resolved_type().clone()
                } else {
                    gather_index_resolved_type()
                },
                Shape::from_dims([3]),
            )
            .unwrap(),
    };
    let axis = match defect {
        Defect::Axis(axis) => axis,
        _ => 0,
    };
    // A rank-one source owes no source coordinate and a rank-one index owes
    // exactly one, so the admitted runs are empty and `[c0]`; each arity defect
    // moves one run by one and leaves the other alone.
    let source_coordinates: &[_] = match defect {
        Defect::ExtraSourceCoordinate => &[c0],
        _ => &[],
    };
    let index_coordinates: &[_] = match defect {
        Defect::MissingIndexCoordinate => &[],
        _ => &[c0],
    };
    builder
        .gather_read(
            source,
            index,
            &[d0],
            source_coordinates,
            index_coordinates,
            Axis::new(axis),
        )
        .expect_err("this fixture is authored to be refused")
}

/// Neither operand may be an output boundary, and each is named on its own.
///
/// Driven per operand rather than once, because a single role check would report
/// whichever boundary it read first and could not tell a caller which of the two
/// it had mis-declared.
#[test]
fn a_gather_operand_outside_the_input_role_is_refused_by_name() {
    assert!(matches!(
        refusal(Defect::SourceIsOutput),
        IndexBuildError::GatherSourceNotInput { .. }
    ));
    assert!(matches!(
        refusal(Defect::IndexIsOutput),
        IndexBuildError::GatherIndexNotInput { .. }
    ));
}

/// A rank-zero source is refused as such, before any axis is considered.
///
/// The gathered axis is zero here, which is out of range for a rank of nothing —
/// so a boundary that checked the axis first would report `GatherAxisOutOfRange`
/// and describe a source that has no axes as having the wrong one.
#[test]
fn a_rank_zero_source_is_refused_before_the_axis_is_judged() {
    assert!(matches!(
        refusal(Defect::SourceIsScalar),
        IndexBuildError::GatherSourceRankZero { .. }
    ));
}

/// Each coordinate run's arity is judged against its **own** rank.
///
/// The two runs derive their arities from different ranks — the source owes one
/// coordinate per source axis *except* the gathered one, the index owes one per
/// index axis — so a boundary that compared both against a single rank would
/// admit one of these two and refuse the other for the wrong reason. The
/// expected and actual counts are asserted, not merely the variant, because the
/// off-by-one direction is what a caller acts on.
#[test]
fn each_coordinate_run_is_refused_against_its_own_rank() {
    assert!(matches!(
        refusal(Defect::ExtraSourceCoordinate),
        IndexBuildError::GatherSourceCoordinateRank {
            expected: 0,
            actual: 1,
        }
    ));
    assert!(matches!(
        refusal(Defect::MissingIndexCoordinate),
        IndexBuildError::GatherIndexCoordinateRank {
            expected: 1,
            actual: 0,
        }
    ));
}

#[test]
fn one_tensor_cannot_play_both_gather_roles() {
    assert!(matches!(
        refusal(Defect::Aliased),
        IndexBuildError::GatherAliasedTensors { .. }
    ));
}

#[test]
fn a_non_f32_source_is_refused_by_name() {
    assert!(matches!(
        refusal(Defect::SourceIsU32),
        IndexBuildError::GatherSourceNotF32 { .. }
    ));
}

#[test]
fn a_non_u32_index_is_refused_by_name() {
    assert!(matches!(
        refusal(Defect::IndexIsF32),
        IndexBuildError::GatherIndexNotU32 { .. }
    ));
}

/// Each of the three literal refusals fires independently and *before* the
/// domain-shape comparison.
///
/// The environment here determines every symbol, so a rule that consulted it
/// would derive a concrete shape and report a shape disagreement — or worse,
/// admit the region. Reporting the authored spelling is what keeps sourced
/// gather support a named refusal rather than an accident of binding.
#[test]
fn each_sourced_boundary_and_domain_extent_is_refused_before_the_domain_shape() {
    assert!(
        matches!(
            refusal(Defect::SourcedSource),
            IndexBuildError::GatherSourceShapeNotLiteral { .. }
        ),
        "a sourced source boundary is refused under its own name",
    );
    assert!(
        matches!(
            refusal(Defect::SourcedIndex),
            IndexBuildError::GatherIndexShapeNotLiteral { .. }
        ),
        "a sourced index boundary is refused under its own name",
    );
    assert!(
        matches!(
            refusal(Defect::SourcedDomain),
            IndexBuildError::GatherDomainExtentNotLiteral { .. }
        ),
        "a sourced result-domain extent is refused under its own name",
    );
}

#[test]
fn an_axis_outside_the_source_is_refused() {
    assert!(matches!(
        refusal(Defect::Axis(7)),
        IndexBuildError::GatherAxisOutOfRange { source_rank: 1, .. }
    ));
}

/// A declared domain that disagrees with the derived result shape is refused.
#[test]
fn a_domain_disagreeing_with_the_derived_result_shape_is_refused() {
    let mut builder = IndexRegionBuilder::new(registry()).unwrap();
    let d0 = builder
        .dimension(DomainRole::Parallel, Extent::new(7))
        .unwrap();
    let c0 = builder.dimension_expr(d0).unwrap();
    let source = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type().clone(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let index = builder
        .tensor(
            TensorRole::Input,
            gather_index_resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let error = builder
        .gather_read(source, index, &[d0], &[], &[c0], Axis::new(0))
        .expect_err("a domain of [7] is not the derived [3]");
    assert!(matches!(error, IndexBuildError::GatherDomainShape { .. }));
}

// ---------------------------------------------------------------------------
// The exhaustive view
// ---------------------------------------------------------------------------

/// Every access answers exactly one arm of the exhaustive view.
#[test]
fn the_access_view_is_exhaustive_and_the_arms_are_exclusive() {
    let region = GatherCase::new(&[4, 5], &[3], 1).build(false).unwrap();
    let mut direct = 0_usize;
    let mut gathered = 0_usize;
    for access in region.accesses() {
        match access.view() {
            TensorAccessView::Direct(_) => direct += 1,
            TensorAccessView::GatherRead(_) => gathered += 1,
        }
        assert_ne!(
            access.view().direct().is_some(),
            access.view().gather_read().is_some(),
            "an access is exactly one kind",
        );
    }
    assert_eq!(gathered, 1, "the fixture authors exactly one gather");
    assert_eq!(direct, 1, "and exactly one direct write");
}

/// `GatherAccessRule` is inspectable and the census covers all of it.
///
/// The rule vocabulary is what the later verifier reports under; naming it here
/// keeps the enum reachable from outside the crate and pins that the diagnostic
/// does not collapse into `CoordinateOutOfBounds`.
///
/// The two assertions below are load-bearing in different directions and
/// neither is redundant. `variant_count` is read from the **type**, so a
/// widened or narrowed vocabulary is a failure here rather than a census that
/// silently stops covering its domain — a hand-written length would pass
/// unchanged through either. The pairwise comparison then rules out the one
/// list a correct length still admits: one naming a variant twice and omitting
/// another. Together they pin the list as exactly the vocabulary.
///
/// `BoundsResolution` is in the list and **no production site constructs it**;
/// `verify_gather_access` raises the other fourteen and never that one. It is
/// carried here as vocabulary, not as evidence that anything can raise it.
#[test]
fn the_gather_rule_vocabulary_is_publicly_inspectable() {
    let rules = [
        GatherAccessRule::SourceRole,
        GatherAccessRule::IndexRole,
        GatherAccessRule::SourceType,
        GatherAccessRule::IndexType,
        GatherAccessRule::SourceShapeLiteral,
        GatherAccessRule::IndexShapeLiteral,
        GatherAccessRule::SourceRank,
        GatherAccessRule::Axis,
        GatherAccessRule::SourceCoordinateRank,
        GatherAccessRule::IndexCoordinateRank,
        GatherAccessRule::DomainExtentLiteral,
        GatherAccessRule::DomainShape,
        GatherAccessRule::SourceCoordinateScope,
        GatherAccessRule::IndexCoordinateScope,
        GatherAccessRule::BoundsResolution,
    ];
    assert_eq!(
        rules.len(),
        variant_count::<GatherAccessRule>(),
        "the census must name every rule the vocabulary admits",
    );
    for (position, rule) in rules.iter().enumerate() {
        for other in &rules[position + 1..] {
            assert_ne!(rule, other, "each rule names one obligation");
        }
    }
}

// ---------------------------------------------------------------------------
// Declaration order: the domain is a set, and both validators read it as one
// ---------------------------------------------------------------------------

/// One `out = gather(source=[4, 5], index=[3], axis=1)` under a chosen order.
///
/// The derived result is `[4, 3]`. `creation_reversed` declares the extent-3
/// result dimension **first**, so the ascending-ordinal run the builder commits
/// is the reverse of the result order; `slice_by_ordinal` then hands
/// `gather_read` its domain in ordinal order rather than in result order. Only
/// the declaration order varies — the dimension set, the extents, the output
/// shape, the coordinates, and the axis are identical across all four.
///
/// That two-by-two is the whole space, because the two validators read the
/// domain from opposite ends: `prepare_gather_access` sees the caller's slice
/// and `verify_gather_access` sees the committed sorted run. Nothing else can
/// separate them, and every other fixture in this file, in
/// `tiler-reference`'s region oracle, and in `proof.rs`'s own `admitted_gather`
/// declares result dimensions ascending, so caller order and ordinal order
/// coincide there and none of them can see this at all.
fn ordered_domain_gather(
    creation_reversed: bool,
    slice_by_ordinal: bool,
) -> Result<VerifiedIndexRegion, String> {
    let mut builder = IndexRegionBuilder::new(registry()).map_err(|error| format!("{error:?}"))?;
    // Result axis 0 has extent 4 and result axis 1 has extent 3, whichever
    // order the two are declared in.
    let (four, three) = if creation_reversed {
        let three = builder
            .dimension(DomainRole::Parallel, Extent::new(3))
            .unwrap();
        let four = builder
            .dimension(DomainRole::Parallel, Extent::new(4))
            .unwrap();
        (four, three)
    } else {
        let four = builder
            .dimension(DomainRole::Parallel, Extent::new(4))
            .unwrap();
        let three = builder
            .dimension(DomainRole::Parallel, Extent::new(3))
            .unwrap();
        (four, three)
    };
    let source = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type().clone(),
            Shape::from_dims([4, 5]),
        )
        .unwrap();
    let index = builder
        .tensor(
            TensorRole::Input,
            gather_index_resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type().clone(),
            Shape::from_dims([4, 3]),
        )
        .unwrap();
    let outer = builder.dimension_expr(four).unwrap();
    let inner = builder.dimension_expr(three).unwrap();
    let result_order = [four, three];
    let ordinal_order = if creation_reversed {
        [three, four]
    } else {
        [four, three]
    };
    let domain = if slice_by_ordinal {
        ordinal_order
    } else {
        result_order
    };
    // Source axis 0 is the one the loaded U32 does not supply; the index run is
    // the single index axis. Both are unchanged by the declaration order.
    let value = builder
        .gather_read(source, index, &domain, &[outer], &[inner], Axis::new(1))
        .map_err(|error| format!("gather_read refused: {error:?}"))?;
    let write = builder
        .write(output, &result_order, &[outer, inner])
        .map_err(|error| format!("{error:?}"))?;
    builder
        .output(write, value)
        .map_err(|error| format!("{error:?}"))?;
    builder
        .build()
        .map_err(|error| format!("build refused: {error:?}"))
}

/// Every declaration order of one gather is admitted by **both** validators.
///
/// The defect this pins is not a wrong answer but a disagreement: the caller's
/// slice order and the committed ordinal order are two spellings of one set,
/// and a rule that read one of them refused a region the other admitted. Three
/// of these four cases pass under a rule that compares sequences; the fourth —
/// dimensions declared in reverse, domain supplied in result order — is
/// admitted by `gather_read` and then refused by `build`, at a different layer
/// under a different diagnostic, which is a region no caller can author in
/// either spelling.
///
/// Driving all four rather than that one case is deliberate: the two orders are
/// independent inputs, and a repair that fixed the authoring check by making it
/// read the sorted run would move the refusal rather than remove it, leaving
/// this test's fourth row green and its third row red.
#[test]
fn every_declaration_order_of_one_gather_is_admitted_by_both_validators() {
    for creation_reversed in [false, true] {
        for slice_by_ordinal in [false, true] {
            assert!(
                ordered_domain_gather(creation_reversed, slice_by_ordinal).is_ok(),
                "a gather declaring its result dimensions reversed={creation_reversed} and \
                 supplying its domain by-ordinal={slice_by_ordinal} names the same set of \
                 dimensions as every other spelling and must be admitted alike",
            );
        }
    }
}

/// Two orders of one domain intern to one access, and so to one value.
///
/// This is the identity half of the same property: the committed `domain` is
/// the ascending collection of a `BTreeSet`, so two callers naming the same
/// dimensions in different orders build the *same* record. Asserting it here
/// keeps a later order-carrying `domain` from splitting one meaning across two
/// identities without that being a deliberate, reviewed decision — under such a
/// change these two calls would return different values and this reddens.
#[test]
fn two_orders_of_one_gather_domain_intern_to_one_value() {
    let mut builder = IndexRegionBuilder::new(registry()).unwrap();
    let four = builder
        .dimension(DomainRole::Parallel, Extent::new(4))
        .unwrap();
    let three = builder
        .dimension(DomainRole::Parallel, Extent::new(3))
        .unwrap();
    let source = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type().clone(),
            Shape::from_dims([4, 5]),
        )
        .unwrap();
    let index = builder
        .tensor(
            TensorRole::Input,
            gather_index_resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let outer = builder.dimension_expr(four).unwrap();
    let inner = builder.dimension_expr(three).unwrap();
    let ascending = builder
        .gather_read(
            source,
            index,
            &[four, three],
            &[outer],
            &[inner],
            Axis::new(1),
        )
        .expect("the result-order spelling is admitted");
    let descending = builder
        .gather_read(
            source,
            index,
            &[three, four],
            &[outer],
            &[inner],
            Axis::new(1),
        )
        .expect("the reversed spelling names the same set and is admitted alike");
    assert_eq!(
        ascending, descending,
        "the domain is a set at rest, so two orders of it are one access and one value",
    );
}

/// A domain carrying the wrong extents is still refused, in both spellings.
///
/// The negative control for the two tests above: order-insensitivity must not
/// have been bought by dropping the rule. `[4, 4]` is neither the derived
/// `[4, 3]` nor any permutation of it, and the refusal must arrive from
/// `gather_read` rather than from `build`, because the authoring surface owns
/// the caller's diagnostics.
#[test]
fn a_domain_whose_extents_are_not_the_result_extents_is_still_refused() {
    let mut builder = IndexRegionBuilder::new(registry()).unwrap();
    let four = builder
        .dimension(DomainRole::Parallel, Extent::new(4))
        .unwrap();
    let other = builder
        .dimension(DomainRole::Parallel, Extent::new(4))
        .unwrap();
    let source = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type().clone(),
            Shape::from_dims([4, 5]),
        )
        .unwrap();
    let index = builder
        .tensor(
            TensorRole::Input,
            gather_index_resolved_type(),
            Shape::from_dims([3]),
        )
        .unwrap();
    let outer = builder.dimension_expr(four).unwrap();
    let inner = builder.dimension_expr(other).unwrap();
    for domain in [[four, other], [other, four]] {
        let error = builder
            .gather_read(source, index, &domain, &[outer], &[inner], Axis::new(1))
            .expect_err("a domain of [4, 4] is not the derived [4, 3] in any order");
        assert!(
            matches!(error, IndexBuildError::GatherDomainShape { .. }),
            "the extent census still refuses under its own name, not another rule",
        );
    }
}

/// A gather commits its domain in ascending ordinal order, whatever order the
/// caller wrote it in.
///
/// The subject is **compaction's** sort, not the draft builder's. A gather's
/// domain is sorted three separate times on the way here — the draft commits
/// the ascending collection of a `BTreeSet`, compaction remaps and re-sorts,
/// and the alpha access key sorts again before hashing — so the draft's own
/// order is unobservable, and replacing its `collect` with `rev().collect()`
/// leaves the entire workspace suite green. What this test can lose is the
/// middle sort: `encode_gather_bounds_identity` frames `subject.domain` as a
/// run of ordinals taken from the compacted access in stored order, so if
/// compaction stopped sorting, one gather would take two bounds-proof
/// identities depending on which order its author happened to write.
///
/// That the alpha key sorts the domain independently is also the evidence that
/// the order was never part of a gather's identity, which is what makes the
/// multiset comparison in `GatherDomainShape` a repair rather than a
/// weakening: the identity boundary already treated this field as a set.
#[test]
fn a_gather_commits_its_domain_in_ascending_ordinal_order() {
    for creation_reversed in [false, true] {
        for slice_by_ordinal in [false, true] {
            let region = ordered_domain_gather(creation_reversed, slice_by_ordinal)
                .expect("every declaration order is admitted");
            let domain: Vec<_> = region
                .accesses()
                .find(|access| access.view().gather_read().is_some())
                .expect("the fixture authors exactly one gather")
                .domain()
                .collect();
            assert_eq!(
                domain.len(),
                2,
                "the fixture's gather iterates two dimensions"
            );
            assert!(
                domain.is_sorted(),
                "a gather authored reversed={creation_reversed} by-ordinal={slice_by_ordinal} \
                 must still commit its domain ascending, because both the extent-multiset rule \
                 and the bounds-proof identity read the committed run as canonical",
            );
        }
    }
}
