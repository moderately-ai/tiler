//! The nodefold backend: its declared target, its translation, and its assembly.
//!
//! Everything here is stated by a producer that lives outside every crate in
//! the workspace and compiles against public surfaces alone. No crate under
//! `crates/` was changed to admit it, and nothing here is Metal's, the scalar
//! CPU spike's, or `tiler-build`'s own test backend's.
//!
//! # What this backend claims, and what it deliberately does not
//!
//! It claims four rows of the ADR 0090 responsibility matrix and no others: a
//! target profile it declares itself, an executable representation, the
//! validation of that representation's bytes, and a runtime adapter (in
//! [`super::nodefold_adapter`]). It installs no semantic authority, no lowering
//! capability, and no physical implementation provider, and it needs none — the
//! governed vertical supplies those, and a backend that had to install one to
//! reach this seam would be evidence that the seam is in the wrong place.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactExecutionPolicy, BackendEntryKey, BackendKey, BindingKind,
    PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadPlatform, PayloadProvenance,
    RepresentationKey, SchemaVersion, ToolComponent, VerifiedArtifactProgram,
};
use tiler_build::{BackendEntryDeclaration, PlanDeterminismDeclaration, assemble_plan_artifact};
use tiler_compiler::session::PlanAlternative;
use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuildError, TargetProfileBuilder, TargetProfileKey,
};
use tiler_ir::kernel::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, CompareOp, ConvertOp,
    KernelConstant, KernelType, OperationView, VerifiedBufferId, VerifiedKernel, VerifiedValueId,
};
use tiler_ir::program::StageRef;
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    SubnormalMode,
};
use tiler_ir::semantic::{F32, SemanticProgram};

use crate::nodefold_graph::{Graph, GraphBuffer, GraphEntry, Node, StorePlan, encode};

/// Governed profile key of the target this backend declares.
pub(crate) const PROFILE_KEY: &str = "tiler.test.nodefold-host-v1";

/// Governed backend family key this backend's payloads declare.
pub(crate) const BACKEND_KEY: &str = "tiler.test.nodefold";

/// Governed executable-representation key of its carried payload.
pub(crate) const REPRESENTATION_KEY: &str = "tiler.test.nodefold-graph-v1";

/// Governed representation key of the source the payload retains.
///
/// There is no source file behind this payload: it is translated from verified
/// structured kernels by an in-process translator. The retained source is the
/// canonical kernel-identity run it was translated from, named as its own
/// representation so the compilation subject stays checkable.
pub(crate) const SOURCE_REPRESENTATION_KEY: &str = "tiler.test.nodefold-kernel-identity-run-v1";

/// Governed toolchain identity of this backend's in-process translator.
pub(crate) const TOOLCHAIN_KEY: &str = "tiler.test.nodefold-translator";

/// Declared target triple this backend's payload provenance records.
pub(crate) const TARGET_TRIPLE: &str = "aarch64-apple-darwin";

/// Governed schema version of this backend's payload.
pub(crate) const PAYLOAD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0);

/// The one delivery position this backend declares.
pub(crate) const SOLE_DELIVERY: usize = 0;

/// Threads per workgroup this backend's execution model admits.
///
/// One, and it is a compile-time fact rather than a prepared-entry query: this
/// backend runs each invocation to completion on one worker thread, so there is
/// no pipeline to interrogate and no deferred predicate to mint. An artifact
/// assembled here therefore carries **zero** deferred prepared-entry
/// predicates, which is a branch of the routing path a backend that can only
/// learn its capacity from a built pipeline never takes.
pub(crate) const WORKGROUP_THREADS: u32 = 1;

/// Explicitly staged local memory this backend's execution model admits.
pub(crate) const LOCAL_MEMORY_BYTES: u64 = 0;

/// Buffer bindings one entry of this backend admits.
pub(crate) const BUFFER_BINDINGS: u32 = 2;

/// Grid extent one launch admits along its single axis.
pub(crate) const GRID_AXIS_THREADS: u64 = 1 << 20;

/// Byte width of one `f32` element in this backend's storage.
pub(crate) const F32_BYTES: u64 = 4;

/// The dtype dispatchability this profile declares, in one place.
///
/// Read by the profile declaration **and** by the execution environment the
/// adapter binds, so the two cannot drift into disagreeing about what this
/// target family can dispatch. A host that stated one thing to the compiler and
/// another to the loader would be two backends wearing one key.
pub(crate) const DTYPE_ROWS: [(KernelType, DTypeDispatchability); 1] =
    [(KernelType::F32, DTypeDispatchability::Dispatchable)];

/// Why this backend refused a plan, a payload, an assembly, or a dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NodefoldRefusal {
    /// A kernel binds a number of buffers this backend cannot place.
    UnsupportedSignature {
        /// Ordinal of the refused kernel.
        entry: usize,
        /// Buffers the kernel binds.
        buffers: usize,
    },
    /// Translation refused a construct by name.
    ///
    /// A `&'static str` rather than a rendered message: the refused construct
    /// is a closed vocabulary this backend states, and a test may compare it
    /// without parsing prose.
    Untranslatable(&'static str),
    /// The artifact layer or the assembly seam refused a declaration.
    Assembly(String),
    /// The assembled artifact could not be encoded.
    Encoding(String),
}

impl fmt::Display for NodefoldRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSignature { entry, buffers } => write!(
                formatter,
                "nodefold: kernel {entry} binds {buffers} buffer(s) and this backend places {BUFFER_BINDINGS}",
            ),
            Self::Untranslatable(construct) => write!(
                formatter,
                "nodefold.translate: this backend's vocabulary does not name {construct}",
            ),
            Self::Assembly(message) => write!(formatter, "nodefold.assemble: {message}"),
            Self::Encoding(message) => write!(formatter, "nodefold.encode: {message}"),
        }
    }
}

impl Error for NodefoldRefusal {}

/// Which entry-mapping statement one assembly makes.
///
/// The sound value is [`Self::Derived`]. The other two are the two ways a
/// producer could try to certify a fact the plan already decided, and they
/// exist so a test can watch the stack refuse them rather than assert that it
/// would.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum EntryPerturbation {
    /// Map each entry under the key the plan's own stage kernel decided.
    #[default]
    Derived,
    /// Map each entry under a key this backend minted for itself.
    ForgedEntryKey,
    /// Map each entry under a symbol the emitted graph does not carry.
    UnmappedSymbol,
    /// Declare the identity transport map while the graph stays rotated.
    ///
    /// Not a forgery: every identity here is the plan's. It is the hazard
    /// [`transport_of`] names, made reachable — a backend whose declared
    /// mapping disagrees with the way its own payload indexes buffers. Nothing
    /// above the backend can catch it, because the mapping is a statement no
    /// plan makes and the payload's bytes are opaque to every layer that could
    /// compare them.
    IdentityTransports,
}

fn source() -> Result<TargetFactSource, TargetProfileBuildError> {
    let producer = TargetFactProducerIdentity::new("tiler.test.nodefold-backend".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    // An external guarantee, not a measurement: this backend evaluates `f32`
    // arithmetic with the host's own IEEE-754 binary32 operations and declares
    // what that standard requires. It measures no host and must not claim to.
    let reference = TargetNormativeReferenceIdentity::new("ieee.754.2019.binary32".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    Ok(TargetFactSource::external_guarantee(producer, reference))
}

/// Builds the target profile this backend compiles against.
///
/// # Errors
///
/// Returns the first typed declaration or freeze diagnostic. Returned rather
/// than unwrapped so a widening of the profile vocabulary surfaces here as a
/// refusal a test reports instead of a panic inside setup.
pub(crate) fn nodefold_profile() -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new(PROFILE_KEY.to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = TargetProfileBuilder::new(key);
    let source = source()?;

    builder.declare_max_threads_per_grid_axis(GRID_AXIS_THREADS, source.clone())?;
    builder.declare_max_threads_per_workgroup(WORKGROUP_THREADS, source.clone())?;
    builder.declare_max_buffer_bindings_per_entry(BUFFER_BINDINGS, source.clone())?;
    builder.declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())?;
    builder.declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())?;
    // The axis asks whether an addressable memory space exists, not whether it
    // is distinct from the host's. This backend's is the process heap.
    builder.declare_device_memory(true, source.clone())?;
    builder.declare_local_memory_bytes(LOCAL_MEMORY_BYTES, source.clone())?;

    declare_numerics(&mut builder, &source)?;

    for (dtype, verdict) in DTYPE_ROWS {
        let resolved = match dtype {
            KernelType::F32 => F32::resolved_type(),
            _ => return Err(TargetProfileBuildError::InvalidProducerClaim),
        };
        builder.declare_dtype_dispatchability(resolved, verdict, source.clone())?;
    }
    builder.build()
}

/// Declares the exact numerical behaviours this backend's evaluator honours.
///
/// A single-assignment node table evaluated in one forward pass performs each
/// declared operation exactly once, in the order the graph states, so it never
/// contracts, reassociates, permutes, or eliminates a signed zero. Those are
/// declared `Forbidden`-exact and `Permitted`-unsupported, which makes a
/// contract that needs any of them a compile-time refusal here rather than an
/// approximation discovered at run time.
fn declare_numerics(
    builder: &mut TargetProfileBuilder,
    source: &TargetFactSource,
) -> Result<(), TargetProfileBuildError> {
    let subject = ScalarArithmetic::f32();

    let subnormals = [
        (SubnormalMode::Preserve, ScalarSupport::Exact),
        (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            ScalarSupport::Unsupported,
        ),
        (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
            ScalarSupport::Unsupported,
        ),
    ];
    for (behaviour, support) in subnormals {
        builder.declare_input_subnormals(subject.clone(), behaviour, support, source.clone())?;
        builder.declare_result_subnormals(subject.clone(), behaviour, support, source.clone())?;
    }

    let permissions = [
        (NumericalPermission::Forbidden, ScalarSupport::Exact),
        (NumericalPermission::Permitted, ScalarSupport::Unsupported),
    ];
    for (permission, support) in permissions {
        builder.declare_contraction(subject.clone(), permission, support, source.clone())?;
        builder.declare_reassociation(subject.clone(), permission, support, source.clone())?;
    }
    builder.declare_permutation(
        subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_signed_zero(
        subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_reciprocal_transform(
        subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_approximate_intrinsics(
        subject.clone(),
        ApproximationEnvelope::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_nan_assumptions(
        subject.clone(),
        ExceptionalValueAssumption::MakeNoAssumption,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_infinity_assumptions(
        subject,
        ExceptionalValueAssumption::MakeNoAssumption,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    Ok(())
}

/// Returns the transport slot one ABI binding slot occupies.
///
/// A rotation by one, and the reason it is not the identity is a hazard rather
/// than a preference: the transport mapping is the backend's own statement, and
/// an adapter that placed a binding by its ABI slot number would bind the read
/// where the write goes and nothing above it would notice. `tests/custom_backend`
/// found the same hazard independently and states its two-binding mapping as a
/// reversal; at two bindings a rotation by one *is* that reversal, which is
/// convergence on a real property of the seam rather than a borrowed trick.
pub(crate) fn transport_of(slot: usize, bindings: usize) -> u32 {
    let rotated = (slot + 1) % bindings;
    u32::try_from(rotated).expect("a bounded binding count fits u32")
}

/// Names one entry within this backend's own payload.
///
/// **Positional, and carrying no identity, which is a decision rather than an
/// omission.** The durable identity of a packaged entry is the
/// `BackendEntryKey` the artifact carries, and `assemble_plan_artifact` derives
/// that from the stage kernel with no parameter a producer could supply
/// instead. A symbol that re-encoded the same identity would be a second copy
/// of a fact that already has an owner, and two copies can disagree; this one
/// is only a name for an entry inside one emitted graph, where the ordinal
/// already makes it unique.
///
/// It is also the row where a rendered identity would have been actively
/// misleading here. This backend's canonical kernel identities open with a
/// governed namespace every kernel in the workspace shares and end in zero
/// bytes, so both a rendered prefix and a rendered suffix produce the same
/// characters for every entry — a discriminator that discriminates nothing.
/// Reaching a digest to fix that would add a dependency this backend otherwise
/// does not need, to compress an identity it does not have to carry.
fn symbol_for(ordinal: usize) -> String {
    format!("nodefold.entry.{ordinal}")
}

/// Translates verified structured kernels into this backend's graph.
///
/// # Errors
///
/// Returns [`NodefoldRefusal::UnsupportedSignature`] for a kernel whose buffer
/// count this backend cannot place, and [`NodefoldRefusal::Untranslatable`]
/// naming the first construct outside its vocabulary.
pub(crate) fn translate(kernels: &[&VerifiedKernel]) -> Result<Graph, NodefoldRefusal> {
    let mut entries = Vec::with_capacity(kernels.len());
    for (ordinal, kernel) in kernels.iter().enumerate() {
        let buffers: Vec<BufferParameter> = kernel.buffers().collect();
        if buffers.len() != BUFFER_BINDINGS as usize {
            return Err(NodefoldRefusal::UnsupportedSignature {
                entry: ordinal,
                buffers: buffers.len(),
            });
        }
        entries.push(translate_kernel(ordinal, kernel)?);
    }
    Ok(Graph { entries })
}

struct Translator<'kernel> {
    kernel: &'kernel VerifiedKernel,
    transports: HashMap<VerifiedBufferId, u32>,
    nodes: Vec<Node>,
    ordinals: HashMap<VerifiedValueId, u32>,
}

impl Translator<'_> {
    fn push(&mut self, node: Node) -> Result<u32, NodefoldRefusal> {
        let ordinal = u32::try_from(self.nodes.len())
            .map_err(|_| NodefoldRefusal::Untranslatable("a node table this large"))?;
        self.nodes.push(node);
        Ok(ordinal)
    }

    fn define(&mut self, value: VerifiedValueId, node: Node) -> Result<u32, NodefoldRefusal> {
        let ordinal = self.push(node)?;
        self.ordinals.insert(value, ordinal);
        Ok(ordinal)
    }

    fn ordinal(&self, value: VerifiedValueId) -> Result<u32, NodefoldRefusal> {
        self.ordinals
            .get(&value)
            .copied()
            .ok_or(NodefoldRefusal::Untranslatable(
                "an operand defined outside this entry's own body",
            ))
    }

    fn transport(&self, buffer: VerifiedBufferId) -> Result<u32, NodefoldRefusal> {
        self.transports
            .get(&buffer)
            .copied()
            .ok_or(NodefoldRefusal::Untranslatable(
                "a buffer the signature does not declare",
            ))
    }

    fn sole_result(results: &[VerifiedValueId]) -> Result<VerifiedValueId, NodefoldRefusal> {
        match results {
            [result] => Ok(*result),
            _ => Err(NodefoldRefusal::Untranslatable(
                "an operation defining other than exactly one result",
            )),
        }
    }

    /// Lowers every value-producing operation of one block into the flat table.
    ///
    /// Returns the store plan when the block ends in a write. A block is
    /// admitted only if at most one of its operations is a store and that store
    /// is its last, which is what makes the decoded entry's single write site
    /// structural rather than checked.
    fn block(
        &mut self,
        block: BlockRef<'_>,
        guard: Option<u32>,
    ) -> Result<Option<StorePlan>, NodefoldRefusal> {
        let mut store = None;
        for operation in block.operations() {
            if store.is_some() {
                return Err(NodefoldRefusal::Untranslatable(
                    "an operation following this block's store",
                ));
            }
            let results: Vec<VerifiedValueId> = operation.results().collect();
            match operation.view() {
                OperationView::Store {
                    buffer,
                    offset,
                    value,
                    ..
                } => {
                    store = Some(StorePlan {
                        guard,
                        buffer: self.transport(buffer)?,
                        offset: self.ordinal(offset)?,
                        value: self.ordinal(value)?,
                    });
                }
                OperationView::Predicated { predicate, body } => {
                    if guard.is_some() {
                        return Err(NodefoldRefusal::Untranslatable("nested predication"));
                    }
                    let predicate = self.ordinal(predicate)?;
                    store = self.block(body, Some(predicate))?;
                    if store.is_none() {
                        return Err(NodefoldRefusal::Untranslatable(
                            "a predicated block that performs no store",
                        ));
                    }
                }
                view => {
                    let node = self.value_node(&view)?;
                    self.define(Self::sole_result(&results)?, node)?;
                }
            }
        }
        Ok(store)
    }

    /// Maps one value-producing operation onto this representation's vocabulary.
    fn value_node(&self, view: &OperationView<'_>) -> Result<Node, NodefoldRefusal> {
        Ok(match view {
            OperationView::Builtin { builtin } => match builtin {
                tiler_ir::kernel::Builtin::GlobalInvocationIndex => Node::InvocationIndex,
                tiler_ir::kernel::Builtin::LocalInvocationIndex => {
                    return Err(NodefoldRefusal::Untranslatable("a workgroup-local index"));
                }
                _ => return Err(NodefoldRefusal::Untranslatable("an unadmitted launch builtin")),
            },
            OperationView::Constant { value } => match value {
                KernelConstant::F32Bits(bits) => Node::F32Constant(*bits),
                KernelConstant::Index(index) => Node::IndexConstant(*index),
                KernelConstant::Bool(_) => {
                    return Err(NodefoldRefusal::Untranslatable("a boolean constant"));
                }
                _ => {
                    return Err(NodefoldRefusal::Untranslatable(
                        "a constant outside this vocabulary",
                    ));
                }
            },
            OperationView::Binary { op, lhs, rhs } => {
                let lhs = self.ordinal(*lhs)?;
                let rhs = self.ordinal(*rhs)?;
                match op {
                    BinaryOp::F32Multiply => Node::F32Multiply(lhs, rhs),
                    BinaryOp::F32Add => Node::F32Add(lhs, rhs),
                    BinaryOp::IndexAdd => Node::IndexAdd(lhs, rhs),
                    BinaryOp::IndexMultiply => Node::IndexMultiply(lhs, rhs),
                    _ => {
                        return Err(NodefoldRefusal::Untranslatable(
                            "an unadmitted binary operation",
                        ));
                    }
                }
            }
            OperationView::Compare { op, lhs, rhs } => match op {
                CompareOp::IndexLessThan => {
                    Node::IndexLessThan(self.ordinal(*lhs)?, self.ordinal(*rhs)?)
                }
                _ => {
                    return Err(NodefoldRefusal::Untranslatable(
                        "a comparison outside unsigned index ordering",
                    ));
                }
            },
            OperationView::Convert { op, source } => match op {
                ConvertOp::CanonicalizeF32Nan => Node::CanonicalizeF32Nan(self.ordinal(*source)?),
                _ => return Err(NodefoldRefusal::Untranslatable("an unadmitted conversion")),
            },
            OperationView::Load { buffer, offset, .. } => Node::Load {
                buffer: self.transport(*buffer)?,
                offset: self.ordinal(*offset)?,
            },
            OperationView::SerialLoop(_) => {
                return Err(NodefoldRefusal::Untranslatable("a serial loop"));
            }
            OperationView::Barrier { .. } => {
                return Err(NodefoldRefusal::Untranslatable("a barrier"));
            }
            OperationView::Unary { .. } => {
                return Err(NodefoldRefusal::Untranslatable("a unary elementary function"));
            }
            OperationView::PackedExtract { .. } => {
                return Err(NodefoldRefusal::Untranslatable("packed-nibble extraction"));
            }
            _ => {
                return Err(NodefoldRefusal::Untranslatable(
                    "an operation kind this schema does not name",
                ));
            }
        })
    }
}

fn translate_buffer(parameter: &BufferParameter) -> Result<GraphBuffer, NodefoldRefusal> {
    if parameter.element_type != KernelType::F32 {
        return Err(NodefoldRefusal::Untranslatable("a non-f32 buffer"));
    }
    if parameter.address_space != AddressSpace::Device {
        return Err(NodefoldRefusal::Untranslatable("a non-device address space"));
    }
    Ok(GraphBuffer {
        write: parameter.access == BufferAccess::Write,
        element_count: parameter.element_count,
    })
}

fn translate_kernel(
    ordinal: usize,
    kernel: &VerifiedKernel,
) -> Result<GraphEntry, NodefoldRefusal> {
    let declared: Vec<(VerifiedBufferId, BufferParameter)> = kernel.declared_buffers().collect();
    let bindings = declared.len();
    let mut buffers = vec![
        GraphBuffer {
            write: false,
            element_count: 0,
        };
        bindings
    ];
    let mut transports = HashMap::with_capacity(bindings);
    for (slot, (id, parameter)) in declared.iter().enumerate() {
        let transport = transport_of(slot, bindings);
        buffers[transport as usize] = translate_buffer(parameter)?;
        transports.insert(*id, transport);
    }
    let mut translator = Translator {
        kernel,
        transports,
        nodes: Vec::new(),
        ordinals: HashMap::new(),
    };
    let store = translator
        .block(kernel.body(), None)?
        .ok_or(NodefoldRefusal::Untranslatable(
            "an entry body that performs no store",
        ))?;
    Ok(GraphEntry {
        symbol: symbol_for(ordinal),
        canonical_nan: translator.kernel.numerical().canonical_arithmetic_nan_bits,
        buffers,
        nodes: translator.nodes,
        store,
    })
}

/// One complete production run: translate, describe, assemble, encode.
pub(crate) struct Produced {
    /// The verified artifact this backend assembled through the neutral seam.
    pub(crate) artifact: VerifiedArtifactProgram,
    /// Encoded envelope bytes of that artifact.
    pub(crate) bytes: Vec<u8>,
}

/// Assembles one checked plan into a verified artifact carrying a nodefold graph.
///
/// # Errors
///
/// Returns the first translation, assembly, or encoding refusal.
pub(crate) fn assemble(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    perturbation: EntryPerturbation,
) -> Result<Produced, NodefoldRefusal> {
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let graph = translate(&kernels)?;
    let content = payload_content(&kernels, &graph, perturbation)?;
    let artifact = assemble_plan_artifact(
        semantic,
        plan,
        // `Unclaimed`, which is every current build: a `Claimed` declaration
        // needs a compiler witness joined to per-payload receipts, and no
        // accepted provider can mint one. This backend states no determinism
        // claim and could not.
        PlanDeterminismDeclaration::Unclaimed,
        |builder, profile| {
            builder
                .push_carried_payload(
                    BackendKey::new(BACKEND_KEY).expect("a governed backend key"),
                    RepresentationKey::new(REPRESENTATION_KEY)
                        .expect("a governed representation key"),
                    PAYLOAD_SCHEMA,
                    profile,
                    ArtifactExecutionPolicy::NativeImage,
                    // No target-environment declaration: this backend registers
                    // no accepted provider descriptor schema to name one under.
                    None,
                    content.clone(),
                )
                .map(|payload| vec![payload])
        },
        |_, stage: StageRef<'_>| {
            Ok(BackendEntryDeclaration {
                bindings: stage.accesses().map(|_| BindingKind::Buffer).collect(),
                // A zero-thread launch has nothing to fold, and this backend
                // states so rather than dispatching an empty pass.
                zero_work_skips_dispatch: true,
                // No launch precondition: every fact this backend's evaluator
                // needs is in the graph it decoded, and a precondition reading
                // a fact it cannot see would be one it could not check.
                preconditions: Vec::new(),
            })
        },
    )
    .map_err(|error| NodefoldRefusal::Assembly(error.to_string()))?;
    let bytes = artifact
        .encode()
        .map_err(|error| NodefoldRefusal::Encoding(error.to_string()))?;
    Ok(Produced { artifact, bytes })
}

/// Builds the carried payload, including the entry mapping under test.
fn payload_content(
    kernels: &[&VerifiedKernel],
    graph: &Graph,
    perturbation: EntryPerturbation,
) -> Result<PayloadContent, NodefoldRefusal> {
    let mut retained = Vec::new();
    let mut mappings = Vec::with_capacity(kernels.len());
    for (kernel, entry) in kernels.iter().zip(&graph.entries) {
        let identity = kernel.canonical_identity();
        retained.extend_from_slice(identity.as_bytes());
        let key_bytes = match perturbation {
            EntryPerturbation::Derived
            | EntryPerturbation::UnmappedSymbol
            | EntryPerturbation::IdentityTransports => identity.as_bytes().to_vec(),
            // A key this backend minted for itself, in place of the one the
            // plan's stage kernel decided. The whole of the forgery: the
            // backend states an identity rather than transporting one.
            EntryPerturbation::ForgedEntryKey => {
                let mut forged = identity.as_bytes().to_vec();
                let last = forged.len() - 1;
                forged[last] ^= 0xff;
                forged
            }
        };
        let symbol = match perturbation {
            EntryPerturbation::Derived
            | EntryPerturbation::ForgedEntryKey
            | EntryPerturbation::IdentityTransports => entry.symbol.clone(),
            EntryPerturbation::UnmappedSymbol => format!("{}.absent", entry.symbol),
        };
        mappings.push(PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(&key_bytes)
                .map_err(|error| NodefoldRefusal::Assembly(error.to_string()))?,
            symbol,
            transports: (0..entry.buffers.len())
                .map(|slot| match perturbation {
                    EntryPerturbation::IdentityTransports => {
                        u32::try_from(slot).expect("a bounded binding count fits u32")
                    }
                    _ => transport_of(slot, entry.buffers.len()),
                })
                .collect(),
        });
    }
    mappings.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));
    Ok(PayloadContent {
        metadata: PayloadMetadata {
            source_representation: RepresentationKey::new(SOURCE_REPRESENTATION_KEY)
                .map_err(|error| NodefoldRefusal::Assembly(error.to_string()))?,
            source: retained,
            provenance: PayloadProvenance {
                toolchain: TOOLCHAIN_KEY.to_owned(),
                target: TARGET_TRIPLE.to_owned(),
                family: BACKEND_KEY.to_owned(),
                language: "tiler.test.nodefold-graph".to_owned(),
                platform: PayloadPlatform::Unversioned,
                components: vec![ToolComponent {
                    role: "translator".to_owned(),
                    version: "1".to_owned(),
                }],
                compile_flags: Vec::new(),
                link_flags: Vec::new(),
            },
            entries: mappings,
            obligations: Vec::new(),
        },
        code: encode(graph),
    })
}

/// Returns the exact object bytes this backend emits for one checked plan.
///
/// The same run `assemble` carries, reached without decoding an envelope, so a
/// case about this representation's own refusals perturbs the representation
/// rather than the artifact around it.
///
/// # Errors
///
/// Returns the first translation refusal.
pub(crate) fn emitted_object(plan: PlanAlternative<'_>) -> Result<Vec<u8>, NodefoldRefusal> {
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    Ok(encode(&translate(&kernels)?))
}
