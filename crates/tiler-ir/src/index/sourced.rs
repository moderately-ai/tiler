//! Sourced index extents: the index layer's consumption of the `ShapeEnv`
//! authority.
//!
//! # Where this surface is reachable from
//!
//! The accepted subset is re-exported flat from [`crate::index`], like the
//! [`ShapeEnv`] vocabulary it reads is from [`crate::shape`]; this module itself
//! is private, so the canonical encoders stay inside it. A frontend authors a
//! symbolic region through [`IndexRegionBuilder::new_with_shape_environment`],
//! [`IndexRegionBuilder::symbolic_dimension`], and
//! [`IndexRegionBuilder::sourced_tensor`], and inspects one through
//! [`DomainDimensionRef::extent`], [`TensorRef::shape`], and
//! [`VerifiedIndexRegion::extent_sources`].
//!
//! [`IndexRegionBuilder::new_with_shape_environment`]: super::IndexRegionBuilder::new_with_shape_environment
//! [`IndexRegionBuilder::symbolic_dimension`]: super::IndexRegionBuilder::symbolic_dimension
//! [`IndexRegionBuilder::sourced_tensor`]: super::IndexRegionBuilder::sourced_tensor
//! [`DomainDimensionRef::extent`]: super::DomainDimensionRef::extent
//! [`TensorRef::shape`]: super::TensorRef::shape
//! [`VerifiedIndexRegion::extent_sources`]: super::VerifiedIndexRegion::extent_sources
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
//! One ceiling, [`EXTENT_PHASE_CEILING`], binds every kind of sourced extent,
//! but they reach it by different routes and the difference is worth keeping
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
//! - **A floor-division or modulo divisor follows the domain.** It is part of a
//!   coordinate expression evaluated over that domain, so a divisor that first
//!   existed once a pipeline was prepared would put the same cycle — domain to
//!   plan to domain — inside one access rather than across two.
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
use crate::shape::{
    Extent, ExtentInterval, Shape, ShapeEnv, ShapeEnvIdentity, ShapeError, ShapeSymbol,
};

/// The last availability phase a sourced extent may be read from.
///
/// One ceiling for index-domain extents and tensor boundary extents alike. See
/// the module documentation for why the two reach it differently: for a
/// boundary the accepted pre-dispatch host-evaluability decision names "every
/// initial output shape" outright, while for a domain it is an inference from
/// that decision rather than a clause quoted from it.
pub const EXTENT_PHASE_CEILING: AvailabilityPhase = AvailabilityPhase::LiveDevicePreflight;

/// One extent and where its value comes from.
///
/// Deliberately two cases and not an expression tree. A composed extent is a
/// relation in the environment's constraint set, where it can be decided,
/// rather than arithmetic the index layer would have to re-derive.
///
/// This is the crate's *one* constant-or-symbol vocabulary for an index-layer
/// magnitude. A domain extent, a tensor boundary axis, and a floor-division or
/// modulo divisor all use it, and they use the same one deliberately: a second
/// divisor enum would give a frontend two ways to spell the same fact, two
/// encodings to fold into identity, and two places to extend when a third
/// source kind arrives. A pass that only handles constants reads
/// [`Self::as_static`] once and refuses everything else with its own typed
/// reason.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourcedExtent {
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
    #[must_use]
    pub const fn symbol(&self) -> Option<&ShapeSymbol> {
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
    #[must_use]
    pub const fn as_static(&self) -> Option<Extent> {
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
/// [`Shape`] is the crate's public fixed-shape vocabulary, and
/// [`Self::as_static`] returns a *borrow* of one. A wholly static boundary must
/// keep answering that borrow, so the static case holds the `Shape` it would
/// otherwise have to materialize on every call.
///
/// Holding [`Shape`] here also keeps one definition of a static shape rather
/// than two: an all-literal boundary is a `Shape`, never a parallel vector of
/// literal [`SourcedExtent`]s that happens to mean the same thing.
///
/// # The normalization invariant
///
/// Construction collapses an all-literal extent vector into [`Self::Static`],
/// so [`Self::Sourced`] holds at least one symbol and a boundary has exactly
/// one spelling. That is what makes [`Self::as_static`] depend on the boundary
/// rather than on which constructor authored it.
#[derive(Clone, Debug)]
pub enum SourcedShape {
    /// Every extent was a literal fixed when the region was authored.
    Static(Shape),
    /// At least one extent is a declared `ShapeEnv` symbol.
    Sourced(Vec<SourcedExtent>),
}

impl SourcedShape {
    /// Wraps an already-bounded static shape.
    ///
    /// Crate-internal, like [`Self::sourced`]: a frontend states a boundary by
    /// calling [`IndexRegionBuilder::tensor`](super::IndexRegionBuilder::tensor)
    /// or
    /// [`IndexRegionBuilder::sourced_tensor`](super::IndexRegionBuilder::sourced_tensor),
    /// which is where the index layer's own rank and byte limits are enforced.
    /// A publicly constructible boundary would be one that bypassed them.
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
    #[must_use]
    pub fn rank(&self) -> usize {
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
    #[must_use]
    pub const fn as_static(&self) -> Option<&Shape> {
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
    #[must_use]
    pub fn extents(&self) -> impl ExactSizeIterator<Item = SourcedExtent> + '_ {
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
///
/// Every variant is a refusal by the *source environment*: the region's one
/// [`ShapeEnv`] does not declare the symbol, does not supply it in time, or does
/// not prove what using it there requires. None of them is a limit of the index
/// layer's own structure, which is [`IndexBuildError`], or of the shape
/// vocabulary, which is [`ShapeError`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExtentSourceError {
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
    /// The environment does not prove a symbolic divisor is at least one.
    ///
    /// `x floordiv d` and `x mod d` have no meaning at `d == 0` under any plan,
    /// so positivity is a condition of the expression being *defined* rather
    /// than an optimization that would be nice to have. The proof comes from
    /// [`ShapeEnv::proves_positive`], which reads semantic input constraints
    /// and never variant guards: a guard's failure selects another plan, and an
    /// expression whose definedness rested on one would be admitted into a
    /// region that a later plan choice could make meaningless.
    ///
    /// This is a refusal, not a missing optimization. A frontend that means the
    /// divisor to be positive states an interval or an equality constraint on
    /// the symbol; there is nothing to retry without one.
    DivisorNotProvedPositive {
        /// The symbol the rejected divisor named.
        symbol: ShapeSymbol,
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
            Self::DivisorNotProvedPositive { symbol } => write!(
                formatter,
                "index-extent.divisor-not-proved-positive: this region's shape environment does not require {symbol} to be at least one"
            ),
        }
    }
}

impl std::error::Error for ExtentSourceError {}

/// A rejected sourced extent, divisor, or boundary.
///
/// Three authorities can refuse the same call and they stay separable rather
/// than collapsing into one message, because a caller acts differently on each:
/// a structural limit says the region is too large and a smaller one would be
/// admitted; a shape-vocabulary refusal says no [`Shape`] can hold the
/// normalized form; and a source refusal says the environment does not declare,
/// supply, or prove what using the extent there requires, so retrying without
/// changing the environment cannot succeed.
///
/// The [`From`] conversions below are ergonomic only. Each one lands in the
/// variant that names its own authority, so `?` never reports one authority's
/// limit under another's name.
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

/// One region's binding of sourced extents to a single shape environment.
///
/// Holds the environment rather than a copy of anything derived from it, so
/// every answer below is a function of the verified environment and nothing can
/// drift from it.
#[derive(Clone, Debug)]
pub struct ExtentSources {
    environment: Arc<ShapeEnv>,
}

impl ExtentSources {
    /// Binds sourced extents to one verified environment.
    ///
    /// Crate-internal: a region acquires its environment once, at
    /// [`IndexRegionBuilder::new_with_shape_environment`](super::IndexRegionBuilder::new_with_shape_environment),
    /// and a publicly constructible binding would be a second way to name the
    /// environment a region's extents resolve against.
    pub(crate) const fn new(environment: Arc<ShapeEnv>) -> Self {
        Self { environment }
    }

    /// Returns the verified environment every symbolic extent resolves in.
    ///
    /// The read-only view a consumer needs to interpret a symbol it found on a
    /// dimension or a boundary: the symbol alone means nothing without the
    /// environment that declares and binds it.
    #[must_use]
    pub fn environment(&self) -> &ShapeEnv {
        &self.environment
    }

    /// Returns the exact identity of the environment extents resolve against.
    ///
    /// A region folds this into its own identity. Two regions whose extents are
    /// spelled identically but whose symbols are bound differently are
    /// different regions, and this is what makes them different.
    #[must_use]
    pub fn environment_identity(&self) -> &ShapeEnvIdentity {
        self.environment.identity()
    }

    /// Admits one sourced extent and returns the phase it becomes readable at.
    ///
    /// # Errors
    ///
    /// Returns [`ExtentSourceError::UndeclaredSymbol`] when the symbol belongs
    /// to another environment, and [`ExtentSourceError::SourceTooLate`] when its
    /// binding arrives after [`EXTENT_PHASE_CEILING`].
    pub fn admit(&self, extent: &SourcedExtent) -> Result<AvailabilityPhase, ExtentSourceError> {
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
    #[must_use]
    pub fn determined(&self, extent: &SourcedExtent) -> Option<Extent> {
        match extent {
            SourcedExtent::Static(value) => Some(*value),
            SourcedExtent::Symbol(symbol) => {
                let interval = self.environment.extent_interval(symbol)?;
                (interval.lower == interval.upper).then(|| Extent::new(interval.lower))
            }
        }
    }

    /// Returns whether every model assigns this extent at least one.
    ///
    /// The definedness question a floor-division or modulo divisor must answer.
    /// A literal answers it by construction. A symbol answers it only through
    /// [`ShapeEnv::proves_positive`], which reads semantic input constraints and
    /// never variant guards — see that method for why the distinction is
    /// load-bearing rather than cautious.
    ///
    /// One-sided: `false` means *not proved*, never *proved zero*.
    #[must_use]
    pub fn proves_positive(&self, extent: &SourcedExtent) -> bool {
        match extent {
            SourcedExtent::Static(value) => value.get() >= 1,
            SourcedExtent::Symbol(symbol) => self.environment.proves_positive(symbol),
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
    #[must_use]
    pub fn proves_equal(&self, left: &SourcedExtent, right: &SourcedExtent) -> bool {
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
    #[must_use]
    pub fn interval(&self, extent: &SourcedExtent) -> Option<ExtentInterval> {
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
        AccessMode, BoundsProofView, DomainRole, FrozenScalarRegistry, IndexBuildError,
        IndexDomainPredicate, IndexDomainUnknownReason, IndexExprClass, IndexExprView,
        IndexExtentRef, IndexRegionBuildError, IndexRegionBuilder, IndexRegionDiagnostic,
        ScalarArity, ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
        ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract,
        ScalarOperationDefinition, ScalarOperationInferencer, ScalarRegistryBuilder, TensorRole,
        VerifiedIndexRegion, WriteOwnershipProofView,
    };
    use crate::program::abi::AvailabilityPhase;
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    };
    use crate::shape::{
        BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, GuardApplicability,
        InterfaceParameterKey, RootBinding, SemanticInputConstraint, Shape, ShapeEnv,
        ShapeEnvBuilder, ShapeSymbol, SymbolScope, VariantGuard,
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
        // The retained evidence names *how* it was proved. Interval, not
        // enumeration: nothing walked a domain whose size is unknown.
        assert!(
            bounded
                .accesses()
                .any(|access| access.bounds_proof() == Some(BoundsProofView::Interval)),
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
    /// complementarity only a test held. [`SourcedShape`] is matched
    /// exhaustively here instead, and both arms are taken by the one region
    /// under test.
    ///
    /// The last assertion is the normalization invariant, and it is what keeps
    /// [`SourcedShape::as_static`] a fact about the boundary rather than about
    /// which constructor authored it: a boundary written through the *sourced*
    /// path whose extents all turned out to be literals is a
    /// [`SourcedShape::Static`].
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
                let sourced = match tensor.shape() {
                    SourcedShape::Static(shape) => {
                        assert_eq!(tensor.shape().as_static(), Some(shape));
                        Vec::new()
                    }
                    SourcedShape::Sourced(extents) => extents
                        .iter()
                        .filter_map(|extent| extent.symbol().cloned())
                        .collect::<Vec<_>>(),
                };
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
        // nothing could have been walked.
        assert!(
            roomy
                .accesses()
                .any(|access| access.bounds_proof() == Some(BoundsProofView::Interval)),
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
                access.bounds_proof() == Some(BoundsProofView::ProvedExtentEquality)
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
}
