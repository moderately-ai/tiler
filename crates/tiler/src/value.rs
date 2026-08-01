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
//! # What is deliberately absent
//!
//! **Storage access.** Nothing here yields a pointer, a buffer, a byte slice, or
//! a device object. The bounded profile this ticket delivers checks a binding;
//! it dispatches nothing, so a storage-access surface would be a public boundary
//! with no caller to review it against. [`AdapterCapability::DenseRowMajorStorage`]
//! is the reservation: an adapter states the storage property Tiler's first
//! dispatch profile will require, and an adapter that cannot state it is refused
//! now rather than read wrongly later.
//!
//! **Per-value storage properties.** A capability is a claim about the adapter,
//! not about one value. An integration whose values may be strided views is
//! expected to materialize before wrapping, or to decline the capability. Moving
//! density onto [`ValueMetadata`] is the widening path, and it is not taken here
//! because nothing yet consumes the distinction.

use std::error::Error;
use std::fmt;

pub use tiler_ir::program::StorageScalar;

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
            | Self::MalformedRegionFacts { .. } => None,
        }
    }
}
