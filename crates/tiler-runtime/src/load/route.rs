//! The preflight stage and the one-way routing commit.
//!
//! # The two stages are two types, so the order is not a convention
//!
//! ADR 0051 requires routing to commit one way, before program work, and
//! forbids falling back after it. That is enforced here by construction rather
//! than by documentation. Every obligation that can refuse lives in
//! [`super::DecodedProgram::preflight`], which returns a [`Preflight`] or a
//! typed rejection; [`Preflight::commit`] consumes that value and is
//! **infallible**. So there is nothing left that can fail after the commit, and
//! no way to hold both a committed route and an uncommitted one — the value the
//! fallback would have needed is gone.
//!
//! A caller that wants a fallback takes it by not calling [`Preflight::commit`],
//! which is exactly ADR 0051's "fallback only before program work".
//!
//! # What a committed route names
//!
//! Everything one dispatch needs, all of it read from the artifact's own bytes.
//! A [`RoutedDispatch`] names the carried object, the descriptor identifying it,
//! the backend entry symbol to look up inside it, the evaluated launch geometry,
//! and — per ABI slot — the backend transport it occupies, what it addresses,
//! and how many bytes must be reachable through it.
//!
//! **Four claims this module previously made are retracted.** It stated that a
//! committed route "does not name an entry symbol, a binding-to-buffer
//! correspondence, or an evaluated launch extent, because a decoded envelope
//! publishes none of those", and concluded that "a caller that does not hold the
//! program it compiled cannot dispatch from an artifact alone". All four were
//! true when written and none is true now:
//! [`DecodedEntry::backend_symbol`](tiler_artifact::program::DecodedEntry::backend_symbol),
//! [`DecodedEntry::transport_slots`](tiler_artifact::program::DecodedEntry::transport_slots),
//! [`DecodedBinding::target`](tiler_artifact::program::DecodedBinding::target),
//! and `DecodedExpr::evaluate` publish exactly those facts, and this module
//! routes through them.
//!
//! # Why the two stages publish different things
//!
//! A [`Preflight`] publishes what a caller *judges*: the identity, the
//! descriptor, the geometry, and the bindings. Those decide whether to commit at
//! all — a launch wider than the host's storage is a reason to abandon this
//! route, and abandoning is only permitted while this value is still held.
//!
//! The object bytes and the entry symbol are published only by
//! [`RoutedDispatch`], because they are what a caller *executes*. Reaching them
//! should require having made the decision rather than merely having considered
//! it, which makes "no program work before the commit" a property of the type
//! rather than a rule to remember.

use tiler_artifact::program::{
    ArtifactExecutionPolicy, BackendPayloadDescriptor, CanonicalArtifactProgramIdentity,
    DecodedBinding, DecodedEntry,
};

/// The evaluated launch geometry of one routed entry.
///
/// Scalars rather than expressions. The artifact carries formulas over its own
/// interface, and they are evaluated against the facts the host bound during
/// preflight — the only point at which an evaluation failure can still be
/// reported as a refusal instead of arriving after the routing commit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RoutedLaunch {
    pub(super) grid_threads: u64,
    pub(super) threads_per_workgroup: u64,
    pub(super) zero_work_skips_dispatch: bool,
}

impl RoutedLaunch {
    /// Returns the total number of threads this launch covers.
    #[must_use]
    pub const fn grid_threads(self) -> u64 {
        self.grid_threads
    }

    /// Returns the number of threads in one workgroup.
    #[must_use]
    pub const fn threads_per_workgroup(self) -> u64 {
        self.threads_per_workgroup
    }

    /// Returns whether a zero-thread launch is skipped rather than encoded.
    ///
    /// Returned rather than assumed. Encoding a zero-thread dispatch against a
    /// backend that refuses one would turn a well-formed empty launch into a
    /// submission failure.
    #[must_use]
    pub const fn zero_work_skips_dispatch(self) -> bool {
        self.zero_work_skips_dispatch
    }
}

/// One ABI binding of a routed entry: where it goes, and how large it must be.
///
/// The two facts the loader *derived* — the backend transport slot and the
/// evaluated byte range — are published beside the decoded binding they came
/// from rather than instead of it. A host needing the element type, address
/// space, or access mode reads them through [`Self::binding`]. Naming those
/// types here would give this crate a direct `tiler-ir` edge, and its dependency
/// closure is a decided property under ADR 0081 rather than an accident of
/// ordering.
#[derive(Clone, Copy, Debug)]
pub struct RoutedBinding<'a> {
    pub(super) binding: DecodedBinding<'a>,
    pub(super) transport: u32,
    pub(super) accessible_bytes: u64,
}

impl<'a> RoutedBinding<'a> {
    /// Returns the zero-based ABI slot, in the kernel signature's own order.
    #[must_use]
    pub fn slot(self) -> usize {
        self.binding.slot()
    }

    /// Returns the backend transport index this slot occupies.
    ///
    /// Deliberately not the same number as [`Self::slot`]: an artifact orders
    /// its bindings by the kernel signature, and a backend places them wherever
    /// its own argument table says. Collapsing the two would bind the right
    /// storage to the wrong index on any backend whose mapping is not the
    /// identity.
    #[must_use]
    pub const fn transport_slot(self) -> u32 {
        self.transport
    }

    /// Returns the minimum number of bytes reachable through this binding.
    ///
    /// Evaluated from the artifact's own accessible-range expression against the
    /// facts the host bound, so a host compares it against the storage it holds
    /// rather than re-deriving an extent the artifact already derived.
    #[must_use]
    pub const fn accessible_bytes(self) -> u64 {
        self.accessible_bytes
    }

    /// Returns the decoded binding this slot was routed from.
    ///
    /// Carries what the slot addresses — a named program input, a named program
    /// output, or entry-internal storage — with every declared storage fact the
    /// artifact records about it.
    #[must_use]
    pub const fn binding(self) -> DecodedBinding<'a> {
        self.binding
    }
}

/// One artifact that passed every obligation this loader can decide.
///
/// Deliberately neither [`Clone`] nor [`Copy`]. A route that could be duplicated
/// could be committed twice, and "committed once" is the property ADR 0051
/// asks for.
#[derive(Debug)]
#[must_use = "a preflight that is neither committed nor abandoned decides nothing"]
pub struct Preflight<'a> {
    pub(super) identity: CanonicalArtifactProgramIdentity,
    pub(super) kernel_program: &'a [u8],
    pub(super) payload: &'a BackendPayloadDescriptor,
    pub(super) object: &'a [u8],
    pub(super) entry: DecodedEntry<'a>,
    pub(super) symbol: &'a str,
    pub(super) launch: RoutedLaunch,
    pub(super) bindings: Vec<RoutedBinding<'a>>,
}

impl<'a> Preflight<'a> {
    /// Returns the identity of the artifact this route would execute.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the canonical identity of the kernel program this route runs.
    ///
    /// The identity alone; the program is not carried and cannot be rebuilt
    /// from an envelope. It is published before the commit because it is the
    /// strongest binding available to a caller that *does* hold the program it
    /// compiled: comparing it proves these bytes package that exact program,
    /// which no artifact identity from a sidecar can establish. A caller that
    /// holds no program ignores it, and has correspondingly less evidence.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.kernel_program
    }

    /// Returns the descriptor of the payload this route selected.
    #[must_use]
    pub const fn payload(&self) -> &'a BackendPayloadDescriptor {
        self.payload
    }

    /// Returns the evaluated launch geometry this route would encode.
    #[must_use]
    pub const fn launch(&self) -> RoutedLaunch {
        self.launch
    }

    /// Returns the routed ABI bindings in the kernel signature's own order.
    #[must_use]
    pub fn bindings(&self) -> &[RoutedBinding<'a>] {
        &self.bindings
    }

    /// Commits to executing this route. One way, and infallible.
    ///
    /// There is no `Result` here on purpose. Every decidable obligation was
    /// discharged by the preflight that produced this value, so a failure at
    /// this point would mean an obligation was checked in the wrong stage.
    /// Consuming `self` is what makes the commit one-way: the caller cannot
    /// afterwards hold this value to fall back to.
    #[must_use]
    pub fn commit(self) -> RoutedDispatch<'a> {
        let Self {
            identity,
            kernel_program,
            payload,
            object,
            entry,
            symbol,
            launch,
            bindings,
        } = self;
        RoutedDispatch {
            identity,
            kernel_program,
            payload,
            object,
            entry,
            symbol,
            launch,
            bindings,
        }
    }
}

/// A committed route: one entry, its object, and everything needed to encode it.
///
/// Reaching this type is the boundary ADR 0051 draws. Everything before it may
/// be abandoned for a fallback; everything after it is program work, and a
/// failure there is reported rather than retried on another route.
/// `Clone` here is deliberate and is not the permission [`Preflight`] withholds.
/// Cloning a route that is already committed cannot un-commit it or produce a
/// second choice; it only lets a host hand the committed decision to the code
/// that encodes it.
#[derive(Clone, Debug)]
pub struct RoutedDispatch<'a> {
    identity: CanonicalArtifactProgramIdentity,
    kernel_program: &'a [u8],
    payload: &'a BackendPayloadDescriptor,
    object: &'a [u8],
    entry: DecodedEntry<'a>,
    symbol: &'a str,
    launch: RoutedLaunch,
    bindings: Vec<RoutedBinding<'a>>,
}

impl<'a> RoutedDispatch<'a> {
    /// Returns the identity of the artifact being executed.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the canonical identity of the kernel program being executed.
    ///
    /// The identity alone; the program is not carried. Republished after the
    /// commit so a host can record *what* it ran beside the result, which is the
    /// value a numerical comparison needs to be attributable.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.kernel_program
    }

    /// Returns the descriptor of the payload this route committed to.
    #[must_use]
    pub const fn payload(&self) -> &'a BackendPayloadDescriptor {
        self.payload
    }

    /// Returns how the committed object reaches an executable state.
    ///
    /// Always [`ArtifactExecutionPolicy::NativeImage`] in this build — preflight
    /// refuses anything else — and returned rather than assumed, so a host does
    /// not hard-code the assumption at its own load site.
    #[must_use]
    pub const fn execution_policy(&self) -> ArtifactExecutionPolicy {
        self.payload.execution_policy
    }

    /// Returns the exact emitted object bytes the artifact carries.
    ///
    /// These are the bytes the producer handed to `push_carried_payload`,
    /// byte for byte: the envelope's framing is stripped by the decoder and the
    /// section body is the object itself. A host loads *these* and nothing it
    /// held before, which is what makes the envelope load-bearing rather than
    /// descriptive.
    #[must_use]
    pub const fn object(&self) -> &'a [u8] {
        self.object
    }

    /// Returns the backend's own entry-point symbol to look up in that object.
    ///
    /// Read from the carried payload's compilation subject, never supplied by
    /// the host. A host naming the symbol itself would be asserting which
    /// function the object contains, and that is a claim only the producer which
    /// compiled it can make.
    #[must_use]
    pub const fn entry_symbol(&self) -> &'a str {
        self.symbol
    }

    /// Returns the evaluated launch geometry to encode.
    #[must_use]
    pub const fn launch(&self) -> RoutedLaunch {
        self.launch
    }

    /// Returns the routed ABI bindings in the kernel signature's own order.
    #[must_use]
    pub fn bindings(&self) -> &[RoutedBinding<'a>] {
        &self.bindings
    }

    /// Returns the decoded entry this route committed to.
    ///
    /// The declared resource requirements and numerical realization hang off it,
    /// so a host reporting what it is about to run — or refusing a kernel whose
    /// declared numerics are not the ones it asked for — reads them here.
    #[must_use]
    pub const fn entry(&self) -> DecodedEntry<'a> {
        self.entry
    }
}
