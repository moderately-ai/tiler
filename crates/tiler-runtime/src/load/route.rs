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
///
/// The published range always starts at byte zero, and by refusal rather than
/// by silence: an artifact binding may start elsewhere, and `place_bindings`
/// rejects one that does as `UnpublishedBindingOffset`, because a host that
/// never learns an offset existed cannot tell that it defaulted.
/// `carry-the-binding-offset-through-the-runtime-route` owns publishing and
/// honouring it instead.
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

/// One entry of a routed variant, with everything its dispatch needs.
///
/// A route carries one of these per stage, in execution order. Every fact here
/// is per *entry* rather than per route, and that is not uniformity for its own
/// sake: nothing requires two entries of one variant to be realized by the same
/// payload, so the object, the symbol, and the descriptor are resolved and
/// checked for each. A loader that validated one entry's payload and executed
/// another's would be routing on a fact it never checked.
#[derive(Clone, Debug)]
pub struct RoutedEntry<'a> {
    pub(super) payload: &'a BackendPayloadDescriptor,
    pub(super) object: &'a [u8],
    pub(super) entry: DecodedEntry<'a>,
    pub(super) symbol: &'a str,
    pub(super) launch: RoutedLaunch,
    pub(super) bindings: Vec<RoutedBinding<'a>>,
}

impl<'a> RoutedEntry<'a> {
    /// Returns the descriptor of the payload realizing this entry.
    #[must_use]
    pub const fn payload(&self) -> &'a BackendPayloadDescriptor {
        self.payload
    }

    /// Returns the exact emitted object bytes this entry executes from.
    #[must_use]
    pub const fn object(&self) -> &'a [u8] {
        self.object
    }

    /// Returns the backend's own entry-point symbol to look up in that object.
    #[must_use]
    pub const fn entry_symbol(&self) -> &'a str {
        self.symbol
    }

    /// Returns the evaluated launch geometry this entry encodes.
    #[must_use]
    pub const fn launch(&self) -> RoutedLaunch {
        self.launch
    }

    /// Returns this entry's routed ABI bindings in the kernel signature's order.
    #[must_use]
    pub fn bindings(&self) -> &[RoutedBinding<'a>] {
        &self.bindings
    }

    /// Returns the decoded entry this was routed from.
    #[must_use]
    pub const fn entry(&self) -> DecodedEntry<'a> {
        self.entry
    }
}

/// Two ABI slots of two entries that must be backed by **one** allocation.
///
/// # Why a loader cannot work this out for itself
///
/// A binding addressing entry-internal storage carries no name — two `Internal`
/// slots are indistinguishable by design, because the artifact layer has no
/// durable name for a program value. So a loader allocating per binding gives
/// the consumer a *fresh* buffer, the producer's result never reaches it, and
/// the dispatch reads uninitialised device memory. That is a wrong answer rather
/// than a refusal, and it is the one place in this stack that would fail open.
///
/// The pairing is derived from the variant's own typed data dependencies, so it
/// states what the packaged program proved rather than what a loader guessed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedAllocation {
    pub(super) producer: EntrySlot,
    pub(super) consumer: EntrySlot,
}

impl SharedAllocation {
    /// Returns the entry and slot that writes the shared storage.
    #[must_use]
    pub const fn producer(self) -> EntrySlot {
        self.producer
    }

    /// Returns the entry and slot that reads it.
    #[must_use]
    pub const fn consumer(self) -> EntrySlot {
        self.consumer
    }
}

/// One ABI slot of one entry, both indices into the route's own execution order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntrySlot {
    pub(super) entry: usize,
    pub(super) slot: usize,
}

impl EntrySlot {
    /// Returns the position of the entry in the route's execution order.
    #[must_use]
    pub const fn entry(self) -> usize {
        self.entry
    }

    /// Returns the zero-based ABI slot within that entry.
    #[must_use]
    pub const fn slot(self) -> usize {
        self.slot
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
    pub(super) entries: Vec<RoutedEntry<'a>>,
    pub(super) shared: Vec<SharedAllocation>,
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

    /// Returns this route's entries **in the order they must be dispatched**.
    ///
    /// The variant's own execution order, not the entry table's canonical
    /// stage-key order. A caller dispatches this sequence front to back.
    #[must_use]
    pub fn entries(&self) -> &[RoutedEntry<'a>] {
        &self.entries
    }

    /// Returns the slot pairs that must be backed by one allocation each.
    ///
    /// Empty for a single-entry route. See [`SharedAllocation`] for why a loader
    /// cannot derive these from the bindings alone.
    #[must_use]
    pub fn shared_allocations(&self) -> &[SharedAllocation] {
        &self.shared
    }

    /// Commits to executing this route. One way, and infallible.
    ///
    /// There is no `Result` here on purpose. Every decidable obligation was
    /// discharged by the preflight that produced this value, so a failure at
    /// this point would mean an obligation was checked in the wrong stage.
    /// Consuming `self` is what makes the commit one-way: the caller cannot
    /// afterwards hold this value to fall back to.
    ///
    /// # The one-way property is checked by the compiler
    ///
    /// The three examples below are the evidence that ADR 0051's commit is
    /// structural here rather than a rule a caller is trusted to follow. Each
    /// is compiled by `cargo test`; the two negative ones pin the exact
    /// diagnostic, so a change that made either *compile* — or that made it
    /// fail for some unrelated reason — fails the gate.
    ///
    /// Committing once is the whole of what a caller may do with this value:
    ///
    /// ```
    /// use tiler_runtime::load::{Preflight, RoutedDispatch};
    ///
    /// fn route(preflight: Preflight<'_>) -> RoutedDispatch<'_> {
    ///     preflight.commit()
    /// }
    /// ```
    ///
    /// Committing a second time does not compile, because the first commit
    /// moved the value the second one would need (`E0382`). This is what
    /// "committed once" means:
    ///
    /// ```compile_fail,E0382
    /// use tiler_runtime::load::Preflight;
    ///
    /// fn commit_twice(preflight: Preflight<'_>) {
    ///     let _first = preflight.commit();
    ///     let _second = preflight.commit();
    /// }
    /// ```
    ///
    /// Keeping a spare to fall back to after committing does not compile
    /// either, because [`Preflight`] is deliberately not [`Clone`] (`E0277`).
    /// Without that, a caller could duplicate the route, commit one copy, and
    /// still hold an uncommitted one — which is exactly the state the commit
    /// exists to make unreachable:
    ///
    /// ```compile_fail,E0277
    /// use tiler_runtime::load::Preflight;
    ///
    /// fn duplicate<T: Clone>(value: T) -> (T, T) {
    ///     (value.clone(), value)
    /// }
    ///
    /// fn keep_a_fallback(preflight: Preflight<'_>) {
    ///     let (_spare, _route) = duplicate(preflight);
    /// }
    /// ```
    ///
    /// # Neither can a second authority be minted
    ///
    /// The three examples above all start from a `Preflight` a caller already
    /// holds, so on their own they prove only that *one* authority is
    /// single-use. They were the whole of the evidence until
    /// `make-runtime-routing-commit-authority-one-shot`, and they left the real
    /// hole open: a caller could mint a second authority from the program and
    /// commit that instead.
    ///
    /// Holding a committed route keeps the program exclusively borrowed, so
    /// preflighting it again does not compile (`E0499`):
    ///
    /// ```compile_fail,E0499
    /// use tiler_artifact::program::AbiFacts;
    /// use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment};
    ///
    /// fn commit_then_mint_another(
    ///     program: &mut DecodedProgram,
    ///     environment: &ExecutionEnvironment,
    ///     expected: &[u8],
    ///     facts: &AbiFacts,
    /// ) {
    ///     let route = program.preflight(environment, expected, facts).unwrap().commit();
    ///     let _second = program.preflight(environment, expected, facts);
    ///     let _still_held = route;
    /// }
    /// ```
    ///
    /// And the program cannot be duplicated to escape that borrow, because
    /// [`DecodedProgram`] is deliberately not [`Clone`] (`E0277`):
    ///
    /// ```compile_fail,E0277
    /// use tiler_runtime::load::DecodedProgram;
    ///
    /// fn duplicate<T: Clone>(value: T) -> (T, T) {
    ///     (value.clone(), value)
    /// }
    ///
    /// fn two_programs_one_artifact(program: DecodedProgram) {
    ///     let (_spare, _original) = duplicate(program);
    /// }
    /// ```
    ///
    /// [`DecodedProgram`]: super::DecodedProgram
    #[must_use]
    pub fn commit(self) -> RoutedDispatch<'a> {
        let Self {
            identity,
            kernel_program,
            entries,
            shared,
        } = self;
        RoutedDispatch {
            identity,
            kernel_program,
            entries,
            shared,
        }
    }
}

/// A committed route: every entry, in dispatch order, and what each one needs.
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
    entries: Vec<RoutedEntry<'a>>,
    shared: Vec<SharedAllocation>,
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

    /// Returns the committed entries **in the order they must be dispatched**.
    ///
    /// The same sequence [`Preflight::entries`] published, carried across the
    /// commit unchanged. A host encodes them front to back, and each carries its
    /// own object, symbol, launch geometry, and bindings.
    #[must_use]
    pub fn entries(&self) -> &[RoutedEntry<'a>] {
        &self.entries
    }

    /// Returns the slot pairs that must be backed by one allocation each.
    ///
    /// Empty for a single-entry route. See [`SharedAllocation`] for why a loader
    /// that ignored these would read uninitialised storage rather than refuse.
    #[must_use]
    pub fn shared_allocations(&self) -> &[SharedAllocation] {
        &self.shared
    }

    /// Returns how each committed object reaches an executable state.
    ///
    /// Always [`ArtifactExecutionPolicy::NativeImage`] in this build — preflight
    /// refuses anything else, for *every* entry — and returned rather than
    /// assumed, so a host does not hard-code the assumption at its own load site.
    ///
    /// Per entry rather than per route, because nothing requires two entries to
    /// name one payload and a single answer would be a claim about one of them.
    #[must_use]
    pub fn execution_policies(
        &self,
    ) -> impl ExactSizeIterator<Item = ArtifactExecutionPolicy> + '_ {
        self.entries
            .iter()
            .map(|entry| entry.payload.execution_policy)
    }
}
