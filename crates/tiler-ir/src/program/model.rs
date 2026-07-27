//! Kernel-program data model, read-only views, verified wrapper, and identity.
//!
//! The vocabulary is deliberately explicit: a consumer reads stages bound to
//! exact verified structured kernels, the semantic occurrences each stage
//! covers, materialized values with their origin, role, storage requirements
//! and defining stage, byte views through which stages address those values,
//! storage allocations with ownership, typed dependency edges that state *why*
//! two stages are ordered, and an ordered list of named outputs. Nothing here
//! requires a consumer to reconstruct a schedule, a kernel body, an access
//! relation, or a semantic graph.
//!
//! Only [`super::KernelProgramBuilder::build`] can bind a draft into an opaque
//! [`VerifiedKernelProgram`]. The verified wrapper exposes read-only meaning
//! and never mutation, thawing, or unchecked construction.

use std::fmt;

use crate::identity::{push_len, push_slice};
use crate::kernel::{KernelType, VerifiedKernel};
use crate::schedule::TensorRole;
use crate::semantic::{InputKey, OutputKey, SemanticGraphIdentity};
use crate::shape::Shape;

use super::MAX_PROGRAM_IDENTITY_BYTES;
use super::abi::{AbiArenaTraversal, ExprNode, canonical_arena_traversal};
use super::error::KernelProgramDiagnostic;
use super::handles::{AbiExprId, ViewId};

/// Converts a stored compact arena ordinal into a host index.
///
/// Every stored ordinal was minted from a `usize` length by a checked handle
/// constructor, so the conversion cannot fail on a supported host.
fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Converts a host index of a verified arena into its compact ordinal.
///
/// Every verified arena was bounded by a governed limit far below `u32::MAX`
/// during construction, so the conversion cannot fail.
fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("a bounded verified arena fits u32")
}

/// A graph-local ordinal of one operation of the bound semantic program.
///
/// The ordinal is the *occurrence*: it names where in one exact graph an
/// implementation is bound, exactly as `RegionOccurrenceIdentity` does for a
/// region. It is meaningful only alongside the
/// [`SemanticGraphIdentity`] encoded with it, and it is never a substitute for
/// semantic-graph meaning.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticOccurrence(u32);

impl SemanticOccurrence {
    /// Wraps a graph-local operation ordinal.
    #[must_use]
    pub const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }

    /// Returns the graph-local operation ordinal.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The program role one materialized value plays.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueRole {
    /// An externally bound program input.
    Input,
    /// An internally produced value consumed by a later stage.
    Temporary,
    /// An internally produced value published as a named program output.
    Output,
}

impl ValueRole {
    const fn tag(self) -> u8 {
        match self {
            Self::Input => 0x01,
            Self::Temporary => 0x02,
            Self::Output => 0x03,
        }
    }

    /// Returns the scheduled boundary tensor role this program role realizes.
    #[must_use]
    pub const fn tensor_role(self) -> TensorRole {
        match self {
            Self::Input => TensorRole::Input,
            Self::Temporary => TensorRole::Intermediate,
            Self::Output => TensorRole::Output,
        }
    }
}

/// A governed memory domain in which program storage lives.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemorySpace {
    /// Device-visible memory that outlives one dispatch.
    Device,
}

impl MemorySpace {
    const fn tag(self) -> u8 {
        match self {
            Self::Device => 0x01,
        }
    }
}

/// Who owns the bytes of one program storage allocation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AllocationOwnership {
    /// Storage bound by the caller; the program never allocates or frees it.
    External,
    /// Storage the program acquires after routing commit and retains itself.
    Program,
}

impl AllocationOwnership {
    const fn tag(self) -> u8 {
        match self {
            Self::External => 0x01,
            Self::Program => 0x02,
        }
    }
}

/// Whether one stage access reads or writes the value it addresses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StageAccessMode {
    /// The stage reads through the view.
    Read,
    /// The stage fully initializes the addressed range.
    Write,
}

impl StageAccessMode {
    const fn tag(self) -> u8 {
        match self {
            Self::Read => 0x01,
            Self::Write => 0x02,
        }
    }
}

/// Where one materialized value comes from.
///
/// The origin answers a different question from the role: it says whether the
/// bytes enter the program across the public interface or are produced inside
/// it. It deliberately does not claim which semantic value a temporary
/// realizes; proving that is compiler-owned refinement evidence, not
/// target-neutral program structure.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MaterializedOrigin {
    /// The value is the externally bound tensor of one named program input.
    ProgramInput {
        /// Interface key of the bound input.
        key: InputKey,
    },
    /// The value is produced by a stage of this program.
    Internal,
}

/// A byte range of one materialized value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteWindow {
    /// First addressed byte, relative to the value's own storage.
    pub offset: u64,
    /// Addressed byte count.
    pub length: u64,
}

/// One stage access: the view it addresses, whether it reads or writes, and the
/// ABI expression computing the byte count the entry may address through it.
///
/// The accessible range travels with the access rather than beside it: a
/// consumer that binds a buffer needs the range for exactly the access it is
/// binding, and a parallel list would let the two drift apart.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StageAccess {
    /// View the stage addresses.
    pub view: ViewId,
    /// Whether the stage reads or writes through it.
    pub mode: StageAccessMode,
    /// ABI expression computing the addressable byte count of this access.
    pub accessible_bytes: AbiExprId,
}

/// The launch geometry one program stage's entry declares.
///
/// Both are ABI expressions rather than resolved numbers because a program
/// whose extents are not yet known must still state how its launch is computed;
/// the bounded static-shape profile simply resolves them at compile time.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StageLaunch {
    /// ABI expression computing the total launch thread count.
    pub grid_threads: AbiExprId,
    /// ABI expression computing the per-workgroup thread count.
    pub threads_per_workgroup: AbiExprId,
}

/// One ordered state of a program's routing-commit lifecycle.
///
/// This is the per-program contract ADR 0072 lists beside ABI and guards, and
/// the one `AGENTS.md` states as "preflight before routing commit, fallback
/// only before program work". It is **not**
/// `tiler_artifact::program::RoutingPolicy`, which orders the variants of a
/// portfolio against each other: a rank is a relation among variants, and one
/// program in isolation has no rank to carry. The two concepts share the word
/// "routing" and nothing else.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RoutingCommitState {
    /// Applicability and feasibility are still being decided; no work is owned.
    Preflight,
    /// This program is the committed choice; alternatives are no longer live.
    Committed,
    /// The program's work has been submitted to a device.
    Executing,
    /// The program's results have been published to the caller.
    Published,
}

impl RoutingCommitState {
    /// Returns the governed wire tag of this variant.
    ///
    /// Written by an exhaustive match rather than read from the discriminant,
    /// so inserting or reordering a variant is a build error here instead of a
    /// silent re-encoding of every program identity ever produced (ADR 0074
    /// convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Preflight => 0x01,
            Self::Committed => 0x02,
            Self::Executing => 0x03,
            Self::Published => 0x04,
        }
    }

    /// Returns the state this one advances to, or `None` for the final state.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Preflight => Some(Self::Committed),
            Self::Committed => Some(Self::Executing),
            Self::Executing => Some(Self::Published),
            Self::Published => None,
        }
    }
}

impl fmt::Display for RoutingCommitState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// One declared step of a program's routing-commit lifecycle.
///
/// A program declares the whole ordered chain and, for each step, whether
/// falling back to another program is still permitted while taking it. The
/// verifier proves the chain is complete and that only the step leaving
/// [`RoutingCommitState::Preflight`] may permit fallback, so a producer states
/// its own fallback intent and cannot state an unsound one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RoutingCommitTransition {
    /// State the step leaves.
    pub from: RoutingCommitState,
    /// State the step reaches.
    pub to: RoutingCommitState,
    /// Whether abandoning this program for another is permitted at this step.
    pub fallback_permitted: bool,
}

/// The complete ordered routing-commit lifecycle every program must span.
pub(super) const ROUTING_COMMIT_TRANSITIONS: usize = 3;

/// The declared facts of one materialized program value.
///
/// The required byte count is derived from the shape and element type rather
/// than declared, so a producer cannot claim a size its own shape contradicts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializedValueSpec {
    /// Where the value comes from.
    pub origin: MaterializedOrigin,
    /// Program role of the value.
    pub role: ValueRole,
    /// Logical tensor shape of the value.
    pub shape: Shape,
    /// Element type stored in the value's bytes.
    pub element_type: KernelType,
    /// Byte alignment the value requires.
    pub alignment: u32,
    /// Memory domain the value lives in.
    pub memory_space: MemorySpace,
}

/// The declared facts of one program storage allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllocationSpec {
    /// Bytes the allocation provides.
    pub capacity_bytes: u64,
    /// Byte alignment the allocation guarantees.
    pub alignment: u32,
    /// Memory domain the allocation lives in.
    pub memory_space: MemorySpace,
    /// Who owns the bytes.
    pub ownership: AllocationOwnership,
}

/// Returns the byte width of one structured-kernel element type.
pub(super) const fn element_bytes(element_type: KernelType) -> u64 {
    match element_type {
        KernelType::Bool => 1,
        KernelType::Index => 8,
        KernelType::F32 => 4,
    }
}

/// Storage for one program stage.
#[derive(Clone, Debug)]
pub(super) struct StageData {
    pub(super) kernel: VerifiedKernel,
    /// Covered occurrences in ascending order.
    pub(super) coverage: Vec<SemanticOccurrence>,
    pub(super) accesses: Vec<StageAccessData>,
    pub(super) launch: StageLaunchData,
}

/// Storage for one stage access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StageAccessData {
    pub(super) view: u32,
    pub(super) mode: StageAccessMode,
    pub(super) accessible_bytes: u32,
}

/// Storage for one stage's declared launch geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StageLaunchData {
    pub(super) grid_threads: u32,
    pub(super) threads_per_workgroup: u32,
}

/// Storage for one materialized program value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MaterializedValueData {
    pub(super) origin: MaterializedOrigin,
    pub(super) role: ValueRole,
    pub(super) shape: Shape,
    pub(super) element_type: KernelType,
    pub(super) required_bytes: u64,
    pub(super) alignment: u32,
    pub(super) memory_space: MemorySpace,
    pub(super) allocation: u32,
}

/// Storage for one byte view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ViewData {
    pub(super) value: u32,
    pub(super) window: ByteWindow,
}

/// Storage for one allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AllocationData {
    pub(super) capacity_bytes: u64,
    pub(super) alignment: u32,
    pub(super) memory_space: MemorySpace,
    pub(super) ownership: AllocationOwnership,
}

/// Storage for one typed dependency edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DependencyData {
    pub(super) predecessor: u32,
    pub(super) successor: u32,
    pub(super) reason: DependencyReasonData,
}

/// Storage for why two stages are ordered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DependencyReasonData {
    /// The successor reads a value the predecessor defines.
    Data(u32),
    /// The successor reuses storage the predecessor's value released.
    StorageHandoff(u32),
}

/// Storage for one named program output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProgramOutputData {
    pub(super) key: OutputKey,
    pub(super) value: u32,
}

/// The assembled, not-yet-verified kernel program.
#[derive(Clone, Debug)]
pub(super) struct KernelProgramData {
    pub(super) semantic_graph: SemanticGraphIdentity,
    pub(super) stages: Vec<StageData>,
    pub(super) values: Vec<MaterializedValueData>,
    pub(super) views: Vec<ViewData>,
    pub(super) allocations: Vec<AllocationData>,
    pub(super) dependencies: Vec<DependencyData>,
    pub(super) outputs: Vec<ProgramOutputData>,
    /// The shared ABI expression arena, in canonical arena order.
    pub(super) abi_expressions: Vec<ExprNode>,
    /// Arena position of the applicability guard, absent until declared.
    pub(super) applicability_guard: Option<u32>,
    /// The ordered routing-commit lifecycle this program declares.
    pub(super) routing_commit: Vec<RoutingCommitTransition>,
}

/// The stage and ordered write position that fully initializes one value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ValueDefinition {
    pub(super) stage: u32,
    /// Ordered position among that stage's write accesses.
    pub(super) write_position: u32,
}

/// Facts the whole-program verifier derives and the verified product retains.
#[derive(Clone, Debug)]
pub(super) struct DerivedProgramFacts {
    /// Defining stage of each value, absent for an externally bound input.
    pub(super) definitions: Vec<Option<ValueDefinition>>,
    /// Canonical topological stage order of the single-stream execution profile.
    pub(super) execution_order: Vec<u32>,
}

/// Opaque canonical bytes identifying one verified kernel program.
///
/// The identity folds every ADR 0072 layer a complete program owns: the
/// canonical [`SemanticGraphIdentity`] of the program it realizes, the exact
/// bound implementation of every stage through each stage's
/// [`CanonicalKernelIdentity`](crate::kernel::CanonicalKernelIdentity) (which
/// itself folds the canonical scheduled region it refines), the complete
/// semantic coverage those stages claim, the materializations, buffers, typed
/// dependencies and named outputs that structure them, the entry ABI, the
/// applicability guard, and the routing-commit lifecycle.
///
/// The ABI expression arena is folded *transitively*: every use site is encoded
/// by its canonical content key, a content key names the node's whole subtree,
/// and whole-program verification rejects an arena node no use site reaches. So
/// no retained expression escapes identity and no node is encoded twice.
///
/// It excludes every transient ordinal: builder insertion order, the program's
/// own stage/value/view/allocation/arena positions, and the planning `RegionId`
/// already excluded by the kernel and schedule identities. Cross-references are
/// encoded by canonical content key, never by position, so two structurally
/// equal programs assembled in different orders share bytes.
///
/// It still deliberately excludes what a later artifact-facing projection owns:
/// packaged admission, selected-provider provenance, the wire encoding of the
/// ABI, and a portfolio's variant priority.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalKernelProgramIdentity(Vec<u8>);

impl CanonicalKernelProgramIdentity {
    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An immutable, verified target-neutral kernel program.
///
/// Only [`super::KernelProgramBuilder::build`] produces one. Equality compares
/// the canonical identity, so two independently assembled programs with the
/// same semantic subject, bound implementations, coverage, and structure
/// compare equal even though their builder ownership tags differ.
#[derive(Clone, Debug)]
pub struct VerifiedKernelProgram {
    pub(super) data: KernelProgramData,
    pub(super) derived: DerivedProgramFacts,
    pub(super) identity: CanonicalKernelProgramIdentity,
}

impl PartialEq for VerifiedKernelProgram {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for VerifiedKernelProgram {}

impl VerifiedKernelProgram {
    /// Returns the canonical graph identity of the semantic program realized.
    #[must_use]
    pub const fn semantic_graph_identity(&self) -> &SemanticGraphIdentity {
        &self.data.semantic_graph
    }

    /// Returns the canonical structural identity of this program.
    #[must_use]
    pub const fn canonical_identity(&self) -> &CanonicalKernelProgramIdentity {
        &self.identity
    }

    /// Returns the program stages in declaration order.
    #[must_use]
    pub fn stages(&self) -> impl ExactSizeIterator<Item = StageRef<'_>> {
        (0..self.data.stages.len()).map(move |index| StageRef {
            program: self,
            stage: index,
        })
    }

    /// Returns the stages in canonical single-stream execution order.
    ///
    /// The order is a deterministic topological order of the dependency graph,
    /// broken by canonical stage content rather than by insertion. It is an
    /// execution-profile guarantee, not additional dependency information: a
    /// correct executor must preserve every typed dependency, and incomparable
    /// stages are not promised to run concurrently in this profile.
    #[must_use]
    pub fn execution_order(&self) -> impl ExactSizeIterator<Item = StageRef<'_>> {
        self.derived
            .execution_order
            .iter()
            .map(move |stage| StageRef {
                program: self,
                stage: position(*stage),
            })
    }

    /// Returns the materialized program values in declaration order.
    #[must_use]
    pub fn values(&self) -> impl ExactSizeIterator<Item = MaterializedValueRef<'_>> {
        (0..self.data.values.len()).map(move |index| MaterializedValueRef {
            program: self,
            value: index,
        })
    }

    /// Returns the declared byte views in declaration order.
    #[must_use]
    pub fn views(&self) -> impl ExactSizeIterator<Item = ViewRef<'_>> {
        (0..self.data.views.len()).map(move |index| ViewRef {
            program: self,
            view: index,
        })
    }

    /// Returns the program storage allocations in declaration order.
    #[must_use]
    pub fn allocations(&self) -> impl ExactSizeIterator<Item = AllocationRef<'_>> {
        (0..self.data.allocations.len()).map(move |index| AllocationRef {
            program: self,
            allocation: index,
        })
    }

    /// Returns the typed dependency edges in declaration order.
    #[must_use]
    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = DependencyRef<'_>> {
        (0..self.data.dependencies.len()).map(move |index| DependencyRef {
            program: self,
            dependency: index,
        })
    }

    /// Returns the ordered named program outputs.
    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = ProgramOutputRef<'_>> {
        (0..self.data.outputs.len()).map(move |index| ProgramOutputRef {
            program: self,
            output: index,
        })
    }

    /// Returns the ABI expression arena in canonical arena order.
    ///
    /// Every operand position is strictly smaller than the node naming it, so a
    /// consumer replaying the arena front to back always has its operands
    /// already minted.
    #[must_use]
    pub fn abi_expressions(&self) -> &[ExprNode] {
        &self.data.abi_expressions
    }

    /// Returns the arena position of the guard deciding whether to route here.
    ///
    /// The position indexes [`Self::abi_expressions`].
    ///
    /// # Panics
    ///
    /// Panics when the program declares no guard, which whole-program
    /// verification rejects as
    /// [`KernelProgramDiagnostic::MissingApplicabilityGuard`] before a verified
    /// program exists. A panic here therefore means an unverified program was
    /// constructed, not that a caller supplied a bad value.
    #[must_use]
    pub fn applicability_guard(&self) -> u32 {
        self.data
            .applicability_guard
            .expect("verification proves a verified program declares its applicability guard")
    }

    /// Returns the declared routing-commit lifecycle in lifecycle order.
    #[must_use]
    pub fn routing_commit_contract(&self) -> &[RoutingCommitTransition] {
        &self.data.routing_commit
    }
}

/// A read-only view of one program stage.
#[derive(Clone, Copy, Debug)]
pub struct StageRef<'a> {
    program: &'a VerifiedKernelProgram,
    stage: usize,
}

impl<'a> StageRef<'a> {
    /// Returns the exact verified structured kernel this stage dispatches.
    #[must_use]
    pub fn kernel(self) -> &'a VerifiedKernel {
        &self.data().kernel
    }

    /// Returns the semantic occurrences this stage covers, in ascending order.
    #[must_use]
    pub fn coverage(self) -> &'a [SemanticOccurrence] {
        &self.data().coverage
    }

    /// Returns the launch geometry this stage's entry declares.
    ///
    /// Both positions index
    /// [`VerifiedKernelProgram::abi_expressions`].
    #[must_use]
    pub fn launch(self) -> StageLaunchView {
        let launch = self.data().launch;
        StageLaunchView {
            grid_threads: launch.grid_threads,
            threads_per_workgroup: launch.threads_per_workgroup,
        }
    }

    /// Returns the stage's accesses in kernel buffer-parameter order.
    #[must_use]
    pub fn accesses(self) -> impl ExactSizeIterator<Item = StageAccessRef<'a>> {
        let program = self.program;
        let stage = self.stage;
        (0..self.data().accesses.len()).map(move |position| StageAccessRef {
            program,
            stage,
            position,
        })
    }

    fn data(self) -> &'a StageData {
        &self.program.data.stages[self.stage]
    }
}

impl PartialEq for StageRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.program, other.program) && self.stage == other.stage
    }
}

impl Eq for StageRef<'_> {}

/// A read-only view of one stage access.
#[derive(Clone, Copy, Debug)]
pub struct StageAccessRef<'a> {
    program: &'a VerifiedKernelProgram,
    stage: usize,
    position: usize,
}

impl<'a> StageAccessRef<'a> {
    /// Returns the view this access addresses.
    #[must_use]
    pub fn view(self) -> ViewRef<'a> {
        ViewRef {
            program: self.program,
            view: position(self.data().view),
        }
    }

    /// Returns whether the access reads or fully initializes the range.
    #[must_use]
    pub fn mode(self) -> StageAccessMode {
        self.data().mode
    }

    /// Returns the arena position of this access's accessible-byte expression.
    ///
    /// The position indexes [`VerifiedKernelProgram::abi_expressions`].
    #[must_use]
    pub fn accessible_bytes(self) -> u32 {
        self.data().accessible_bytes
    }

    fn data(self) -> StageAccessData {
        self.program.data.stages[self.stage].accesses[self.position]
    }
}

/// A read-only view of one stage's declared launch geometry.
///
/// Each field is an arena position into
/// [`VerifiedKernelProgram::abi_expressions`], never a resolved number: a
/// consumer that resolved the geometry itself would be a second derivation of
/// what the program already decided.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StageLaunchView {
    /// Arena position of the total launch thread count.
    pub grid_threads: u32,
    /// Arena position of the per-workgroup thread count.
    pub threads_per_workgroup: u32,
}

/// A read-only view of one byte view of a materialized value.
#[derive(Clone, Copy, Debug)]
pub struct ViewRef<'a> {
    program: &'a VerifiedKernelProgram,
    view: usize,
}

impl<'a> ViewRef<'a> {
    /// Returns the materialized value this view addresses.
    #[must_use]
    pub fn value(self) -> MaterializedValueRef<'a> {
        MaterializedValueRef {
            program: self.program,
            value: position(self.data().value),
        }
    }

    /// Returns the addressed byte range within that value.
    #[must_use]
    pub fn window(self) -> ByteWindow {
        self.data().window
    }

    fn data(self) -> ViewData {
        self.program.data.views[self.view]
    }
}

/// A read-only view of one materialized program value.
#[derive(Clone, Copy, Debug)]
pub struct MaterializedValueRef<'a> {
    program: &'a VerifiedKernelProgram,
    value: usize,
}

impl<'a> MaterializedValueRef<'a> {
    /// Returns where the value comes from.
    #[must_use]
    pub fn origin(self) -> &'a MaterializedOrigin {
        &self.data().origin
    }

    /// Returns the program role of the value.
    #[must_use]
    pub fn role(self) -> ValueRole {
        self.data().role
    }

    /// Returns the logical tensor shape.
    #[must_use]
    pub fn shape(self) -> &'a Shape {
        &self.data().shape
    }

    /// Returns the element type stored in the value's bytes.
    #[must_use]
    pub fn element_type(self) -> KernelType {
        self.data().element_type
    }

    /// Returns the derived byte count the value requires.
    #[must_use]
    pub fn required_bytes(self) -> u64 {
        self.data().required_bytes
    }

    /// Returns the byte alignment the value requires.
    #[must_use]
    pub fn alignment(self) -> u32 {
        self.data().alignment
    }

    /// Returns the memory domain the value lives in.
    #[must_use]
    pub fn memory_space(self) -> MemorySpace {
        self.data().memory_space
    }

    /// Returns the allocation that owns the value's bytes.
    #[must_use]
    pub fn allocation(self) -> AllocationRef<'a> {
        AllocationRef {
            program: self.program,
            allocation: position(self.data().allocation),
        }
    }

    /// Returns the stage that fully initializes the value, if any.
    ///
    /// An externally bound program input has no defining stage.
    #[must_use]
    pub fn definition(self) -> Option<StageRef<'a>> {
        self.program.derived.definitions[self.value].map(|definition| StageRef {
            program: self.program,
            stage: position(definition.stage),
        })
    }

    fn data(self) -> &'a MaterializedValueData {
        &self.program.data.values[self.value]
    }
}

impl PartialEq for MaterializedValueRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.program, other.program) && self.value == other.value
    }
}

impl Eq for MaterializedValueRef<'_> {}

/// A read-only view of one program storage allocation.
#[derive(Clone, Copy, Debug)]
pub struct AllocationRef<'a> {
    program: &'a VerifiedKernelProgram,
    allocation: usize,
}

impl<'a> AllocationRef<'a> {
    /// Returns the bytes the allocation provides.
    #[must_use]
    pub fn capacity_bytes(self) -> u64 {
        self.data().capacity_bytes
    }

    /// Returns the byte alignment the allocation guarantees.
    #[must_use]
    pub fn alignment(self) -> u32 {
        self.data().alignment
    }

    /// Returns the memory domain the allocation lives in.
    #[must_use]
    pub fn memory_space(self) -> MemorySpace {
        self.data().memory_space
    }

    /// Returns who owns the bytes.
    #[must_use]
    pub fn ownership(self) -> AllocationOwnership {
        self.data().ownership
    }

    /// Returns the materialized values bound to this allocation.
    pub fn values(self) -> impl Iterator<Item = MaterializedValueRef<'a>> {
        let program = self.program;
        let allocation = ordinal(self.allocation);
        program
            .data
            .values
            .iter()
            .enumerate()
            .filter(move |(_, value)| value.allocation == allocation)
            .map(move |(index, _)| MaterializedValueRef {
                program,
                value: index,
            })
    }

    fn data(self) -> AllocationData {
        self.program.data.allocations[self.allocation]
    }
}

impl PartialEq for AllocationRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.program, other.program) && self.allocation == other.allocation
    }
}

impl Eq for AllocationRef<'_> {}

/// A read-only view of one typed dependency edge.
#[derive(Clone, Copy, Debug)]
pub struct DependencyRef<'a> {
    program: &'a VerifiedKernelProgram,
    dependency: usize,
}

impl<'a> DependencyRef<'a> {
    /// Returns the stage that must precede.
    #[must_use]
    pub fn predecessor(self) -> StageRef<'a> {
        StageRef {
            program: self.program,
            stage: position(self.data().predecessor),
        }
    }

    /// Returns the stage that must follow.
    #[must_use]
    pub fn successor(self) -> StageRef<'a> {
        StageRef {
            program: self.program,
            stage: position(self.data().successor),
        }
    }

    /// Returns why the two stages are ordered.
    #[must_use]
    pub fn reason(self) -> DependencyReasonView<'a> {
        match self.data().reason {
            DependencyReasonData::Data(value) => DependencyReasonView::Data(MaterializedValueRef {
                program: self.program,
                value: position(value),
            }),
            DependencyReasonData::StorageHandoff(allocation) => {
                DependencyReasonView::StorageHandoff(AllocationRef {
                    program: self.program,
                    allocation: position(allocation),
                })
            }
        }
    }

    fn data(self) -> DependencyData {
        self.program.data.dependencies[self.dependency]
    }
}

/// The typed reason one stage must precede another.
///
/// An accidental order between otherwise independent stages is not a reusable
/// correctness proof, so every edge names the obligation it discharges.
#[derive(Clone, Copy, Debug)]
pub enum DependencyReasonView<'a> {
    /// The successor reads a value the predecessor fully initializes.
    Data(MaterializedValueRef<'a>),
    /// The successor reuses storage whose previous value the predecessor released.
    StorageHandoff(AllocationRef<'a>),
}

/// A read-only view of one named program output.
#[derive(Clone, Copy, Debug)]
pub struct ProgramOutputRef<'a> {
    program: &'a VerifiedKernelProgram,
    output: usize,
}

impl<'a> ProgramOutputRef<'a> {
    /// Returns the stable interface key of the output.
    #[must_use]
    pub fn key(self) -> &'a OutputKey {
        &self.data().key
    }

    /// Returns the materialized value published under that key.
    #[must_use]
    pub fn value(self) -> MaterializedValueRef<'a> {
        MaterializedValueRef {
            program: self.program,
            value: position(self.data().value),
        }
    }

    fn data(self) -> &'a ProgramOutputData {
        &self.program.data.outputs[self.output]
    }
}

/// Cross-reference key domains, unchanged at `v1`.
///
/// These keys are what dependency edges, value definitions and allocation
/// bindings name each other by. `complete-program-identity-with-abi-guards-and-routing`
/// added the entry ABI, the applicability guard and the routing-commit contract
/// to *program* identity without changing what any of these keys means: a stage
/// is still identified by the implementation it binds and the occurrences it
/// covers, and its launch geometry is folded beside it in the program encoding
/// rather than into the key other entities cross-reference it by.
const STAGE_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.stage.v1\0";
const VALUE_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.value.v1\0";
const VIEW_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.view.v1\0";
const ALLOCATION_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.allocation.v1\0";
/// Program identity domain, bumped to `v3`.
///
/// `v2` folded the semantic graph, bound implementations, coverage, program
/// structure, the entry ABI, the applicability guard and the routing-commit
/// contract — and `v3` folds exactly the same subject. What changed is the
/// *encoding* of the ABI expressions inside it, and that is precisely why the
/// tag steps: every program ever encoded maps to different bytes now, so a
/// cache or artifact holding a `v2` identity must miss rather than match.
///
/// `v2` named each use site's expression by [`expr_key`], a standalone content
/// key that embeds its operands' keys. A key therefore restated its whole
/// subtree, so an identity was quadratic in arena size along a chain and
/// doubled per level wherever one node was shared — a five-operation program
/// measured 13,623 bytes. `v3` writes the reachable arena once, in the
/// canonical order of the use sites that reach it, and each use site names its
/// expression by canonical position. Injectivity is unchanged: the arena
/// section determines the whole DAG including its sharing, and an 8-byte
/// canonical position determines which node a use site means.
///
/// The change is deliberate and its cost is stated: no external consumer holds
/// a `v2` identity, so invalidating every artifact identity and cache entry
/// costs a rebuild rather than a migration. See
/// `encode-abi-expression-identity-in-linear-space`.
const PROGRAM_DOMAIN: &[u8] = b"tiler.kernel-program.v3\0";

fn push_shape(bytes: &mut Vec<u8>, shape: &Shape) {
    push_len(bytes, shape.rank());
    for extent in shape.extents() {
        bytes.extend_from_slice(&extent.get().to_be_bytes());
    }
}

fn push_element_type(bytes: &mut Vec<u8>, element_type: KernelType) {
    bytes.push(match element_type {
        KernelType::Bool => 0x01,
        KernelType::Index => 0x02,
        KernelType::F32 => 0x03,
    });
}

fn push_origin(bytes: &mut Vec<u8>, origin: &MaterializedOrigin) {
    match origin {
        MaterializedOrigin::ProgramInput { key } => {
            bytes.push(0x01);
            push_slice(bytes, key.as_str().as_bytes());
        }
        MaterializedOrigin::Internal => bytes.push(0x02),
    }
}

/// The canonical content keys of every program entity.
///
/// Each key is a domain-tagged, length-prefixed encoding of what the entity
/// *is*, never of where it happens to sit in a builder arena. The verifier
/// proves the keys of each category are pairwise distinct, which makes the
/// canonical order they induce total.
///
/// That order is what these keys are for. They are **not** written into
/// identity: [`encode_identity`] writes each entity once and cross-references
/// it by its rank in this order, because a key that embeds its operands' keys
/// restates a whole subtree everywhere it appears. Ranking still needs the
/// nested form — a rank has to be a function of the entity's full content — but
/// only one comparison sort pays for it, instead of every reference.
#[derive(Clone, Debug)]
pub(super) struct CanonicalKeys {
    pub(super) stages: Vec<Vec<u8>>,
    pub(super) values: Vec<Vec<u8>>,
    pub(super) views: Vec<Vec<u8>>,
    pub(super) allocations: Vec<Vec<u8>>,
}

/// Derives the canonical content key of every entity.
///
/// The layering is acyclic by construction: a stage key names only its bound
/// kernel and coverage, a value key names its defining stage key, a view key
/// names its base value key, and an allocation key names the value keys it
/// binds.
pub(super) fn canonical_keys(
    data: &KernelProgramData,
    definitions: &[Option<ValueDefinition>],
) -> CanonicalKeys {
    let stages: Vec<Vec<u8>> = data.stages.iter().map(stage_key).collect();
    let values: Vec<Vec<u8>> = data
        .values
        .iter()
        .zip(definitions)
        .map(|(value, definition)| value_key(value, definition.as_ref(), &stages))
        .collect();
    let views: Vec<Vec<u8>> = data
        .views
        .iter()
        .map(|view| view_key(view, &values))
        .collect();
    let allocations: Vec<Vec<u8>> = data
        .allocations
        .iter()
        .enumerate()
        .map(|(index, allocation)| allocation_key(index, allocation, data, &values))
        .collect();
    CanonicalKeys {
        stages,
        values,
        views,
        allocations,
    }
}

fn stage_key(stage: &StageData) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(STAGE_KEY_DOMAIN);
    push_slice(&mut bytes, stage.kernel.canonical_identity().as_bytes());
    push_len(&mut bytes, stage.coverage.len());
    for occurrence in &stage.coverage {
        bytes.extend_from_slice(&occurrence.get().to_be_bytes());
    }
    bytes
}

fn value_key(
    value: &MaterializedValueData,
    definition: Option<&ValueDefinition>,
    stages: &[Vec<u8>],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(VALUE_KEY_DOMAIN);
    bytes.push(value.role.tag());
    push_origin(&mut bytes, &value.origin);
    push_shape(&mut bytes, &value.shape);
    push_element_type(&mut bytes, value.element_type);
    bytes.extend_from_slice(&value.required_bytes.to_be_bytes());
    bytes.extend_from_slice(&value.alignment.to_be_bytes());
    bytes.push(value.memory_space.tag());
    match definition {
        None => bytes.push(0x00),
        Some(definition) => {
            bytes.push(0x01);
            push_slice(&mut bytes, &stages[position(definition.stage)]);
            bytes.extend_from_slice(&definition.write_position.to_be_bytes());
        }
    }
    bytes
}

fn view_key(view: &ViewData, values: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(VIEW_KEY_DOMAIN);
    push_slice(&mut bytes, &values[position(view.value)]);
    bytes.extend_from_slice(&view.window.offset.to_be_bytes());
    bytes.extend_from_slice(&view.window.length.to_be_bytes());
    bytes
}

fn allocation_key(
    index: usize,
    allocation: &AllocationData,
    data: &KernelProgramData,
    values: &[Vec<u8>],
) -> Vec<u8> {
    let index = u32::try_from(index).expect("bounded allocation count fits u32");
    let mut bound: Vec<&[u8]> = data
        .values
        .iter()
        .enumerate()
        .filter(|(_, value)| value.allocation == index)
        .map(|(position, _)| values[position].as_slice())
        .collect();
    bound.sort_unstable();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ALLOCATION_KEY_DOMAIN);
    bytes.extend_from_slice(&allocation.capacity_bytes.to_be_bytes());
    bytes.extend_from_slice(&allocation.alignment.to_be_bytes());
    bytes.push(allocation.memory_space.tag());
    bytes.push(allocation.ownership.tag());
    push_len(&mut bytes, bound.len());
    for key in bound {
        push_slice(&mut bytes, key);
    }
    bytes
}

/// Returns the positions of `keys` in ascending canonical-key order.
fn canonical_order(keys: &[Vec<u8>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_unstable_by(|left, right| keys[*left].cmp(&keys[*right]));
    order
}

/// One canonical entity numbering: declared position in, canonical position out.
///
/// The numbering is content-derived because [`canonical_order`] ranks by
/// canonical key, and the verifier proves those keys pairwise distinct within a
/// category — so the rank is total and two programs that mean the same thing
/// number their entities identically.
struct CanonicalIds {
    /// Canonical position of each declared entity.
    of_declared: Vec<u64>,
    /// Declared positions in canonical order.
    order: Vec<usize>,
}

fn canonical_ids(keys: &[Vec<u8>]) -> CanonicalIds {
    let order = canonical_order(keys);
    let mut of_declared = vec![0_u64; order.len()];
    for (canonical, declared) in order.iter().enumerate() {
        of_declared[*declared] = u64::try_from(canonical).expect("a bounded program fits u64");
    }
    CanonicalIds { of_declared, order }
}

impl CanonicalIds {
    /// Names one entity by its canonical position, at fixed width.
    ///
    /// Eight big-endian bytes are self-delimiting, so this replaces a
    /// length-prefixed key without losing framing.
    fn push(&self, bytes: &mut Vec<u8>, declared: u32) {
        bytes.extend_from_slice(&self.of_declared[position(declared)].to_be_bytes());
    }
}

/// The ABI use sites of one program, in the order identity folds them.
///
/// This seeds the canonical arena numbering, so it must be an order derived
/// from program content — hence the canonical stage order rather than the
/// declared one. It is deliberately *not* the authority on where each reference
/// is written: [`encode_identity`] names each use site directly, and a drift
/// between the two would change which node gets which number without making an
/// identity ambiguous, because the arena is written in whatever numbering this
/// produced and every reference resolves through that same numbering.
fn abi_use_sites(data: &KernelProgramData, stage_order: &[usize]) -> Vec<u32> {
    // The applicability guard leads because it decides whether any stage runs
    // at all.
    let mut sites = vec![
        data.applicability_guard
            .expect("verification proves an applicability guard is declared"),
    ];
    for stage in stage_order {
        let stage = &data.stages[*stage];
        sites.push(stage.launch.grid_threads);
        sites.push(stage.launch.threads_per_workgroup);
        sites.extend(stage.accesses.iter().map(|access| access.accessible_bytes));
    }
    sites
}

/// Names one arena node by its canonical position, at fixed width.
///
/// Fixed width is what lets this replace a length-prefixed key without losing
/// framing: eight big-endian bytes are self-delimiting on their own.
fn push_abi_reference(bytes: &mut Vec<u8>, arena: &AbiArenaTraversal, node: u32) {
    bytes.extend_from_slice(&arena.canonical_id(node).to_be_bytes());
}

/// Encodes the canonical identity of one verified kernel program.
///
/// # Shape
///
/// Every entity is written **once**, in a count-prefixed section in canonical
/// order, and every cross-reference to it is its canonical position. Sections
/// are ordered so a record only names entities an earlier section already
/// defined — stages, then values, then views and allocations, then the program
/// structure that names all of them.
///
/// # Injectivity
///
/// Each section is a framed count followed by that many self-delimiting
/// records, so a reader recovers the exact entity list from these bytes alone.
/// A canonical position is eight fixed-width bytes and the section it indexes
/// is complete, so a reference determines its entity exactly as a full copy of
/// that entity's key did. What the encoding stops restating, it does not stop
/// determining.
///
/// See [`PROGRAM_DOMAIN`] for why this is a `v3` step and what `v2` did instead.
///
/// # Errors
///
/// Returns [`KernelProgramDiagnostic::IdentityLimit`] when the encoding exceeds
/// the governed byte bound.
pub(super) fn encode_identity(
    data: &KernelProgramData,
    keys: &CanonicalKeys,
    definitions: &[Option<ValueDefinition>],
) -> Result<CanonicalKernelProgramIdentity, KernelProgramDiagnostic> {
    let stages = canonical_ids(&keys.stages);
    let values = canonical_ids(&keys.values);
    let views = canonical_ids(&keys.views);
    let allocations = canonical_ids(&keys.allocations);

    let use_sites = abi_use_sites(data, &stages.order);
    let arena = canonical_arena_traversal(&data.abi_expressions, use_sites.iter().copied());
    debug_assert_eq!(
        arena.reached(),
        data.abi_expressions.len(),
        "verification proves every arena node is reached by a use site"
    );

    let mut bytes = Vec::new();
    bytes.extend_from_slice(PROGRAM_DOMAIN);
    push_slice(&mut bytes, data.semantic_graph.as_bytes());
    arena.encode(&data.abi_expressions, &mut bytes);

    // A stage names only its bound implementation and the occurrences it
    // covers, so it depends on no other entity and leads.
    push_len(&mut bytes, data.stages.len());
    for stage in &stages.order {
        let stage = &data.stages[*stage];
        push_slice(&mut bytes, stage.kernel.canonical_identity().as_bytes());
        push_len(&mut bytes, stage.coverage.len());
        for occurrence in &stage.coverage {
            bytes.extend_from_slice(&occurrence.get().to_be_bytes());
        }
    }

    push_len(&mut bytes, data.values.len());
    for value in &values.order {
        push_value(
            &mut bytes,
            &data.values[*value],
            definitions[*value],
            &stages,
        );
    }

    push_len(&mut bytes, data.views.len());
    for view in &views.order {
        let view = &data.views[*view];
        values.push(&mut bytes, view.value);
        bytes.extend_from_slice(&view.window.offset.to_be_bytes());
        bytes.extend_from_slice(&view.window.length.to_be_bytes());
    }

    push_len(&mut bytes, data.allocations.len());
    for allocation in &allocations.order {
        push_allocation(&mut bytes, *allocation, data, &values);
    }

    // The applicability guard is folded before the stage detail because it
    // decides whether any stage runs at all.
    push_abi_reference(
        &mut bytes,
        &arena,
        data.applicability_guard
            .expect("verification proves an applicability guard is declared"),
    );

    // The launch geometry and accesses of each stage, in the same canonical
    // stage order the section above established.
    for stage in &stages.order {
        let stage = &data.stages[*stage];
        push_abi_reference(&mut bytes, &arena, stage.launch.grid_threads);
        push_abi_reference(&mut bytes, &arena, stage.launch.threads_per_workgroup);
        push_len(&mut bytes, stage.accesses.len());
        for access in &stage.accesses {
            views.push(&mut bytes, access.view);
            bytes.push(access.mode.tag());
            push_abi_reference(&mut bytes, &arena, access.accessible_bytes);
        }
    }

    let mut edges: Vec<Vec<u8>> = data
        .dependencies
        .iter()
        .map(|dependency| encode_dependency(dependency, &stages, &values, &allocations))
        .collect();
    edges.sort_unstable();
    push_len(&mut bytes, edges.len());
    for edge in edges {
        push_slice(&mut bytes, &edge);
    }

    let mut outputs: Vec<Vec<u8>> = data
        .outputs
        .iter()
        .map(|output| {
            let mut encoded = Vec::new();
            push_slice(&mut encoded, output.key.as_str().as_bytes());
            values.push(&mut encoded, output.value);
            encoded
        })
        .collect();
    outputs.sort_unstable();
    push_len(&mut bytes, outputs.len());
    for output in outputs {
        push_slice(&mut bytes, &output);
    }

    // Lifecycle order, not insertion order: verification proves the declared
    // transitions form the one ordered chain, so this sequence is content.
    push_len(&mut bytes, data.routing_commit.len());
    for transition in &data.routing_commit {
        bytes.push(transition.from.tag());
        bytes.push(transition.to.tag());
        bytes.push(u8::from(transition.fallback_permitted));
    }

    if bytes.len() > MAX_PROGRAM_IDENTITY_BYTES {
        return Err(KernelProgramDiagnostic::IdentityLimit {
            bytes: bytes.len(),
            limit: MAX_PROGRAM_IDENTITY_BYTES,
        });
    }
    Ok(CanonicalKernelProgramIdentity(bytes))
}

fn push_value(
    bytes: &mut Vec<u8>,
    value: &MaterializedValueData,
    definition: Option<ValueDefinition>,
    stages: &CanonicalIds,
) {
    bytes.push(value.role.tag());
    push_origin(bytes, &value.origin);
    push_shape(bytes, &value.shape);
    push_element_type(bytes, value.element_type);
    bytes.extend_from_slice(&value.required_bytes.to_be_bytes());
    bytes.extend_from_slice(&value.alignment.to_be_bytes());
    bytes.push(value.memory_space.tag());
    match definition {
        None => bytes.push(0x00),
        Some(definition) => {
            bytes.push(0x01);
            stages.push(bytes, definition.stage);
            bytes.extend_from_slice(&definition.write_position.to_be_bytes());
        }
    }
}

/// Writes one allocation, naming the values bound to it in canonical order.
///
/// The bound set is what carries the storage binding into identity — a value's
/// own record does not name its allocation — so it stays part of the encoding
/// rather than being dropped along with the nesting.
fn push_allocation(
    bytes: &mut Vec<u8>,
    index: usize,
    data: &KernelProgramData,
    values: &CanonicalIds,
) {
    let index = u32::try_from(index).expect("bounded allocation count fits u32");
    let allocation = &data.allocations[position(index)];
    let mut bound: Vec<u64> = data
        .values
        .iter()
        .enumerate()
        .filter(|(_, value)| value.allocation == index)
        .map(|(declared, _)| values.of_declared[declared])
        .collect();
    bound.sort_unstable();
    bytes.extend_from_slice(&allocation.capacity_bytes.to_be_bytes());
    bytes.extend_from_slice(&allocation.alignment.to_be_bytes());
    bytes.push(allocation.memory_space.tag());
    bytes.push(allocation.ownership.tag());
    push_len(bytes, bound.len());
    for value in bound {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
}

fn encode_dependency(
    dependency: &DependencyData,
    stages: &CanonicalIds,
    values: &CanonicalIds,
    allocations: &CanonicalIds,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    stages.push(&mut bytes, dependency.predecessor);
    stages.push(&mut bytes, dependency.successor);
    match dependency.reason {
        DependencyReasonData::Data(value) => {
            bytes.push(0x01);
            values.push(&mut bytes, value);
        }
        DependencyReasonData::StorageHandoff(allocation) => {
            bytes.push(0x02);
            allocations.push(&mut bytes, allocation);
        }
    }
    bytes
}
