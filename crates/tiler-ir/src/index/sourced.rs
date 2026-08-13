//! The index layer's sourced-scalar vocabulary, and the three authorities that
//! can refuse a sourced construction call.
//!
//! # What lives at the shape layer instead
//!
//! The constant-or-symbol *magnitude* vocabulary this builds on —
//! [`SourcedExtent`], [`SourcedShape`], [`ExtentSources`],
//! [`ExtentSourceError`], and [`EXTENT_PHASE_CEILING`] — is
//! [`crate::shape`]'s, because `SourcedExtent` is `Extent | ShapeSymbol` and
//! both of those are shape-layer types. That module owns the four rejections,
//! the phase ceiling, and what a [`ShapeEnv`] lets a verifier prove, and none of
//! it is restated here.
//!
//! # What stays here, and why it could not move with them
//!
//! Both remaining types name an index-layer type in their own definition, so
//! siting them at the shape layer would make the crate's base vocabulary depend
//! on one of its consumers:
//!
//! - [`SourcedIndexInteger`] is `IndexInteger | ShapeSymbol`. A coefficient is a
//!   signed, arbitrary-precision index scalar rather than a magnitude, which is
//!   why it is a second type at all; the type's own documentation gives that
//!   argument. Its *symbol* half is not duplicated — it converts from the one
//!   magnitude vocabulary and admits through [`ExtentSources::admit`].
//! - [`SymbolicExtentError`] is the union of the three authorities that can
//!   refuse one of this layer's construction calls, and one of them is
//!   [`IndexBuildError`]. It is deliberately *not* shared: another consumer of
//!   the sourced vocabulary would name its own structural authority in that
//!   slot, so one shared union would report one layer's limit under another
//!   layer's name — the exact collapse the [`SymbolicExtentError::ShapeVocabulary`]
//!   variant already argues against for the neighbouring pair.
//!
//! # Why the tests are here
//!
//! Every fixture below builds an [`IndexRegionBuilder`], because an admission,
//! a canonical identity, and a bounds proof are only observable through a
//! region. They therefore exercise the relocated shape-layer vocabulary end to
//! end as well, and are kept in one module rather than split so that the
//! environment fixtures they share are declared once.
//!
//! [`SourcedExtent`]: crate::shape::SourcedExtent
//! [`SourcedShape`]: crate::shape::SourcedShape
//! [`ExtentSources`]: crate::shape::ExtentSources
//! [`ExtentSources::admit`]: crate::shape::ExtentSources::admit
//! [`ExtentSourceError`]: crate::shape::ExtentSourceError
//! [`EXTENT_PHASE_CEILING`]: crate::shape::EXTENT_PHASE_CEILING
//! [`IndexRegionBuilder`]: super::IndexRegionBuilder

use super::{IndexBuildError, IndexInteger};
use crate::shape::{ExtentSourceError, ShapeError, ShapeSymbol, SourcedExtent};

/// One signed index-expression scalar, and where its value comes from.
///
/// **Draft surface, not yet accepted.** This type, its variants, its
/// conversions, its tag byte, and its canonical encoding are a concrete draft
/// pending Tom's acceptance of the widened linear-combination boundary, along
/// with [`IndexRegionBuilder::sourced_linear_combination`],
/// [`LinearTermRef::coefficient`], and the `constant` field of
/// [`IndexExprView::LinearCombination`]. It is in use inside `tiler-ir`
/// meanwhile, exactly as the sourced divisor's vocabulary was; the label is
/// what an acceptance flips.
///
/// [`IndexRegionBuilder::sourced_linear_combination`]: super::IndexRegionBuilder::sourced_linear_combination
/// [`LinearTermRef::coefficient`]: super::LinearTermRef::coefficient
/// [`IndexExprView::LinearCombination`]: super::IndexExprView::LinearCombination
/// [`ExtentSources::admit`]: crate::shape::ExtentSources::admit
/// [`SourcedShape::as_static`]: crate::shape::SourcedShape::as_static
///
/// # Why this is not [`SourcedExtent`]
///
/// [`SourcedExtent`] is the crate's one constant-or-symbol vocabulary for a
/// *magnitude*, and a coefficient is not a magnitude: it is a
/// signed, arbitrary-precision [`IndexInteger`], and existing regions are
/// authored with negative ones. Widening `SourcedExtent` to carry a sign would
/// put a signed value where a domain extent, a boundary axis, and a divisor all
/// require a nonnegative one, so the two domains stay distinct types.
///
/// What is *not* duplicated is the symbol half. A frontend that holds the
/// [`SourcedExtent`] it sized a domain or a boundary with converts *that* into
/// this type, so there is exactly one spelling for "this value comes from a
/// declared symbol", one admission path ([`ExtentSources::admit`]), and one
/// place to extend when a third source kind arrives.
///
/// # The normalization invariant
///
/// The [`SourcedExtent`] conversion collapses [`SourcedExtent::Static`] into
/// [`Self::Literal`], so [`Self::Symbol`] holds a symbol and a value has exactly
/// one spelling. That is what makes [`Self::as_literal`] a fact about the value
/// rather than about which constructor authored it, exactly as
/// [`SourcedShape::as_static`] is about its boundary.
///
/// # Why a symbolic value carries no sign
///
/// [`Self::Symbol`] resolves through a [`ShapeSymbol`], which names an extent
/// and is therefore never negative. That loses nothing, because multiplication
/// is commutative and the *operand* carries the sign: `-B * p` is written as the
/// term `B * (-p)`, and a negated symbolic addend is the term `S * (-1)` over a
/// constant operand. A signed symbolic variant would add a second spelling for
/// each of those without adding a program.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourcedIndexInteger {
    /// An exact signed integer fixed when the region was authored.
    Literal(IndexInteger),
    /// A declared `ShapeEnv` symbol, resolved through that environment alone.
    Symbol(ShapeSymbol),
}

impl SourcedIndexInteger {
    /// Returns the governed tag of this source kind, exhaustively.
    ///
    /// Written by a match rather than read from the discriminant, for the
    /// reason [`SourcedExtent`]'s own tag gives: adding a kind is a build error
    /// here instead of a silent re-encoding of every region identity ever
    /// derived (ADR 0074 convention 3).
    const fn tag(&self) -> u8 {
        match self {
            Self::Literal(_) => 0x01,
            Self::Symbol(_) => 0x02,
        }
    }

    /// Returns the symbol this value names, if it names one.
    #[must_use]
    pub const fn symbol(&self) -> Option<&ShapeSymbol> {
        match self {
            Self::Symbol(symbol) => Some(symbol),
            Self::Literal(_) => None,
        }
    }

    /// Returns the exact integer, for a literally authored value only.
    ///
    /// `None` for a symbolic value even when its environment determines one:
    /// this asks what was *written*, exactly as [`SourcedExtent::as_static`]
    /// does, and identity is a function of that. A pass that can use a pinned
    /// value reads [`ExtentSources::determined`] explicitly.
    ///
    /// [`ExtentSources::determined`]: crate::shape::ExtentSources::determined
    #[must_use]
    pub const fn as_literal(&self) -> Option<&IndexInteger> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Symbol(_) => None,
        }
    }

    /// Appends this value's canonical bytes.
    ///
    /// A symbolic value encodes its symbol, never a resolved value, for the
    /// reason [`SourcedExtent::encode`] states: folding a bound value in here
    /// would collapse `graph identity` into `specialized identity`. The region
    /// folds the environment's own identity once, so the symbol reference is
    /// complete.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::Literal(value) => value.encode(bytes),
            Self::Symbol(symbol) => symbol.encode(bytes),
        }
    }

    /// Returns the exact canonical byte length [`Self::encode`] appends.
    pub(crate) fn encoded_len(&self) -> usize {
        1_usize.saturating_add(match self {
            Self::Literal(value) => value.encoded_len(),
            Self::Symbol(symbol) => symbol.encoded_len(),
        })
    }
}

impl From<IndexInteger> for SourcedIndexInteger {
    fn from(value: IndexInteger) -> Self {
        Self::Literal(value)
    }
}

impl From<i128> for SourcedIndexInteger {
    fn from(value: i128) -> Self {
        Self::Literal(IndexInteger::from_i128(value))
    }
}

impl From<u64> for SourcedIndexInteger {
    fn from(value: u64) -> Self {
        Self::Literal(IndexInteger::from_u64(value))
    }
}

impl From<ShapeSymbol> for SourcedIndexInteger {
    fn from(value: ShapeSymbol) -> Self {
        Self::Symbol(value)
    }
}

impl From<SourcedExtent> for SourcedIndexInteger {
    /// Reads the crate's one magnitude vocabulary as a signed index scalar.
    ///
    /// This is the bridge that keeps the symbol half single: a frontend holding
    /// the [`SourcedExtent`] it sized a domain or a boundary with multiplies by
    /// *that* rather than re-naming its symbol through a second type. A literal
    /// extent normalizes to [`Self::Literal`] so the two vocabularies cannot
    /// disagree about what `4` is.
    fn from(value: SourcedExtent) -> Self {
        match value {
            SourcedExtent::Static(extent) => Self::Literal(IndexInteger::from_u64(extent.get())),
            SourcedExtent::Symbol(symbol) => Self::Symbol(symbol),
        }
    }
}

/// A rejected sourced extent, divisor, coefficient, or boundary.
///
/// Three authorities can refuse the same call and they stay separable rather
/// than collapsing into one message, because a caller acts differently on each:
/// a structural limit says the region is too large and a smaller one would be
/// admitted; a shape-vocabulary refusal says no [`Shape`] can hold the
/// normalized form; and a source refusal says the environment does not declare,
/// supply, or prove what using the extent there requires, so retrying without
/// changing the environment cannot succeed.
///
/// **An index-layer type, unlike the vocabulary it reports on.** Only
/// [`Self::Source`] is the shared environment authority; the other two name
/// *this* layer's structural limits and the shape vocabulary's. A second
/// consumer of [`SourcedExtent`] would fill the structural slot with its own
/// build error, so one shared union would report one layer's limit under
/// another's name — which is exactly what [`Self::ShapeVocabulary`] is spelled
/// apart from [`Self::Structural`] to prevent one level down.
///
/// The [`From`] conversions below are ergonomic only. Each one lands in the
/// variant that names its own authority, so `?` never reports one authority's
/// limit under another's name.
///
/// [`Shape`]: crate::shape::Shape
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolicExtentError {
    /// The extent's source was refused by this region's shape environment.
    Source(ExtentSourceError),
    /// A governed structural limit or handle rule refused the insertion.
    Structural(IndexBuildError),
    /// The shape vocabulary refused to represent the boundary at all.
    ///
    /// Distinct from [`Self::Structural`] because the two name different
    /// authorities: the index layer's own `MAX_TENSOR_RANK` governs how large a
    /// boundary *this* IR admits, while [`ShapeError`] is the shape
    /// vocabulary's bound on what a [`Shape`] can hold. Collapsing them would
    /// report one limit's rejection under the other's name.
    ///
    /// Spelled `ShapeVocabulary` rather than `Shape`: a public variant named
    /// `Shape` puts a second `Shape` in the crate's exported name table, and
    /// rustc then stops printing the short path for the [`Shape`] *type* in
    /// every diagnostic about it. That is an observed regression in this
    /// crate's byte-compared `trybuild` goldens, not a style preference.
    ///
    /// [`Shape`]: crate::shape::Shape
    ShapeVocabulary(ShapeError),
}

impl std::fmt::Display for SymbolicExtentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "{error}"),
            Self::Structural(error) => write!(formatter, "{error}"),
            Self::ShapeVocabulary(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SymbolicExtentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::Structural(error) => Some(error),
            Self::ShapeVocabulary(error) => Some(error),
        }
    }
}

impl From<ExtentSourceError> for SymbolicExtentError {
    fn from(value: ExtentSourceError) -> Self {
        Self::Source(value)
    }
}

impl From<IndexBuildError> for SymbolicExtentError {
    fn from(value: IndexBuildError) -> Self {
        Self::Structural(value)
    }
}

impl From<ShapeError> for SymbolicExtentError {
    fn from(value: ShapeError) -> Self {
        Self::ShapeVocabulary(value)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::variant_count;
    use std::num::NonZeroU64;
    use std::sync::Arc;

    use super::{SourcedIndexInteger, SymbolicExtentError};
    use crate::index::{
        AccessMode, BoundsProofView, DomainRole, FrozenScalarRegistry, IndexBuildError,
        IndexDomainFactSource, IndexDomainPredicate, IndexDomainUnknownReason, IndexExprClass,
        IndexExprView, IndexExtentRef, IndexRegionBuildError, IndexRegionBuilder,
        IndexRegionDiagnostic, ScalarArity, ScalarAttributeSchema, ScalarAttributes, ScalarEffect,
        ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey,
        ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
        ScalarRegistryBuilder, TensorRole, VerifiedIndexRegion, WriteOwnershipProofView,
    };
    use crate::program::abi::AvailabilityPhase;
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    };
    use crate::shape::{
        BindingSource, EXTENT_PHASE_CEILING, Extent, ExtentRelation, ExtentSourceError,
        ExtentSources, ExtentTerm, FactProvenance, GuardApplicability, InterfaceParameterKey,
        RootBinding, SemanticInputConstraint, Shape, ShapeEnv, ShapeEnvBuilder, ShapeSymbol,
        SourcedExtent, SourcedShape, SymbolScope, VariantGuard,
    };

    #[test]
    fn the_sourced_index_integer_tag_table_is_injective_over_its_variant_set() {
        let values: [SourcedIndexInteger; variant_count::<SourcedIndexInteger>()] = [
            SourcedIndexInteger::Literal(0_i128.into()),
            SourcedIndexInteger::Symbol(symbol("tag-source")),
        ];
        crate::exhaustive_injectivity::assert_tag_table_ref(
            "SourcedIndexInteger::tag",
            &values,
            SourcedIndexInteger::tag,
        );
    }

    struct Types;
    impl SemanticRegistryProvider for Types {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("example", "types", 1).unwrap()
        }
        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("example", "pixel", 1).unwrap()),
                NormativeDefinitionRef::new("urn:example:pixel:v1").unwrap(),
                TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            ))
        }
    }

    fn value_type() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("example", "pixel", 1).unwrap())
    }

    /// Produces the fixture element type, whatever it is applied to.
    struct Produce;
    impl ScalarOperationInferencer for Produce {
        fn infer(
            &self,
            _: ScalarInferenceRequest<'_>,
            outputs: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            outputs.try_push(value_type())
        }
    }

    fn zero_key() -> ScalarOpKey {
        ScalarOpKey::new("example", "zero", 1).unwrap()
    }

    fn step_key() -> ScalarOpKey {
        ScalarOpKey::new("example", "step", 1).unwrap()
    }

    fn definition(key: ScalarOpKey, operands: usize) -> ScalarOperationDefinition {
        ScalarOperationDefinition::new(
            key,
            NormativeDefinitionRef::new("urn:example:scalar:v1").unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(operands).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::record([]).unwrap(),
                CanonicalValue::record([]).unwrap(),
            ),
            Arc::new(Produce),
        )
    }

    fn registry() -> FrozenScalarRegistry {
        let mut semantic = SemanticRegistryBuilder::new();
        semantic.register_provider(&Types).unwrap();
        // Ad-hoc: `example::scalars` over a sourced-extent fixture. The subject is the
        // sourced-shape plumbing rather than the governed vocabulary.
        let mut scalar = ScalarRegistryBuilder::new(semantic.freeze().unwrap());
        let provider = ProviderIdentity::new("example", "scalars", 1).unwrap();
        scalar
            .register(provider.clone(), definition(zero_key(), 0))
            .unwrap();
        scalar
            .register(provider, definition(step_key(), 2))
            .unwrap();
        scalar.freeze()
    }

    fn symbol(name: &str) -> ShapeSymbol {
        ShapeSymbol::new(SymbolScope::new("region/0").unwrap(), name).unwrap()
    }

    fn term(name: &str) -> ExtentTerm {
        ExtentTerm::Symbol(symbol(name))
    }

    /// An interface parameter, whose source class floors exactly at the ceiling.
    ///
    /// One parameter key per symbol: two symbols bound to *one* parameter would
    /// be one value, and an environment that recorded that in its bindings
    /// without recording it as a constraint would be stating a fact its own
    /// decision procedure could not see.
    fn parameter_binding(name: &str, phase: AvailabilityPhase) -> RootBinding {
        RootBinding::new(
            BindingSource::InterfaceParameter {
                key: InterfaceParameterKey::new(name).unwrap(),
            },
            phase,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    /// An environment declaring each named symbol once, with given relations.
    fn environment_over(
        phase: AvailabilityPhase,
        names: &[&str],
        relations: &[ExtentRelation],
    ) -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        for name in names {
            let declared = symbol(name);
            draft.declare(declared.clone()).unwrap();
            draft
                .bind(&declared, parameter_binding(name, phase))
                .unwrap();
        }
        for relation in relations {
            draft
                .require(SemanticInputConstraint::new(
                    relation.clone(),
                    FactProvenance::FrontendRequired,
                ))
                .unwrap();
        }
        Arc::new(draft.build().unwrap())
    }

    /// An environment declaring `n` with one root binding and given relations.
    fn environment(phase: AvailabilityPhase, relations: &[ExtentRelation]) -> Arc<ShapeEnv> {
        environment_over(phase, &["n"], relations)
    }

    /// An environment that only *guards* the named relations.
    ///
    /// The neighbour of [`environment_over`] with the same relations recorded on
    /// the other side of the contract's semantic-constraint/variant-guard line.
    fn guarded_environment(names: &[&str], relations: &[ExtentRelation]) -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        for name in names {
            let declared = symbol(name);
            draft.declare(declared.clone()).unwrap();
            draft
                .bind(&declared, parameter_binding(name, EXTENT_PHASE_CEILING))
                .unwrap();
        }
        for relation in relations {
            draft
                .guard(VariantGuard::new(
                    relation.clone(),
                    GuardApplicability::Schedule,
                ))
                .unwrap();
        }
        Arc::new(draft.build().unwrap())
    }

    /// Copies `input[i / d]` into `output[i]` over a static domain of eight.
    ///
    /// The divisor is the only symbolic thing in the region, so what the region
    /// can prove about the read turns on the environment's facts about `d` and
    /// on nothing else. The write is a plain permutation of the static parallel
    /// dimension, so ownership is discharged whatever the divisor does and the
    /// fixture measures the bounds argument alone.
    fn divided_copy(
        environment: Option<Arc<ShapeEnv>>,
        divisor: SourcedExtent,
    ) -> Result<Result<VerifiedIndexRegion, IndexRegionBuildError>, SymbolicExtentError> {
        let mut builder = match environment {
            Some(environment) => {
                IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap()
            }
            None => IndexRegionBuilder::new(registry()).unwrap(),
        };
        let input = builder
            .tensor(TensorRole::Input, value_type(), Shape::from_dims([8]))
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type(), Shape::from_dims([8]))
            .unwrap();
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(8))
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let quotient = builder.floor_div(coordinate, divisor)?;
        let value = builder.read(input, &[dimension], &[quotient]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        Ok(builder.build())
    }

    /// Reduces one input axis of symbolic extent into a rank-zero output.
    ///
    /// The symbolic dimension is a *reduction* dimension deliberately. A write
    /// must cover every parallel dimension and prove it owns its output
    /// exactly, which needs a determined extent; a read over a reduction axis
    /// only has to prove it stays in bounds, which the environment's interval
    /// can establish on its own. This is therefore the fixture that exercises a
    /// bounded-but-undetermined extent, and it is also the realistic case — a
    /// sum over an axis whose length the caller supplies.
    fn region(
        environment: Option<Arc<ShapeEnv>>,
        input_extent: u64,
    ) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
        let mut builder = match environment {
            Some(environment) => {
                IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap()
            }
            None => IndexRegionBuilder::new(registry()).unwrap(),
        };
        let input = builder
            .tensor(
                TensorRole::Input,
                value_type(),
                Shape::from_dims([input_extent]),
            )
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type(), Shape::from_dims([]))
            .unwrap();
        let reduced = builder
            .symbolic_dimension(DomainRole::Reduction, symbol("n"))
            .expect("the source is admissible in these fixtures");
        let coordinate = builder.dimension_expr(reduced).unwrap();
        let contributor = builder.read(input, &[reduced], &[coordinate]).unwrap();
        let initial = builder
            .apply(zero_key(), ScalarAttributes::empty(), &[])
            .unwrap()
            .get(0)
            .unwrap();
        let total = builder
            .reduce(&[reduced], &[initial], &[contributor], |body| {
                let state = body.state(0).expect("one accumulator");
                let contributed = body.contributor(0).expect("one contributor");
                let stepped =
                    body.apply(step_key(), ScalarAttributes::empty(), &[state, contributed])?;
                let stepped = stepped.get(0).expect("one result");
                body.yield_values(&[stepped])
            })
            .unwrap()
            .get(0)
            .unwrap();
        let write = builder.write(output, &[], &[]).unwrap();
        builder.output(write, total).unwrap();
        builder.build()
    }

    fn reports(error: &IndexRegionBuildError, wanted: fn(&IndexRegionDiagnostic) -> bool) -> bool {
        error.diagnostics().iter().any(wanted)
    }

    /// A symbolic domain is bounded by the environment, not by a literal.
    ///
    /// This is what the ticket is for: the region proves its access in bounds
    /// without ever knowing the extent, because the constraint environment
    /// bounds the symbol tightly enough. The rejecting neighbour differs only
    /// in that bound, so the acceptance is evidence about the environment
    /// rather than about the region's shape.
    #[test]
    fn a_symbolic_domain_proves_its_bounds_from_the_environment() {
        let bounded = environment(
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(term("n"), 1, 4).unwrap()],
        );
        let bounded = region(Some(bounded), 4).expect("`n <= 4` bounds a read of a 4-element axis");
        // The retained evidence names *how* it was proved and *what it read*.
        // Interval, not enumeration: nothing walked a domain whose size is
        // unknown. And the environment, not the program: the bound came from
        // `n`'s declared extent and nothing in the region carries it.
        assert!(
            bounded.accesses().any(|access| access.bounds_proof()
                == Some(BoundsProofView::Interval {
                    facts: IndexDomainFactSource::ShapeEnvironment,
                })),
            "the symbolic read is proved by the environment's interval",
        );

        // One larger admissible extent leaves only the upper-bound atom unresolved.
        let too_wide = environment(
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(term("n"), 1, 5).unwrap()],
        );
        let too_wide = region(Some(too_wide), 4).expect("an unknown read bound is retained");
        let read = too_wide
            .accesses()
            .find(|access| access.mode() == AccessMode::Read)
            .unwrap();
        let expression = read.coordinates().next().unwrap();
        let predicate = IndexDomainPredicate::LessThanExtent {
            expression,
            extent: IndexExtentRef::TensorAxis {
                tensor: read.tensor(),
                axis: 0,
            },
        };
        let unknown = too_wide
            .index_domain_unknown(read.id(), predicate)
            .unwrap()
            .expect("only the upper bound is unknown");
        assert_eq!(
            unknown.reason(),
            IndexDomainUnknownReason::InsufficientFacts
        );
        assert_eq!(too_wide.unknown_index_domain_predicates().count(), 1);
    }

    /// An unbounded symbolic extent is retained as an exact semantic obligation.
    #[test]
    fn an_unbounded_symbolic_extent_is_retained_without_enumeration() {
        let region = region(Some(environment(EXTENT_PHASE_CEILING, &[])), 4)
            .expect("missing facts are not a disproval");
        let unknown = region.unknown_index_domain_predicates().collect::<Vec<_>>();
        assert_eq!(unknown.len(), 1);
        assert!(matches!(
            unknown[0].predicate(),
            IndexDomainPredicate::LessThanExtent { .. }
        ));
        assert_eq!(
            unknown[0].reason(),
            IndexDomainUnknownReason::InsufficientFacts
        );
    }

    /// A source that arrives after the ceiling is refused where it is written.
    ///
    /// The accepted contract admits only bindings "evaluable on the host before
    /// any device work begins". A domain extent that first exists once a
    /// pipeline is prepared would make the iteration domain depend on a plan
    /// derived from that same domain.
    #[test]
    fn a_source_after_the_phase_ceiling_is_refused_at_the_dimension() {
        for phase in [
            AvailabilityPhase::PreparedKernelPreflight,
            AvailabilityPhase::LaunchPreflight,
        ] {
            let mut builder =
                IndexRegionBuilder::new_with_shape_environment(registry(), environment(phase, &[]))
                    .unwrap();
            assert_eq!(
                builder.symbolic_dimension(DomainRole::Parallel, symbol("n")),
                Err(SymbolicExtentError::Source(
                    ExtentSourceError::SourceTooLate {
                        symbol: symbol("n"),
                        available: phase,
                        ceiling: EXTENT_PHASE_CEILING,
                    }
                )),
            );
        }

        // The same source one phase earlier is admitted, so the refusals above
        // are about when the value arrives rather than about the source class.
        let mut admitted = IndexRegionBuilder::new_with_shape_environment(
            registry(),
            environment(EXTENT_PHASE_CEILING, &[]),
        )
        .unwrap();
        admitted
            .symbolic_dimension(DomainRole::Parallel, symbol("n"))
            .expect("a value available before device work may size a domain");
    }

    /// An extent may only name a symbol its own region's environment declares.
    ///
    /// `ShapeEnv` already makes free and ambiguous bindings impossible inside
    /// one environment. What is left for the index layer is that a region has
    /// exactly one environment and resolves every symbol there.
    #[test]
    fn a_symbol_outside_this_regions_environment_is_undeclared_here() {
        let mut foreign = IndexRegionBuilder::new_with_shape_environment(
            registry(),
            environment(EXTENT_PHASE_CEILING, &[]),
        )
        .unwrap();
        assert_eq!(
            foreign.symbolic_dimension(DomainRole::Parallel, symbol("elsewhere")),
            Err(SymbolicExtentError::Source(
                ExtentSourceError::UndeclaredSymbol {
                    symbol: symbol("elsewhere"),
                }
            )),
        );

        // A region with no environment declares nothing at all, so every symbol
        // is undeclared rather than silently admitted.
        let mut unbound = IndexRegionBuilder::new(registry()).unwrap();
        assert_eq!(
            unbound.symbolic_dimension(DomainRole::Parallel, symbol("n")),
            Err(SymbolicExtentError::Source(
                ExtentSourceError::UndeclaredSymbol {
                    symbol: symbol("n"),
                }
            )),
        );
    }

    /// A determined symbolic extent proves exactly what a literal proves.
    ///
    /// The accepted rule is that inference succeeds "only when the available
    /// semantic constraints determine exactly one nonnegative extent". When
    /// they do, the write-ownership argument that needs an exact extent
    /// succeeds; the neighbour constrains the same symbol to a range instead,
    /// and the same argument fails.
    #[test]
    fn a_determined_symbolic_extent_proves_what_a_literal_proves() {
        fn write_over_symbolic_domain(
            environment: Arc<ShapeEnv>,
        ) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
            let mut builder =
                IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap();
            let input = builder
                .tensor(TensorRole::Input, value_type(), Shape::from_dims([4]))
                .unwrap();
            let output = builder
                .tensor(TensorRole::Output, value_type(), Shape::from_dims([4]))
                .unwrap();
            let dimension = builder
                .symbolic_dimension(DomainRole::Parallel, symbol("n"))
                .expect("admissible in both fixtures");
            let coordinate = builder.dimension_expr(dimension).unwrap();
            let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
            let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
            builder.output(write, value).unwrap();
            builder.build()
        }

        let determined = environment(
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::equal(term("n"), ExtentTerm::Constant(4))],
        );
        write_over_symbolic_domain(determined)
            .expect("`n == 4` covers a 4-element output exactly once");

        let merely_bounded = environment(
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(term("n"), 1, 4).unwrap()],
        );
        let error = write_over_symbolic_domain(merely_bounded).unwrap_err();
        assert!(
            reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::WriteOwnershipNotProven { .. }
            )),
            "an extent that may be below 4 does not cover a 4-element output: {error}",
        );
    }

    /// A region's identity names the environment its symbols resolve in.
    ///
    /// Two regions spelled identically over differently constrained
    /// environments are different programs. Without the environment's identity
    /// folded in, their canonical bytes would be equal and a cache would serve
    /// one for the other.
    #[test]
    fn region_identity_names_the_environment_its_symbols_resolve_in() {
        let tight = environment(
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(term("n"), 1, 4).unwrap()],
        );
        let divisible = environment(
            EXTENT_PHASE_CEILING,
            &[
                ExtentRelation::interval(term("n"), 1, 4).unwrap(),
                ExtentRelation::divisible(term("n"), NonZeroU64::new(2).unwrap()),
            ],
        );

        let first = region(Some(Arc::clone(&tight)), 4).unwrap();
        let repeated = region(Some(tight), 4).unwrap();
        let other = region(Some(divisible), 4).unwrap();

        assert_eq!(
            first.canonical_identity(),
            repeated.canonical_identity(),
            "one environment and one structure name one region",
        );
        assert_ne!(
            first.canonical_identity(),
            other.canonical_identity(),
            "a differently constrained environment is a different program",
        );
    }

    /// Every dimension is visible through one total view.
    ///
    /// **The successor to `static_extent_is_absent_exactly_for_a_symbolic_dimension`.**
    /// That test asserted a *rule about a pair of accessors* — that exactly one
    /// of `static_extent()` and the symbol accessor answered `Some` — which was
    /// unenforceable: a third source kind would have made both `None` and every
    /// consumer reading "not static, therefore symbolic" would have been
    /// silently wrong with no test failing anywhere near it.
    ///
    /// This asserts the property that replaced the rule. The match below is
    /// exhaustive over [`SourcedExtent`] with no wildcard arm, so a new source
    /// kind is a build error here and at every other consumer, and both arms are
    /// taken — by a symbolically sized region and a statically sized one — so
    /// neither is merely compiled.
    #[test]
    fn every_dimension_is_visible_through_one_total_extent_view() {
        /// Partitions a region's dimensions through the one total view.
        fn partition(region: &VerifiedIndexRegion) -> (Vec<Extent>, Vec<ShapeSymbol>) {
            let (mut literals, mut symbols) = (Vec::new(), Vec::new());
            for dimension in region.dimensions() {
                match dimension.extent() {
                    SourcedExtent::Static(extent) => literals.push(*extent),
                    SourcedExtent::Symbol(named) => symbols.push(named.clone()),
                }
            }
            assert_eq!(
                literals.len() + symbols.len(),
                region.dimensions().len(),
                "the view is total: every dimension took exactly one arm",
            );
            (literals, symbols)
        }

        let symbolic = region(
            Some(environment(
                EXTENT_PHASE_CEILING,
                &[ExtentRelation::interval(term("n"), 1, 4).unwrap()],
            )),
            4,
        )
        .unwrap();
        assert_eq!(
            partition(&symbolic),
            (Vec::new(), vec![symbol("n")]),
            "the symbolic dimension exposes the symbol it was sourced from",
        );
        assert!(
            symbolic.extent_sources().is_some(),
            "a region with a symbolic extent retains the environment resolving it",
        );

        let static_domain = read_from_symbolic_axis(environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::interval(term("m"), 8, 16).unwrap()],
        ))
        .expect("a domain of 4 indexes an axis of at least 8");
        assert_eq!(
            partition(&static_domain),
            (vec![Extent::new(4)], Vec::new()),
            "a literal dimension answers the same view with the extent it was written with",
        );
    }

    /// Copies one element of a symbolic-boundary input into a static output.
    ///
    /// The read is what the boundary work adds: the coordinate is bounded by a
    /// *static* domain and the axis it indexes is symbolic, so whether it is in
    /// bounds is a question about the environment's bound on that axis and
    /// nothing else. The output stays static and rank-zero so the write proves
    /// itself without involving the boundary under test.
    fn read_from_symbolic_axis(
        environment: Arc<ShapeEnv>,
    ) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
        let mut builder =
            IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap();
        let input = builder
            .sourced_tensor(
                TensorRole::Input,
                value_type(),
                vec![SourcedExtent::Symbol(symbol("m"))],
            )
            .expect("the source is admissible in these fixtures");
        let output = builder
            .tensor(TensorRole::Output, value_type(), Shape::from_dims([]))
            .unwrap();
        let reduced = builder
            .dimension(DomainRole::Reduction, Extent::new(4))
            .unwrap();
        let coordinate = builder.dimension_expr(reduced).unwrap();
        let contributor = builder.read(input, &[reduced], &[coordinate]).unwrap();
        let initial = builder
            .apply(zero_key(), ScalarAttributes::empty(), &[])
            .unwrap()
            .get(0)
            .unwrap();
        let total = builder
            .reduce(&[reduced], &[initial], &[contributor], |body| {
                let state = body.state(0).expect("one accumulator");
                let contributed = body.contributor(0).expect("one contributor");
                let stepped =
                    body.apply(step_key(), ScalarAttributes::empty(), &[state, contributed])?;
                let stepped = stepped.get(0).expect("one result");
                body.yield_values(&[stepped])
            })
            .unwrap()
            .get(0)
            .unwrap();
        let write = builder.write(output, &[], &[]).unwrap();
        builder.output(write, total).unwrap();
        builder.build()
    }

    /// Every boundary is visible through one total view.
    ///
    /// The boundary counterpart of
    /// `every_dimension_is_visible_through_one_total_extent_view`, and it
    /// replaced the same defect: a `static_shape()` beside a symbol accessor
    /// made "wholly literal" and "symbolic" two independent answers whose
    /// complementarity only a test held. [`SourcedShape`]'s total views exercise
    /// both representations through the one region under test.
    ///
    /// The last assertion is the normalization invariant, and it is what keeps
    /// [`SourcedShape::as_static`] a fact about the boundary rather than about
    /// which constructor authored it: a boundary written through the *sourced*
    /// path whose extents all turned out to be literals has the one static
    /// representation.
    #[test]
    fn every_boundary_is_visible_through_one_total_shape_view() {
        let region = read_from_symbolic_axis(environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::interval(term("m"), 8, 16).unwrap()],
        ))
        .expect("a domain of 4 indexes an axis of at least 8");

        let boundaries: Vec<_> = region
            .tensors()
            .map(|tensor| {
                let extents: Vec<_> = tensor.shape().extents().collect();
                let all_static = extents.iter().all(|extent| extent.as_static().is_some());
                let sourced = extents
                    .iter()
                    .filter_map(|extent| extent.symbol().cloned())
                    .collect::<Vec<_>>();
                assert_eq!(tensor.shape().as_static().is_some(), all_static);
                (tensor.role(), sourced)
            })
            .collect();
        assert!(
            boundaries
                .iter()
                .any(|(role, symbols)| *role == TensorRole::Input && symbols == &[symbol("m")]),
            "the symbolic input exposes the symbol it was sourced from: {boundaries:?}",
        );
        assert!(
            boundaries
                .iter()
                .any(|(role, symbols)| *role == TensorRole::Output && symbols.is_empty()),
            "the literal output takes the static arm of the same view: {boundaries:?}",
        );

        // Authored through the sourced path, but every extent is a literal, so
        // it normalizes to the same boundary a static caller would have written.
        let normalized = SourcedShape::sourced(vec![
            SourcedExtent::Static(Extent::new(2)),
            SourcedExtent::Static(Extent::new(3)),
        ])
        .expect("rank two is within the governed shape bound");
        assert_eq!(
            normalized.as_static(),
            Some(&Shape::from_dims([2, 3])),
            "an all-literal sourced boundary is a static shape, not a second spelling of one",
        );
    }

    /// Two extents can be proved equal with no value known for either.
    ///
    /// The mechanism the dynamically shaped case rests on, isolated from the
    /// region verifier. Neither symbol is determined here — the environment
    /// pins no constant at all — so a predicate that compared resolved values
    /// would answer "not proved", and the equality class is the only thing that
    /// can answer at all. The neighbour drops the one relation.
    #[test]
    fn two_undetermined_symbols_are_proved_equal_by_their_equality_class() {
        let related = ExtentSources::new(environment_over(
            EXTENT_PHASE_CEILING,
            &["m", "n"],
            &[ExtentRelation::equal(term("m"), term("n"))],
        ));
        let m = SourcedExtent::Symbol(symbol("m"));
        let n = SourcedExtent::Symbol(symbol("n"));

        assert_eq!(
            (related.determined(&m), related.determined(&n)),
            (None, None),
            "neither symbol has a value, so no comparison of values could decide this",
        );
        assert!(
            related.proves_equal(&m, &n),
            "`m == n` makes them one extent in every model",
        );

        let unrelated = ExtentSources::new(environment_over(
            EXTENT_PHASE_CEILING,
            &["m", "n"],
            &[
                ExtentRelation::interval(term("m"), 1, 4).unwrap(),
                ExtentRelation::interval(term("n"), 1, 4).unwrap(),
            ],
        ));
        assert!(
            !unrelated.proves_equal(&m, &n),
            "sharing an interval is not being one extent: the answer is not-proved, \
             and a caller may not read it as proved-different either",
        );
    }

    /// A symbolic tensor boundary is proved in bounds from the environment.
    ///
    /// The boundary counterpart of the domain case: nothing here knows how long
    /// the input axis is, and the read is admitted because the constraint
    /// environment bounds that axis *below* every coordinate the static domain
    /// can produce. The rejecting neighbour differs only in that bound, so the
    /// acceptance is evidence about the environment rather than about the
    /// region's shape.
    #[test]
    fn a_symbolic_boundary_axis_is_proved_in_bounds_by_its_environment_floor() {
        let roomy = read_from_symbolic_axis(environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::interval(term("m"), 8, 16).unwrap()],
        ))
        .expect("`m >= 8` admits every coordinate a 4-point domain produces");
        // Interval, not enumeration: the axis length is still unknown, so
        // nothing could have been walked. The symbolic axis is what the proof
        // read, so the facts name the environment.
        assert!(
            roomy.accesses().any(|access| access.bounds_proof()
                == Some(BoundsProofView::Interval {
                    facts: IndexDomainFactSource::ShapeEnvironment,
                })),
            "the read into a symbolic axis is proved by the environment's interval",
        );

        // One admissible extent below the domain leaves the upper-bound atom unresolved.
        let too_short = read_from_symbolic_axis(environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::interval(term("m"), 3, 16).unwrap()],
        ))
        .expect("an unknown read bound remains a semantic obligation");
        let unknown = too_short
            .unknown_index_domain_predicates()
            .collect::<Vec<_>>();
        assert_eq!(unknown.len(), 1);
        assert!(matches!(
            unknown[0].predicate(),
            IndexDomainPredicate::LessThanExtent { .. }
        ));
        assert_eq!(
            unknown[0].reason(),
            IndexDomainUnknownReason::InsufficientFacts
        );
    }

    /// A boundary source arriving after the ceiling is refused where it is written.
    ///
    /// For a *boundary* this is the quoted clause rather than the inference the
    /// domain case rests on: an output boundary's extent is an "initial output
    /// shape", and the accepted contract requires every one of those to be
    /// "evaluable on the host before any device work begins".
    #[test]
    fn a_boundary_source_after_the_phase_ceiling_is_refused_where_it_is_written() {
        for phase in [
            AvailabilityPhase::PreparedKernelPreflight,
            AvailabilityPhase::LaunchPreflight,
        ] {
            let mut builder =
                IndexRegionBuilder::new_with_shape_environment(registry(), environment(phase, &[]))
                    .unwrap();
            assert_eq!(
                builder.sourced_tensor(
                    TensorRole::Output,
                    value_type(),
                    vec![SourcedExtent::Symbol(symbol("n"))],
                ),
                Err(SymbolicExtentError::Source(
                    ExtentSourceError::SourceTooLate {
                        symbol: symbol("n"),
                        available: phase,
                        ceiling: EXTENT_PHASE_CEILING,
                    }
                )),
            );
        }

        // The same source one phase earlier is admitted, so the refusals above
        // are about when the value arrives rather than about the source class.
        let mut admitted = IndexRegionBuilder::new_with_shape_environment(
            registry(),
            environment(EXTENT_PHASE_CEILING, &[]),
        )
        .unwrap();
        admitted
            .sourced_tensor(
                TensorRole::Output,
                value_type(),
                vec![SourcedExtent::Symbol(symbol("n"))],
            )
            .expect("a value available before device work may size an output");
    }

    /// A dynamically shaped write is owned exactly when the environment proves
    /// the domain and boundary extents equal.
    ///
    /// This is what a caller-sized program needs. `write_is_permutation` used to
    /// compare a determined extent against a literal; the symbolic form of that
    /// comparison is symbol equality, and the constraint environment decides it
    /// from the equality classes alone. Both fixtures below prove their bounds
    /// by interval, so the pair turns on ownership and nothing else: the
    /// accepted one asserts `m == n` and the rejected one bounds `m` to a range
    /// that contains `n`'s value without ever saying they are the same extent.
    #[test]
    fn a_dynamically_shaped_write_is_owned_when_the_environment_proves_the_extents_equal() {
        fn write_to_symbolic_boundary(
            environment: Arc<ShapeEnv>,
        ) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
            let mut builder =
                IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap();
            let input = builder
                .tensor(TensorRole::Input, value_type(), Shape::from_dims([4]))
                .unwrap();
            let output = builder
                .sourced_tensor(
                    TensorRole::Output,
                    value_type(),
                    vec![SourcedExtent::Symbol(symbol("m"))],
                )
                .expect("admissible in both fixtures");
            let dimension = builder
                .symbolic_dimension(DomainRole::Parallel, symbol("n"))
                .expect("admissible in both fixtures");
            let coordinate = builder.dimension_expr(dimension).unwrap();
            let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
            let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
            builder.output(write, value).unwrap();
            builder.build()
        }

        let same_extent = write_to_symbolic_boundary(environment_over(
            EXTENT_PHASE_CEILING,
            &["m", "n"],
            &[
                ExtentRelation::equal(term("n"), ExtentTerm::Constant(4)),
                ExtentRelation::equal(term("m"), term("n")),
            ],
        ))
        .expect("an output sized `m == n` is covered exactly by a domain sized `n`");
        assert!(
            same_extent
                .accesses()
                .any(|access| access.write_ownership_proof()
                    == Some(WriteOwnershipProofView::CoordinatePermutation)),
            "ownership is the permutation argument, discharged through the environment",
        );

        // `m` is bounded tightly enough for the coordinate to be in bounds, and
        // its range even contains `n`'s value — but the environment never says
        // the two are one extent, so the write is not proved to cover it.
        let merely_overlapping = write_to_symbolic_boundary(environment_over(
            EXTENT_PHASE_CEILING,
            &["m", "n"],
            &[
                ExtentRelation::equal(term("n"), ExtentTerm::Constant(4)),
                ExtentRelation::interval(term("m"), 4, 5).unwrap(),
            ],
        ))
        .unwrap_err();
        assert!(
            reports(&merely_overlapping, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::WriteOwnershipNotProven { .. }
            )),
            "an output that may hold 5 elements is not covered by 4 writes: {merely_overlapping}",
        );
        assert!(
            !reports(&merely_overlapping, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::ProofResourceLimit { .. }
            )),
            "this is a refusal, not an enumeration that ran out of budget: {merely_overlapping}",
        );
    }

    /// A boundary's identity names the symbol it was written with, not the
    /// value its environment resolves that symbol to.
    ///
    /// The accepted contract keeps `graph identity`, `interface identity`, and
    /// `specialized identity` distinguishable. An output written as `[m]` in an
    /// environment that happens to pin `m == 4` is a program that adapts to its
    /// caller; one written as `[4]` is a program that does not. Folding the
    /// resolved value in here would collapse the first into the second, and a
    /// cache would then serve either for the other.
    #[test]
    fn a_boundary_identity_names_its_symbol_rather_than_a_resolved_value() {
        fn region(environment: Arc<ShapeEnv>, symbolic_output: bool) -> VerifiedIndexRegion {
            let mut builder =
                IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap();
            let input = builder
                .tensor(TensorRole::Input, value_type(), Shape::from_dims([4]))
                .unwrap();
            let output = if symbolic_output {
                builder
                    .sourced_tensor(
                        TensorRole::Output,
                        value_type(),
                        vec![SourcedExtent::Symbol(symbol("m"))],
                    )
                    .expect("`m` is declared and available in time")
            } else {
                builder
                    .tensor(TensorRole::Output, value_type(), Shape::from_dims([4]))
                    .unwrap()
            };
            let dimension = builder
                .dimension(DomainRole::Parallel, Extent::new(4))
                .unwrap();
            let coordinate = builder.dimension_expr(dimension).unwrap();
            let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
            let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
            builder.output(write, value).unwrap();
            builder
                .build()
                .expect("both spellings verify under `m == 4`")
        }

        let pinned = environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::equal(term("m"), ExtentTerm::Constant(4))],
        );
        let symbolic = region(Arc::clone(&pinned), true);
        let literal = region(Arc::clone(&pinned), false);

        assert_eq!(
            symbolic.canonical_identity(),
            region(pinned, true).canonical_identity(),
            "one environment and one structure name one region",
        );
        assert_ne!(
            symbolic.canonical_identity(),
            literal.canonical_identity(),
            "a boundary sized by a symbol is a different program from one sized by that symbol's value",
        );
        assert!(
            literal
                .tensors()
                .all(|tensor| tensor.shape().as_static().is_some()),
            "the literal spelling stays wholly static even though `m` resolves to it",
        );
    }

    /// A wholly undetermined `[n] -> [n]` copy verifies, and its evidence names
    /// the argument that actually proved it.
    ///
    /// **The successor to `a_wholly_undetermined_dynamic_copy_is_refused_rather_than_approximated`.**
    /// That case measured this profile's boundary: ownership was proved — the
    /// environment decides `m == n` from the equality class — while bounds were
    /// not, and the reason was structural rather than a gap in the environment.
    /// Proving `i < m` from `0 <= i < n` and `m == n` is an equality-class
    /// argument, not an interval comparison: `n`'s interval is the whole extent
    /// domain, so `max(i)` is nowhere below `m`'s floor. The argument was sound
    /// and cheap, and what blocked it was that [`BoundsProofView`] had no name
    /// for it — `VacuousEmptyDomain`, `Interval`, and `Exhaustive` would each
    /// have misdescribed how such an access was proved, and an access whose
    /// retained evidence names the wrong proof is worse than one that is
    /// refused.
    ///
    /// `name-the-proved-extent-equality-bounds-proof` added
    /// [`BoundsProofView::ProvedExtentEquality`], so the region now verifies.
    /// This asserts the part that made naming it necessary: the *read* records
    /// the equality argument rather than an interval or an enumeration, and
    /// nothing was enumerated at all.
    #[test]
    fn a_wholly_undetermined_dynamic_copy_verifies_by_proved_extent_equality() {
        let region = undetermined_dynamic_copy(environment_over(
            EXTENT_PHASE_CEILING,
            &["m", "n"],
            &[ExtentRelation::equal(term("m"), term("n"))],
        ))
        .expect("`i` is a dimension sized `n`, and the environment proves `m == n`");

        assert!(
            region.accesses().all(|access| {
                access.bounds_proof()
                    == Some(BoundsProofView::ProvedExtentEquality {
                        facts: IndexDomainFactSource::ShapeEnvironment,
                    })
            }),
            "neither interval propagation nor an enumeration closed this; the equality did",
        );
        assert!(
            region
                .accesses()
                .filter(|access| access.mode() == AccessMode::Write)
                .all(|access| access.write_ownership_proof()
                    == Some(WriteOwnershipProofView::CoordinatePermutation)),
            "ownership stays the permutation argument, which was already discharged",
        );
    }

    /// The neighbour whose environment never proves write ownership.
    #[test]
    fn an_undetermined_copy_whose_extents_are_never_proved_equal_is_still_refused() {
        let error =
            undetermined_dynamic_copy(environment_over(EXTENT_PHASE_CEILING, &["m", "n"], &[]))
                .unwrap_err();
        assert!(
            reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::WriteOwnershipNotProven { .. }
            )),
            "residual bounds cannot substitute for total, injective write ownership: {error}",
        );
        assert!(
            !reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::ProofResourceLimit { .. }
            )),
            "nothing was enumerated, so nothing ran out of budget: {error}",
        );
    }

    /// Builds the caller-sized `[m] -> [m]` copy over a domain sized `n`.
    ///
    /// Nothing in either fixture determines a value for `m` or `n`; the two
    /// differ only in whether the environment proves them one extent.
    fn undetermined_dynamic_copy(
        environment: Arc<ShapeEnv>,
    ) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
        let mut builder =
            IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap();
        let boundary = |builder: &mut IndexRegionBuilder, role| {
            builder
                .sourced_tensor(role, value_type(), vec![SourcedExtent::Symbol(symbol("m"))])
                .expect("`m` is declared and available in time")
        };
        let input = boundary(&mut builder, TensorRole::Input);
        let output = boundary(&mut builder, TensorRole::Output);
        let dimension = builder
            .symbolic_dimension(DomainRole::Parallel, symbol("n"))
            .expect("`n` is declared and available in time");
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        builder.build()
    }

    /// One divisor vocabulary carries both a literal and a declared symbol.
    ///
    /// The argument position is the same in both calls and there is no second
    /// divisor type to choose between, which is what stops a frontend from
    /// having two spellings for the same fact and the encoder from having two
    /// forms to fold into identity.
    #[test]
    fn one_divisor_vocabulary_carries_a_literal_and_a_symbol() {
        divided_copy(None, SourcedExtent::Static(Extent::new(2)))
            .expect("a literal divisor needs no environment")
            .expect("`i / 2` over a domain of eight stays inside an eight-element axis");

        divided_copy(
            Some(environment_over(
                EXTENT_PHASE_CEILING,
                &["d"],
                &[ExtentRelation::interval(term("d"), 2, 2).unwrap()],
            )),
            SourcedExtent::Symbol(symbol("d")),
        )
        .expect("`d` is declared, in time, and proved positive")
        .expect("a divisor the environment pins to two proves the same bound the literal did");
    }

    /// A symbolic divisor's positivity comes from a constraint, never a guard.
    ///
    /// **Tested from both sides, because the discriminator is the whole point.**
    /// The same relation — `d` in `[1, 64]` — admits the divisor when it is
    /// *required* and refuses it when it is merely *guarded*. A builder that
    /// folded guards in would pass the first half and fail the second, and that
    /// is exactly the defect: a guard's failure selects another plan, so an
    /// expression whose definedness rested on one could be left meaningless by
    /// a later planning choice rather than rejected here.
    #[test]
    fn a_symbolic_divisor_is_proved_positive_by_a_constraint_and_not_by_a_guard() {
        let at_least_one = ExtentRelation::interval(term("d"), 1, 64).unwrap();

        divided_copy(
            Some(environment_over(
                EXTENT_PHASE_CEILING,
                &["d"],
                std::slice::from_ref(&at_least_one),
            )),
            SourcedExtent::Symbol(symbol("d")),
        )
        .expect("a required extent of at least one proves the divisor positive")
        .expect("the read's residual bound is an obligation, not a verification failure");

        assert_eq!(
            divided_copy(
                Some(guarded_environment(&["d"], &[at_least_one])),
                SourcedExtent::Symbol(symbol("d")),
            )
            .unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::DivisorNotProvedPositive {
                symbol: symbol("d"),
            }),
        );

        // An environment that says nothing about `d` proves nothing about it.
        assert_eq!(
            divided_copy(
                Some(environment_over(EXTENT_PHASE_CEILING, &["d"], &[])),
                SourcedExtent::Symbol(symbol("d")),
            )
            .unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::DivisorNotProvedPositive {
                symbol: symbol("d"),
            }),
        );
    }

    /// A divisor's source is refused by the same authorities an extent's is.
    ///
    /// Three different refusals, each reported under its own authority's name:
    /// the environment does not declare the symbol, does not supply it before
    /// device work begins, and — for a literal — the index layer itself refuses
    /// a zero divisor as a structural rule rather than as a source fact.
    #[test]
    fn a_divisor_source_is_refused_under_the_authority_that_refused_it() {
        assert_eq!(
            divided_copy(None, SourcedExtent::Symbol(symbol("d"))).unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol("d"),
            }),
            "a region with no environment declares no symbol, so none is admissible",
        );

        assert_eq!(
            divided_copy(
                Some(environment_over(
                    EXTENT_PHASE_CEILING,
                    &["d"],
                    &[ExtentRelation::interval(term("d"), 1, 64).unwrap()],
                )),
                SourcedExtent::Symbol(symbol("elsewhere")),
            )
            .unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol("elsewhere"),
            }),
        );

        for phase in [
            AvailabilityPhase::PreparedKernelPreflight,
            AvailabilityPhase::LaunchPreflight,
        ] {
            let mut draft = ShapeEnvBuilder::new();
            let declared = symbol("d");
            draft.declare(declared.clone()).unwrap();
            draft
                .bind(&declared, parameter_binding("d", phase))
                .unwrap();
            draft
                .require(SemanticInputConstraint::new(
                    ExtentRelation::interval(term("d"), 1, 64).unwrap(),
                    FactProvenance::FrontendRequired,
                ))
                .unwrap();
            assert_eq!(
                divided_copy(
                    Some(Arc::new(draft.build().unwrap())),
                    SourcedExtent::Symbol(symbol("d")),
                )
                .unwrap_err(),
                SymbolicExtentError::Source(ExtentSourceError::SourceTooLate {
                    symbol: symbol("d"),
                    available: phase,
                    ceiling: EXTENT_PHASE_CEILING,
                }),
                "a divisor obeys the same ceiling a domain extent does",
            );
        }

        assert_eq!(
            divided_copy(None, SourcedExtent::Static(Extent::new(0))).unwrap_err(),
            SymbolicExtentError::Structural(IndexBuildError::NonPositiveDivisor),
            "a zero literal is the index layer's own rule, not the environment's",
        );
    }

    /// A semi-affine expression is classed as one, and its residual bound is
    /// retained rather than approximated.
    ///
    /// The class is a fact about the expression's *form*: `d` is pinned to two
    /// in the second fixture, so the quotient's range is exactly what the
    /// literal spelling produces — and the class stays [`IndexExprClass::SemiAffine`]
    /// anyway, because the region's canonical bytes name the symbol and another
    /// environment could bind it differently.
    ///
    /// The first fixture is the declining half. With `d` bounded but not fixed
    /// there is no divisor to divide by, so interval propagation states nothing
    /// and no enumeration exists to fall back to. The obligation is retained as
    /// [`IndexDomainUnknownReason::InsufficientFacts`] — and explicitly *not* as
    /// a proof-resource limit, which is what a budget charged for a walk that
    /// could never run would have produced.
    #[test]
    fn a_semi_affine_quotient_is_classed_and_its_residual_bound_is_retained() {
        let bounded = divided_copy(
            Some(environment_over(
                EXTENT_PHASE_CEILING,
                &["d"],
                &[ExtentRelation::interval(term("d"), 1, 64).unwrap()],
            )),
            SourcedExtent::Symbol(symbol("d")),
        )
        .expect("`d >= 1` proves the divisor positive")
        .expect("an unproved read bound is an obligation, not a verification failure");

        // Both halves of the read's bound are open, and that is the honest
        // answer: with no value for `d` the quotient's range states neither a
        // floor nor a ceiling, so neither predicate was decided.
        let unknown: Vec<_> = bounded.unknown_index_domain_predicates().collect();
        assert_eq!(unknown.len(), 2);
        assert!(
            unknown.iter().any(|record| matches!(
                record.predicate(),
                IndexDomainPredicate::NonNegative { .. }
            ))
        );
        assert!(unknown.iter().any(|record| matches!(
            record.predicate(),
            IndexDomainPredicate::LessThanExtent { .. }
        )));
        assert!(
            unknown
                .iter()
                .all(|record| record.reason() == IndexDomainUnknownReason::InsufficientFacts),
            "no enumeration existed to exhaust, so this is missing facts and not a budget",
        );
        assert!(
            bounded
                .accesses()
                .filter(|access| access.mode() == AccessMode::Read)
                .all(|access| access.bounds_proof().is_none()),
            "nothing proved the read in bounds, and no proof kind claims otherwise",
        );

        let pinned = divided_copy(
            Some(environment_over(
                EXTENT_PHASE_CEILING,
                &["d"],
                &[ExtentRelation::interval(term("d"), 2, 2).unwrap()],
            )),
            SourcedExtent::Symbol(symbol("d")),
        )
        .expect("`d` is proved positive")
        .expect("a pinned divisor closes the read bound by interval");
        assert_eq!(pinned.unknown_index_domain_predicates().count(), 0);

        for region in [&bounded, &pinned] {
            let classes: Vec<_> = region
                .index_expressions()
                .map(|expression| (expression.class(), expression.view()))
                .collect();
            assert!(
                classes
                    .iter()
                    .any(|(class, view)| *class == IndexExprClass::SemiAffine
                        && matches!(view, IndexExprView::FloorDiv { divisor, .. }
                        if divisor.symbol() == Some(&symbol("d")))),
                "the quotient by a named symbol is semi-affine: {classes:?}",
            );
        }

        // The literal spelling of the same arithmetic stays quasi-affine, so the
        // class tracks the form rather than the environment's resolution of it.
        let literal = divided_copy(None, SourcedExtent::Static(Extent::new(2)))
            .expect("a literal divisor needs no environment")
            .expect("`i / 2` is in bounds by interval");
        assert!(
            literal
                .index_expressions()
                .all(|expression| expression.class() != IndexExprClass::SemiAffine),
            "no expression divided by a symbol, so none is semi-affine",
        );
    }

    /// Copies `input[addend + coefficient * i]` into `output[i]` over eight points.
    ///
    /// The coefficient half's counterpart of [`divided_copy`], built the same
    /// way and for the same reason: the only symbolic things in the region are
    /// the two scalars under test, so what the region can prove about the read
    /// turns on the environment's facts about them and on nothing else. The
    /// write is a plain permutation of the static parallel dimension, so
    /// ownership is discharged whatever the scalars do.
    ///
    /// The input axis is 64 against a domain of 8 so that the *literal*
    /// neighbours every test below compares against — `1 * i`, `2 * i`, `4 * i`
    /// — are provably in bounds. A shorter axis would refute them, and the
    /// contrast under test would become "declined versus refuted" rather than
    /// "declined versus proved by interval".
    fn scaled_copy(
        environment: Option<Arc<ShapeEnv>>,
        addend: SourcedIndexInteger,
        coefficient: SourcedIndexInteger,
    ) -> Result<Result<VerifiedIndexRegion, IndexRegionBuildError>, SymbolicExtentError> {
        let mut builder = match environment {
            Some(environment) => {
                IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap()
            }
            None => IndexRegionBuilder::new(registry()).unwrap(),
        };
        let input = builder
            .tensor(TensorRole::Input, value_type(), Shape::from_dims([64]))
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type(), Shape::from_dims([8]))
            .unwrap();
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(8))
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let scaled = builder.sourced_linear_combination(addend, &[(coefficient, coordinate)])?;
        let value = builder.read(input, &[dimension], &[scaled]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        Ok(builder.build())
    }

    /// A symbolic coefficient and a symbolic addend are both expressible, and
    /// the expression that carries them is semi-affine.
    ///
    /// This is what the ticket is for. ADR 0046 admits "symbolic coefficients
    /// **or** proven-positive symbolic divisors" and only the divisor was
    /// implemented; `i * B + S` is the other half, written through the same
    /// sourced vocabulary the divisor uses.
    ///
    /// The class assertion is about the expression's *form*, exactly as the
    /// divisor's is: `B` is pinned to two here, so the region could have been
    /// spelled with literals, and it stays [`IndexExprClass::SemiAffine`]
    /// anyway because the canonical bytes name the symbols and another
    /// environment could bind them differently.
    #[test]
    fn a_symbolic_coefficient_and_addend_are_expressible_and_classed_semi_affine() {
        let region = scaled_copy(
            Some(environment_over(
                EXTENT_PHASE_CEILING,
                &["b", "s"],
                &[
                    ExtentRelation::interval(term("b"), 2, 2).unwrap(),
                    ExtentRelation::interval(term("s"), 0, 0).unwrap(),
                ],
            )),
            SourcedIndexInteger::Symbol(symbol("s")),
            SourcedIndexInteger::Symbol(symbol("b")),
        )
        .expect("both symbols are declared and available in time")
        .expect("`b == 2` and `s == 0` bound `s + b * i` below a 64-element axis");

        let combination = region
            .index_expressions()
            .find(|expression| matches!(expression.view(), IndexExprView::LinearCombination { .. }))
            .expect("the coordinate is a linear combination");
        assert_eq!(
            combination.class(),
            IndexExprClass::SemiAffine,
            "a symbol the region names but does not fix makes the form semi-affine",
        );

        let IndexExprView::LinearCombination { constant, terms } = combination.view() else {
            unreachable!("matched above")
        };
        // The additive slot stays exact; the symbolic addend became a term.
        assert_eq!(constant.to_string(), "0");
        let coefficients: Vec<_> = terms.map(|term| term.coefficient().clone()).collect();
        assert!(
            coefficients.contains(&SourcedIndexInteger::Symbol(symbol("b"))),
            "the coefficient names `b`: {coefficients:?}",
        );
        assert!(
            coefficients.contains(&SourcedIndexInteger::Symbol(symbol("s"))),
            "the addend is carried as a term scaled by `s`: {coefficients:?}",
        );

        // The literal spelling of the same arithmetic is affine, so the class
        // tracks the form rather than the environment's resolution of it.
        let literal = scaled_copy(None, 0_i128.into(), 2_i128.into())
            .expect("literals need no environment")
            .expect("`2 * i` over eight points is in bounds by interval");
        assert!(
            literal
                .index_expressions()
                .all(|expression| expression.class() != IndexExprClass::SemiAffine),
            "no expression named a symbol, so none is semi-affine",
        );
    }

    /// A symbolic coefficient's source is refused under the authority that
    /// refused it, and positivity is never the question.
    ///
    /// Three refusals, each with its own typed cause: the region has no
    /// environment and so declares nothing; the environment declares other
    /// symbols but not this one; and the symbol's value arrives after the
    /// ceiling. All three come from [`ExtentSources::admit`] — the same path a
    /// domain extent, a boundary axis, and a divisor use.
    ///
    /// **`proves_positive` is deliberately not consulted, and the last case is
    /// what proves it.** An environment that says nothing at all about `b`
    /// refuses a *divisor* under
    /// [`ExtentSourceError::DivisorNotProvedPositive`], because `x floordiv 0`
    /// is undefined; the same environment admits `b` as a coefficient, because
    /// every magnitude it could take denotes a coordinate.
    #[test]
    fn a_symbolic_coefficient_source_is_refused_under_the_authority_that_refused_it() {
        assert_eq!(
            scaled_copy(
                None,
                0_i128.into(),
                SourcedIndexInteger::Symbol(symbol("b"))
            )
            .unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol("b"),
            }),
            "a region with no environment declares no symbol, so none is admissible",
        );

        assert_eq!(
            scaled_copy(
                Some(environment_over(EXTENT_PHASE_CEILING, &["b"], &[])),
                0_i128.into(),
                SourcedIndexInteger::Symbol(symbol("elsewhere")),
            )
            .unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol("elsewhere"),
            }),
            "a symbol from another environment is undeclared in this one",
        );

        for phase in [
            AvailabilityPhase::PreparedKernelPreflight,
            AvailabilityPhase::LaunchPreflight,
        ] {
            assert_eq!(
                scaled_copy(
                    Some(environment_over(phase, &["b"], &[])),
                    0_i128.into(),
                    SourcedIndexInteger::Symbol(symbol("b")),
                )
                .unwrap_err(),
                SymbolicExtentError::Source(ExtentSourceError::SourceTooLate {
                    symbol: symbol("b"),
                    available: phase,
                    ceiling: EXTENT_PHASE_CEILING,
                }),
                "a coefficient obeys the same ceiling a domain extent does",
            );
            // The addend reaches the same authority, so neither scalar position
            // is admitted by a path the other is not.
            assert_eq!(
                scaled_copy(
                    Some(environment_over(phase, &["b"], &[])),
                    SourcedIndexInteger::Symbol(symbol("b")),
                    1_i128.into(),
                )
                .unwrap_err(),
                SymbolicExtentError::Source(ExtentSourceError::SourceTooLate {
                    symbol: symbol("b"),
                    available: phase,
                    ceiling: EXTENT_PHASE_CEILING,
                }),
            );
        }

        // An environment that proves nothing about `b` still admits it as a
        // coefficient. The divisor neighbour below is the same environment and
        // the same symbol, refused — so the difference is the predicate and not
        // the environment.
        scaled_copy(
            Some(environment_over(EXTENT_PHASE_CEILING, &["b"], &[])),
            0_i128.into(),
            SourcedIndexInteger::Symbol(symbol("b")),
        )
        .expect("a coefficient is never required to be proved positive")
        .expect("its unproved read bound is an obligation");
        assert_eq!(
            divided_copy(
                Some(environment_over(EXTENT_PHASE_CEILING, &["d"], &[])),
                SourcedExtent::Symbol(symbol("d")),
            )
            .unwrap_err(),
            SymbolicExtentError::Source(ExtentSourceError::DivisorNotProvedPositive {
                symbol: symbol("d"),
            }),
            "the divisor's extra predicate is what a coefficient does not carry",
        );
    }

    /// A refused coefficient leaves the draft exactly as it was.
    ///
    /// The refusal happens before any operand is resolved, any expression is
    /// interned, or any constant is created, so a builder that survived one is
    /// byte-identical to a builder that never attempted it. Asserted over
    /// canonical identity rather than over an internal counter, because that is
    /// what a half-applied draft would actually corrupt.
    #[test]
    fn a_refused_coefficient_leaves_the_draft_unchanged() {
        fn region(attempt_refused: bool) -> VerifiedIndexRegion {
            let mut builder = IndexRegionBuilder::new_with_shape_environment(
                registry(),
                environment_over(EXTENT_PHASE_CEILING, &["b"], &[]),
            )
            .unwrap();
            let input = builder
                .tensor(TensorRole::Input, value_type(), Shape::from_dims([8]))
                .unwrap();
            let output = builder
                .tensor(TensorRole::Output, value_type(), Shape::from_dims([8]))
                .unwrap();
            let dimension = builder
                .dimension(DomainRole::Parallel, Extent::new(8))
                .unwrap();
            let coordinate = builder.dimension_expr(dimension).unwrap();
            if attempt_refused {
                assert_eq!(
                    builder.sourced_linear_combination(
                        0_i128.into(),
                        &[(SourcedIndexInteger::Symbol(symbol("elsewhere")), coordinate)],
                    ),
                    Err(SymbolicExtentError::Source(
                        ExtentSourceError::UndeclaredSymbol {
                            symbol: symbol("elsewhere"),
                        }
                    )),
                );
            }
            let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
            let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
            builder.output(write, value).unwrap();
            builder.build().expect("a static copy verifies")
        }

        assert_eq!(
            region(true).canonical_identity(),
            region(false).canonical_identity(),
            "a refusal is not a mutation: the surviving draft names the same region",
        );
    }

    /// A symbolic coefficient is bounded from its declared extent, and the
    /// evidence says the environment is what bounded it.
    ///
    /// **The successor to `an_interval_over_a_symbolic_coefficient_declines_with_a_named_reason`.**
    /// That case asserted the deliberate decline
    /// [`admit-symbolic-index-expression-coefficients`] left in
    /// [`interval_linear`], on the ground that a bound read from the
    /// environment would make an expression's interval a function of the
    /// binding. The ground did not survive inspection: a `ShapeEnv` holds no
    /// values, a region folds its environment's *identity* into its own, and
    /// every neighbouring proof in this crate already reads that environment —
    /// a symbolic dimension's interval, a symbolic axis, a pinned divisor's
    /// quotient interval, and the enumeration's own divisors. The coefficient
    /// was the only exception, and it is not one now.
    ///
    /// [`admit-symbolic-index-expression-coefficients`]: super::super::IndexRegionBuilder
    ///
    /// Three fixtures, and the contrast between them is the whole point. `b`
    /// pinned to one proves the bound and records
    /// [`IndexDomainFactSource::ShapeEnvironment`]; the literal `1` proves the
    /// same bound and records [`IndexDomainFactSource::Program`], so a caller
    /// can tell which it got. `b` bounded nowhere above still leaves the upper
    /// bound open under [`IndexDomainUnknownReason::InsufficientFacts`] — and
    /// explicitly not [`IndexRegionDiagnostic::ProofResourceLimit`], because
    /// nothing could be enumerated either.
    #[test]
    fn a_symbolic_coefficient_is_bounded_from_its_declared_extent() {
        let pinned = environment_over(
            EXTENT_PHASE_CEILING,
            &["b"],
            &[ExtentRelation::interval(term("b"), 1, 1).unwrap()],
        );
        let symbolic = scaled_copy(
            Some(pinned),
            0_i128.into(),
            SourcedIndexInteger::Symbol(symbol("b")),
        )
        .expect("`b` is declared and available in time")
        .expect("`b == 1` bounds `b * i` below a 64-element axis");
        assert_eq!(symbolic.unknown_index_domain_predicates().count(), 0);
        assert!(
            symbolic
                .accesses()
                .filter(|access| access.mode() == AccessMode::Read)
                .all(|access| access.bounds_proof()
                    == Some(BoundsProofView::Interval {
                        facts: IndexDomainFactSource::ShapeEnvironment,
                    })),
            "the bound exists only because the environment declares `b`'s extent",
        );

        // The same arithmetic written as the literal the environment pins `b`
        // to. It proves the same bound and records the *strong* claim, which is
        // the distinction a caller needs and the reason the fact source is
        // retained rather than left to be re-derived.
        let literal = scaled_copy(None, 0_i128.into(), 1_i128.into())
            .expect("literals need no environment")
            .expect("`1 * i` over eight points is in bounds by interval");
        assert_eq!(literal.unknown_index_domain_predicates().count(), 0);
        assert!(
            literal
                .accesses()
                .filter(|access| access.mode() == AccessMode::Read)
                .all(|access| access.bounds_proof()
                    == Some(BoundsProofView::Interval {
                        facts: IndexDomainFactSource::Program,
                    })),
        );

        // An environment that bounds `b` nowhere above proves no upper bound,
        // and the obligation is retained rather than approximated. The *lower*
        // bound still closes, and that is not an accident of the fixture: `b`
        // resolves through a `ShapeSymbol`, which names an extent, so every
        // model assigns it at least zero.
        let unbounded = scaled_copy(
            Some(environment_over(EXTENT_PHASE_CEILING, &["b"], &[])),
            0_i128.into(),
            SourcedIndexInteger::Symbol(symbol("b")),
        )
        .expect("`b` is declared and available in time")
        .expect("an unproved read bound is an obligation, not a verification failure");
        let unknown: Vec<_> = unbounded.unknown_index_domain_predicates().collect();
        assert_eq!(unknown.len(), 1);
        assert!(matches!(
            unknown[0].predicate(),
            IndexDomainPredicate::LessThanExtent { .. }
        ));
        assert_eq!(
            unknown[0].reason(),
            IndexDomainUnknownReason::InsufficientFacts,
            "no enumeration existed to exhaust, so this is missing facts and not a budget",
        );
        assert!(
            unbounded
                .accesses()
                .filter(|access| access.mode() == AccessMode::Read)
                .all(|access| access.bounds_proof().is_none()),
            "one open atom leaves the access unproved, and no proof kind claims otherwise",
        );
    }

    /// Copies `input[(B * i) mod 2 + (B * i) / 2]` into `output[i]` over five
    /// points, against a three-element input axis.
    ///
    /// The shape that reaches the *enumeration* fallback with a symbolic
    /// coefficient inside it. A bare linear form never can: its propagated
    /// interval is exact, so an interval that fails to close there means a point
    /// that really is out of bounds. Splitting the scaled coordinate into a
    /// remainder and a quotient and adding them back makes propagation bound the
    /// two branches independently — `[0, 1] + [0, 2]` reaches `[0, 3]`, which
    /// does not close against an axis of three — while no visited point exceeds
    /// two.
    fn split_copy(
        environment: Arc<ShapeEnv>,
        coefficient: SourcedIndexInteger,
    ) -> Result<Result<VerifiedIndexRegion, IndexRegionBuildError>, SymbolicExtentError> {
        let mut builder =
            IndexRegionBuilder::new_with_shape_environment(registry(), environment).unwrap();
        let input = builder
            .tensor(TensorRole::Input, value_type(), Shape::from_dims([3]))
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, value_type(), Shape::from_dims([5]))
            .unwrap();
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(5))
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let scaled =
            builder.sourced_linear_combination(0_i128.into(), &[(coefficient, coordinate)])?;
        let two = SourcedExtent::Static(Extent::new(2));
        let remainder = builder.modulo(scaled, two.clone())?;
        let quotient = builder.floor_div(scaled, two)?;
        let split = builder.linear_combination(
            0_i128.into(),
            &[(1_i128.into(), remainder), (1_i128.into(), quotient)],
        )?;
        let value = builder.read(input, &[dimension], &[split]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
        Ok(builder.build())
    }

    /// A symbolic coefficient the environment pins is *enumerated* through the
    /// same query the divisor already used, so the two halves end on one rule.
    ///
    /// The walk needs the coefficient's value at a point and takes it from the
    /// environment exactly as `plan_scalars` takes a divisor's. The retained
    /// evidence is [`BoundsProofView::Exhaustive`] over
    /// [`IndexDomainFactSource::ShapeEnvironment`], which is the pairing this
    /// ticket's closing condition demands: no path derives a bound from the
    /// environment without its evidence saying so.
    ///
    /// **The write in the same region is the control.** Its coordinate is the
    /// bare dimension and its axis and domain are literals, so it records
    /// [`IndexDomainFactSource::Program`] — the axis therefore separates
    /// accesses *within* one region rather than only regions from one another.
    #[test]
    fn an_enumerated_symbolic_coefficient_resolves_as_a_symbolic_divisor_does() {
        let region = split_copy(
            environment_over(
                EXTENT_PHASE_CEILING,
                &["b"],
                &[ExtentRelation::interval(term("b"), 1, 1).unwrap()],
            ),
            SourcedIndexInteger::Symbol(symbol("b")),
        )
        .expect("`b` is declared and available in time")
        .expect("every visited point lands inside the three-element axis");

        assert_eq!(region.unknown_index_domain_predicates().count(), 0);
        assert_eq!(
            region
                .accesses()
                .find(|access| access.mode() == AccessMode::Read)
                .unwrap()
                .bounds_proof(),
            Some(BoundsProofView::Exhaustive {
                points: 5,
                facts: IndexDomainFactSource::ShapeEnvironment,
            }),
            "the walk resolved `b` from the environment, and the evidence says so",
        );
        assert_eq!(
            region
                .accesses()
                .find(|access| access.mode() == AccessMode::Write)
                .unwrap()
                .bounds_proof(),
            Some(BoundsProofView::Interval {
                facts: IndexDomainFactSource::Program,
            }),
            "the write names no symbol, so its own bound is the program's",
        );

        // The neighbour differs only in that nothing pins `b`. There is no value
        // to multiply by at a point, so no enumeration exists — and the
        // obligation is missing facts rather than a spent budget, exactly as an
        // undetermined divisor's is.
        let unpinned = split_copy(
            environment_over(
                EXTENT_PHASE_CEILING,
                &["b"],
                &[ExtentRelation::interval(term("b"), 1, 4).unwrap()],
            ),
            SourcedIndexInteger::Symbol(symbol("b")),
        )
        .expect("`b` is declared and available in time")
        .expect("an unproved read bound is an obligation, not a verification failure");
        let unknown: Vec<_> = unpinned.unknown_index_domain_predicates().collect();
        assert_eq!(unknown.len(), 1);
        assert!(matches!(
            unknown[0].predicate(),
            IndexDomainPredicate::LessThanExtent { .. }
        ));
        assert_eq!(
            unknown[0].reason(),
            IndexDomainUnknownReason::InsufficientFacts,
            "a coefficient nothing fixes leaves no walk to budget",
        );
    }

    /// A coefficient's identity names the symbol it was written with, not the
    /// value its environment resolves that symbol to.
    ///
    /// The coefficient counterpart of
    /// `a_boundary_identity_names_its_symbol_rather_than_a_resolved_value`, and
    /// the reason is the same accepted contract: `graph identity`, `interface
    /// identity`, and `specialized identity` stay distinguishable. A region
    /// scaled by `b` in an environment that happens to pin `b == 4` is a
    /// program that adapts to its caller; one scaled by the literal `4` is a
    /// program that does not. Folding the resolved value in would collapse the
    /// first into the second and a cache would then serve either for the other.
    ///
    /// This is also what makes the normalization decision observable. Declining
    /// to fold a symbolic coefficient is *why* these two identities differ; a
    /// builder that resolved `b` to `4` because the environment allowed it
    /// would make this assertion fail. **That the two spellings now prove the
    /// same bound is exactly the separation under test**: reading `b`'s
    /// declared extent inside a proof leaves the node naming `b`, so the
    /// identities stay apart while the analysis closes on both.
    #[test]
    fn a_coefficient_identity_names_its_symbol_rather_than_a_resolved_value() {
        let pinned = environment_over(
            EXTENT_PHASE_CEILING,
            &["b"],
            &[ExtentRelation::interval(term("b"), 4, 4).unwrap()],
        );
        let region = |coefficient: SourcedIndexInteger| {
            scaled_copy(Some(Arc::clone(&pinned)), 0_i128.into(), coefficient)
                .expect("`b` is declared and available in time")
                .expect("both spellings prove their bound; only their identities differ")
        };

        let symbolic = region(SourcedIndexInteger::Symbol(symbol("b")));
        let literal = region(4_i128.into());
        assert_eq!(
            symbolic.canonical_identity(),
            region(SourcedIndexInteger::Symbol(symbol("b"))).canonical_identity(),
            "one environment and one structure name one region",
        );
        assert_ne!(
            symbolic.canonical_identity(),
            literal.canonical_identity(),
            "a coefficient written as a symbol is a different program from one \
             written as that symbol's value",
        );
    }

    /// Normalization declines on a symbolic coefficient, term by term.
    ///
    /// Each assertion is one rewrite the literal path performs and the symbolic
    /// path does not, and each is a deliberate decision rather than a gap: none
    /// is available without a value the environment need not pin, and doing it
    /// *when* the environment happens to pin one would make canonicalization a
    /// function of the binding. The environment here pins `z == 0` and `u == 1`
    /// precisely so that a builder which resolved symbols would visibly fold.
    #[test]
    fn a_symbolic_coefficient_declines_every_fold_a_literal_takes() {
        let environment = environment_over(
            EXTENT_PHASE_CEILING,
            &["z", "u"],
            &[
                ExtentRelation::interval(term("z"), 0, 0).unwrap(),
                ExtentRelation::interval(term("u"), 1, 1).unwrap(),
            ],
        );
        let mut builder =
            IndexRegionBuilder::new_with_shape_environment(registry(), Arc::clone(&environment))
                .unwrap();
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(8))
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();

        // Interning makes these identity comparisons total: two spellings that
        // normalized to one node are one `IndexExprId`, and two that did not
        // are not. No view is needed to see which rewrites ran.
        //
        // A zero-pinned symbol is not dropped, where a literal zero is.
        let zeroed = builder
            .sourced_linear_combination(
                1_i128.into(),
                &[(SourcedIndexInteger::Symbol(symbol("z")), coordinate)],
            )
            .unwrap();
        let one = builder.constant(1_i128.into()).unwrap();
        assert_ne!(
            zeroed, one,
            "`1 + z * i` keeps its term even though the environment pins `z == 0`",
        );
        assert_eq!(
            builder
                .linear_combination(1_i128.into(), &[(0_i128.into(), coordinate)])
                .unwrap(),
            one,
            "the literal zero `z` is pinned to *is* dropped, leaving the constant",
        );

        // A one-pinned symbol is not unwrapped, where a literal one is.
        let scaled = builder
            .sourced_linear_combination(
                0_i128.into(),
                &[(SourcedIndexInteger::Symbol(symbol("u")), coordinate)],
            )
            .unwrap();
        assert_ne!(
            scaled, coordinate,
            "`u * i` is not the dimension itself even though `u == 1`",
        );
        assert_eq!(
            builder
                .linear_combination(0_i128.into(), &[(1_i128.into(), coordinate)])
                .unwrap(),
            coordinate,
            "the literal one `u` is pinned to *is* unwrapped",
        );

        // Two symbolic terms over one operand are not merged. Read through a
        // verified region because the count is a property of the retained node.
        let twice = scaled_copy(
            Some(environment),
            0_i128.into(),
            SourcedIndexInteger::Symbol(symbol("u")),
        )
        .expect("`u` is declared and available in time")
        .expect("`u == 1` bounds `u * i` below a 64-element axis");
        let terms = twice
            .index_expressions()
            .find_map(|expression| match expression.view() {
                IndexExprView::LinearCombination { terms, .. } => Some(terms.len()),
                _ => None,
            })
            .expect("the coordinate is a linear combination");
        assert_eq!(
            terms, 1,
            "one symbolic term stays one term, and it is not folded into the constant",
        );
    }
}
