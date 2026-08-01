//! A statically linked custom backend producer that is not Metal.
//!
//! It lives outside every crate in the workspace and compiles against public
//! surfaces alone. Its four stages are kept apart on purpose, because
//! [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! requires a backend to separate them even when one component implements all
//! four:
//!
//! 1. [`require_compiled_under`] — refuse a plan assessed against another target
//!    *before* any translation, for the reason the Metal path refuses one before
//!    emission: everything translation then decides was decided against a target
//!    the artifact will not declare.
//! 2. [`emit`] — pure translation of verified structured kernels into this
//!    backend's own representation. It touches no artifact type, invokes no
//!    tool, and its refusals are its own.
//! 3. [`payload_metadata`] — the compilation subject the artifact layer derives
//!    the payload digest from. This backend runs no external compiler, so its
//!    provenance names an in-process translator and no SDK; that honesty is what
//!    makes the digest mean something.
//! 4. [`assemble`] — orchestration through `tiler_build::assemble_plan_artifact`
//!    and `tiler_build::accept_or_publish_delivered_payload_artifact`, which is
//!    where this backend stops making statements the plan already made.
//!
//! # What this backend states to the promoted cache seam, and what it does not
//!
//! Two statements, and they are of two kinds. [`PayloadDeclaration`] is *data*:
//! the governed keys, schema, and policy its sole payload declares, its derived
//! compilation digest, and the canonical compilation bytes the cache subject
//! names. [`correspondence`] is *behaviour*: which compilation facts this
//! backend authored, in which order they are compared, and what each is called
//! when it disagrees. Everything else — subject composition, miss-only
//! compilation, identity agreement before publication, re-validation of every
//! result — is the seam's, and there is no parameter to supply any of it
//! through.
//!
//! # What this producer deliberately cannot do
//!
//! It cannot state the artifact's target-profile reference, feasibility rules,
//! selected providers, deferred predicates, or any entry's backend entry key.
//! Those are derived from the owner-linked plan by the facade and there is no
//! parameter to supply them through, which is the property the suite's mutation
//! cases are written against.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    AbiBinaryOp, AbiExprId, AbiRoot, ArtifactBuildError, ArtifactExecutionPolicy,
    ArtifactProgramBuilder, AvailabilityPhase, BackendEntryKey, BackendKey,
    BackendPayloadDescriptor, BindingKind, DecodedArtifact, DigestAlgorithm, PayloadContent,
    PayloadDigest, PayloadEntryMapping, PayloadMetadata, PayloadPlatform, PayloadProvenance,
    RepresentationKey, SchemaVersion, TargetPropertyKey, ToolComponent, VerifiedArtifactProgram,
};
use tiler_build::{
    BackendEntryDeclaration, DeclaredPayload, PlanArtifactError, assemble_plan_artifact,
};
use tiler_cache::expansion::{
    ComposedSubject, ExpansionCache, PublishFailure, SubjectFacets, SubjectRefusal,
};
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::kernel::{BufferAccess, VerifiedKernel};
use tiler_ir::program::StageRef;
use tiler_ir::semantic::SemanticProgram;

use crate::image::{ScalarEntry, ScalarImage, ScalarImageRefusal, decode};
use crate::profile::{
    BACKEND_KEY, PROFILE_KEY, REPRESENTATION_KEY, SOURCE_REPRESENTATION_KEY, TARGET_TRIPLE,
    TOOLCHAIN_KEY,
};

/// Governed launch-time property this backend's entries place a floor on.
pub const SCRATCH_PROPERTY_KEY: &str = "tiler.test.scalar-host.launch.scratch-bytes";

/// Scratch bytes this backend's entries require at launch.
pub const SCRATCH_BYTES_FLOOR: u64 = 4_096;

/// Governed schema version of this backend's payload.
pub const PAYLOAD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0);

/// Domain separator under which entry-point symbols are derived.
const SYMBOL_DOMAIN: &[u8] = b"tiler.test.scalar-host.symbol.v1\0";

/// Why this backend refused to produce or accept a payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScalarHostRefusal {
    /// The plan was compiled under another target-profile key.
    ForeignProfileKey {
        /// Key this backend declares.
        declared: String,
        /// Key the plan was compiled under.
        compiled: String,
    },
    /// The plan was compiled under this key but a different profile revision.
    ForeignProfileDescriptor,
    /// A kernel binds a number of buffers this bounded backend cannot place.
    UnsupportedBufferCount {
        /// Ordinal of the refused kernel.
        entry: usize,
        /// Buffers the kernel binds.
        buffers: usize,
    },
    /// A kernel's buffer access pattern is not one read and one write.
    UnsupportedAccessPattern {
        /// Ordinal of the refused kernel.
        entry: usize,
    },
    /// The artifact layer rejected a declaration this backend made.
    Artifact(String),
    /// The neutral plan-assembly facade refused the declared artifact.
    Assembly(String),
    /// The carried bytes are not a scalar image this build executes.
    Payload(ScalarImageRefusal),
    /// The artifact declares a payload it does not carry.
    MissingPayloadObject,
    /// An artifact entry names a backend entry the payload maps no symbol for.
    UnmappedBackendEntry,
    /// The mapped symbol names no entry of the carried image.
    UnmappedSymbol,
    /// The image places bindings on transports the artifact's mapping does not.
    TransportDisagreement,
    /// The complete cache subject could not be composed.
    CacheSubject(SubjectRefusal),
    /// The verified artifact could not be encoded for publication.
    CacheEncoding(String),
    /// The cache's governed validator rejected the produced envelope.
    CacheArtifact(String),
}

impl fmt::Display for ScalarHostRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignProfileKey { declared, compiled } => write!(
                formatter,
                "this backend declares `{declared}` and the plan was compiled under `{compiled}`",
            ),
            Self::ForeignProfileDescriptor => {
                formatter.write_str("the plan was compiled under another revision of this profile")
            }
            Self::UnsupportedBufferCount { entry, buffers } => write!(
                formatter,
                "kernel {entry} binds {buffers} buffers and this backend places two",
            ),
            Self::UnsupportedAccessPattern { entry } => {
                write!(formatter, "kernel {entry} is not one read and one write")
            }
            Self::Artifact(message) => write!(formatter, "artifact layer refusal: {message}"),
            Self::Assembly(message) => write!(formatter, "plan assembly refusal: {message}"),
            Self::Payload(refusal) => refusal.fmt(formatter),
            Self::MissingPayloadObject => {
                formatter.write_str("the artifact declares a payload it does not carry")
            }
            Self::UnmappedBackendEntry => {
                formatter.write_str("an entry reaches no mapping in the carried payload")
            }
            Self::UnmappedSymbol => {
                formatter.write_str("the mapped symbol names no entry of the carried image")
            }
            Self::TransportDisagreement => formatter
                .write_str("the image places bindings on transports the artifact does not state"),
            Self::CacheSubject(refusal) => refusal.fmt(formatter),
            Self::CacheEncoding(message) => write!(formatter, "artifact encoding: {message}"),
            Self::CacheArtifact(message) => {
                write!(formatter, "cache artifact validator: {message}")
            }
        }
    }
}

impl Error for ScalarHostRefusal {}

/// One compilation fact this backend authored and compares on every payload.
///
/// The naming half the promoted seam delegates: only this backend knows which
/// facts its metadata asserts, so only it can say which one disagreed. The
/// Apple-shaped provenance fields
/// [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
/// item 14 names as meaningless here — the platform family, the language, the
/// deployment minimum, the SDK, and both flag lists — are deliberately outside
/// this check, as are the entry mappings and target obligations, which are
/// translation facts rather than compilation ones. The seam's descriptor
/// comparison subsumes every one of them, because the payload digest is derived
/// from the canonical metadata bytes; what this enum adds is the *name*.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarHostFact {
    /// Governed representation key of the retained source.
    SourceRepresentation,
    /// The exact kernel-identity list this image was translated from.
    Source,
    /// Governed identity of the in-process translator.
    Toolchain,
    /// Declared target triple.
    Target,
    /// Ordered translator components.
    Components,
}

impl ScalarHostFact {
    /// Returns the stable diagnostic spelling of this fact.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceRepresentation => "source-representation",
            Self::Source => "source",
            Self::Toolchain => "toolchain",
            Self::Target => "target",
            Self::Components => "components",
        }
    }
}

impl fmt::Display for ScalarHostFact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "carried scalar-host payload disagrees with compilation fact `{}`",
            self.as_str(),
        )
    }
}

impl Error for ScalarHostFact {}

impl From<ArtifactBuildError> for ScalarHostRefusal {
    fn from(error: ArtifactBuildError) -> Self {
        Self::Artifact(error.to_string())
    }
}

impl From<PlanArtifactError> for ScalarHostRefusal {
    fn from(error: PlanArtifactError) -> Self {
        Self::Assembly(error.to_string())
    }
}

/// Returns this backend's governed family key.
///
/// # Panics
///
/// Panics when the constant is not a governed key, which is a defect in this
/// file rather than a case under test.
#[must_use]
pub fn backend() -> BackendKey {
    BackendKey::new(BACKEND_KEY).expect("a governed backend key")
}

/// Returns this backend's governed representation key.
///
/// # Panics
///
/// Panics when the constant is not a governed key.
#[must_use]
pub fn representation() -> RepresentationKey {
    RepresentationKey::new(REPRESENTATION_KEY).expect("a governed representation key")
}

/// Refuses a plan assessed against another target before any translation runs.
///
/// # Errors
///
/// Returns the exact half that disagreed. A different key and a different
/// revision of the same key are two findings with two remedies, so they stay
/// two refusals — the same distinction the Metal declaration draws.
pub fn require_compiled_under(compilation: &Compilation) -> Result<(), ScalarHostRefusal> {
    if compilation.target_profile_key() != PROFILE_KEY {
        return Err(ScalarHostRefusal::ForeignProfileKey {
            declared: PROFILE_KEY.to_owned(),
            compiled: compilation.target_profile_key().to_owned(),
        });
    }
    let declared = crate::profile::scalar_host_profile()
        .expect("this backend's own profile declares")
        .canonical_descriptor()
        .to_vec();
    if compilation.target_profile_descriptor() != declared.as_slice() {
        return Err(ScalarHostRefusal::ForeignProfileDescriptor);
    }
    Ok(())
}

/// Translates verified structured kernels into this backend's representation.
///
/// Pure: it reads the kernels, invokes nothing, and knows no artifact type. The
/// transport map it writes is deliberately **not** the identity — binding slot
/// `n` occupies transport `bindings - 1 - n` — so a consumer that assumed the
/// two coincide binds the wrong storage and this suite's validation says so.
///
/// # Errors
///
/// Returns the first kernel this bounded backend cannot place, naming its
/// ordinal. A wider signature is refused rather than partially bound.
pub fn emit(kernels: &[&VerifiedKernel]) -> Result<ScalarImage, ScalarHostRefusal> {
    let mut entries = Vec::with_capacity(kernels.len());
    for (entry, kernel) in kernels.iter().enumerate() {
        let buffers: Vec<_> = kernel.buffers().collect();
        if buffers.len() != 2 {
            return Err(ScalarHostRefusal::UnsupportedBufferCount {
                entry,
                buffers: buffers.len(),
            });
        }
        let writes: Vec<_> = buffers
            .iter()
            .filter(|buffer| buffer.access == BufferAccess::Write)
            .collect();
        let [write] = writes.as_slice() else {
            return Err(ScalarHostRefusal::UnsupportedAccessPattern { entry });
        };
        let transports = (0..u32::try_from(buffers.len()).expect("two bindings"))
            .rev()
            .collect();
        entries.push(ScalarEntry {
            symbol: symbol_for(kernel),
            transports,
            work_items: write.element_count,
        });
    }
    Ok(ScalarImage { entries })
}

/// Derives one entry-point symbol from a kernel's canonical identity.
///
/// Identity-derived rather than named, so two implementations of one region
/// cannot collide on a symbol and a renamed kernel cannot silently keep one.
#[must_use]
pub fn symbol_for(kernel: &VerifiedKernel) -> String {
    let digest = DigestAlgorithm::GOVERNED
        .digest(SYMBOL_DOMAIN, kernel.canonical_identity().as_bytes())
        .label();
    format!("scalar_host_{}", &digest[..16])
}

/// Builds the compilation subject the artifact layer digests this payload under.
///
/// The retained "source" is the canonical kernel identity list this image was
/// translated from. There is no source file behind this payload and claiming one
/// would be provenance nobody can check.
///
/// # Errors
///
/// Returns the artifact layer's typed key failure.
pub fn payload_metadata(
    kernels: &[&VerifiedKernel],
    image: &ScalarImage,
) -> Result<PayloadMetadata, ScalarHostRefusal> {
    let mut source = Vec::new();
    let mut entries = Vec::with_capacity(kernels.len());
    for (kernel, entry) in kernels.iter().zip(&image.entries) {
        source.extend_from_slice(kernel.canonical_identity().as_bytes());
        entries.push(PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(kernel.canonical_identity().as_bytes())?,
            symbol: entry.symbol.clone(),
            transports: entry.transports.clone(),
        });
    }
    entries.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));
    Ok(PayloadMetadata {
        source_representation: RepresentationKey::new(SOURCE_REPRESENTATION_KEY)?,
        source,
        provenance: PayloadProvenance {
            toolchain: TOOLCHAIN_KEY.to_owned(),
            target: TARGET_TRIPLE.to_owned(),
            family: BACKEND_KEY.to_owned(),
            language: "tiler.test.scalar-host-image".to_owned(),
            // This backend translates kernel IR to its own host image. There is
            // no SDK and no platform deployment minimum to state, and it says so
            // rather than minting values with no referent on its target — which
            // is the whole of what ADR 0090 item 14 named. Every field it does
            // owe is above; the four this shape does not owe are absent rather
            // than approximated.
            platform: PayloadPlatform::Unversioned,
            components: vec![ToolComponent {
                role: "translator".to_owned(),
                version: "1".to_owned(),
            }],
            compile_flags: Vec::new(),
            link_flags: Vec::new(),
        },
        entries,
        obligations: Vec::new(),
    })
}

/// What this backend states to the promoted cache seam, as data.
///
/// The fields are owned and public so a case can perturb exactly one of them
/// before handing the declaration over. Nothing here is derived by the seam: a
/// producer that declared another representation, schema, policy, or digest is
/// declaring a payload it did not prepare, and the seam says so.
#[derive(Clone, Debug)]
pub struct PayloadDeclaration {
    /// Governed backend family the sole payload must declare.
    pub backend: BackendKey,
    /// Governed executable representation it must declare.
    pub representation: RepresentationKey,
    /// Schema version of this backend's payload metadata.
    pub payload_schema: SchemaVersion,
    /// How this backend's payload reaches an executable state.
    pub execution_policy: ArtifactExecutionPolicy,
    /// Digest the artifact layer derived from the metadata's canonical bytes.
    pub digest: PayloadDigest,
    /// Canonical bytes of the compilation the cache subject names.
    pub compilation: Vec<u8>,
}

impl PayloadDeclaration {
    /// Borrows this declaration in the shape the promoted seam reads.
    #[must_use]
    pub fn declared(&self) -> DeclaredPayload<'_> {
        DeclaredPayload {
            backend: &self.backend,
            representation: &self.representation,
            payload_schema: self.payload_schema,
            execution_policy: self.execution_policy,
            digest: &self.digest,
            compilation: &self.compilation,
        }
    }
}

/// One translated image, its compilation subject, and what it declares.
pub struct PreparedScalarPayload {
    /// The image this backend translated the kernels into.
    pub image: ScalarImage,
    /// The compilation subject the artifact layer digests this payload under.
    pub metadata: PayloadMetadata,
    /// What the promoted cache seam compares every result against.
    pub declaration: PayloadDeclaration,
}

/// Translates one plan's kernels and derives everything the cache seam needs.
///
/// # Errors
///
/// Returns this backend's translation refusal or the artifact layer's typed key
/// or digest failure.
pub fn prepare(kernels: &[&VerifiedKernel]) -> Result<PreparedScalarPayload, ScalarHostRefusal> {
    let image = emit(kernels)?;
    let metadata = payload_metadata(kernels, &image)?;
    let digest = metadata.identity()?;
    Ok(PreparedScalarPayload {
        declaration: PayloadDeclaration {
            backend: backend(),
            representation: representation(),
            payload_schema: PAYLOAD_SCHEMA,
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            // This backend runs no external compiler, so the compilation the
            // cache subject names is the payload's own derived digest. Naming a
            // toolchain resolution it never performed would be a facet nobody
            // could check, and the seam wraps this run rather than parsing it.
            compilation: digest.as_bytes().to_vec(),
            digest,
        },
        image,
        metadata,
    })
}

/// Compares one carried payload's metadata against the compilation performed.
///
/// The order is the contract, exactly as it is for the Metal path: a producer
/// reading two refusals in sequence is reading them in this order.
///
/// # Errors
///
/// Returns the first fact that disagreed.
pub fn correspondence(
    expected: &PayloadMetadata,
    actual: &PayloadMetadata,
) -> Result<(), ScalarHostFact> {
    let facts = [
        (
            actual.source_representation == expected.source_representation,
            ScalarHostFact::SourceRepresentation,
        ),
        (actual.source == expected.source, ScalarHostFact::Source),
        (
            actual.provenance.toolchain == expected.provenance.toolchain,
            ScalarHostFact::Toolchain,
        ),
        (
            actual.provenance.target == expected.provenance.target,
            ScalarHostFact::Target,
        ),
        (
            actual.provenance.components == expected.provenance.components,
            ScalarHostFact::Components,
        ),
    ];
    for (agrees, fact) in facts {
        if !agrees {
            return Err(fact);
        }
    }
    Ok(())
}

/// What this backend declares per entry, and the one knob a perturbation moves.
///
/// `bindings` exists so a case can declare a binding count other than the
/// stage's without a second copy of [`assemble`]. It is `None` for the
/// backend's real answer — one binding per stage access — and a count for the
/// perturbation that must be refused.
#[derive(Clone, Copy, Debug, Default)]
pub struct EntryPerturbation {
    /// Bindings to declare per entry, or `None` for the stage's own count.
    pub bindings: Option<usize>,
    /// Whether to declare a zero-thread launch as unskippable.
    pub forbid_zero_work_skip: bool,
}

/// Assembles one verified artifact through the neutral build orchestration seam.
///
/// The two closures are the whole of what this backend states: which payload it
/// carries, and — per stage — how each binding is transported, whether a
/// zero-thread launch is skippable, and what must hold at launch time. The
/// launch precondition is minted on the builder the facade hands over, so its
/// handle belongs to that builder and one from anywhere else is a typed refusal.
///
/// # Errors
///
/// Returns this backend's wrapping of the facade's typed refusal.
pub fn assemble(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    content: PayloadContent,
    perturbation: EntryPerturbation,
) -> Result<VerifiedArtifactProgram, ScalarHostRefusal> {
    Ok(assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| {
            builder
                .push_carried_payload(
                    backend(),
                    representation(),
                    PAYLOAD_SCHEMA,
                    profile,
                    ArtifactExecutionPolicy::NativeImage,
                    content,
                )
                .map(|payload| vec![payload])
        },
        |builder, stage| entry_declaration(builder, stage, perturbation),
    )?)
}

/// Assembles the descriptor-only artifact whose identity the cache subject names.
///
/// The pending and carried assemblies share [`entry_declaration`] rather than
/// each spelling the three launch statements, because the promoted seam requires
/// their canonical identities to agree and two copies that could drift apart
/// would fail that agreement rather than the statement they disagreed about.
///
/// # Errors
///
/// Returns the neutral facade's typed refusal.
pub fn assemble_pending(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    declaration: &PayloadDeclaration,
    perturbation: EntryPerturbation,
) -> Result<VerifiedArtifactProgram, ScalarHostRefusal> {
    Ok(assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| {
            builder
                .push_payload(BackendPayloadDescriptor {
                    backend: declaration.backend.clone(),
                    representation: declaration.representation.clone(),
                    payload_schema: declaration.payload_schema,
                    compatibility: profile,
                    execution_policy: declaration.execution_policy,
                    digest: declaration.digest.clone(),
                })
                .map(|payload| vec![payload])
        },
        |builder, stage| entry_declaration(builder, stage, perturbation),
    )?)
}

/// The three launch statements this backend makes, for either assembly.
fn entry_declaration(
    builder: &mut ArtifactProgramBuilder,
    stage: StageRef<'_>,
    perturbation: EntryPerturbation,
) -> Result<BackendEntryDeclaration, ArtifactBuildError> {
    let declared = perturbation
        .bindings
        .unwrap_or_else(|| stage.accesses().len());
    Ok(BackendEntryDeclaration {
        bindings: vec![BindingKind::Buffer; declared],
        zero_work_skips_dispatch: !perturbation.forbid_zero_work_skip,
        preconditions: vec![scratch_precondition(builder)?],
    })
}

/// Mints this backend's launch-time scratch floor on the facade's own builder.
///
/// # Panics
///
/// Panics when [`SCRATCH_PROPERTY_KEY`] is not a governed property key, which is
/// a defect in this file rather than a case under test.
fn scratch_precondition(
    builder: &mut ArtifactProgramBuilder,
) -> Result<AbiExprId, ArtifactBuildError> {
    let observed = builder.push_root(AbiRoot::TargetProperty {
        key: TargetPropertyKey::new(SCRATCH_PROPERTY_KEY).expect("a governed property key"),
        phase: AvailabilityPhase::LaunchPreflight,
    })?;
    let required = builder.push_root(AbiRoot::UnsignedLiteral(SCRATCH_BYTES_FLOOR))?;
    builder.push_binary(AbiBinaryOp::LessOrEqual, required, observed)
}

/// Validates this backend's own payload from the bytes an envelope carries.
///
/// The obligation ADR 0090 item 8 places on every backend, and the one the
/// artifact layer provably cannot discharge: the image is decoded from bytes
/// that could have come from anywhere, and every artifact entry is proven to
/// reach a symbol the image actually declares, on the transports the artifact's
/// own mapping states.
///
/// # Errors
///
/// Returns the first disagreement in the order it is checked.
pub fn validate_from_bytes(artifact: &DecodedArtifact) -> Result<(), ScalarHostRefusal> {
    for variant in artifact.variants() {
        for entry in variant.entries() {
            // Every delivery position, because a consumer at any of them loads
            // that position's object and this backend validates what it would
            // execute rather than a representative of it.
            for delivery in 0..entry.delivery_positions() {
                let object = entry
                    .payload(delivery)
                    .and_then(|payload| artifact.payload_object(payload))
                    .ok_or(ScalarHostRefusal::MissingPayloadObject)?;
                let image = decode(object).map_err(ScalarHostRefusal::Payload)?;
                let symbol = entry
                    .backend_symbol(delivery)
                    .ok_or(ScalarHostRefusal::UnmappedBackendEntry)?;
                let transports = entry
                    .transport_slots(delivery)
                    .ok_or(ScalarHostRefusal::UnmappedBackendEntry)?;
                let placed = image
                    .entries
                    .iter()
                    .find(|candidate| candidate.symbol == symbol)
                    .ok_or(ScalarHostRefusal::UnmappedSymbol)?;
                if placed.transports.as_slice() != transports {
                    return Err(ScalarHostRefusal::TransportDisagreement);
                }
            }
        }
    }
    Ok(())
}

/// Files one artifact's envelope under a *different* artifact's cache subject.
///
/// A deliberate protocol violation, used by one case to prove the re-check after
/// resolution can say no. Nothing in a production path may do this; it exists
/// because a check nobody has watched fail is not evidence.
///
/// # Errors
///
/// Returns the typed subject or publication refusal.
pub fn publish_under_foreign_subject(
    cache: &ExpansionCache,
    subject_of: &VerifiedArtifactProgram,
    compilation: &[u8],
    published: &VerifiedArtifactProgram,
) -> Result<(), ScalarHostRefusal> {
    let compilations = [compilation];
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &compilations,
        artifact_program: subject_of.canonical_identity().as_bytes(),
    })
    .map_err(ScalarHostRefusal::CacheSubject)?;
    cache
        .get_or_publish(&subject, || published.encode())
        .map_err(|failure| match failure {
            PublishFailure::Build(error) => ScalarHostRefusal::CacheEncoding(error.to_string()),
            PublishFailure::Artifact(error) => ScalarHostRefusal::CacheArtifact(error.to_string()),
        })?;
    Ok(())
}
