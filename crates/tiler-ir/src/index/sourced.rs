#![allow(
    dead_code,
    reason = "ADR 0074 convention 7 draft: the symbolic index profile is crate-internal until it is reviewed, so no public constructor reaches it and its only callers are this module's tests. `implement-shapeenv-index-bindings` records promoting the boundary as a separate reviewed step; wiring a public caller to satisfy the lint would perform that promotion without the review"
)]

//! Sourced index extents: the index layer's consumption of the `ShapeEnv`
//! authority.
//!
//! # Draft status
//!
//! This module is `pub(crate)`, like the [`ShapeEnv`] authority it reads.
//! `implement-shapeenv-index-bindings` states that "any consequential public or
//! cross-crate boundary remains a draft until Tom reviews and accepts the exact
//! implementation commit", so no symbolic extent is reachable through the
//! crate's public index API and promoting it is a separate reviewed step. Every
//! public constructor still produces a static extent, which is why the public
//! `static_extent()` and `static_shape()` accessors keep returning `Some` for
//! every region a public caller can build.
//!
//! # What this module owns, and what it deliberately does not
//!
//! It owns the question *may this extent be sourced here, and what does the
//! environment let me prove about it*. It owns no symbols. A symbolic extent is
//! a [`ShapeSymbol`] declared in a [`ShapeEnv`] and nothing else: there is no
//! index-local symbol table, no index-local binding, and no way to name an
//! extent this module resolves without the environment that declares it. That
//! is `docs/ir.md`'s requirement that unsupported dynamic cases "reject rather
//! than entering an index-local symbol or untyped predicate escape hatch".
//!
//! # The four rejections
//!
//! The ticket names free, ambiguous, tensor-data-derived, and too-late sources.
//! They are refused at three different places, and the difference matters
//! because only one of them is this module's own work:
//!
//! - **Free** and **ambiguous** sources are already impossible. `ShapeEnv`
//!   gives every symbol exactly one declaration and exactly one root binding,
//!   fails `build` on an unbound symbol, and rejects a second binding rather
//!   than overwriting the first. A verified environment therefore has no free
//!   and no ambiguous symbol in it, and re-deciding that here would be a second
//!   authority over a settled question. What this module does check is that the
//!   symbol belongs to *this* environment — [`ExtentSourceError::UndeclaredSymbol`] —
//!   because an extent naming a symbol from some other environment would make
//!   the region's identity ambiguous even though each environment alone is not.
//!
//! - **Tensor-data-derived** sources are unrepresentable rather than rejected,
//!   and that is a weaker claim stated deliberately. [`SourcedExtent`] admits a
//!   literal or a declared symbol; a symbol's root binding admits a static
//!   extent, an input's *shape metadata*, a host interface parameter, or a
//!   governed target property. A scalar value read out of a tensor is none of
//!   those and there is no constructor that would accept one. The accepted
//!   contract draws the same line — "tensor element data is never an index
//!   parameter in the initial model", and a runtime integer is modelled as an
//!   explicit interface parameter "rather than encode metadata as a tensor
//!   shape".
//!
//! - **Too-late** sources are this module's own check, and the first place the
//!   availability ladder carries weight. See below.
//!
//! # The phase ceiling
//!
//! **Fact.** The accepted shape-environment contract requires that "every
//! initial output shape, temporary allocation size, applicability guard,
//! routing expression, and launch expression must be evaluable on the host
//! before any device work begins", and admits exactly "static constants, input
//! tensor metadata, explicit host interface parameters, and admitted target
//! properties available from a compile profile or live-device preflight".
//! Properties available only after selecting or preparing a pipeline "cannot
//! initially determine semantic output shapes: doing so would create a
//! dependency from shape to plan/pipeline and back to shape".
//!
//! **Inference.** An index-domain extent is upstream of exactly those
//! quantities — it fixes the iteration domain a launch geometry is derived
//! from — so the same rule binds it. On ADR 0043's ladder that makes
//! [`EXTENT_PHASE_CEILING`] the last admissible phase, and
//! [`AvailabilityPhase::PreparedKernelPreflight`] and
//! [`AvailabilityPhase::LaunchPreflight`] too late. The corpus states the rule
//! for semantic extents and does not restate it for index domains; this module
//! is where that inference is written down rather than assumed.
//!
//! The check is a comparison because ADR 0043's order is total and documented
//! as load-bearing: "a use site evaluated at one phase may only name roots
//! available no later than it".
//!
//! # What the environment lets a verifier prove
//!
//! A symbolic extent is not an opaque hole. [`ExtentSources::interval`] asks the
//! environment for the closed interval every model confines the symbol to, so a
//! coordinate derived from a symbolic domain can still be proved in bounds
//! against a static tensor axis whenever the constraint environment bounds the
//! symbol tightly enough. [`ExtentSources::determined`] answers the stronger
//! question — does the environment fix exactly one value — which is what an
//! exhaustive enumeration or a write-permutation argument needs.
//!
//! Neither answer is ever guessed. An extent the environment does not bound is
//! reported as unprovable by the region verifier rather than approximated, and
//! an extent the environment does not determine is never enumerated.

use std::sync::Arc;

use super::IndexBuildError;
use crate::program::abi::AvailabilityPhase;
use crate::shape::Extent;
use crate::shape::env::constraint::ExtentInterval;
use crate::shape::env::{ShapeEnv, ShapeEnvIdentity, ShapeSymbol};

/// The last availability phase an index-domain extent may be sourced from.
///
/// See the module documentation: this is an inference from the accepted
/// pre-dispatch host-evaluability decision, not a clause quoted from it.
pub(crate) const EXTENT_PHASE_CEILING: AvailabilityPhase = AvailabilityPhase::LiveDevicePreflight;

/// One extent and where its value comes from.
///
/// Deliberately two cases and not an expression tree. A composed extent is a
/// relation in the environment's constraint set, where it can be decided,
/// rather than arithmetic the index layer would have to re-derive.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SourcedExtent {
    /// A literal extent fixed when the region was authored.
    Static(Extent),
    /// A declared `ShapeEnv` symbol, resolved through that environment alone.
    Symbol(ShapeSymbol),
}

impl SourcedExtent {
    /// Returns the governed tag of this source kind, exhaustively.
    ///
    /// Written by a match rather than read from the discriminant, so adding a
    /// kind is a build error here instead of a silent re-encoding of every
    /// region identity ever derived (ADR 0074 convention 3).
    const fn tag(&self) -> u8 {
        match self {
            Self::Static(_) => 0x01,
            Self::Symbol(_) => 0x02,
        }
    }

    /// Returns the symbol this extent names, if it names one.
    pub(crate) const fn symbol(&self) -> Option<&ShapeSymbol> {
        match self {
            Self::Symbol(symbol) => Some(symbol),
            Self::Static(_) => None,
        }
    }

    /// Returns the literal extent, for a statically authored one only.
    ///
    /// A symbolic extent returns `None` even when its environment determines a
    /// value: this asks what was *written*, and identity is a function of that.
    /// Use [`ExtentSources::determined`] for what the environment fixes.
    pub(crate) const fn as_static(&self) -> Option<Extent> {
        match self {
            Self::Static(extent) => Some(*extent),
            Self::Symbol(_) => None,
        }
    }

    /// Appends this extent's canonical bytes.
    ///
    /// A symbolic extent encodes its symbol, not a resolved value: the accepted
    /// contract keeps `graph identity`, `interface identity`, and `specialized
    /// identity` distinguishable, and folding a bound value in here would
    /// collapse the first into the last. The environment's own identity is
    /// folded once by the region, so the symbol reference is complete.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::Static(extent) => bytes.extend_from_slice(&extent.get().to_be_bytes()),
            Self::Symbol(symbol) => symbol.encode(bytes),
        }
    }

    /// Returns the exact canonical byte length [`Self::encode`] appends.
    pub(crate) fn encoded_len(&self) -> usize {
        match self {
            Self::Static(_) => 1 + 8,
            Self::Symbol(symbol) => 1 + symbol.encoded_len(),
        }
    }
}

/// Why one sourced extent may not be used where it was written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExtentSourceError {
    /// The extent named a symbol this region's environment does not declare.
    ///
    /// A region resolves every symbolic extent against exactly one environment.
    /// Admitting a symbol from another one would leave the region's identity
    /// naming a binding no consumer of this region can resolve.
    UndeclaredSymbol {
        /// The symbol the rejected extent named.
        symbol: ShapeSymbol,
    },
    /// The extent named a symbol whose value arrives after it is needed.
    ///
    /// Admitting it would make the iteration domain depend on a selected or
    /// prepared pipeline, and the pipeline depend on the iteration domain.
    SourceTooLate {
        /// The symbol the rejected extent named.
        symbol: ShapeSymbol,
        /// The phase its root binding declares.
        available: AvailabilityPhase,
        /// The last phase an index extent may be sourced from.
        ceiling: AvailabilityPhase,
    },
}

impl std::fmt::Display for ExtentSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndeclaredSymbol { symbol } => write!(
                formatter,
                "index-extent.undeclared-symbol: {symbol} is not declared by this region's shape environment"
            ),
            Self::SourceTooLate {
                symbol,
                available,
                ceiling,
            } => write!(
                formatter,
                "index-extent.source-too-late: {symbol} is available at {available}, after {ceiling}"
            ),
        }
    }
}

impl std::error::Error for ExtentSourceError {}

/// A rejected symbolic dimension.
///
/// Two causes stay separable rather than collapsing into one message: a
/// structural limit says the region is too large, and a source refusal says the
/// extent may not be sourced there at all. A caller that retried the first
/// would be right to and a caller that retried the second would not.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SymbolicExtentError {
    /// The extent's source was refused.
    Source(ExtentSourceError),
    /// A governed structural limit or handle rule refused the dimension.
    Structural(IndexBuildError),
}

impl std::fmt::Display for SymbolicExtentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "{error}"),
            Self::Structural(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SymbolicExtentError {}

/// One region's binding of sourced extents to a single shape environment.
///
/// Holds the environment rather than a copy of anything derived from it, so
/// every answer below is a function of the verified environment and nothing can
/// drift from it.
#[derive(Clone, Debug)]
pub(crate) struct ExtentSources {
    environment: Arc<ShapeEnv>,
}

impl ExtentSources {
    /// Binds sourced extents to one verified environment.
    pub(crate) const fn new(environment: Arc<ShapeEnv>) -> Self {
        Self { environment }
    }

    /// Returns the exact identity of the environment extents resolve against.
    ///
    /// A region folds this into its own identity. Two regions whose extents are
    /// spelled identically but whose symbols are bound differently are
    /// different regions, and this is what makes them different.
    pub(crate) fn environment_identity(&self) -> &ShapeEnvIdentity {
        self.environment.identity()
    }

    /// Admits one sourced extent and returns the phase it becomes readable at.
    ///
    /// # Errors
    ///
    /// Returns [`ExtentSourceError::UndeclaredSymbol`] when the symbol belongs
    /// to another environment, and [`ExtentSourceError::SourceTooLate`] when its
    /// binding arrives after [`EXTENT_PHASE_CEILING`].
    pub(crate) fn admit(
        &self,
        extent: &SourcedExtent,
    ) -> Result<AvailabilityPhase, ExtentSourceError> {
        let Some(symbol) = extent.symbol() else {
            return Ok(AvailabilityPhase::CompileProfile);
        };
        let Some(binding) = self.environment.binding(symbol) else {
            return Err(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol.clone(),
            });
        };
        let available = binding.phase();
        if available > EXTENT_PHASE_CEILING {
            return Err(ExtentSourceError::SourceTooLate {
                symbol: symbol.clone(),
                available,
                ceiling: EXTENT_PHASE_CEILING,
            });
        }
        Ok(available)
    }

    /// Returns the single value the environment fixes for this extent, if any.
    ///
    /// A literal is fixed by construction. A symbol is fixed when the interval
    /// every model confines it to is a single point — which is sound because
    /// the interval contains every model, so a one-point interval leaves one
    /// admissible value. That is the accepted rule that "extent inference
    /// succeeds only when the available semantic constraints determine exactly
    /// one nonnegative extent"; anything short of it stays undetermined rather
    /// than being narrowed to a convenient value.
    pub(crate) fn determined(&self, extent: &SourcedExtent) -> Option<Extent> {
        match extent {
            SourcedExtent::Static(value) => Some(*value),
            SourcedExtent::Symbol(symbol) => {
                let interval = self.environment.extent_interval(symbol)?;
                (interval.lower == interval.upper).then(|| Extent::new(interval.lower))
            }
        }
    }

    /// Returns the closed interval every model confines this extent to.
    ///
    /// Sound to prove an upper bound against and never sound to enumerate: a
    /// congruence in the environment can exclude interior values.
    pub(crate) fn interval(&self, extent: &SourcedExtent) -> Option<ExtentInterval> {
        match extent {
            SourcedExtent::Static(value) => Some(ExtentInterval {
                lower: value.get(),
                upper: value.get(),
            }),
            SourcedExtent::Symbol(symbol) => self.environment.extent_interval(symbol),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;
    use std::sync::Arc;

    use super::{EXTENT_PHASE_CEILING, ExtentSourceError, SymbolicExtentError};
    use crate::index::{
        BoundsProofView, DomainRole, FrozenScalarRegistry, IndexRegionBuildError,
        IndexRegionBuilder, IndexRegionDiagnostic, ScalarArity, ScalarAttributeSchema,
        ScalarAttributes, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
        ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
        ScalarOperationInferencer, ScalarRegistryBuilder, TensorRole, VerifiedIndexRegion,
    };
    use crate::program::abi::AvailabilityPhase;
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    };
    use crate::shape::Shape;
    use crate::shape::env::constraint::{ExtentRelation, ExtentTerm, SemanticInputConstraint};
    use crate::shape::env::{
        BindingSource, FactProvenance, InterfaceParameterKey, RootBinding, ShapeEnv,
        ShapeEnvBuilder, ShapeSymbol, SymbolScope,
    };

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
    fn parameter_binding(phase: AvailabilityPhase) -> RootBinding {
        RootBinding::new(
            BindingSource::InterfaceParameter {
                key: InterfaceParameterKey::new("n").unwrap(),
            },
            phase,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    /// An environment declaring `n` with one root binding and given relations.
    fn environment(phase: AvailabilityPhase, relations: &[ExtentRelation]) -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        let n = symbol("n");
        draft.declare(n.clone()).unwrap();
        draft.bind(&n, parameter_binding(phase)).unwrap();
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
        let mut builder = IndexRegionBuilder::new(registry()).unwrap();
        if let Some(environment) = environment {
            builder = builder.with_shape_environment(environment);
        }
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
        // The retained evidence names *how* it was proved. Interval, not
        // enumeration: nothing walked a domain whose size is unknown.
        assert!(
            bounded
                .accesses()
                .any(|access| access.bounds_proof() == BoundsProofView::Interval),
            "the symbolic read is proved by the environment's interval",
        );

        // One larger admissible extent is one too many, and the same region is
        // refused. Nothing else about it changed.
        let too_wide = environment(
            EXTENT_PHASE_CEILING,
            &[ExtentRelation::interval(term("n"), 1, 5).unwrap()],
        );
        let error = region(Some(too_wide), 4).unwrap_err();
        assert!(
            reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::BoundsNotProven { .. }
            )),
            "an admissible extent of 5 can index a 4-element axis out of bounds: {error}",
        );
    }

    /// An unbounded symbolic extent is refused, never enumerated or assumed.
    ///
    /// `docs/ir.md` requires that "unsupported dynamic cases must reject rather
    /// than entering an index-local symbol or untyped predicate escape hatch".
    /// The refusal must also not be the proof-resource diagnostic, which the
    /// same contract defines as meaning "the enumeration stopped — not that the
    /// region was disproved". Those are the two ways this could have gone
    /// wrong, so both are asserted.
    #[test]
    fn an_unbounded_symbolic_extent_is_refused_rather_than_enumerated() {
        let error = region(Some(environment(EXTENT_PHASE_CEILING, &[])), 4).unwrap_err();
        assert!(
            reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::BoundsNotProven { .. }
            )),
            "an extent with no admitted bound proves nothing: {error}",
        );
        assert!(
            !reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::ProofResourceLimit { .. }
            )),
            "this is a refusal, not an enumeration that ran out of budget: {error}",
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
            let mut builder = IndexRegionBuilder::new(registry())
                .unwrap()
                .with_shape_environment(environment(phase, &[]));
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
        let mut admitted = IndexRegionBuilder::new(registry())
            .unwrap()
            .with_shape_environment(environment(EXTENT_PHASE_CEILING, &[]));
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
        let mut foreign = IndexRegionBuilder::new(registry())
            .unwrap()
            .with_shape_environment(environment(EXTENT_PHASE_CEILING, &[]));
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
            let mut builder = IndexRegionBuilder::new(registry())
                .unwrap()
                .with_shape_environment(environment);
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

    /// `static_extent()` is absent exactly for a symbolic dimension.
    ///
    /// `docs/ir.md` reserved this: static extents "return `Some` throughout
    /// this bounded profile. A future symbolic profile can return `None` and
    /// expose its `ShapeEnv` expression through an additive borrowed view
    /// instead of changing the meaning of an existing accessor."
    #[test]
    fn static_extent_is_absent_exactly_for_a_symbolic_dimension() {
        let region = region(
            Some(environment(
                EXTENT_PHASE_CEILING,
                &[ExtentRelation::interval(term("n"), 1, 4).unwrap()],
            )),
            4,
        )
        .unwrap();

        let extents: Vec<_> = region
            .dimensions()
            .map(|dimension| {
                (
                    dimension.static_extent(),
                    dimension.sourced_extent().symbol().cloned(),
                )
            })
            .collect();
        assert!(
            extents
                .iter()
                .all(|(literal, sourced)| literal.is_some() != sourced.is_some()),
            "a dimension is either literal or symbolic and never both: {extents:?}",
        );
        assert!(
            extents
                .iter()
                .any(|(_, sourced)| sourced.as_ref() == Some(&symbol("n"))),
            "the symbolic dimension exposes the symbol it was sourced from: {extents:?}",
        );
        assert!(
            region.extent_sources().is_some(),
            "a region with a symbolic extent retains the environment resolving it",
        );
    }
}
