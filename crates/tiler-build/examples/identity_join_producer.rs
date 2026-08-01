//! The build-time half of the cross-process identity join, as its own program.
//!
//! # Why this is an executable rather than a test module
//!
//! The claim under test is that a build-time producer and a runtime adapter join
//! through durable artifact identity and nothing else. A fixture that produced
//! and consumed in one process cannot show that: every Rust value the producer
//! built would still be alive when the consumer ran, so "the consumer used only
//! the bytes" would be a property of how the test was written rather than of the
//! seam. Splitting the halves across two programs removes the possibility.
//!
//! The split is asymmetric on purpose. This program links `tiler-compiler`,
//! `tiler-metal-aot`, and the whole of `tiler-build`; the consumer
//! (`crates/tiler-runtime/tests/identity_join`) links `tiler-artifact`,
//! `tiler-ir`, and `tiler-reference` and *cannot* link any of them — ADR 0081
//! item 2 fixes `tiler-runtime`'s dependency closure, and the consumer asserts
//! that closure from `Cargo.lock` rather than claiming it. So "the consumer
//! constructs no compiler, emitter, AOT driver, or build-provider object" is a
//! statement about what its binary contains, not about what its source happens
//! to call.
//!
//! # What crosses the boundary
//!
//! Two files per variant, and nothing else. `artifact.bin` is the exact envelope
//! the expansion cache returned, and `sidecar.txt` is the durable identity record
//! a consumer is configured with — governed keys and digests over canonical
//! bytes, no pointer, no `TypeId`, no symbol address, no process-scoped handle.
//! There is no callback, no shared memory, and no dynamic loading: the consumer
//! re-reads and re-validates everything from the bytes.
//!
//! # The runs
//!
//! `run-a/` holds every variant, produced by this process. `run-b/` holds the
//! sound variant alone, produced by a **re-executed child of this program**
//! against the same cache root, from a different working directory and with an
//! extra environment variable set. Two live processes producing byte-identical
//! envelopes is what makes "no process-local identity leaked" a measurement: an
//! address, an allocation, or a load order that reached identity would move
//! between them, and the second run resolves the sound subject as a cache *hit*
//! that its own validation re-checks from bytes.
//!
//! # Usage
//!
//! ```text
//! cargo run -p tiler-build --example identity_join_producer -- <root-directory>
//! ```
//!
//! The consumer suite invokes exactly that; running it by hand writes the same
//! tree and prints nothing on success.

use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactExecutionPolicy, BackendEntryKey, BackendKey,
    BackendPayloadDescriptor, BindingKind, DigestAlgorithm, PayloadContent, PayloadDigest,
    PayloadEntryMapping, PayloadMetadata, PayloadProvenance, PayloadSdkIdentity, RepresentationKey,
    SchemaVersion, TargetProfileDescriptorDigest, TargetProfileRef, ToolComponent,
    VerifiedArtifactProgram,
};
use tiler_build::{
    BackendEntryDeclaration, DeclaredPayload, accept_or_publish_single_payload_artifact,
    assemble_plan_artifact,
};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{
    Compilation, CompileRequest, NumericalContract, PlanAlternative, compile,
};
use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuildError, TargetProfileBuilder, TargetProfileKey, TargetRequest,
};
use tiler_ir::kernel::{
    BinaryOp, BlockRef, BufferAccess, KernelConstant, OperationView, VerifiedKernel,
};
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

// -------------------------------------------------------------------------
// The governed vocabulary both halves join on
// -------------------------------------------------------------------------

/// Governed target-profile key this producer compiles against.
const PROFILE_KEY: &str = "tiler.test.identity-join-scalar-host";
/// Governed backend family its payloads declare.
const BACKEND_KEY: &str = "tiler.test.scalar-host";
/// Governed executable representation its payloads declare.
const REPRESENTATION_KEY: &str = "tiler.test.scalar-host-image-v1";
/// Governed representation of the source the payload retains.
const SOURCE_REPRESENTATION_KEY: &str = "tiler.test.scalar-host-kernel-identity-list-v1";
/// Governed identity of this in-process translator.
const TOOLCHAIN_KEY: &str = "tiler.test.scalar-host-translator";
/// Declared target triple the payload provenance names.
const TARGET_TRIPLE: &str = "aarch64-apple-darwin";
/// Governed schema version of this backend's payload metadata.
const PAYLOAD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0);
/// Domain separator under which entry-point symbols are derived.
const SYMBOL_DOMAIN: &[u8] = b"tiler.test.scalar-host.symbol.v1\0";

/// Grid extent the sound profile admits along its single axis.
const GRID_AXIS_THREADS: u64 = 1 << 24;
/// Grid extent the *other revision* of that profile admits.
///
/// One declared fact, moved. The key is unchanged, so the two profiles differ
/// only in their exact descriptor — which is the distinction ADR 0043 exists to
/// preserve and the one a producer cannot forge, because
/// [`assemble_plan_artifact`] derives the variant's profile from the compilation
/// rather than from anything stated here.
const OTHER_GRID_AXIS_THREADS: u64 = 1 << 20;

/// Rows of the packaged input, which is also the output element count.
const ROWS: u64 = 2;
/// Columns of the packaged input, which is the reduction extent.
const COLUMNS: u64 = 3;
/// Bit pattern of the pointwise scale constant the graph applies, `2.0f32`.
const SCALE_BITS: u32 = 0x4000_0000;
/// Bit pattern of the pointwise bias constant the graph applies, `1.0f32`.
const BIAS_BITS: u32 = 0x3f80_0000;

// -------------------------------------------------------------------------
// The transported executable representation
// -------------------------------------------------------------------------

/// Domain separator of `tiler.test.scalar-host-image-v1`.
///
/// The encoder below is deliberately *not* shared with the consumer's decoder.
/// A wire format crossing a process boundary is a byte contract, and two halves
/// that shared one Rust type would be agreeing through the type rather than
/// through the bytes — which is the whole thing this fixture exists to test.
/// Drift is not silent: the consumer's decoder refuses, and its route fails.
const IMAGE_DOMAIN: &[u8; 16] = b"tiler.scalar-img";
/// Schema version of the encoding this producer writes.
const IMAGE_SCHEMA: (u16, u16) = (1, 0);

/// One executable entry of the emitted scalar image.
#[derive(Clone, Debug)]
struct ScalarEntry {
    /// This backend's own entry-point symbol, derived from kernel identity.
    symbol: String,
    /// Backend transport slot the entry reads its input through.
    read_transport: u32,
    /// Backend transport slot the entry writes its output through.
    write_transport: u32,
    /// Rows of the input, which is also the output element count.
    rows: u32,
    /// Columns of the input, which is the reduction extent.
    columns: u32,
    /// Bit pattern of the recognized affine scale.
    scale_bits: u32,
    /// Bit pattern of the recognized affine bias.
    bias_bits: u32,
}

/// Encodes the emitted image into the bytes the payload carries.
fn encode_image(entries: &[ScalarEntry]) -> Vec<u8> {
    let mut bytes = IMAGE_DOMAIN.to_vec();
    bytes.extend_from_slice(&IMAGE_SCHEMA.0.to_le_bytes());
    bytes.extend_from_slice(&IMAGE_SCHEMA.1.to_le_bytes());
    let count = u32::try_from(entries.len()).expect("a fixture emits few entries");
    bytes.extend_from_slice(&count.to_le_bytes());
    for entry in entries {
        let symbol = entry.symbol.as_bytes();
        let length = u32::try_from(symbol.len()).expect("a derived symbol is short");
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(symbol);
        for value in [
            entry.read_transport,
            entry.write_transport,
            entry.rows,
            entry.columns,
            entry.scale_bits,
            entry.bias_bits,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

// -------------------------------------------------------------------------
// Translation: verified structured kernels into this backend's representation
// -------------------------------------------------------------------------

/// Why this bounded backend refused to translate a kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
enum TranslationRefusal {
    /// The kernel binds a number of buffers this backend cannot place.
    UnsupportedBufferCount(usize),
    /// The kernel's buffers are not exactly one read and one write.
    UnsupportedAccessPattern,
    /// The read extent is not a whole number of contributors per output.
    UnsupportedContributorShape {
        /// Elements the read buffer addresses.
        read: u64,
        /// Elements the write buffer addresses.
        write: u64,
    },
    /// Two affine coefficients of the same role disagree within one kernel.
    AmbiguousAffineMap {
        /// Which coefficient disagreed.
        role: &'static str,
    },
}

impl fmt::Display for TranslationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedBufferCount(buffers) => write!(
                formatter,
                "the kernel binds {buffers} buffer(s) and this backend places two",
            ),
            Self::UnsupportedAccessPattern => {
                formatter.write_str("the kernel is not one read and one write")
            }
            Self::UnsupportedContributorShape { read, write } => write!(
                formatter,
                "a {read}-element read does not divide into {write} output(s)",
            ),
            Self::AmbiguousAffineMap { role } => {
                write!(
                    formatter,
                    "the kernel applies two different {role} constants"
                )
            }
        }
    }
}

/// Recognizes one verified kernel as an affine-contributor strict serial sum.
///
/// This backend's profile admits exactly one kernel family, so translation is
/// recognition rather than general code generation: two buffers, one read and
/// one write, a contributor count derived from their extents, and an affine
/// contributor map read out of the structured body. Anything else is refused —
/// the arms below are what stop a widened kernel from being emitted as a
/// narrower one that runs and returns the wrong answer.
///
/// The affine coefficients are read from the kernel rather than restated from
/// the semantic graph this producer happens to have built. A translator that
/// restated them would emit an image agreeing with the graph even when the plan
/// it was handed computed something else.
fn translate(
    kernel: &VerifiedKernel,
    transports: (u32, u32),
) -> Result<ScalarEntry, TranslationRefusal> {
    let buffers: Vec<_> = kernel.buffers().collect();
    if buffers.len() != 2 {
        return Err(TranslationRefusal::UnsupportedBufferCount(buffers.len()));
    }
    let reads: Vec<_> = buffers
        .iter()
        .filter(|buffer| buffer.access == BufferAccess::Read)
        .collect();
    let writes: Vec<_> = buffers
        .iter()
        .filter(|buffer| buffer.access == BufferAccess::Write)
        .collect();
    let ([read], [write]) = (reads.as_slice(), writes.as_slice()) else {
        return Err(TranslationRefusal::UnsupportedAccessPattern);
    };
    if write.element_count == 0 || read.element_count % write.element_count != 0 {
        return Err(TranslationRefusal::UnsupportedContributorShape {
            read: read.element_count,
            write: write.element_count,
        });
    }
    let rows = write.element_count;
    let columns = read.element_count / rows;

    let mut affine = AffineMap::default();
    affine.recognize(kernel, kernel.body())?;
    let (read_transport, write_transport) = transports;
    Ok(ScalarEntry {
        symbol: symbol_for(kernel),
        read_transport,
        write_transport,
        rows: u32::try_from(rows).expect("a fixture extent is small"),
        columns: u32::try_from(columns).expect("a fixture extent is small"),
        // An absent coefficient is the exact bitwise identity rather than a
        // guess: `x * 1.0` preserves sign and magnitude and `x + (-0.0)` returns
        // `x` for both zeros, so a reduction with no affine prologue emits a map
        // that changes no contributor.
        scale_bits: affine.scale.unwrap_or(0x3f80_0000),
        bias_bits: affine.bias.unwrap_or(0x8000_0000),
    })
}

/// The affine coefficients recognized so far, and their agreement obligation.
#[derive(Clone, Copy, Debug, Default)]
struct AffineMap {
    scale: Option<u32>,
    bias: Option<u32>,
}

impl AffineMap {
    /// Walks one structured block and every block nested in it.
    ///
    /// A reduction's prologue applies the map to the seed and its loop body
    /// applies it again to each later contributor, so the same two constants are
    /// seen more than once and must agree every time. The fold's own
    /// accumulate step is an `F32Add` whose right operand is a loaded
    /// contributor rather than a constant, which is what keeps it out of this.
    fn recognize(
        &mut self,
        kernel: &VerifiedKernel,
        block: BlockRef<'_>,
    ) -> Result<(), TranslationRefusal> {
        for operation in block.operations() {
            match operation.view() {
                OperationView::Binary { op, rhs, .. } => {
                    let Ok(Some(KernelConstant::F32Bits(bits))) = kernel.value_constant(rhs) else {
                        continue;
                    };
                    let (slot, role) = match op {
                        BinaryOp::F32Multiply => (&mut self.scale, "scale"),
                        BinaryOp::F32Add => (&mut self.bias, "bias"),
                        _ => continue,
                    };
                    match slot {
                        Some(seen) if *seen != bits => {
                            return Err(TranslationRefusal::AmbiguousAffineMap { role });
                        }
                        _ => *slot = Some(bits),
                    }
                }
                OperationView::Predicated { body, .. } => self.recognize(kernel, body)?,
                OperationView::SerialLoop(nested) => self.recognize(kernel, nested.body())?,
                _ => {}
            }
        }
        Ok(())
    }
}

/// Derives one entry-point symbol from a kernel's canonical identity.
fn symbol_for(kernel: &VerifiedKernel) -> String {
    let digest = DigestAlgorithm::GOVERNED
        .digest(SYMBOL_DOMAIN, kernel.canonical_identity().as_bytes())
        .label();
    format!("scalar_host_{}", &digest[..16])
}

// -------------------------------------------------------------------------
// The target profile this producer compiles against
// -------------------------------------------------------------------------

/// Declares one revision of the scalar-host target profile.
///
/// `grid_axis_threads` is the single declared fact a perturbation moves. Every
/// other axis is fixed, so two revisions differ in their exact descriptor and
/// agree on their key — the pair of subjects the loader classifies apart.
fn declare_profile(
    key: &str,
    grid_axis_threads: u64,
) -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new(key.to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = TargetProfileBuilder::new(key);
    let producer =
        TargetFactProducerIdentity::new("tiler.test.identity-join-backend".to_owned(), 1)
            .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let reference = TargetNormativeReferenceIdentity::new("ieee.754.2019.binary32".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let source = TargetFactSource::external_guarantee(producer, reference);

    builder.declare_max_threads_per_grid_axis(grid_axis_threads, source.clone())?;
    // A scalar host runs each invocation independently and stages no local
    // memory. Omitting either axis would leave it `Unknown`, which is a
    // different claim from one and zero.
    builder.declare_max_threads_per_workgroup(1, source.clone())?;
    builder.declare_max_buffer_bindings_per_entry(2, source.clone())?;
    builder.declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())?;
    builder.declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())?;
    builder.declare_device_memory(true, source.clone())?;
    builder.declare_local_memory_bytes(0, source.clone())?;
    declare_numerics(&mut builder, &source)?;
    builder.declare_dtype_dispatchability(
        F32::resolved_type(),
        DTypeDispatchability::Dispatchable,
        source,
    )?;
    builder.build()
}

/// Declares the exact numerical behaviours this backend's interpreter honours.
///
/// Narrower than the governed Metal-shaped profile: the consumer evaluates the
/// contributor sequence in the order the kernel states, so it never contracts,
/// reassociates, permutes, or eliminates a signed zero, and a contract requiring
/// any of those is refused at compile time rather than approximated at run time.
fn declare_numerics(
    builder: &mut TargetProfileBuilder,
    source: &TargetFactSource,
) -> Result<(), TargetProfileBuildError> {
    let f32_subject = ScalarArithmetic::f32();
    let rows = [
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
    for (behaviour, support) in rows {
        builder.declare_input_subnormals(
            f32_subject.clone(),
            behaviour,
            support,
            source.clone(),
        )?;
        builder.declare_result_subnormals(
            f32_subject.clone(),
            behaviour,
            support,
            source.clone(),
        )?;
    }
    for (permission, support) in [
        (NumericalPermission::Forbidden, ScalarSupport::Exact),
        (NumericalPermission::Permitted, ScalarSupport::Unsupported),
    ] {
        builder.declare_contraction(f32_subject.clone(), permission, support, source.clone())?;
        builder.declare_reassociation(f32_subject.clone(), permission, support, source.clone())?;
    }
    builder.declare_permutation(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_signed_zero(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_nan_assumptions(
        f32_subject.clone(),
        ExceptionalValueAssumption::MakeNoAssumption,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_infinity_assumptions(
        f32_subject,
        ExceptionalValueAssumption::MakeNoAssumption,
        ScalarSupport::Exact,
        source.clone(),
    )
}

// -------------------------------------------------------------------------
// The semantic program, and the compilation of it
// -------------------------------------------------------------------------

/// Builds the verified semantic graph every variant packages a plan for.
///
/// The consumer builds the same graph independently and evaluates it through
/// `tiler-reference`. Neither half reads the other's construction, so their
/// agreement on the routed result is a statement about the artifact rather than
/// about a shared expression.
fn semantic_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([ROWS, COLUMNS]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, SCALE_BITS).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, BIAS_BITS).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Compiles that graph against one revision of this backend's profile.
fn compilation_for(program: &SemanticProgram, grid_axis_threads: u64) -> Compilation {
    let profile = declare_profile(PROFILE_KEY, grid_axis_threads).expect("the profile declares");
    compile(CompileRequest::new(
        program,
        NumericalContract::StrictF32,
        TargetRequest::new([profile]).expect("a singleton target request"),
    ))
    .expect("the program compiles against the scalar-host profile")
    .into_targets()
    .pop()
    .expect("one target outcome")
    .into_parts()
    .1
    .expect("the scalar-host target compiles")
}

// -------------------------------------------------------------------------
// The variants, and what each one moves
// -------------------------------------------------------------------------

/// One join subject this producer deliberately moves, or none.
///
/// Each value moves exactly one subject and leaves every other one alone, which
/// is what makes the consumer's refusal attributable. A variant that moved two
/// would show that *something* was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Variant {
    /// Nothing moved.
    Sound,
    /// The payload declares another governed backend family.
    ForeignBackend,
    /// The payload declares another governed executable representation.
    ForeignRepresentation,
    /// The plan is compiled against another revision of the target profile.
    ForeignTargetProfile,
    /// The payload's compilation subject moves, and identity moves with it.
    MovedCompilationSubject,
    /// The payload declares it was built for another target profile.
    ForeignPayloadCompatibility,
    /// The entry mapping names a symbol the emitted object does not define.
    ForeignEntrySymbol,
    /// The entry mapping is keyed by an entry no variant entry references.
    UnmappedEntryKey,
    /// The emitted object's bytes move while every durable identity holds still.
    MovedEmittedObject,
}

impl Variant {
    /// Every variant this producer writes, in the order it writes them.
    const ALL: [Self; 9] = [
        Self::Sound,
        Self::ForeignBackend,
        Self::ForeignRepresentation,
        Self::ForeignTargetProfile,
        Self::MovedCompilationSubject,
        Self::ForeignPayloadCompatibility,
        Self::ForeignEntrySymbol,
        Self::UnmappedEntryKey,
        Self::MovedEmittedObject,
    ];

    /// Returns the directory name this variant is written under.
    const fn directory(self) -> &'static str {
        match self {
            Self::Sound => "sound",
            Self::ForeignBackend => "foreign-backend",
            Self::ForeignRepresentation => "foreign-representation",
            Self::ForeignTargetProfile => "foreign-target-profile",
            Self::MovedCompilationSubject => "moved-compilation-subject",
            Self::ForeignPayloadCompatibility => "foreign-payload-compatibility",
            Self::ForeignEntrySymbol => "foreign-entry-symbol",
            Self::UnmappedEntryKey => "unmapped-entry-key",
            Self::MovedEmittedObject => "moved-emitted-object",
        }
    }

    /// Returns the governed backend family the payload declares.
    fn backend(self) -> BackendKey {
        let key = if self == Self::ForeignBackend {
            "tiler.test.other-scalar-host"
        } else {
            BACKEND_KEY
        };
        BackendKey::new(key).expect("a governed backend key")
    }

    /// Returns the governed executable representation the payload declares.
    fn representation(self) -> RepresentationKey {
        let key = if self == Self::ForeignRepresentation {
            "tiler.test.scalar-host-image-v2"
        } else {
            REPRESENTATION_KEY
        };
        RepresentationKey::new(key).expect("a governed representation key")
    }

    /// Returns the grid-axis fact the compiled profile revision declares.
    const fn grid_axis_threads(self) -> u64 {
        if matches!(self, Self::ForeignTargetProfile) {
            OTHER_GRID_AXIS_THREADS
        } else {
            GRID_AXIS_THREADS
        }
    }

    /// Returns why this variant's envelope cannot come from the cache seam.
    ///
    /// Two variants cannot, for opposite reasons, and both are recorded in the
    /// sidecar rather than hidden. Neither is a cache defect: the first is the
    /// seam refusing to publish an envelope it cannot decode, and the second is
    /// the subject *correctly* not distinguishing two artifacts that differ only
    /// where identity deliberately does not look. Publishing the second under
    /// the sound subject would hand a later reader the sound object, which is
    /// exactly what the seam does and exactly why this variant sidesteps it.
    const fn cache_bypass(self) -> Option<&'static str> {
        match self {
            Self::MovedEmittedObject => Some(
                "artifact identity excludes the emitted object, so this variant shares the sound \
                 variant's cache subject and the seam would return the sound envelope",
            ),
            Self::UnmappedEntryKey => {
                Some("the produced envelope does not decode, so the seam refuses to publish it")
            }
            _ => None,
        }
    }
}

// -------------------------------------------------------------------------
// One produced variant
// -------------------------------------------------------------------------

/// What a produced variant hands to the consumer.
struct Produced {
    /// The exact envelope bytes the cache returned.
    envelope: Vec<u8>,
    /// The durable identity record written beside them.
    sidecar: String,
}

/// Why this producer could not write a variant.
#[derive(Debug)]
enum ProducerFailure {
    /// This backend refused to translate the plan's kernels.
    Translation(TranslationRefusal),
    /// The artifact layer refused a declaration this producer made.
    Artifact(String),
    /// Cache orchestration refused the produced artifact.
    Cache(String),
}

impl fmt::Display for ProducerFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Translation(refusal) => write!(formatter, "translation refused: {refusal}"),
            Self::Artifact(message) => write!(formatter, "artifact layer refused: {message}"),
            Self::Cache(message) => write!(formatter, "cache orchestration refused: {message}"),
        }
    }
}

impl From<ArtifactBuildError> for ProducerFailure {
    fn from(error: ArtifactBuildError) -> Self {
        Self::Artifact(error.to_string())
    }
}

/// Builds the payload metadata the artifact layer derives this payload's digest from.
///
/// The retained source is the canonical kernel-identity list the image was
/// translated from. There is no source file behind this payload, and claiming
/// one would be provenance nobody could check.
fn payload_metadata(
    kernels: &[&VerifiedKernel],
    entries: &[ScalarEntry],
    variant: Variant,
) -> Result<PayloadMetadata, ProducerFailure> {
    let mut source = Vec::new();
    let mut mappings = Vec::with_capacity(kernels.len());
    for (kernel, entry) in kernels.iter().zip(entries) {
        source.extend_from_slice(kernel.canonical_identity().as_bytes());
        let identity = kernel.canonical_identity().as_bytes();
        let entry_key = if variant == Variant::UnmappedEntryKey {
            // A key no packaged entry references. The artifact still builds and
            // encodes — the builder never re-reads a carried mapping — and the
            // decoder is what refuses, in this producer *and* in the consumer.
            BackendEntryKey::from_bytes(b"an entry key no kernel minted")?
        } else {
            BackendEntryKey::from_bytes(identity)?
        };
        let symbol = if variant == Variant::ForeignEntrySymbol {
            "scalar_host_0000000000000000".to_owned()
        } else {
            entry.symbol.clone()
        };
        mappings.push(PayloadEntryMapping {
            entry_key,
            symbol,
            transports: vec![entry.read_transport, entry.write_transport],
        });
    }
    mappings.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));
    Ok(PayloadMetadata {
        source_representation: RepresentationKey::new(SOURCE_REPRESENTATION_KEY)?,
        source,
        provenance: PayloadProvenance {
            toolchain: TOOLCHAIN_KEY.to_owned(),
            target: TARGET_TRIPLE.to_owned(),
            family: BACKEND_KEY.to_owned(),
            language: "tiler.test.scalar-host-image".to_owned(),
            // Apple-shaped required fields with no meaning for this backend.
            // ADR 0090 item 14 names that gap; stating this representation's own
            // version rather than a platform claim is the compromise the bounded
            // scalar vertical already recorded.
            deployment_major: 1,
            deployment_minor: 0,
            components: vec![ToolComponent {
                role: "translator".to_owned(),
                version: "1".to_owned(),
            }],
            sdk: PayloadSdkIdentity {
                name: TOOLCHAIN_KEY.to_owned(),
                version: "1".to_owned(),
                build: "0".to_owned(),
            },
            // The one output-affecting producer statement a variant moves
            // without touching anything else. It reaches the canonical metadata
            // bytes, so the payload digest, the artifact identity, and the cache
            // subject all move together and none of them can be moved alone.
            compile_flags: if variant == Variant::MovedCompilationSubject {
                vec!["-identity-join-revision-2".to_owned()]
            } else {
                Vec::new()
            },
            link_flags: Vec::new(),
        },
        entries: mappings,
        obligations: Vec::new(),
    })
}

/// Returns the target profile the payload declares it was built for.
fn payload_compatibility(
    variant: Variant,
    derived: &TargetProfileRef,
) -> Result<TargetProfileRef, ArtifactBuildError> {
    if variant != Variant::ForeignPayloadCompatibility {
        return Ok(derived.clone());
    }
    let other = declare_profile(PROFILE_KEY, OTHER_GRID_AXIS_THREADS)
        .expect("the other profile revision declares");
    Ok(TargetProfileRef {
        key: derived.key.clone(),
        descriptor: TargetProfileDescriptorDigest::from_bytes(other.canonical_descriptor())?,
    })
}

/// The three launch statements this backend makes, for either assembly.
fn entry_declaration(stage: tiler_ir::program::StageRef<'_>) -> BackendEntryDeclaration {
    BackendEntryDeclaration {
        bindings: vec![BindingKind::Buffer; stage.accesses().len()],
        zero_work_skips_dispatch: true,
        preconditions: Vec::new(),
    }
}

/// Assembles the descriptor-only artifact whose identity the cache subject names.
fn assemble_pending(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    variant: Variant,
    digest: &PayloadDigest,
) -> Result<VerifiedArtifactProgram, ProducerFailure> {
    assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| {
            let compatibility = payload_compatibility(variant, &profile)?;
            builder.push_payload(BackendPayloadDescriptor {
                backend: variant.backend(),
                representation: variant.representation(),
                payload_schema: PAYLOAD_SCHEMA,
                compatibility,
                execution_policy: ArtifactExecutionPolicy::NativeImage,
                digest: digest.clone(),
            })
        },
        |_, stage| Ok(entry_declaration(stage)),
    )
    .map_err(|error| ProducerFailure::Artifact(error.to_string()))
}

/// Assembles the carried artifact this producer publishes.
fn assemble_carried(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    variant: Variant,
    content: PayloadContent,
) -> Result<VerifiedArtifactProgram, ProducerFailure> {
    let mut content = Some(content);
    assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| {
            let compatibility = payload_compatibility(variant, &profile)?;
            builder.push_carried_payload(
                variant.backend(),
                variant.representation(),
                PAYLOAD_SCHEMA,
                compatibility,
                ArtifactExecutionPolicy::NativeImage,
                content.take().expect("the payload is declared once"),
            )
        },
        |_, stage| Ok(entry_declaration(stage)),
    )
    .map_err(|error| ProducerFailure::Artifact(error.to_string()))
}

/// Produces one variant: compile, translate, describe, assemble, resolve.
fn produce(cache: &ExpansionCache, variant: Variant) -> Result<Produced, ProducerFailure> {
    let semantic = semantic_program();
    let compilation = compilation_for(&semantic, variant.grid_axis_threads());
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();

    // Deliberately not the identity: ABI slot 0 is the read binding and slot 1
    // the write binding, and this backend places them on transports 1 and 0. A
    // consumer that assumed a slot occupies the transport of the same number
    // would bind the input where the output goes, and only a non-identity map
    // makes that a checked fact rather than a coincidence.
    let mut entries = Vec::with_capacity(kernels.len());
    for kernel in &kernels {
        entries.push(translate(kernel, (1, 0)).map_err(ProducerFailure::Translation)?);
    }
    let metadata = payload_metadata(&kernels, &entries, variant)?;
    let digest = metadata.identity()?;
    let mut object = encode_image(&entries);
    if variant == Variant::MovedEmittedObject {
        // The emitted object alone. Artifact identity deliberately excludes it,
        // so this variant carries the *same* durable identity as the sound one
        // and only the backend's own from-bytes validation can tell them apart.
        object.extend_from_slice(&[0x00, 0x01, 0x02]);
    }
    let content = PayloadContent {
        metadata: metadata.clone(),
        code: object,
    };
    let pending = assemble_pending(&semantic, plan, variant, &digest)?;

    let backend = variant.backend();
    let representation = variant.representation();
    let declared = DeclaredPayload {
        backend: &backend,
        representation: &representation,
        payload_schema: PAYLOAD_SCHEMA,
        execution_policy: ArtifactExecutionPolicy::NativeImage,
        digest: &digest,
        // This backend runs no external compiler, so the compilation the cache
        // subject names is the payload's own derived digest.
        compilation: digest.as_bytes(),
    };
    let expected = metadata.clone();
    let mut bypass = variant.cache_bypass().map(str::to_owned);
    if variant == Variant::UnmappedEntryKey {
        // Run the seam anyway, and record what it said. "The producer's own
        // cache orchestration refuses to publish this envelope" is half the
        // evidence; the consumer refusing the same bytes from its own decode is
        // the other half, and neither substitutes for the other.
        let refusal = accept_or_publish_single_payload_artifact(
            cache,
            &pending,
            &declared,
            |actual: &PayloadMetadata| correspondence(&expected, actual),
            || Ok::<PayloadContent, String>(content.clone()),
            |content| assemble_carried(&semantic, plan, variant, content),
        )
        .expect_err("an envelope that does not decode cannot reach the cache");
        let note = bypass.as_mut().expect("this variant bypasses the cache");
        write!(note, "; the seam refused it: {refusal}").expect("writing to a string cannot fail");
    }

    if let Some(reason) = bypass {
        let bypassed = assemble_carried(&semantic, plan, variant, content)?;
        let envelope = bypassed
            .encode()
            .map_err(|error| ProducerFailure::Artifact(error.to_string()))?;
        let sidecar = sidecar(
            variant,
            "bypassed",
            Some(&reason),
            &[],
            bypassed.canonical_identity().as_bytes(),
            &metadata,
            bypassed.payloads(),
            &derived_profile(&compilation)?,
        );
        return Ok(Produced { envelope, sidecar });
    }

    let accepted = accept_or_publish_single_payload_artifact(
        cache,
        &pending,
        &declared,
        |actual: &PayloadMetadata| correspondence(&expected, actual),
        || Ok::<PayloadContent, String>(content.clone()),
        |content| assemble_carried(&semantic, plan, variant, content),
    )
    .map_err(|failure| ProducerFailure::Cache(failure.to_string()))?;

    let resolution = match accepted.resolution() {
        Resolution::Published { .. } => "published",
        Resolution::Hit { .. } => "hit",
        Resolution::Uncached { .. } => "uncached",
    };
    let envelope = match accepted.resolution() {
        Resolution::Published { entry, .. } | Resolution::Hit { entry, .. } => {
            entry.envelope_bytes().to_vec()
        }
        Resolution::Uncached { envelope, .. } => envelope.clone(),
    };
    let sidecar = sidecar(
        variant,
        resolution,
        None,
        accepted.cache_subject().as_bytes(),
        accepted.decoded().identity().as_bytes(),
        &metadata,
        accepted.decoded().payloads(),
        accepted
            .decoded()
            .variants()
            .next()
            .expect("one packaged variant")
            .target_profile(),
    );
    Ok(Produced { envelope, sidecar })
}

/// Returns the variant target profile the compilation derives.
///
/// Only the bypass path needs it: every other variant reads the profile back
/// out of the artifact it actually produced.
fn derived_profile(compilation: &Compilation) -> Result<TargetProfileRef, ProducerFailure> {
    Ok(TargetProfileRef {
        key: tiler_artifact::program::TargetProfileKey::new(compilation.target_profile_key())?,
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )?,
    })
}

/// Compares one carried payload's metadata against the compilation performed.
fn correspondence(
    expected: &PayloadMetadata,
    actual: &PayloadMetadata,
) -> Result<(), &'static str> {
    let facts = [
        (
            actual.source_representation == expected.source_representation,
            "source-representation",
        ),
        (actual.source == expected.source, "source"),
        (
            actual.provenance.toolchain == expected.provenance.toolchain,
            "toolchain",
        ),
        (
            actual.provenance.target == expected.provenance.target,
            "target",
        ),
        (actual.entries == expected.entries, "entries"),
    ];
    for (agrees, fact) in facts {
        if !agrees {
            return Err(fact);
        }
    }
    Ok(())
}

// -------------------------------------------------------------------------
// The durable identity record
// -------------------------------------------------------------------------

/// Renders one byte run as lower-case hexadecimal.
fn hex(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(rendered, "{byte:02x}").expect("writing to a string cannot fail");
    }
    rendered
}

/// Writes the durable record a consumer joins on.
///
/// Every line is a governed key or a digest over canonical bytes. Nothing here
/// is an address, a `TypeId`, a vtable, a symbol resolution, or a registration
/// order, which is why two processes writing this file write the same bytes.
#[expect(
    clippy::too_many_arguments,
    reason = "one record with one field per governed join subject; grouping them into a struct \
              would add a type whose only purpose is to be destructured here"
)]
fn sidecar(
    variant: Variant,
    resolution: &str,
    bypass_reason: Option<&str>,
    cache_subject: &[u8],
    artifact_identity: &[u8],
    metadata: &PayloadMetadata,
    payloads: &[BackendPayloadDescriptor],
    variant_profile: &TargetProfileRef,
) -> String {
    let [descriptor] = payloads else {
        panic!("this producer declares exactly one payload per artifact");
    };
    let mut record = String::new();
    let mut line = |key: &str, value: &str| {
        writeln!(record, "{key} = {value}").expect("writing to a string cannot fail");
    };
    line("variant", variant.directory());
    line("resolution", resolution);
    line("producer-pid", &std::process::id().to_string());
    if let Some(reason) = bypass_reason {
        line("cache-bypass", reason);
    }
    line("artifact-identity", &hex(artifact_identity));
    line("cache-subject", &hex(cache_subject));
    line("backend", descriptor.backend.as_str());
    line("representation", descriptor.representation.as_str());
    line(
        "payload-schema",
        &format!(
            "{}.{}",
            descriptor.payload_schema.major(),
            descriptor.payload_schema.minor(),
        ),
    );
    line(
        "execution-policy",
        &format!("{:?}", descriptor.execution_policy),
    );
    line("payload-digest", &hex(descriptor.digest.as_bytes()));
    line(
        "payload-compatibility-key",
        descriptor.compatibility.key.as_str(),
    );
    line(
        "payload-compatibility-descriptor",
        &hex(descriptor.compatibility.descriptor.as_bytes()),
    );
    line("target-profile-key", variant_profile.key.as_str());
    line(
        "target-profile-descriptor",
        &hex(variant_profile.descriptor.as_bytes()),
    );
    for mapping in &metadata.entries {
        line(
            "entry",
            &format!("{} {}", hex(mapping.entry_key.as_bytes()), mapping.symbol),
        );
    }
    record
}

// -------------------------------------------------------------------------
// The two runs
// -------------------------------------------------------------------------

/// Writes one run's variants under `directory`, sharing `cache`.
fn write_run(cache: &ExpansionCache, directory: &Path, variants: &[Variant]) {
    for variant in variants {
        let produced = produce(cache, *variant)
            .unwrap_or_else(|failure| panic!("{}: {failure}", variant.directory()));
        let target = directory.join(variant.directory());
        std::fs::create_dir_all(&target).expect("the variant directory is creatable");
        std::fs::write(target.join("artifact.bin"), &produced.envelope)
            .expect("the envelope is writable");
        std::fs::write(target.join("sidecar.txt"), produced.sidecar.as_bytes())
            .expect("the sidecar is writable");
    }
}

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let root = PathBuf::from(
        arguments
            .next()
            .expect("usage: identity_join_producer <root-directory> [--child]"),
    );
    let child = arguments.next().is_some_and(|flag| flag == "--child");
    let cache = ExpansionCache::open(root.join("cache"));

    if child {
        write_run(&cache, &root.join("run-b"), &[Variant::Sound]);
        return;
    }

    write_run(&cache, &root.join("run-a"), &Variant::ALL);

    // A second live process against the same cache root, from a different
    // working directory and with an extra variable in its environment. Anything
    // process-scoped that had reached a durable identity would move here.
    let status = Command::new(std::env::current_exe().expect("a program knows its own path"))
        .arg(&root)
        .arg("--child")
        .current_dir(&root)
        .env("TILER_IDENTITY_JOIN_CHILD", "1")
        .status()
        .expect("the producer re-executes itself");
    assert!(status.success(), "the second producer run failed: {status}");
}
