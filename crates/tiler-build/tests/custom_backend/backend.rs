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
//! 4. [`assemble`] — orchestration through `tiler_build::assemble_plan_artifact`,
//!    which is where this backend stops making statements the plan already made.
//!
//! # What this producer deliberately cannot do
//!
//! It cannot state the artifact's target-profile reference, feasibility rules,
//! selected providers, deferred predicates, or any entry's backend entry key.
//! Those are derived from the owner-linked plan by the facade and there is no
//! parameter to supply them through, which is the property the suite's mutation
//! cases are written against.

use tiler_artifact::program::{
    AbiBinaryOp, AbiExprId, AbiRoot, ArtifactBuildError, ArtifactExecutionPolicy,
    ArtifactProgramBuilder, AvailabilityPhase, BackendEntryKey, BackendKey, BindingKind,
    DecodedArtifact, DigestAlgorithm, PayloadContent, PayloadEntryMapping, PayloadMetadata,
    PayloadProvenance, PayloadSdkIdentity, RepresentationKey, SchemaVersion, TargetPropertyKey,
    ToolComponent, VerifiedArtifactProgram,
};
use tiler_build::{BackendEntryDeclaration, PlanArtifactError, assemble_plan_artifact};
use tiler_cache::expansion::{
    ComposedSubject, ExpansionCache, PublishFailure, Resolution, SubjectFacets, SubjectRefusal,
};
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::kernel::{BufferAccess, VerifiedKernel};
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
    /// A cache result carries an artifact other than the one published.
    CacheIdentity,
}

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
            // Apple-shaped required fields with no meaning for this backend.
            // ADR 0090 item 14 names that gap; stating this representation's own
            // version rather than a platform claim is the compromise the CPU
            // vertical recorded, not a new one.
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
            compile_flags: Vec::new(),
            link_flags: Vec::new(),
        },
        entries,
        obligations: Vec::new(),
    })
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
            builder.push_carried_payload(
                backend(),
                representation(),
                PAYLOAD_SCHEMA,
                profile,
                ArtifactExecutionPolicy::NativeImage,
                content,
            )
        },
        |builder, stage| {
            let declared = perturbation
                .bindings
                .unwrap_or_else(|| stage.accesses().len());
            Ok(BackendEntryDeclaration {
                bindings: vec![BindingKind::Buffer; declared],
                zero_work_skips_dispatch: !perturbation.forbid_zero_work_skip,
                preconditions: vec![scratch_precondition(builder)?],
            })
        },
    )?)
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
            let object = artifact
                .payload_object(entry.payload())
                .ok_or(ScalarHostRefusal::MissingPayloadObject)?;
            let image = decode(object).map_err(ScalarHostRefusal::Payload)?;
            let symbol = entry
                .backend_symbol()
                .ok_or(ScalarHostRefusal::UnmappedBackendEntry)?;
            let transports = entry
                .transport_slots()
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
    Ok(())
}

/// Publishes or accepts one artifact under its complete composed cache subject.
///
/// The subject is composed by `tiler_cache`'s own constructor from two facets:
/// the payload's derived compilation digest and the artifact's canonical
/// identity. Neither is this backend's to state — the first is derived by the
/// artifact builder from the metadata's canonical bytes and the second by
/// whole-artifact verification — so a producer cannot file bytes under a subject
/// naming a compilation it did not perform.
///
/// # Errors
///
/// Returns the typed subject, codec, or protocol refusal. A cache result whose
/// artifact identity differs from the published one is hard: it is never
/// translated into a miss or an automatic rebuild.
pub fn accept_or_publish(
    cache: &ExpansionCache,
    artifact: &VerifiedArtifactProgram,
    compilation: &[u8],
) -> Result<Resolution, ScalarHostRefusal> {
    let expected = artifact.canonical_identity().clone();
    let compilations = [compilation];
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &compilations,
        artifact_program: expected.as_bytes(),
    })
    .map_err(ScalarHostRefusal::CacheSubject)?;
    let resolution = cache
        .get_or_publish(&subject, || artifact.encode())
        .map_err(|failure| match failure {
            PublishFailure::Build(error) => ScalarHostRefusal::CacheEncoding(error.to_string()),
            PublishFailure::Artifact(error) => ScalarHostRefusal::CacheArtifact(error.to_string()),
        })?;
    let decoded = match &resolution {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => entry.artifact(),
        Resolution::Uncached { artifact, .. } => artifact,
    };
    if decoded.identity().as_bytes() != expected.as_bytes() {
        return Err(ScalarHostRefusal::CacheIdentity);
    }
    validate_from_bytes(decoded)?;
    Ok(resolution)
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
