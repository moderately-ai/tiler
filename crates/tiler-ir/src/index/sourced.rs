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
//! One ceiling, [`EXTENT_PHASE_CEILING`], binds both kinds of sourced extent,
//! but the two reach it by different routes and the difference is worth keeping
//! legible:
//!
//! - **A tensor boundary extent is the quoted case.** An output boundary's
//!   extent *is* an "initial output shape", which the clause above names
//!   outright. Nothing has to be inferred.
//! - **An index-domain extent is the inferred case.** It is upstream of exactly
//!   those quantities — it fixes the iteration domain a launch geometry is
//!   derived from — so the same rule binds it. The corpus states the rule for
//!   semantic extents and does not restate it for index domains; this module is
//!   where that inference is written down rather than assumed.
//!
//! Either way [`AvailabilityPhase::PreparedKernelPreflight`] and
//! [`AvailabilityPhase::LaunchPreflight`] are too late. The check is a
//! comparison because ADR 0043's order is total and documented as load-bearing:
//! "a use site evaluated at one phase may only name roots available no later
//! than it".
//!
//! # What the environment lets a verifier prove
//!
//! A symbolic extent is not an opaque hole. Three questions are answerable, and
//! they are deliberately three rather than one because each admits a different
//! proof and a caller that confused them would claim more than it proved:
//!
//! - [`ExtentSources::interval`] returns the closed interval every model
//!   confines the symbol to, so a coordinate can be proved in bounds against an
//!   axis whenever the constraint environment bounds the two tightly enough.
//! - [`ExtentSources::determined`] answers the stronger question — does the
//!   environment fix exactly one value — which is what an exhaustive
//!   enumeration needs.
//! - [`ExtentSources::proves_equal`] answers a question neither of the others
//!   can: whether two extents are the *same* extent in every model, even when
//!   no model-independent value is known for either. That is what a
//!   dynamically shaped output needs — a write covers its boundary exactly when
//!   the domain it iterates and the boundary it writes are the same size, and
//!   with both symbolic that is an equality-class fact rather than an
//!   arithmetic one.
//!
//! Neither answer is ever guessed. An extent the environment does not bound is
//! reported as unprovable by the region verifier rather than approximated, and
//! an extent the environment does not determine is never enumerated.

use std::sync::Arc;

use super::IndexBuildError;
use crate::program::abi::AvailabilityPhase;
use crate::shape::env::constraint::ExtentInterval;
use crate::shape::env::{ShapeEnv, ShapeEnvIdentity, ShapeSymbol};
use crate::shape::{Extent, Shape, ShapeError};

/// The last availability phase a sourced extent may be read from.
///
/// One ceiling for index-domain extents and tensor boundary extents alike. See
/// the module documentation for why the two reach it differently: for a
/// boundary the accepted pre-dispatch host-evaluability decision names "every
/// initial output shape" outright, while for a domain it is an inference from
/// that decision rather than a clause quoted from it.
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

/// One tensor boundary's ordered extents, and where each one comes from.
///
/// # Why an enum rather than a bare `Vec<SourcedExtent>`
///
/// [`Shape`] is the crate's public shape vocabulary and
/// [`TensorRef::static_shape`](super::TensorRef::static_shape) returns a
/// *borrow* of one. A wholly static boundary must keep answering that borrow,
/// so the static case holds the `Shape` it would otherwise have to materialize
/// on every call. Widening the accessor to return a `Shape` by value instead
/// would change a public signature to express something no public caller can
/// yet observe.
///
/// Holding [`Shape`] here also keeps one definition of a static shape rather
/// than two: an all-literal boundary is a `Shape`, never a parallel vector of
/// literal [`SourcedExtent`]s that happens to mean the same thing.
///
/// # The normalization invariant
///
/// [`Self::sourced`] collapses an all-literal extent vector into
/// [`Self::Static`], so [`Self::Sourced`] holds at least one symbol and a
/// boundary has exactly one spelling. That is what makes `static_shape()`
/// depend on the boundary rather than on which constructor authored it.
#[derive(Clone, Debug)]
pub(crate) enum SourcedShape {
    /// Every extent was a literal fixed when the region was authored.
    Static(Shape),
    /// At least one extent is a declared `ShapeEnv` symbol.
    Sourced(Vec<SourcedExtent>),
}

impl SourcedShape {
    /// Wraps an already-bounded static shape.
    pub(crate) const fn from_shape(shape: Shape) -> Self {
        Self::Static(shape)
    }

    /// Builds a boundary from ordered sourced extents, normalizing as above.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError::RankTooLarge`] when an all-literal boundary's rank
    /// exceeds the governed shape bound. The index layer's own, tighter
    /// `MAX_TENSOR_RANK` is checked separately by the builder; this is the
    /// shape vocabulary refusing to represent the normalized form at all, and
    /// the two limits stay distinct rather than one standing in for the other.
    pub(crate) fn sourced(extents: Vec<SourcedExtent>) -> Result<Self, ShapeError> {
        let literals: Option<Vec<Extent>> = extents.iter().map(SourcedExtent::as_static).collect();
        match literals {
            Some(literals) => Shape::try_new(literals).map(Self::Static),
            None => Ok(Self::Sourced(extents)),
        }
    }

    /// Returns the logical rank.
    pub(crate) fn rank(&self) -> usize {
        match self {
            Self::Static(shape) => shape.rank(),
            Self::Sourced(extents) => extents.len(),
        }
    }

    /// Returns the borrowed static shape, for a wholly literal boundary only.
    ///
    /// `None` for a boundary any of whose extents names a symbol, even when the
    /// environment determines every one of them: this asks what was *written*,
    /// exactly as [`SourcedExtent::as_static`] does, and identity is a function
    /// of that rather than of what a particular environment resolves it to.
    pub(crate) const fn as_static(&self) -> Option<&Shape> {
        match self {
            Self::Static(shape) => Some(shape),
            Self::Sourced(_) => None,
        }
    }

    /// Returns the ordered extents with their sources, outermost first.
    ///
    /// A static boundary is projected extent by extent rather than kept on a
    /// separate path, so every verifier reads one shape vocabulary and a rule
    /// cannot be extended for one representation and forgotten for the other.
    pub(crate) fn extents(&self) -> impl ExactSizeIterator<Item = SourcedExtent> + '_ {
        // Indexed rather than chained so the result stays `ExactSizeIterator`.
        // Each arm indexes the slice whose length `rank()` returned, so both
        // indices are in range by construction; a panic here would be a broken
        // invariant rather than an input a caller can reach.
        (0..self.rank()).map(move |axis| match self {
            Self::Static(shape) => SourcedExtent::Static(shape.extents()[axis]),
            Self::Sourced(extents) => extents[axis].clone(),
        })
    }

    /// Appends this boundary's canonical bytes.
    ///
    /// Length-framed and then extent by extent through
    /// [`SourcedExtent::encode`], so a literal axis encodes identically whether
    /// it was authored as a [`Shape`] or normalized out of an extent vector.
    /// One encoding for one boundary is what keeps the representation choice
    /// above from being observable in identity.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        crate::identity::push_len(bytes, self.rank());
        for extent in self.extents() {
            extent.encode(bytes);
        }
    }

    /// Returns the exact canonical byte length [`Self::encode`] appends.
    pub(crate) fn encoded_len(&self) -> usize {
        self.extents()
            .map(|extent| extent.encoded_len())
            .fold(8_usize, usize::saturating_add)
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
    /// The shape vocabulary refused to represent the boundary at all.
    ///
    /// Distinct from [`Self::Structural`] because the two name different
    /// authorities: the index layer's own `MAX_TENSOR_RANK` governs how large a
    /// boundary *this* IR admits, while [`ShapeError`] is the shape
    /// vocabulary's bound on what a [`Shape`] can hold. Collapsing them would
    /// report one limit's rejection under the other's name.
    Shape(ShapeError),
}

impl std::fmt::Display for SymbolicExtentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "{error}"),
            Self::Structural(error) => write!(formatter, "{error}"),
            Self::Shape(error) => write!(formatter, "{error}"),
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

    /// Returns whether every model assigns these two extents the same value.
    ///
    /// One-sided, and deliberately so. `true` is a proof of equality; `false`
    /// means *not proved*, never *proved different*. A caller that read `false`
    /// as a disequality would be inventing a fact the environment did not
    /// state.
    ///
    /// Two independent routes reach `true`, and both are needed because neither
    /// subsumes the other:
    ///
    /// - **The equality class.** Two symbols the environment forces together —
    ///   by an asserted symbol equality, or by a `>=` cycle, which forces
    ///   equality just as directly — share a class, and every model therefore
    ///   assigns them one value. This route proves equality with no value
    ///   known for either side, which is exactly the dynamically shaped case:
    ///   an output boundary sized `n` and a domain iterating `n` are the same
    ///   size whatever `n` turns out to be.
    /// - **A common determined value.** Two extents the environment pins to the
    ///   same single value are equal even when nothing relates them
    ///   syntactically, and this is the only route available when one side is a
    ///   literal.
    pub(crate) fn proves_equal(&self, left: &SourcedExtent, right: &SourcedExtent) -> bool {
        if let (Some(left), Some(right)) = (left.symbol(), right.symbol())
            && self.environment.proves_equal(left, right)
        {
            return true;
        }
        match (self.determined(left), self.determined(right)) {
            (Some(left), Some(right)) => left == right,
            _ => false,
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
    use super::{ExtentSources, SourcedExtent, SourcedShape};
    use crate::index::{
        AccessMode, BoundsProofView, DomainRole, FrozenScalarRegistry, IndexRegionBuildError,
        IndexRegionBuilder, IndexRegionDiagnostic, ScalarArity, ScalarAttributeSchema,
        ScalarAttributes, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
        ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
        ScalarOperationInferencer, ScalarRegistryBuilder, TensorRole, VerifiedIndexRegion,
        WriteOwnershipProofView,
    };
    use crate::program::abi::AvailabilityPhase;
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    };
    use crate::shape::Extent;
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
        let mut builder = IndexRegionBuilder::new(registry())
            .unwrap()
            .with_shape_environment(environment);
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

    /// `static_shape()` is absent exactly for a symbolically sourced boundary.
    ///
    /// `docs/ir.md` reserved this beside the dimension case: static dimensions
    /// and tensor boundaries "return `Some` throughout this bounded profile. A
    /// future symbolic profile can return `None` and expose its `ShapeEnv`
    /// expression through an additive borrowed view instead of changing the
    /// meaning of an existing accessor."
    ///
    /// The second half is the normalization invariant, and it is what keeps the
    /// accessor a fact about the boundary rather than about which constructor
    /// authored it: a boundary written through the *sourced* path whose extents
    /// all turned out to be literals still answers `Some`.
    #[test]
    fn static_shape_is_absent_exactly_for_a_symbolically_sourced_boundary() {
        let region = read_from_symbolic_axis(environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::interval(term("m"), 8, 16).unwrap()],
        ))
        .expect("a domain of 4 indexes an axis of at least 8");

        let boundaries: Vec<_> = region
            .tensors()
            .map(|tensor| {
                (
                    tensor.role(),
                    tensor.static_shape().is_some(),
                    tensor
                        .sourced_shape()
                        .extents()
                        .filter_map(|extent| extent.symbol().cloned())
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        assert!(
            boundaries
                .iter()
                .all(|(_, literal, symbols)| *literal == symbols.is_empty()),
            "a boundary is either wholly literal or symbolic and never both: {boundaries:?}",
        );
        assert!(
            boundaries
                .iter()
                .any(|(role, _, symbols)| *role == TensorRole::Input && symbols == &[symbol("m")]),
            "the symbolic input exposes the symbol it was sourced from: {boundaries:?}",
        );

        // Authored through the sourced path, but every extent is a literal, so
        // it normalizes to the same boundary a static caller would have
        // written — and answers `static_shape()` accordingly.
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
        // nothing could have been walked.
        assert!(
            roomy
                .accesses()
                .any(|access| access.bounds_proof() == BoundsProofView::Interval),
            "the read into a symbolic axis is proved by the environment's interval",
        );

        // One admissible extent below the domain is one too few, and the same
        // region is refused. Nothing else about it changed.
        let too_short = read_from_symbolic_axis(environment_over(
            EXTENT_PHASE_CEILING,
            &["m"],
            &[ExtentRelation::interval(term("m"), 3, 16).unwrap()],
        ))
        .unwrap_err();
        assert!(
            reports(&too_short, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::BoundsNotProven { .. }
            )),
            "an admissible extent of 3 cannot hold coordinate 3: {too_short}",
        );
        assert!(
            !reports(&too_short, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::ProofResourceLimit { .. }
            )),
            "this is a refusal, not an enumeration that ran out of budget: {too_short}",
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
            let mut builder = IndexRegionBuilder::new(registry())
                .unwrap()
                .with_shape_environment(environment(phase, &[]));
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
        let mut admitted = IndexRegionBuilder::new(registry())
            .unwrap()
            .with_shape_environment(environment(EXTENT_PHASE_CEILING, &[]));
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
            let mut builder = IndexRegionBuilder::new(registry())
                .unwrap()
                .with_shape_environment(environment);
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
            let mut builder = IndexRegionBuilder::new(registry())
                .unwrap()
                .with_shape_environment(environment);
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
                .all(|tensor| tensor.static_shape().is_some()),
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
            region
                .accesses()
                .all(|access| access.bounds_proof() == BoundsProofView::ProvedExtentEquality),
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

    /// The neighbour whose environment never says the two extents are one.
    ///
    /// `m` is declared and available, and nothing bounds it at all. The
    /// structural argument needs the equality and has none, interval
    /// propagation has no interval to compare, and there is no finite domain to
    /// walk — so both accesses are refused. The refusal must stay *explicit*
    /// and must not be the proof-resource diagnostic, which `docs/ir.md`
    /// defines as meaning "the enumeration stopped — not that the region was
    /// disproved".
    #[test]
    fn an_undetermined_copy_whose_extents_are_never_proved_equal_is_still_refused() {
        let error =
            undetermined_dynamic_copy(environment_over(EXTENT_PHASE_CEILING, &["m", "n"], &[]))
                .unwrap_err();
        assert!(
            reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::BoundsNotProven { .. }
            )) && reports(&error, |diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::WriteOwnershipNotProven { .. }
            )),
            "an unproved equality is not a proof, and both accesses rest on it: {error}",
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
        let mut builder = IndexRegionBuilder::new(registry())
            .unwrap()
            .with_shape_environment(environment);
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
}
