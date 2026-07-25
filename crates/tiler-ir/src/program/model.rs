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

use crate::identity::{push_len, push_slice};
use crate::kernel::{KernelType, VerifiedKernel};
use crate::schedule::TensorRole;
use crate::semantic::{InputKey, OutputKey, SemanticGraphIdentity};
use crate::shape::Shape;

use super::MAX_PROGRAM_IDENTITY_BYTES;
use super::error::KernelProgramDiagnostic;
use super::handles::ViewId;

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

/// One stage access: the view it addresses and whether it reads or writes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StageAccess {
    /// View the stage addresses.
    pub view: ViewId,
    /// Whether the stage reads or writes through it.
    pub mode: StageAccessMode,
}

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
}

/// Storage for one stage access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StageAccessData {
    pub(super) view: u32,
    pub(super) mode: StageAccessMode,
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
/// The identity folds the three ADR 0072 layers a complete program owns: the
/// canonical [`SemanticGraphIdentity`] of the program it realizes, the exact
/// bound implementation of every stage through each stage's
/// [`CanonicalKernelIdentity`](crate::kernel::CanonicalKernelIdentity) (which
/// itself folds the canonical scheduled
/// region it refines), and the complete semantic coverage those stages claim.
/// Program structure — materialized values, views, allocations, typed
/// dependencies, and named outputs — is folded in alongside them.
///
/// It excludes every transient ordinal: builder insertion order, the program's
/// own stage/value/view/allocation positions, and the planning `RegionId`
/// already excluded by the kernel and schedule identities. Cross-references are
/// encoded by canonical content key, never by position, so two structurally
/// equal programs assembled in different orders share bytes.
///
/// It also deliberately excludes what a later artifact-facing projection owns:
/// packaged admission, selected-provider provenance, and the artifact's routing
/// and ABI representation.
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

    fn data(self) -> StageAccessData {
        self.program.data.stages[self.stage].accesses[self.position]
    }
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

const STAGE_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.stage.v1\0";
const VALUE_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.value.v1\0";
const VIEW_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.view.v1\0";
const ALLOCATION_KEY_DOMAIN: &[u8] = b"tiler.kernel-program.allocation.v1\0";
const PROGRAM_DOMAIN: &[u8] = b"tiler.kernel-program.v1\0";

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
/// canonical order they induce total and makes cross-references by key
/// injective.
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

/// Encodes the canonical identity of one verified kernel program.
///
/// # Errors
///
/// Returns [`KernelProgramDiagnostic::IdentityLimit`] when the encoding exceeds
/// the governed byte bound.
pub(super) fn encode_identity(
    data: &KernelProgramData,
    keys: &CanonicalKeys,
) -> Result<CanonicalKernelProgramIdentity, KernelProgramDiagnostic> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(PROGRAM_DOMAIN);
    push_slice(&mut bytes, data.semantic_graph.as_bytes());

    push_len(&mut bytes, data.stages.len());
    for stage in canonical_order(&keys.stages) {
        push_slice(&mut bytes, &keys.stages[stage]);
        let accesses = &data.stages[stage].accesses;
        push_len(&mut bytes, accesses.len());
        for access in accesses {
            push_slice(&mut bytes, &keys.views[position(access.view)]);
            bytes.push(access.mode.tag());
        }
    }

    push_len(&mut bytes, data.values.len());
    for value in canonical_order(&keys.values) {
        push_slice(&mut bytes, &keys.values[value]);
    }

    push_len(&mut bytes, data.allocations.len());
    for allocation in canonical_order(&keys.allocations) {
        push_slice(&mut bytes, &keys.allocations[allocation]);
    }

    let mut edges: Vec<Vec<u8>> = data
        .dependencies
        .iter()
        .map(|dependency| encode_dependency(dependency, keys))
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
            push_slice(&mut encoded, &keys.values[position(output.value)]);
            encoded
        })
        .collect();
    outputs.sort_unstable();
    push_len(&mut bytes, outputs.len());
    for output in outputs {
        push_slice(&mut bytes, &output);
    }

    if bytes.len() > MAX_PROGRAM_IDENTITY_BYTES {
        return Err(KernelProgramDiagnostic::IdentityLimit {
            bytes: bytes.len(),
            limit: MAX_PROGRAM_IDENTITY_BYTES,
        });
    }
    Ok(CanonicalKernelProgramIdentity(bytes))
}

fn encode_dependency(dependency: &DependencyData, keys: &CanonicalKeys) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, &keys.stages[position(dependency.predecessor)]);
    push_slice(&mut bytes, &keys.stages[position(dependency.successor)]);
    match dependency.reason {
        DependencyReasonData::Data(value) => {
            bytes.push(0x01);
            push_slice(&mut bytes, &keys.values[position(value)]);
        }
        DependencyReasonData::StorageHandoff(allocation) => {
            bytes.push(0x02);
            push_slice(&mut bytes, &keys.allocations[position(allocation)]);
        }
    }
    bytes
}
