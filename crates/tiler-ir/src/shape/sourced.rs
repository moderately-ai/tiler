//! Sourced extents: the crate's one constant-or-symbol magnitude vocabulary,
//! and what a [`ShapeEnv`] lets a consumer prove about one.
//!
//! # Where this surface is reachable from
//!
//! The accepted subset is re-exported flat from [`crate::shape`], exactly as the
//! [`ShapeEnv`] vocabulary it reads is; this module itself is private, so the
//! canonical encoders stay inside it. A caller that could encode an extent could
//! derive an identity under rules the encoder does not establish.
//!
//! **This vocabulary sits at the shape layer because that is where its
//! components live.** [`SourcedExtent`] is `Extent | ShapeSymbol` and both of
//! those are shape-layer types; nothing in it is specific to any one consumer.
//! Siting it here is what lets two layers share one spelling, one encoding, and
//! one place to extend when a third source kind arrives, rather than each
//! minting a mirror — which is the defect [`SourcedExtent`]'s own documentation
//! is written against.
//!
//! **Fact — the index and semantic layers are both consumers at this commit.**
//! The index layer authors a symbolic region through
//! [`IndexRegionBuilder::new_with_shape_environment`](crate::index::IndexRegionBuilder::new_with_shape_environment),
//! [`IndexRegionBuilder::symbolic_dimension`](crate::index::IndexRegionBuilder::symbolic_dimension),
//! and
//! [`IndexRegionBuilder::sourced_tensor`](crate::index::IndexRegionBuilder::sourced_tensor),
//! and inspects one through
//! [`DomainDimensionRef::extent`](crate::index::DomainDimensionRef::extent),
//! [`TensorRef::shape`](crate::index::TensorRef::shape), and
//! [`VerifiedIndexRegion::extent_sources`](crate::index::VerifiedIndexRegion::extent_sources).
//! The semantic layer authors a symbolic *input* through
//! [`SemanticProgramBuilder::try_standard_with_shape_environment`](crate::semantic::SemanticProgramBuilder::try_standard_with_shape_environment),
//! [`SemanticProgramBuilder::input_sourced`](crate::semantic::SemanticProgramBuilder::input_sourced),
//! and
//! [`SemanticProgramBuilder::input_resolved_sourced`](crate::semantic::SemanticProgramBuilder::input_resolved_sourced),
//! and inspects one through
//! [`SemanticProgram::shape`](crate::semantic::SemanticProgram::shape) and
//! [`SemanticProgram::extent_sources`](crate::semantic::SemanticProgram::extent_sources).
//! Every claim below about "a consumer" is stated once here rather than
//! duplicated into each layer, which would then have to be kept in step.
//!
//! An *inferred* semantic result shape is still a [`Shape`]: shape inference
//! over symbolic operands is a separate delivery, so a symbolic value cannot be
//! an operation operand yet and only a program input may name a symbol. That is
//! a boundary rather than a partial state — every representable program is
//! constructible, verifiable, and identifiable — and it is what
//! `resolve-semantic-shape-inference-over-symbolic-extents` moves.
//!
//! # What this module owns, and what it deliberately does not
//!
//! It owns the question *may this extent be sourced here, and what does the
//! environment let me prove about it*. It owns no symbols. A symbolic extent is
//! a [`ShapeSymbol`] declared in a [`ShapeEnv`] and nothing else: there is no
//! consumer-local symbol table, no consumer-local binding, and no way to name an
//! extent this module resolves without the environment that declares it. That
//! is `docs/ir.md`'s requirement that unsupported dynamic cases "reject rather
//! than entering an index-local symbol or untyped predicate escape hatch".
//!
//! # The four rejections
//!
//! Free, ambiguous, tensor-data-derived, and too-late sources are refused at
//! three different places, and the difference matters because only one of them
//! is this module's own work:
//!
//! - **Free** and **ambiguous** sources are already impossible. `ShapeEnv`
//!   gives every symbol exactly one declaration and exactly one root binding,
//!   fails `build` on an unbound symbol, and rejects a second binding rather
//!   than overwriting the first. A verified environment therefore has no free
//!   and no ambiguous symbol in it, and re-deciding that here would be a second
//!   authority over a settled question. What this module does check is that the
//!   symbol belongs to *this* environment — [`ExtentSourceError::UndeclaredSymbol`] —
//!   because an extent naming a symbol from some other environment would make
//!   the consuming region's identity ambiguous even though each environment
//!   alone is not.
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
//! legible. The ladder is stated here, at the layer that owns the vocabulary,
//! rather than at any one consumer:
//!
//! - **A semantic output extent is the quoted case.** The clause above is about
//!   "initial output shape[s]", which is a semantic-layer quantity, and it names
//!   them outright. Nothing has to be inferred. It is stated first because it is
//!   the rule the other three read across from rather than re-derive.
//!   [`SemanticProgramBuilder::input_resolved_sourced`](crate::semantic::SemanticProgramBuilder::input_resolved_sourced)
//!   enforces it on the semantic side, at the constructor rather than at build,
//!   so a refused source leaves the draft exactly as it was.
//! - **An index-layer tensor boundary extent inherits it directly.** An index
//!   region's output boundary *is* the realization of an initial output shape,
//!   so the clause reaches it with no further step than identifying the two.
//! - **An index-domain extent is the inferred case.** It is upstream of exactly
//!   those quantities — it fixes the iteration domain a launch geometry is
//!   derived from — so the same rule binds it. The corpus states the rule for
//!   semantic extents and does not restate it for index domains; this is where
//!   that inference is written down rather than assumed.
//! - **A divisor or a linear-combination coefficient follows the domain.** Each
//!   is part of a coordinate expression evaluated over that domain, so a scalar
//!   that first existed once a pipeline was prepared would put the same cycle —
//!   domain to plan to domain — inside one access rather than across two.
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
//! reported as unprovable by the consuming verifier rather than approximated,
//! and an extent the environment does not determine is never enumerated.

use std::sync::Arc;

use crate::program::abi::AvailabilityPhase;

use super::{Extent, ExtentInterval, Shape, ShapeEnv, ShapeEnvIdentity, ShapeError, ShapeSymbol};

/// The last availability phase a sourced extent may be read from.
///
/// One ceiling for every consumer of this vocabulary. See the module
/// documentation for why they reach it differently: the accepted pre-dispatch
/// host-evaluability decision names "every initial output shape" outright, an
/// index-layer tensor boundary is the realization of one, and an index domain,
/// a divisor, and a coefficient are inferences from that same decision rather
/// than clauses quoted from it.
pub const EXTENT_PHASE_CEILING: AvailabilityPhase = AvailabilityPhase::LiveDevicePreflight;

/// One extent and where its value comes from.
///
/// Deliberately two cases and not an expression tree. A composed extent is a
/// relation in the environment's constraint set, where it can be decided,
/// rather than arithmetic a consumer would have to re-derive.
///
/// This is the crate's *one* constant-or-symbol vocabulary for a magnitude. An
/// index domain extent, a tensor boundary axis, and a floor-division or modulo
/// divisor all use it, and they use the same one deliberately: a second divisor
/// enum would give a frontend two ways to spell the same fact, two encodings to
/// fold into identity, and two places to extend when a third source kind
/// arrives. That argument is what sites the type here rather than inside any one
/// consumer — a per-layer copy is the same defect one layer up. A pass that only
/// handles constants reads [`Self::as_static`] once and refuses everything else
/// with its own typed reason.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourcedExtent {
    /// A literal extent fixed when the consuming program was authored.
    Static(Extent),
    /// A declared `ShapeEnv` symbol, resolved through that environment alone.
    Symbol(ShapeSymbol),
}

impl SourcedExtent {
    /// Returns the governed tag of this source kind, exhaustively.
    ///
    /// Written by a match rather than read from the discriminant, so adding a
    /// kind is a build error here instead of a silent re-encoding of every
    /// identity ever derived over one (ADR 0074 convention 3).
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
    /// folded once by the consuming program, so the symbol reference is
    /// complete.
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
/// # Equality
///
/// Structural, and sound because of the normalization invariant above: an
/// all-literal boundary has exactly one spelling, so two equal boundaries are
/// the same boundary and two different boundaries never compare equal. Equality
/// asks what was *written* — two symbols an environment forces together are
/// still two different spellings — which is the same question
/// [`Self::as_static`] answers and the one identity is a function of.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SourcedShape {
    /// Every extent was a literal fixed when the boundary was authored.
    Static(Shape),
    /// At least one extent is a declared `ShapeEnv` symbol.
    Sourced(Vec<SourcedExtent>),
}

impl SourcedShape {
    /// Wraps an already-bounded static shape.
    ///
    /// Crate-internal, like [`Self::sourced`]: a frontend states a boundary
    /// through its own layer's constructor — today
    /// [`IndexRegionBuilder::tensor`](crate::index::IndexRegionBuilder::tensor)
    /// or
    /// [`IndexRegionBuilder::sourced_tensor`](crate::index::IndexRegionBuilder::sourced_tensor)
    /// — which is where that layer's own rank and byte limits are enforced. A
    /// publicly constructible boundary would be one that bypassed them.
    pub(crate) const fn from_shape(shape: Shape) -> Self {
        Self::Static(shape)
    }

    /// Builds a boundary from ordered sourced extents, normalizing as above.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError::RankTooLarge`] when an all-literal boundary's rank
    /// exceeds the governed shape bound. A consuming layer's own, tighter rank
    /// limit is checked separately by that layer's builder; this is the shape
    /// vocabulary refusing to represent the normalized form at all, and the two
    /// limits stay distinct rather than one standing in for the other.
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
        extents_encoded_len(self.extents())
    }

    /// Returns the length [`Self::encode`] appends for a wholly literal shape.
    ///
    /// A consumer's canonical-work accounting reserves an inferred boundary's
    /// bytes before the boundary itself exists, and an inferred boundary is
    /// always literal. This answers that question through the same framing
    /// [`Self::encoded_len`] uses rather than restating it, and without
    /// materializing the [`Self::Static`] it would describe.
    pub(crate) fn static_encoded_len(shape: &Shape) -> usize {
        extents_encoded_len(shape.extents().iter().copied().map(SourcedExtent::Static))
    }
}

/// Frames a run of sourced extents: the length, then each extent's own bytes.
///
/// The single definition of a boundary's framing. Both
/// [`SourcedShape::encoded_len`] and [`SourcedShape::static_encoded_len`]
/// delegate here so a boundary's byte length cannot be described two ways, in
/// the same spirit as this crate's string framing.
fn extents_encoded_len(extents: impl Iterator<Item = SourcedExtent>) -> usize {
    extents
        .map(|extent| extent.encoded_len())
        .fold(SOURCED_SHAPE_LENGTH_BYTES, usize::saturating_add)
}

/// Width of the rank prefix [`crate::identity::push_len`] writes.
const SOURCED_SHAPE_LENGTH_BYTES: usize = std::mem::size_of::<u64>();

/// Why one sourced extent may not be used where it was written.
///
/// Every variant is a refusal by the *source environment*: the consuming
/// program's one [`ShapeEnv`] does not declare the symbol, does not supply it in
/// time, or does not prove what using it there requires. None of them is a limit
/// of a consuming layer's own structure, and none is a limit of the shape
/// vocabulary, which is [`ShapeError`]. A consumer that can be refused by all
/// three keeps them separable in its own error type rather than collapsing them
/// here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExtentSourceError {
    /// The extent named a symbol this program's environment does not declare.
    ///
    /// A program resolves every symbolic extent against exactly one
    /// environment. Admitting a symbol from another one would leave its identity
    /// naming a binding no consumer of it can resolve.
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
        /// The last phase an extent may be sourced from.
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
    /// program that a later plan choice could make meaningless.
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

/// One program's binding of sourced extents to a single shape environment.
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
    /// Crate-internal: a consuming program acquires its environment once, at
    /// construction — today
    /// [`IndexRegionBuilder::new_with_shape_environment`](crate::index::IndexRegionBuilder::new_with_shape_environment)
    /// — and a publicly constructible binding would be a second way to name the
    /// environment a program's extents resolve against.
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
    /// A consuming program folds this into its own identity. Two programs whose
    /// extents are spelled identically but whose symbols are bound differently
    /// are different programs, and this is what makes them different.
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
