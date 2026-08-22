//! Metal's specialization of the neutral expansion-cache seam.
//!
//! Everything structural — subject composition, miss-only compilation, identity
//! agreement before publication, re-validation of every result — belongs to
//! [`crate::payload_cache`] and is shared with every other backend. What is
//! Metal's and stays here is three statements: the governed
//! `tiler.metal`/`metallib`/`NativeImage` payload descriptor this backend
//! declares, the fact-level correspondence between a carried payload's metadata
//! and the Apple compilation that was prepared for it, and what a succeeding
//! compilation retains beside its entry. The first is data and travels in a
//! [`DeclaredPayload`]; the second is a closure, because naming *which*
//! compilation fact disagreed is a judgement only this backend can make; the
//! third is [`stage_retention`], because only this backend knows that a Metal
//! compilation is two tools and which one wrote what.
//!
//! # Several artifact families, one envelope
//!
//! A selection naming several artifact families is one compilation, one plan,
//! one kernel program, and one compiled object per family — so this takes a
//! *run* of prepared payloads in delivery order and the neutral seam resolves
//! each delivery position through the artifact's own entries. The correspondence
//! closure is what makes a wrong-position payload a build error rather than a
//! wrong artifact: position `p`'s decoded metadata is compared against the
//! compilation prepared for position `p`, and two families whose objects were
//! placed the other way round disagree on
//! [`MetalPayloadFact::Target`](crate::MetalPayloadFact::Target) — the AOT
//! triple, which is the one fact that distinguishes them, since the ledger
//! records the artifact family as backend-only and the two share a byte-identical
//! compiler profile descriptor.
//!
//! The refusal vocabulary below is Metal's own and is preserved exactly: the
//! neutral seam's protocol refusals map one-to-one onto it, so a caller reading
//! [`MetalArtifactProtocolError`] sees the same kinds in the same order.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactCodecFailure, ArtifactExecutionPolicy, VerifiedArtifactProgram,
};
use tiler_cache::expansion::{DebugRetention, ExpansionCache, RetentionRefusal, SubjectRefusal};
use tiler_metal_aot::diagnostic::CompileStage;
use tiler_metal_aot::record::StageOutputs;

use crate::MetalPayloadMismatch;
use crate::metal_assembly::{
    BACKEND, CompiledMetalPayload, MetalAssemblyError, PAYLOAD_SCHEMA, PreparedMetalPayload,
};
use crate::metal_payload::validate_metal_payload_metadata;
use crate::payload_cache::{
    AcceptedArtifact, CompiledPayloads, DeclaredPayload, DeliveredPayloadCacheError,
    DeliveredPayloadProtocolError, accept_or_publish_delivered_payload_artifact,
};

/// A decoded or assembled artifact contradicts the prepared Metal subjects.
///
/// Every position-scoped variant names the **delivery position** it is about —
/// the ordered slot a consumer's build target resolves to — because with one
/// object per artifact family "which one disagreed" is the first thing a
/// producer needs, and a descriptor-table index would name a canonical content
/// slot the producer never chose.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalArtifactProtocolError {
    /// The artifact does not carry exactly the backend compilations in the cache subject.
    PayloadPortfolio {
        /// Number of payloads the cache subject names.
        expected: usize,
        /// Number of payload descriptors the artifact carries.
        actual: usize,
    },
    /// The artifact realizes its entries at a different number of delivery positions.
    DeliveryPositions {
        /// Number of delivery positions prepared.
        expected: usize,
        /// Number the artifact realizes its entries at.
        actual: usize,
    },
    /// Two executable entries name different payloads at one delivery position.
    DeliveryRealization {
        /// The delivery position the entries disagreed about.
        delivery: usize,
    },
    /// One position's payload is not the governed Metal native-image descriptor.
    PayloadDescriptor {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position's carried payload omits its compilation metadata.
    MissingPayloadMetadata {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position's carried payload omits its compiled object.
    MissingPayloadObject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// The carried metadata contradicts the compilation prepared for that position.
    ///
    /// This is the refusal a wrong-position payload arrives as: two artifact
    /// families share a compiler profile and differ in their AOT triple, so the
    /// object built for the other family disagrees on
    /// [`MetalPayloadFact::Target`](crate::MetalPayloadFact::Target).
    Correspondence {
        /// The delivery position that disagreed.
        delivery: usize,
        /// The exact compilation fact that disagreed.
        mismatch: MetalPayloadMismatch,
    },
    /// One position's payload names a different complete compilation subject.
    PayloadSubject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// The compile step produced a different number of objects than were prepared.
    CompiledPortfolio {
        /// Number of compilations prepared.
        expected: usize,
        /// Number the compile step produced.
        actual: usize,
    },
    /// The compiled artifact differs from the pending artifact program used in the key.
    ArtifactIdentity,
    /// One position carries object bytes other than the exact miss-produced object.
    PayloadObject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
}

impl fmt::Display for MetalArtifactProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadPortfolio { expected, actual } => write!(
                formatter,
                "Metal cache orchestration requires exactly {expected} payload(s), found {actual}",
            ),
            Self::DeliveryPositions { expected, actual } => write!(
                formatter,
                "{expected} Metal compilation(s) were prepared and the artifact realizes its \
                 entries at {actual} delivery position(s)",
            ),
            Self::DeliveryRealization { delivery } => write!(
                formatter,
                "two executable entries name different Metal payloads at delivery position \
                 {delivery}",
            ),
            Self::PayloadDescriptor { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} is not the governed Metal metallib \
                 native-image descriptor",
            ),
            Self::MissingPayloadMetadata { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} carries no compilation metadata",
            ),
            Self::MissingPayloadObject { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} carries no compiled object",
            ),
            Self::Correspondence { delivery, mismatch } => {
                write!(formatter, "at delivery position {delivery}: {mismatch}")
            }
            Self::PayloadSubject { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} names a different complete \
                 compilation subject",
            ),
            Self::CompiledPortfolio { expected, actual } => write!(
                formatter,
                "{expected} Metal compilation(s) were prepared and the compile step produced \
                 {actual}",
            ),
            Self::ArtifactIdentity => formatter.write_str(
                "compiled artifact identity differs from the pending artifact cache subject",
            ),
            Self::PayloadObject { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} carries a different compiled \
                 object",
            ),
        }
    }
}

impl Error for MetalArtifactProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Correspondence { mismatch, .. } => Some(mismatch),
            _ => None,
        }
    }
}

impl From<DeliveredPayloadProtocolError<MetalPayloadMismatch>> for MetalArtifactProtocolError {
    /// Renames the neutral protocol refusals into Metal's own vocabulary.
    ///
    /// Exhaustive by arm rather than by wildcard, so a refusal added to the
    /// neutral seam is a build error here instead of a Metal diagnostic that
    /// silently loses a case.
    fn from(error: DeliveredPayloadProtocolError<MetalPayloadMismatch>) -> Self {
        match error {
            DeliveredPayloadProtocolError::PayloadPortfolio { expected, actual } => {
                Self::PayloadPortfolio { expected, actual }
            }
            DeliveredPayloadProtocolError::DeliveryPositions { expected, actual } => {
                Self::DeliveryPositions { expected, actual }
            }
            DeliveredPayloadProtocolError::DeliveryRealization { delivery } => {
                Self::DeliveryRealization { delivery }
            }
            DeliveredPayloadProtocolError::PayloadDescriptor { delivery } => {
                Self::PayloadDescriptor { delivery }
            }
            DeliveredPayloadProtocolError::MissingPayloadMetadata { delivery } => {
                Self::MissingPayloadMetadata { delivery }
            }
            DeliveredPayloadProtocolError::Correspondence { delivery, cause } => {
                Self::Correspondence {
                    delivery,
                    mismatch: cause,
                }
            }
            DeliveredPayloadProtocolError::PayloadSubject { delivery } => {
                Self::PayloadSubject { delivery }
            }
            DeliveredPayloadProtocolError::MissingPayloadObject { delivery } => {
                Self::MissingPayloadObject { delivery }
            }
            DeliveredPayloadProtocolError::PayloadObject { delivery } => {
                Self::PayloadObject { delivery }
            }
            DeliveredPayloadProtocolError::CompiledPortfolio { expected, actual } => {
                Self::CompiledPortfolio { expected, actual }
            }
            DeliveredPayloadProtocolError::ArtifactIdentity => Self::ArtifactIdentity,
        }
    }
}

/// Why cache orchestration could not return an accepted Metal artifact.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalCacheError<E> {
    /// The complete cache subject could not be composed.
    Subject(SubjectRefusal),
    /// The prepared Metal compiler or linker failed on a cache miss.
    Compile(MetalAssemblyError),
    /// The caller could not assemble the compiled payload into its artifact.
    Assemble(E),
    /// The caller's verified artifact could not be encoded.
    Encode(ArtifactCodecFailure),
    /// The cache's governed artifact validator rejected the produced envelope.
    CacheArtifact(ArtifactCodecFailure),
    /// The pending, produced, or cached artifact contradicted the prepared operation.
    Protocol(MetalArtifactProtocolError),
}

impl<E: fmt::Display> fmt::Display for MetalCacheError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subject(error) => write!(formatter, "Metal cache subject was refused: {error}"),
            Self::Compile(error) => error.fmt(formatter),
            Self::Assemble(error) => write!(formatter, "Metal artifact assembly failed: {error}"),
            Self::Encode(error) => write!(formatter, "Metal artifact encoding failed: {error}"),
            Self::CacheArtifact(error) => {
                write!(
                    formatter,
                    "expansion cache refused the generated artifact: {error}"
                )
            }
            Self::Protocol(error) => error.fmt(formatter),
        }
    }
}

impl<E: Error + 'static> Error for MetalCacheError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Subject(error) => Some(error),
            Self::Compile(error) => Some(error),
            Self::Assemble(error) => Some(error),
            Self::Encode(error) | Self::CacheArtifact(error) => Some(error),
            Self::Protocol(error) => Some(error),
        }
    }
}

impl<E> From<DeliveredPayloadCacheError<MetalPayloadMismatch, MetalAssemblyError, E>>
    for MetalCacheError<E>
{
    fn from(
        error: DeliveredPayloadCacheError<MetalPayloadMismatch, MetalAssemblyError, E>,
    ) -> Self {
        match error {
            DeliveredPayloadCacheError::Subject(error) => Self::Subject(error),
            DeliveredPayloadCacheError::Compile(error) => Self::Compile(error),
            DeliveredPayloadCacheError::Assemble(error) => Self::Assemble(error),
            DeliveredPayloadCacheError::Encode(error) => Self::Encode(error),
            DeliveredPayloadCacheError::CacheArtifact(error) => Self::CacheArtifact(error),
            DeliveredPayloadCacheError::Protocol(error) => Self::Protocol(error.into()),
        }
    }
}

/// Resolves one plan's delivery-ordered Metal payload run through the expansion cache.
///
/// `pending` is the verified descriptor-only artifact whose canonical identity
/// is available before compilation. `prepared` is one prepared compilation per
/// delivery position, in the order the producer built its artifact families;
/// `assemble` runs only on a cache miss and must assemble the supplied compiled
/// payloads — in that same order — into the corresponding carried artifact. The
/// carried identity and payloads are checked before publication; every cache
/// result is checked again before this function returns.
///
/// This binds [`accept_or_publish_delivered_payload_artifact`] to Metal's two
/// statements and nothing else: each prepared payload already holds its governed
/// descriptor keys and its derived compilation digest, and this crate's own
/// `validate_metal_payload_metadata` is the correspondence closure, applied to
/// the compilation prepared for *that* position. The compilation facet the cache
/// subject names is the AOT driver's own prepared identity, which has no public
/// constructor, so it cannot be minted from invented toolchain facts.
///
/// # What a miss retains beside the entry
///
/// Each position's `metal` and `metallib` runs are retained under their own
/// labels — see `stage_retention` in this module — so a later hit can be asked what the
/// compiler said about the object it is serving. None of it reaches the payload
/// metadata, the payload digest, the composed subject, or the cache key: all of
/// those are derived before either tool runs, from the prepared compilation and
/// the pending artifact. A build whose compiler warns therefore resolves to the
/// same entry as one whose compiler is silent.
///
/// # Errors
///
/// Returns a typed subject, compilation, assembly, codec, or protocol failure.
/// A protocol failure is hard: it is never translated into a cache miss or an
/// automatic rebuild.
pub fn accept_or_publish_delivered_metal_artifact<E>(
    cache: &ExpansionCache,
    pending: &VerifiedArtifactProgram,
    prepared: Vec<PreparedMetalPayload<'_>>,
    assemble: impl FnOnce(Vec<CompiledMetalPayload>) -> Result<VerifiedArtifactProgram, E>,
) -> Result<AcceptedArtifact, MetalCacheError<E>> {
    // Owned before the tokens are consumed: the compile closure consumes each
    // token, and the same declarations are compared again after resolution.
    let backends: Vec<_> = prepared
        .iter()
        .map(|payload| payload.backend().clone())
        .collect();
    let representations: Vec<_> = prepared
        .iter()
        .map(|payload| payload.representation().clone())
        .collect();
    let digests: Vec<_> = prepared
        .iter()
        .map(|payload| payload.digest().clone())
        .collect();
    let expected_metadata: Vec<_> = prepared
        .iter()
        .map(|payload| payload.metadata().clone())
        .collect();
    let compilations: Vec<Vec<u8>> = prepared
        .iter()
        .map(|payload| payload.compilation_identity_bytes().to_vec())
        .collect();
    let declared: Vec<DeclaredPayload<'_>> = (0..prepared.len())
        .map(|delivery| DeclaredPayload {
            backend: &backends[delivery],
            representation: &representations[delivery],
            payload_schema: PAYLOAD_SCHEMA,
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            digest: &digests[delivery],
            compilation: &compilations[delivery],
        })
        .collect();
    let tokens: Vec<_> = prepared
        .into_iter()
        .map(PreparedMetalPayload::into_parts)
        .collect();

    accept_or_publish_delivered_payload_artifact(
        cache,
        pending,
        &declared,
        |delivery, actual| validate_metal_payload_metadata(&expected_metadata[delivery], actual),
        || {
            // Each compilation's object and its stage output are separated here:
            // the object is what the artifact carries and every check below
            // compares, while the output is retained beside the published entry
            // and reaches no identity at all.
            let mut contents = Vec::with_capacity(tokens.len());
            let mut outputs = Vec::with_capacity(tokens.len());
            for (prepared, metadata, _digest) in tokens {
                let (content, stage_outputs) =
                    CompiledMetalPayload::compile_prepared_parts(prepared, metadata)?;
                contents.push(content);
                outputs.push(stage_outputs);
            }
            Ok(CompiledPayloads {
                contents,
                retained: stage_retention(&outputs),
            })
        },
        |contents| {
            assemble(
                contents
                    .into_iter()
                    .map(CompiledMetalPayload::from_content)
                    .collect(),
            )
        },
    )
    .map_err(MetalCacheError::from)
}

/// Names one delivery position's stage run.
///
/// The backend key, the delivery position, and the stage's own tool name, in
/// that order: the run a reader wants is "what did `metallib` say about the
/// object my build target loads", and every part of that question is in the
/// label. The position is included because one entry covers the whole selection
/// — several artifact families are several compilations under one key — so a
/// label naming only the stage would be two runs fighting over one name, which
/// [`DebugRetention::retaining_with_stated_total`] refuses rather than silently
/// merges.
fn stage_label(delivery: usize, stage: CompileStage) -> String {
    format!("{BACKEND}.{delivery}.{}", stage.tool())
}

/// States what every stage of every position in this selection wrote.
///
/// **Always stated, never discovered.** This backend retains its stage output on
/// every publication rather than consulting an environment variable or a build
/// profile, which is the ADR 0089 root policy the retention module restates: the
/// decision lives with the caller that has one, and this caller's decision is
/// that a Metal compilation's own words belong beside the entry it produced. The
/// cost is bounded by the retention's own limits and is two empty runs for a
/// quiet compilation.
///
/// **A silent stage is retained as an empty run.** Both stages ran, so both are
/// named; dropping the quiet one would leave a reader unable to tell a compiler
/// that warned about nothing from an entry published before any of this existed,
/// which is the state [`DebugRetention::is_empty`] already answers.
///
/// **The text is host-specific, which is the second reason it is not identity.**
/// A `metal` diagnostic names the file it diagnosed, and the driver compiles from
/// a per-process scratch directory, so two hosts compiling byte-identical source
/// under one toolchain retain different bytes for one warning. They resolve to
/// one entry regardless, because the key is a function of the composed subject
/// alone; an implementation that folded this text into a subject would have given
/// them two.
///
/// **The stage's own total is stated, not re-derived.**
/// `tiler_metal_aot::diagnostic::ToolOutput` and
/// `tiler_cache::expansion::MAX_RETAINED_RUN_BYTES` bound one run identically, at
/// 16 KiB, so a stage that wrote megabytes arrives here already truncated and
/// exactly at the bound. `ToolOutput::total_bytes` is what the tool actually
/// wrote, and passing it through
/// [`DebugRetention::retaining_with_stated_total`] is what lets a later hit tell
/// that bounded prefix from a whole diagnostic. Pre-truncating below the cache's
/// bound, or editing the tool's bytes to describe themselves, were the two
/// producer-side alternatives; both give up something byte-preserving capture
/// exists to keep.
///
/// # A refusal is not a build failure
///
/// A retention that cannot be stated — a selection wide enough to pass the
/// run-count limit is the reachable case — leaves the compilation entirely
/// correct, so it is recorded as one run saying so rather than returned as an
/// error. Failing a successful compilation over a diagnostic would make a warning
/// a compilation input in the only way that actually matters.
fn stage_retention(outputs: &[StageOutputs]) -> DebugRetention {
    let mut retention = DebugRetention::none();
    for (delivery, stage_outputs) in outputs.iter().enumerate() {
        for stage in CompileStage::ALL {
            let output = stage_outputs.stage(stage);
            match retention.retaining_with_stated_total(
                &stage_label(delivery, stage),
                output.as_bytes(),
                output.total_bytes(),
            ) {
                Ok(extended) => retention = extended,
                // All or nothing: a partial run set reads as a selection with
                // fewer positions than it had, and a reader cannot tell which.
                Err(refusal) => return elided_retention(&refusal),
            }
        }
    }
    retention
}

/// Retains one run stating why no stage output is here.
///
/// A positive statement rather than an absent section, because absence already
/// means "published by a build that retained nothing" and a reader that could
/// not tell the two apart would go looking for a compiler that never spoke.
fn elided_retention(refusal: &RetentionRefusal) -> DebugRetention {
    DebugRetention::none()
        .retaining(
            &format!("{BACKEND}.retention-elided"),
            format!("no Metal stage output was retained: {refusal}").as_bytes(),
        )
        // One governed label and one run cannot exceed a bound, so this is the
        // unreachable arm of a total function rather than a case to handle: it
        // resolves to the same "nothing to show" a non-retaining build leaves.
        .unwrap_or_else(|_| DebugRetention::none())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tiler_cache::expansion::MAX_RETAINED_RUNS;
    use tiler_metal_aot::diagnostic::CompileStage;
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::{
        ApplePlatform, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion,
        NumericalRealization, OptimizationLevel,
    };
    use tiler_metal_aot::record::StageOutputs;

    use super::{stage_label, stage_retention};

    const METAL_TEXT: &str = "warning: the front end said this";
    const METALLIB_TEXT: &str = "warning: the linker said this";

    fn scratch(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tiler-build-metal-cache-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        std::fs::create_dir_all(&path).expect("the scratch directory is creatable");
        path
    }

    fn write_executable(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::write(path, body).expect("the fake tool is writable");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("the fake tool is executable");
    }

    /// A fake toolchain whose two stages write distinguishable text and succeed.
    ///
    /// Fake rather than real for the reason `metal_plan`'s own fixture is: the
    /// installed `metal` warns about whatever it chooses to, so a case built on
    /// real warning text would assert on this host's compiler release rather than
    /// on the retention. What is under test here is the labelling, not the
    /// compiler.
    fn talking_toolchain(directory: &Path) -> Toolchain {
        let metal = directory.join("metal");
        let metallib = directory.join("metallib");
        let launcher = directory.join("xcrun");
        write_executable(
            &metal,
            &format!(
                "#!/bin/sh\n\
                 if [ \"$1\" = \"--version\" ]; then echo 'Metal retention-v1'; exit 0; fi\n\
                 printf '%s' '{METAL_TEXT}' >&2\n\
                 while [ \"$#\" -gt 0 ]; do\n\
                   if [ \"$1\" = \"-o\" ]; then shift; printf AIR > \"$1\"; exit 0; fi\n\
                   shift\n\
                 done\n\
                 exit 1\n",
            ),
        );
        write_executable(
            &metallib,
            &format!(
                "#!/bin/sh\n\
                 if [ \"$1\" = \"--version\" ]; then echo 'metallib retention-v1'; exit 0; fi\n\
                 printf '%s' '{METALLIB_TEXT}' >&2\n\
                 while [ \"$#\" -gt 0 ]; do\n\
                   if [ \"$1\" = \"-o\" ]; then shift; printf MTLBcache > \"$1\"; exit 0; fi\n\
                   shift\n\
                 done\n\
                 exit 1\n",
            ),
        );
        write_executable(
            &launcher,
            &format!(
                "#!/bin/sh\n\
                 shift 2\n\
                 case \"$1\" in\n\
                   --find) if [ \"$2\" = \"metal\" ]; then echo '{}'; else echo '{}'; fi ;;\n\
                   --show-sdk-version) echo 26.5 ;;\n\
                   --show-sdk-build-version) echo 25F70 ;;\n\
                   *) exit 1 ;;\n\
                 esac\n",
                metal.display(),
                metallib.display(),
            ),
        );
        Toolchain::with_launcher(launcher)
    }

    /// Runs one fake compilation and returns exactly what its stages wrote.
    ///
    /// A real driver run rather than a hand-built value, because
    /// [`StageOutputs`] is `#[non_exhaustive]` and has no out-of-crate literal:
    /// the only honest way to hold one here is to have produced it. That is also
    /// the stronger fixture — the retained bytes are what the capture path
    /// actually carried, not what a test assumed it would.
    fn compiled_stage_outputs(directory: &Path) -> StageOutputs {
        let toolchain = talking_toolchain(directory);
        let target = MetalTarget::new(
            ApplePlatform::MacOs,
            DeploymentMinimum::new(26, 0),
            MslVersion::Metal4_0,
        )
        .expect("macOS at its MSL 4.0 floor is a governed compilation target");
        let request = CompileRequest::new(
            "kernel void nothing() {}",
            target,
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        );
        toolchain
            .prepare(&request)
            .expect("the fake toolchain resolves")
            .compile()
            .expect("the fake toolchain compiles")
            .stage_outputs
    }

    /// Each delivery position's stage runs are retained under their own label.
    ///
    /// **The two positions carry byte-identical output on purpose.** That is the
    /// case a label naming only the stage cannot survive: two runs would fight
    /// over `tiler.metal.metal`, `retaining_with_stated_total` refuses a
    /// duplicate label rather than merging it, and `stage_retention`'s
    /// all-or-nothing arm would replace the whole set with the elision run. So a
    /// position dropped from [`stage_label`] is not a subtle mislabelling here —
    /// it removes every stage run in the selection.
    ///
    /// This is the surviving half of the multi-position retention evidence the
    /// required compilation-selection provenance took with the test-only second
    /// Metal declaration: an end-to-end two-family Metal publication now needs
    /// two *measured* declarations, which is
    /// `first-authoritative-ios-metal-compile-declaration`, while the labelling
    /// this function owns is reachable from one compilation's output.
    #[test]
    fn every_delivery_positions_stage_is_retained_under_its_own_governed_label() {
        let directory = scratch("per-position-labels");
        let outputs = compiled_stage_outputs(&directory);
        assert_eq!(outputs.metal.as_bytes(), METAL_TEXT.as_bytes());
        assert_eq!(outputs.metallib.as_bytes(), METALLIB_TEXT.as_bytes());

        let retention = stage_retention(&[outputs.clone(), outputs]);
        let actual: Vec<(&str, &[u8], u64)> = retention
            .runs()
            .iter()
            .map(|run| (run.label(), run.as_bytes(), run.total_bytes()))
            .collect();
        let expected: Vec<(&str, &[u8], u64)> = vec![
            (
                "tiler.metal.0.metal",
                METAL_TEXT.as_bytes(),
                METAL_TEXT.len() as u64,
            ),
            (
                "tiler.metal.0.metallib",
                METALLIB_TEXT.as_bytes(),
                METALLIB_TEXT.len() as u64,
            ),
            (
                "tiler.metal.1.metal",
                METAL_TEXT.as_bytes(),
                METAL_TEXT.len() as u64,
            ),
            (
                "tiler.metal.1.metallib",
                METALLIB_TEXT.as_bytes(),
                METALLIB_TEXT.len() as u64,
            ),
        ];
        assert_eq!(
            actual, expected,
            "each stage of each delivery position keeps its own label, bytes, and stated total",
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    /// Every stage of every position is named, in position-then-stage order.
    ///
    /// The label derivation on its own, so a change to its shape fails at the
    /// derivation rather than only inside the census above. The stage list is
    /// [`CompileStage::ALL`], which is sized by `variant_count`, so a third
    /// offline stage is an array-length error there rather than a run this test
    /// silently stops naming.
    #[test]
    fn a_stage_label_names_its_position_and_its_tool() {
        let labels: Vec<String> = (0..2)
            .flat_map(|delivery| CompileStage::ALL.map(|stage| stage_label(delivery, stage)))
            .collect();
        assert_eq!(
            labels,
            [
                "tiler.metal.0.metal",
                "tiler.metal.0.metallib",
                "tiler.metal.1.metal",
                "tiler.metal.1.metallib",
            ],
        );
    }

    /// A selection too wide to retain states that, rather than retaining part of it.
    ///
    /// The all-or-nothing arm, which no one-position selection can reach:
    /// [`MAX_RETAINED_RUNS`] is 16 and each position contributes one run per
    /// stage, so the first refusable width is nine positions. A partial run set
    /// would read as a selection with fewer positions than it had, and a reader
    /// could not tell which — so the whole set is replaced by one run that says
    /// why.
    #[test]
    fn a_selection_wider_than_the_run_limit_states_the_elision() {
        let directory = scratch("elided-retention");
        let outputs = compiled_stage_outputs(&directory);
        let positions = MAX_RETAINED_RUNS / CompileStage::ALL.len() + 1;
        let wide = vec![outputs; positions];

        let retention = stage_retention(&wide);
        let [elided] = retention.runs() else {
            panic!(
                "an elided retention is exactly one run: {:?}",
                retention.runs()
            );
        };
        assert_eq!(elided.label(), "tiler.metal.retention-elided");
        assert_eq!(
            String::from_utf8_lossy(elided.as_bytes()),
            "no Metal stage output was retained: a retention carries at most 16 runs",
        );
        let _ = std::fs::remove_dir_all(directory);
    }
}
