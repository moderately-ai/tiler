//! The runtime-value boundary: what Tiler reads from, and builds through, an
//! integration's own tensor values.
//!
//! # Public boundary status
//!
//! Every public item here is a **reviewed draft boundary** (ADR 0074 §7, ADR
//! 0075). It is `pub` so its shape can be reviewed as a whole — and because the
//! seam only works if a crate outside this one can implement it — and it is not
//! an accepted public facade until Tom accepts the exact interface.
//!
//! # Why an adapter and not a tensor type
//!
//! Tom ratified the shape on 2026-07-30: the public runtime-value boundary is a
//! facade-owned opaque wrapper parameterized by a consumer-supplied adapter.
//! Three consequences follow, and each removes an option that looks cheaper:
//!
//! - **No consumer type appears here.** Not Candle, not Metal, not a device
//!   object, not a storage layout, not an allocation policy, not a lifetime
//!   belonging to any of them. [`TensorAdapter`] names the integration's value,
//!   its context, and its error as associated types, so this crate never learns
//!   what they are.
//! - **No global registry.** A registry would make "which adapter applies" an
//!   ambient fact that two consumers in one binary could disagree about. The
//!   adapter travels in [`Tensor`]'s type parameter instead, so the answer is
//!   fixed where the value is constructed and checked by the compiler.
//! - **No adapter argument per invocation.** That is the same fact from the call
//!   site's side: a region takes wrapped values, not raw values plus an adapter,
//!   because the wrapper already carries the binding.
//!
//! An integration owns both conversions. It wraps its value going in, and it
//! receives its own value back out — [`build_result`](crate::__private::build_result)
//! returns `A::Value` rather than a wrapper, so the `let d = tiler::tensor! { … }`
//! a consumer writes binds the consumer's own tensor type.
//!
//! # Storage access is here now, and it is owed only where it is used
//!
//! This module previously published none, and said why: "a storage-access
//! surface would be a public boundary with no caller to review it against".
//! `route-an-embedded-artifact-through-a-consumer-storage-seam` is that caller,
//! so the surface exists — as [`DispatchAdapter`], a *second* trait rather than
//! three more methods on [`TensorAdapter`].
//!
//! The split is where the obligation actually falls. A region that states no
//! `deliver` policy embeds nothing, routes nothing, and dispatches nothing; an
//! adapter serving only such regions owes no byte run and no device authority,
//! and making it write both would be charging every consumer for a path it never
//! takes. Generated code names [`crate::__private::bind_and_build`] for those
//! regions and [`crate::__private::bind_route_and_build`] for a delivering one,
//! and only the second is bounded by [`DispatchAdapter`] — so the obligation is
//! demanded by the compiler exactly where a kernel would read the bytes.
//!
//! [`AdapterCapability::DenseRowMajorStorage`] was the reservation this fills.
//! It stays on [`TensorAdapter`] rather than moving, because it is a claim any
//! adapter may state and a region may check without dispatching; what moved onto
//! the new trait is the surface that *reads* the storage the claim describes.
//!
//! # Per-value storage properties are still absent
//!
//! A capability is a claim about the adapter, not about one value. An
//! integration whose values may be strided views is expected to materialize
//! before wrapping, or to decline the capability. Moving density onto
//! [`ValueMetadata`] is the widening path, and it is still not taken here
//! because nothing yet consumes the distinction.
//!
//! What *is* checked per value is length. An adapter claiming
//! [`AdapterCapability::DenseRowMajorStorage`] is claiming that a value's byte
//! run is exactly its element count times its stored scalar's width, and
//! [`BindError::StorageLengthMismatch`] refuses a value whose reported extents
//! and reported bytes disagree — before any of those bytes reach a kernel.
//! Trusting the claim instead would let a short buffer be dispatched against a
//! launch geometry derived from the long shape, which is a read past the end
//! rather than a wrong answer.

use std::error::Error;
use std::fmt;

pub use tiler_ir::program::StorageScalar;

use crate::runtime::adapter::RuntimeAdapter;
use crate::runtime::load::ExecutionEnvironment;

/// Bytes one value of this scalar occupies per element.
///
/// Restated rather than imported: `tiler_ir::program::StorageScalar::byte_width`
/// is private to its own crate. The restatement is safe because the match is
/// exhaustive over a type that is deliberately not `#[non_exhaustive]` — a
/// scalar added to the IR is a build error here rather than a silently wrong
/// length.
const fn byte_width(scalar: StorageScalar) -> u64 {
    match scalar {
        StorageScalar::U8 => 1,
        StorageScalar::F32 => 4,
    }
}

/// Returns the byte run one dense row-major value of this shape occupies.
///
/// `None` on overflow, which is a refusal rather than a wrap: a truncated
/// product would compare a real buffer against a length nothing describes.
pub(crate) fn dense_bytes(scalar: StorageScalar, extents: &[u64]) -> Option<u64> {
    extents
        .iter()
        .try_fold(1_u64, |elements, extent| elements.checked_mul(*extent))?
        .checked_mul(byte_width(scalar))
}

/// One capability a region may require of an adapter.
///
/// Deliberately **not** `#[non_exhaustive]`, under ADR 0074 convention 5c. An
/// adapter matches this to decide support, so it is a recognizer: making it
/// non-exhaustive would force every adapter to grow a wildcard arm, and a
/// capability added later would then reroute silently into "not supported"
/// with nothing failing anywhere. Leaving it exhaustive makes a widened profile
/// a build error in every adapter, which is where the decision belongs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdapterCapability {
    /// Every value this adapter yields stores its elements densely, innermost
    /// axis fastest, with no offset, stride, or padding of its own.
    ///
    /// An integration whose tensors may be strided views declines this, and a
    /// region requiring it refuses rather than reading such a value as though it
    /// were dense. This is the storage property Tiler's first dispatch profile
    /// requires; it is stated as an adapter capability rather than a layout
    /// description because the contract exposes no consumer storage layout.
    DenseRowMajorStorage,
    /// The adapter can construct a new value of a requested element type and
    /// shape from its own context.
    ///
    /// An adapter over a borrowed or read-only tensor type declines this, and a
    /// region that must return a result refuses rather than fabricating one.
    ResultConstruction,
}

impl AdapterCapability {
    /// Returns the stable diagnostic name of this capability.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DenseRowMajorStorage => "dense-row-major-storage",
            Self::ResultConstruction => "result-construction",
        }
    }
}

impl fmt::Display for AdapterCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The read-only metadata Tiler needs about one runtime tensor value.
///
/// An adapter constructs this, so it is an input record from this crate's side
/// and is deliberately exhaustive rather than `#[non_exhaustive]` (ADR 0074
/// convention 5a is asymmetric for exactly this reason: a caller-constructed
/// record marked non-exhaustive is a record no caller outside the crate can
/// construct).
///
/// It carries what a binding check needs and nothing else. The element type is
/// the *stored* scalar, which is what an input buffer must match; the semantic
/// element type of a region's operand is a different subject and lives in the
/// IR.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ValueMetadata {
    storage_scalar: StorageScalar,
    extents: Vec<u64>,
}

impl ValueMetadata {
    /// Reports one value's stored scalar and its extents, outermost first.
    ///
    /// Infallible: an implausible rank is not rejected here, because the check
    /// that matters is against the rank the region declared, and reporting it
    /// truthfully is what lets [`BindError::RankMismatch`] name both numbers.
    pub fn new(storage_scalar: StorageScalar, extents: impl IntoIterator<Item = u64>) -> Self {
        Self {
            storage_scalar,
            extents: extents.into_iter().collect(),
        }
    }

    /// Returns the scalar this value's storage holds.
    #[must_use]
    pub const fn storage_scalar(&self) -> StorageScalar {
        self.storage_scalar
    }

    /// Returns the logical rank.
    #[must_use]
    pub fn rank(&self) -> usize {
        self.extents.len()
    }

    /// Returns the extents, outermost first.
    #[must_use]
    pub fn extents(&self) -> &[u64] {
        &self.extents
    }
}

/// What a region asks an adapter to construct.
///
/// A shape and an element type, resolved from the region's declaration and the
/// symbol values bound from its operands. It names no allocation policy, memory
/// domain, or device: choosing those is the integration's, and stating them here
/// would put a consumer's runtime model inside a consumer-neutral contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResultRequest<'a> {
    storage_scalar: StorageScalar,
    extents: &'a [u64],
}

impl<'a> ResultRequest<'a> {
    /// Builds one request. Crate-internal: a request is derived from a verified
    /// region's facts and its bound extents, and a publicly constructible one
    /// would be a second way to state what a region asks for.
    pub(crate) const fn new(storage_scalar: StorageScalar, extents: &'a [u64]) -> Self {
        Self {
            storage_scalar,
            extents,
        }
    }

    /// Returns the scalar the constructed value's storage must hold.
    #[must_use]
    pub const fn storage_scalar(&self) -> StorageScalar {
        self.storage_scalar
    }

    /// Returns the requested extents, outermost first.
    #[must_use]
    pub const fn extents(&self) -> &'a [u64] {
        self.extents
    }
}

/// One integration's binding between Tiler and its own runtime tensor values.
///
/// Implemented by an integration crate, not by this one. Every method is an
/// associated function rather than a method on `&self`: the adapter is a
/// type-level marker that names three associated types, so there is no adapter
/// instance to store in a [`Tensor`], to thread through generated code, or to
/// look up in a registry. Whatever runtime state an integration needs travels in
/// [`Self::Context`], which the wrapper carries.
///
/// The trait is not object-safe and does not need to be. Generated code is
/// generic over `A` and monomorphized at the call site, so a `dyn` form would
/// buy indirection nothing asks for.
pub trait TensorAdapter {
    /// The integration's own runtime tensor value.
    type Value;
    /// Whatever the integration needs in order to construct a value — a device,
    /// an allocator, a stream, or `()`.
    type Context;
    /// The integration's own failure type, carried rather than flattened
    /// (ADR 0074 convention 1).
    type Error: Error + 'static;

    /// Returns whether this adapter offers one capability.
    ///
    /// Exhaustively matched by an implementor: [`AdapterCapability`] is not
    /// `#[non_exhaustive]` precisely so that a widened profile is a build error
    /// here rather than a silent decline.
    fn supports(capability: AdapterCapability) -> bool;

    /// Reports one value's read-only metadata.
    ///
    /// # Errors
    ///
    /// Returns the integration's own error when the value's metadata cannot be
    /// read. Tiler carries it verbatim in [`BindError::Adapter`].
    fn metadata(value: &Self::Value) -> Result<ValueMetadata, Self::Error>;

    /// Constructs one result value.
    ///
    /// Called only when [`AdapterCapability::ResultConstruction`] is supported.
    ///
    /// # Errors
    ///
    /// Returns the integration's own error when the value cannot be
    /// constructed. Tiler carries it verbatim in [`BindError::Adapter`].
    fn build(
        context: &Self::Context,
        request: &ResultRequest<'_>,
    ) -> Result<Self::Value, Self::Error>;
}

/// One runtime tensor value handed to a Tiler region, opaque to Tiler.
///
/// The wrapper exists to carry two things a region cannot otherwise obtain: the
/// adapter, at the type level, and the context a result is constructed from. It
/// inspects neither the value nor the context, and neither enters graph
/// semantics or artifact identity — a region's identity is a function of its
/// declared program, and two invocations differing only in which tensors they
/// were handed are one program.
pub struct Tensor<A: TensorAdapter> {
    value: A::Value,
    context: A::Context,
}

impl<A: TensorAdapter> Tensor<A> {
    /// Wraps one of the integration's values together with its context.
    pub const fn new(value: A::Value, context: A::Context) -> Self {
        Self { value, context }
    }

    /// Borrows the wrapped value.
    pub const fn value(&self) -> &A::Value {
        &self.value
    }

    /// Borrows the context this value was wrapped with.
    pub const fn context(&self) -> &A::Context {
        &self.context
    }

    /// Unwraps to the integration's own value, discarding the context.
    pub fn into_value(self) -> A::Value {
        self.value
    }

    /// Unwraps to the value and the context.
    pub fn into_parts(self) -> (A::Value, A::Context) {
        (self.value, self.context)
    }
}

impl<A: TensorAdapter> fmt::Debug for Tensor<A>
where
    A::Value: fmt::Debug,
    A::Context: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Tensor")
            .field("value", &self.value)
            .field("context", &self.context)
            .finish()
    }
}

/// One operand's storage, named by the interface key the region declared it under.
///
/// A leaf descriptor over a borrow, so it is `Copy` and its accessors are the
/// whole of it. The key travels with the bytes because a device authority binds
/// storage to the artifact's *named* program inputs — `RoutedBinding::binding`
/// addresses a key, not a position — and matching by position would bind the
/// right bytes to the wrong slot for any interface whose declaration order and
/// canonical order differ.
#[derive(Clone, Copy, Debug)]
pub struct RegionOperand<'a> {
    key: &'static str,
    bytes: &'a [u8],
}

impl<'a> RegionOperand<'a> {
    /// Pairs one checked byte run with the key it was declared under.
    /// Crate-internal: the length check is what makes the run trustworthy, and a
    /// publicly constructible one would let it be skipped.
    pub(crate) const fn new(key: &'static str, bytes: &'a [u8]) -> Self {
        Self { key, bytes }
    }

    /// Returns the stable interface key the region declared this operand under.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        self.key
    }

    /// Returns the operand's dense row-major storage.
    ///
    /// Exactly the run [`AdapterCapability::DenseRowMajorStorage`] describes:
    /// elements innermost-axis-fastest, no offset, no stride, no padding, and a
    /// length already checked against the extents the adapter reported.
    #[must_use]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

/// Everything one delivering region asks its device authority to carry out.
///
/// # Why this is one value rather than several arguments
///
/// The three things an integration needs in order to build a runtime adapter for
/// *this* invocation become true together and are meaningless apart: the operand
/// bytes, the result's storage to write into, and the environment the producer
/// declared for the embedded artifact. Handing them over as one value is also
/// what lets the borrow be stated once — the adapter an integration builds from
/// this holds the region's storage for exactly the route's duration, and nothing
/// outlives it.
///
/// # The environment here is the *producer's*, and an adapter must say so
///
/// [`Self::declared_environment`] is the profile, backend family, and
/// representation the expansion recorded off the artifact it produced. It is
/// **not** an observation of this machine, and nothing in this crate can make
/// one: [ADR 0086](https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0086-require-attributable-or-attested-native-translation.md)
/// decides that native device translation of a payload during pipeline creation
/// is a capability fact whose authority is `Unknown` on every macOS row
/// currently observable, so no host earns the right to offer the profile.
///
/// An adapter that returns this value from
/// [`RuntimeAdapter::bind_execution_context`] is therefore routing on
/// **producer-declared equality, not host-earned eligibility**, and
/// [`crate::__private::PRODUCER_DECLARED_EQUALITY`] is the label it must report
/// beside any result. `prototypes/serial-sum-run` prints the same distinction in
/// the same words, and `crates/tiler/tests/labelled_diagnostic.rs` fails if the
/// two ever stop agreeing.
#[derive(Debug)]
pub struct RegionRequest<'region> {
    operands: Vec<RegionOperand<'region>>,
    result_key: &'static str,
    result: &'region mut [u8],
    declared: ExecutionEnvironment,
}

impl<'region> RegionRequest<'region> {
    /// Builds one request. Crate-internal: every field is derived from a region
    /// whose operands were already checked, and a publicly constructible one
    /// would be a second way to state what a region hands over.
    pub(crate) fn new(
        operands: Vec<RegionOperand<'region>>,
        result_key: &'static str,
        result: &'region mut [u8],
        declared: ExecutionEnvironment,
    ) -> Self {
        Self {
            operands,
            result_key,
            result,
            declared,
        }
    }

    /// Returns every operand's storage, in the order the region's interface
    /// names them.
    #[must_use]
    pub fn operands(&self) -> &[RegionOperand<'region>] {
        &self.operands
    }

    /// Returns one operand's storage by its interface key.
    ///
    /// The lookup a device authority actually needs: a routed binding names a
    /// program input by key, and this is what answers it.
    #[must_use]
    pub fn operand(&self, key: &str) -> Option<&[u8]> {
        self.operands
            .iter()
            .find(|operand| operand.key == key)
            .map(|operand| operand.bytes)
    }

    /// Returns the interface key the region declared its result under.
    #[must_use]
    pub const fn result_key(&self) -> &'static str {
        self.result_key
    }

    /// Returns how many bytes the result's storage holds.
    ///
    /// Readable without the exclusive borrow [`Self::result_mut`] takes, so an
    /// adapter can size its allocations while still holding the request shared.
    #[must_use]
    pub const fn result_len(&self) -> usize {
        self.result.len()
    }

    /// Returns the result's storage, for a dispatch to write into.
    ///
    /// Length-checked on the way in against the extents the region resolved, so
    /// an adapter comparing a routed binding's `accessible_bytes` against this
    /// slice is comparing against the value the caller will actually receive.
    #[must_use]
    pub fn result_mut(&mut self) -> &mut [u8] {
        self.result
    }

    /// Returns the environment the artifact's *producer* declared.
    ///
    /// See the type documentation: this is a producer's declaration, never a
    /// host observation.
    #[must_use]
    pub const fn declared_environment(&self) -> &ExecutionEnvironment {
        &self.declared
    }
}

/// The additional obligations a region that dispatches an embedded artifact places on an adapter.
///
/// Implemented by an integration alongside [`TensorAdapter`], and required only
/// by [`crate::__private::bind_route_and_build`] — the entry point generated for
/// a region whose `deliver` statement selected an artifact family. A consumer
/// whose regions are all fallback-only never writes this trait.
///
/// # Why the device authority is `tiler_runtime`'s trait and not a new one
///
/// [`RuntimeAdapter`] already *is* the accepted vocabulary for "a consumer's
/// statically linked executor for one backend and representation family", under
/// [ADR 0090](https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
/// row 12, and [`crate::runtime::adapter::route_with_adapter`] is the driver
/// that sequences the loader's comparisons against it. Minting a second executor
/// seam here would give the workspace two ways to say the same thing, and the
/// two would drift at exactly the obligations — payload validation before the
/// first device question, both device stages unconditional, the commit one-way —
/// that the existing one exists to fix in place.
///
/// So this trait adds no execution method at all. It adds the two byte-run
/// accessors the reservation named, and one factory that turns them into the
/// integration's own [`RuntimeAdapter`].
///
/// # Why a factory rather than a stored adapter
///
/// A [`Tensor`] is borrowed shared at a call site, so nothing reachable from it
/// can be borrowed mutably — and every [`RuntimeAdapter`] method takes
/// `&mut self`. Storing the authority in [`TensorAdapter::Context`] and lending
/// it out would therefore need interior mutability in every integration, for a
/// value whose whole lifetime is one invocation.
///
/// Building it per invocation removes the problem instead of routing around it,
/// and it is also what makes the region's storage reachable: the adapter is
/// constructed *from* [`RegionRequest`], so it holds the operands and the result
/// by borrow rather than receiving them through a channel the seam would
/// otherwise have to grow. `crates/tiler-runtime/tests/adapter_route` already
/// builds its adapter this way (`ScalarHostAdapter::new(&OPERANDS)`); this makes
/// the idiom the contract.
pub trait DispatchAdapter: TensorAdapter {
    /// Why this integration's device authority refused a route before it committed.
    ///
    /// Named here rather than left on [`Self::Dispatch`] so an outcome can be
    /// spelled without naming a lifetime: see [`Self::Dispatch`].
    type Refusal;

    /// Why a committed dispatch did not complete.
    ///
    /// Separate from [`Self::Refusal`] because ADR 0051 draws the line between
    /// them: a refusal arrives while a fallback is still permitted, and a
    /// failure arrives after the one-way commit and is reported rather than
    /// retried.
    type Failure;

    /// The integration's own runtime adapter for one region invocation.
    ///
    /// Generic over the region's lifetime because it borrows the region's
    /// storage. Its two error types are pinned to this trait's so that
    /// [`crate::__private::RouteOutcome`] can be named without a lifetime — an
    /// outcome outlives the borrow it reports on.
    type Dispatch<'region>: RuntimeAdapter<Refusal = Self::Refusal, Failure = Self::Failure>;

    /// Borrows one value's storage as the dense row-major byte run
    /// [`AdapterCapability::DenseRowMajorStorage`] claims it is.
    ///
    /// Called only for a region that declares the capability, which
    /// [`crate::__private::bind_region`] refuses an adapter for when it is not
    /// offered — so an adapter over strided views is never asked.
    ///
    /// # Errors
    ///
    /// Returns the integration's own error when the storage cannot be borrowed.
    /// Tiler carries it verbatim in [`BindError::Adapter`].
    fn storage(value: &Self::Value) -> Result<&[u8], Self::Error>;

    /// Borrows the same run of a value a dispatch writes into.
    ///
    /// Separate from [`Self::storage`] rather than one method returning a
    /// mutable borrow, because the operands are read through shared references a
    /// caller still holds and only the freshly built result is owned. One
    /// mutable accessor would make every operand require exclusive access the
    /// call site cannot give.
    ///
    /// # Errors
    ///
    /// Returns the integration's own error when the storage cannot be borrowed.
    fn storage_mut(value: &mut Self::Value) -> Result<&mut [u8], Self::Error>;

    /// Builds the device authority that will carry out one region invocation.
    ///
    /// Returning an error refuses the *region*, because a request whose storage
    /// could not be handed over is a failure of this integration rather than a
    /// route the loader declined. An integration that simply has no device on
    /// this build returns an adapter whose
    /// [`RuntimeAdapter::bind_execution_context`] refuses instead: that is a
    /// pre-commit refusal, the semantic fallback runs, and the region still
    /// produces its declared result.
    ///
    /// # Errors
    ///
    /// Returns the integration's own error, carried verbatim in
    /// [`BindError::Adapter`].
    fn dispatcher<'region>(
        context: &Self::Context,
        request: RegionRequest<'region>,
    ) -> Result<Self::Dispatch<'region>, Self::Error>;
}

/// One operand axis, named the way a diagnostic must name it.
///
/// A leaf value-data descriptor with no cross-field invariant, so its fields are
/// public per ADR 0074 convention 6. The interface key rather than a position:
/// "operand 1 axis 0" is not something a consumer can locate in a region they
/// wrote, and "`b` axis 0" is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperandAxis {
    /// Stable interface key of the operand.
    pub input: &'static str,
    /// Zero-based axis position within that operand's shape.
    pub axis: usize,
}

impl fmt::Display for OperandAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` axis {}", self.input, self.axis)
    }
}

/// Why a region refused the values it was handed.
///
/// Generic over the adapter's own error rather than unifying with it, per
/// ADR 0074 convention 1: a caller can still tell an integration's failure from
/// Tiler's refusal, and the integration's type survives to
/// [`Error::source`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BindError<E> {
    /// The region declares a different number of operands than were supplied.
    OperandCountMismatch {
        /// Operands the region declares.
        declared: usize,
        /// Operands the invocation supplied.
        supplied: usize,
    },
    /// The adapter does not offer a capability this region requires.
    UnsupportedCapability {
        /// The capability the region requires.
        capability: AdapterCapability,
    },
    /// The supplied value's rank is not the rank the region declared.
    RankMismatch {
        /// Stable interface key of the operand.
        input: &'static str,
        /// Rank the region declared.
        declared: usize,
        /// Rank the adapter reported.
        actual: usize,
    },
    /// The supplied value's stored scalar is not the one the region declared.
    StorageScalarMismatch {
        /// Stable interface key of the operand.
        input: &'static str,
        /// Scalar the region declared.
        declared: StorageScalar,
        /// Scalar the adapter reported.
        actual: StorageScalar,
    },
    /// The supplied value does not report an extent the region fixed literally.
    ///
    /// The literal counterpart of [`Self::InconsistentExtent`], and not a case
    /// of it: a symbolic extent is read from the operands and only has to agree
    /// with the other axes naming it, while a literal extent is a claim the
    /// region already made, so the supplied value is the side that must agree.
    /// An operand whose axes are all literal names no symbol and would
    /// therefore owe no obligation at all.
    ///
    /// Carries the axis rather than the operand alone, because one operand may
    /// fix several and naming only the operand would not say which to change.
    LiteralExtentMismatch {
        /// The axis whose extent the region fixed.
        axis: OperandAxis,
        /// The extent the region declared for it.
        declared: u64,
        /// The extent the adapter reported.
        actual: u64,
    },
    /// Two operand axes naming one symbol reported different extents.
    ///
    /// This is what operand unification means at runtime: `sym n` takes its
    /// value from one canonical axis, and every other axis naming `n` owes an
    /// equality. The error names both sides because neither alone tells a
    /// consumer which tensor to change.
    InconsistentExtent {
        /// The symbol whose occurrences disagreed.
        symbol: &'static str,
        /// The axis the value was taken from.
        source: OperandAxis,
        /// The extent that axis reported.
        source_extent: u64,
        /// The axis that owed an equality and did not meet it.
        conflicting: OperandAxis,
        /// The extent that axis reported.
        conflicting_extent: u64,
    },
    /// The region's own emitted facts are inconsistent.
    ///
    /// A defect in the expansion that produced them, never in the invocation. It
    /// is a typed refusal rather than a panic so a mistake in generated code
    /// fails closed at the call site instead of aborting the consumer's process.
    MalformedRegionFacts {
        /// What the facts asked for that they do not describe.
        detail: &'static str,
    },
    /// A value's byte run is not the length its own reported extents describe.
    ///
    /// The per-value half of [`AdapterCapability::DenseRowMajorStorage`]. The
    /// capability is a claim about the adapter, and this is the point at which
    /// the claim meets one value: a dense row-major run of these extents is
    /// exactly this many bytes, and a dispatch derives its launch geometry and
    /// its accessible ranges from the extents. Believing the claim over the
    /// length would hand a kernel a short buffer under a long shape, which is a
    /// read past the end rather than a wrong answer.
    ///
    /// Reachable only from a region that dispatches; nothing reads storage
    /// otherwise.
    StorageLengthMismatch {
        /// Stable interface key of the operand, or of the region's result.
        input: &'static str,
        /// Bytes the reported extents and stored scalar describe.
        declared: u64,
        /// Bytes the adapter's storage borrow actually holds.
        actual: u64,
    },
    /// A committed dispatch did not complete, and no fallback follows it.
    ///
    /// The one refusal here that is **not** a region contract failure. ADR 0051
    /// forbids selecting another plan after the routing commit, so a region
    /// whose dispatch failed may not quietly return the semantic fallback's
    /// value: the caller asked for a computation that was committed to and did
    /// not finish, and saying so is the only answer that is not a fabrication.
    ///
    /// The detail is rendered rather than carried typed, because the adapter's
    /// own failure type is [`DispatchAdapter::Failure`] and this enum is generic
    /// over [`TensorAdapter::Error`] alone — two unrelated parameters, and
    /// adding the second to every `BindError` a fallback-only consumer handles
    /// would charge every region for a type only a dispatching one can produce.
    DispatchFailed {
        /// The adapter's own account of what did not complete.
        detail: String,
    },
    /// The adapter failed, carrying its own error unchanged.
    Adapter(E),
}

impl<E: fmt::Display> fmt::Display for BindError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperandCountMismatch { declared, supplied } => write!(
                formatter,
                "tiler.bind.operand-count-mismatch: the region declares {declared} operands and \
                 {supplied} were supplied"
            ),
            Self::UnsupportedCapability { capability } => write!(
                formatter,
                "tiler.bind.unsupported-adapter-capability: this region requires `{capability}`, \
                 which the adapter does not offer"
            ),
            Self::RankMismatch {
                input,
                declared,
                actual,
            } => write!(
                formatter,
                "tiler.bind.rank-mismatch: operand `{input}` is declared with rank {declared} and \
                 the supplied value has rank {actual}"
            ),
            Self::StorageScalarMismatch {
                input,
                declared,
                actual,
            } => write!(
                formatter,
                "tiler.bind.storage-scalar-mismatch: operand `{input}` is declared as \
                 {declared:?} and the supplied value stores {actual:?}"
            ),
            Self::LiteralExtentMismatch {
                axis,
                declared,
                actual,
            } => write!(
                formatter,
                "tiler.bind.literal-extent-mismatch: {axis} is declared with extent {declared} and \
                 the supplied value reports {actual}"
            ),
            Self::InconsistentExtent {
                symbol,
                source,
                source_extent,
                conflicting,
                conflicting_extent,
            } => write!(
                formatter,
                "tiler.bind.inconsistent-extent: `{symbol}` is {source_extent} at {source} and \
                 {conflicting_extent} at {conflicting}"
            ),
            Self::MalformedRegionFacts { detail } => write!(
                formatter,
                "tiler.bind.malformed-region-facts: {detail}; this is a defect in the expansion \
                 that produced this region, not in the invocation"
            ),
            Self::StorageLengthMismatch {
                input,
                declared,
                actual,
            } => write!(
                formatter,
                "tiler.bind.storage-length-mismatch: `{input}` reports extents describing \
                 {declared} dense byte(s) and its storage holds {actual}"
            ),
            Self::DispatchFailed { detail } => write!(
                formatter,
                "tiler.bind.dispatch-failed: the committed route did not complete, and ADR 0051 \
                 permits no fallback after the commit: {detail}"
            ),
            Self::Adapter(source) => write!(formatter, "tiler.bind.adapter: {source}"),
        }
    }
}

impl<E: Error + 'static> Error for BindError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Adapter(source) => Some(source),
            Self::OperandCountMismatch { .. }
            | Self::UnsupportedCapability { .. }
            | Self::RankMismatch { .. }
            | Self::StorageScalarMismatch { .. }
            | Self::LiteralExtentMismatch { .. }
            | Self::InconsistentExtent { .. }
            | Self::MalformedRegionFacts { .. }
            | Self::StorageLengthMismatch { .. }
            | Self::DispatchFailed { .. } => None,
        }
    }
}
