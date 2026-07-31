//! The non-published offline producer for the serial-Sum vertical slice.
//!
//! It drives the already-implemented component capabilities — semantic
//! construction, compilation, MSL emission, and offline Metal compilation —
//! through one path, and implements no component capability of its own.
//! `tiler-build` owns checked-plan consumption, Metal emission and AOT
//! preparation, artifact assembly, and cache acceptance; this prototype
//! supplies proof policy, target facts, output names, and sidecars.
//!
//! # What this proves and what it does not
//!
//! Running it proves the offline path composes end to end: a semantic program
//! reaches a verified kernel through the public compiler boundary, `tiler-build`
//! carries that checked plan through deterministic MSL emission and `xcrun`,
//! and the accepted neutral artifact survives an encode, a decode, and a
//! byte-identical re-encode.
//!
//! It proves nothing about execution — no device is created, no kernel is
//! dispatched, and no output bits are compared. An artifact that assembles,
//! encodes, and re-validates from its own bytes is not an artifact that has run.
//!
//! # Usage, and why `--out` is required
//!
//! ```text
//! cargo run -p tiler-prototype-compile -- --out <path>
//! ```
//!
//! It writes twelve files: one envelope and one `.proof` sidecar for each of
//! six members, named `<path>.<class>.<role>`. The six are three reduction
//! classes — an empty domain, a singleton, and a nontrivial reduction — times
//! two plan roles, the portfolio's selected (fused) plan and the retained
//! materialized alternative that dispatches two stages through one
//! intermediate. Each sidecar carries the artifact identity, the operands, and
//! the expected outputs the governed reference produced for them.
//!
//! `prototypes/serial-sum-run` consumes the set, and those files are the whole
//! interface between the two halves of this vertical slice — no module, type,
//! or Cargo edge crosses it. The proof is that the fused and materialized
//! members of a class, which are genuinely different programs on the device,
//! return the same bits as the reference for every operand class.
//!
//! The path is required rather than defaulted because a producer run that emits
//! nothing produces nothing. It used to take no arguments and print a summary,
//! which was honest while the envelope had no consumer and would now be a run
//! that looks successful and leaves the runner with no artifact.
//!
//! # Why the identity travels beside the envelope rather than inside it
//!
//! `DecodedProgram::preflight` binds a consumer's *expected* identity against the
//! one re-derived from the bytes. An identity read out of those same bytes is a
//! tautology, so the expected one has to come from whatever named the artifact.
//! Here that is this producer, which derives it from the `VerifiedArtifactProgram`
//! it assembled rather than from the encoding — so the sidecar catches a stale
//! envelope, a mixed-up path, or a producer run that did not complete. It does
//! not resist an adversary who rewrites both files, and nothing unsigned could.
//! A torn write between the two is caught the same way: as a mismatch, loudly.
//!
//! # The filename is the interface, and both halves pin it
//!
//! Because no code crosses between the two prototypes, the suffix below is a
//! fact each derives on its own, and nothing mechanical compares them: this
//! producer wrote one name and the runner opened another for a whole commit,
//! while the complete gate stayed green over a slice that could not run. Both
//! crates now pin [`SIDECAR_SUFFIX`] in a test that names the other side, so a
//! rename fails on the half that was not updated.

mod sidecar;

use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

use tiler_artifact::program::ArtifactCodecFailure;
#[cfg(test)]
use tiler_artifact::program::decode_artifact;
use tiler_build::{MetalPlanBuildError, MetalPlanBuildPolicy, accept_or_publish_metal_plan};
#[cfg(test)]
use tiler_build::{metal_compile_request, prepare_metal_payload};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{CompileFailure, NumericalContract, compile_governed};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
#[cfg(test)]
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
    MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
    MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{NumericalRealization, OptimizationLevel};

/// The buffer argument-table capacity Apple's feature tables state per compute
/// function for every current family.
///
/// Stated by the caller rather than derived, so a signature needing more
/// bindings is rejected instead of emitted with an unaddressable attribute.
const BUFFER_BINDING_LIMIT: u32 = 31;

/// Rows of the packaged program's input; each row reduces to one output element.
///
/// One is deliberate. The governed target profile's only portable compile-time
/// grid-axis guarantee is four threads, and the materialized nontrivial plan's
/// pointwise stage launches `rows * columns` threads. With the three
/// contributors required below, one row keeps both the fused and materialized
/// programs feasible without inventing a larger target capability. The
/// sidecar still supplies five independent operand classes for that row,
/// including exceptional and contraction-sensitive values; row count is not
/// being used as a proxy for numerical coverage.
const ROWS: u64 = 1;
/// Columns of the packaged program's input; the reduced axis.
///
/// Three, matching `prototypes/serial-sum-run`, because three contributors per
/// row is what makes a serial reduction's ordering observable — one contributor
/// reduces in every order.
///
/// It was one until `bound-the-backend-entry-key-by-the-identity-it-carries`,
/// because the artifact layer bounded a `BackendEntryKey` at 1,024 bytes while
/// the canonical kernel identity this producer hands it measures 1,121 bytes for
/// any reduction with two or more contributors. The bound is now `tiler-ir`'s
/// own for that value, so the runner's two paths carry the same program.
const COLUMNS: u64 = 3;

/// The reduction classes the proof covers, as `(name, reduced extent)`.
///
/// Three programs, not three operand sets. The reduced extent lives in the
/// input shape, so it changes the semantic graph, the verified kernels, and the
/// artifact identity; an empty domain and a singleton cannot be reached by
/// choosing different numbers for a fixed shape.
///
/// The boundaries are what make the nontrivial case mean anything. A serial
/// reduction over one contributor reduces in every order, so it cannot observe
/// an ordering defect, and an empty domain is where a reduction's identity
/// element is either right or silently invented.
/// The empty domain leads, because it is the boundary the other two cannot
/// speak for: it is where a reduction's identity element is either right or
/// silently invented, and its kernel reads its input buffer never.
const REDUCTION_CLASSES: [(&str, u64); 3] = [
    ("empty-domain", 0),
    ("singleton", 1),
    ("nontrivial", COLUMNS),
];

/// The plan roles the proof publishes for each reduction class.
///
/// `selected` is whatever the portfolio ranks first, which is the fused plan on
/// this profile; `materialized` is the retained alternative that dispatches two
/// stages through one intermediate. Publishing both is the point of the proof:
/// the two are different programs on the device and must agree bit for bit.
const PLAN_ROLES: [&str; 2] = ["selected", "materialized"];

/// Suffix appended to the envelope path to name the proof-case sidecar.
///
/// `prototypes/serial-sum-run` derives the same name from the path it is given.
/// Nothing links the two crates, so each pins this in a test rather than
/// sharing a constant neither may import.
const SIDECAR_SUFFIX: &str = ".proof";

/// The target facts this producer emits for.
///
/// macOS 14.0 under MSL 3.1, declaring the measured Apple subnormal behaviour
/// once per floating-point arithmetic type: `f32` arithmetic flushes subnormal
/// operands and results to zero on every governed family, and `f16` arithmetic
/// preserves them on the same hardware in the same math modes. Declaring them is
/// what lets emission reject a kernel whose numerical contract the target cannot
/// honour, rather than emitting one that silently computes something else — and
/// declaring them separately is what stops the `f32` fact answering for a width
/// it was not measured at.
fn target_facts() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(14, 0),
        MetalSubnormalArithmeticFacts::unmeasured()
            .stating(
                MetalFloatArithmeticType::F32,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            )
            .stating(
                MetalFloatArithmeticType::F16,
                MetalSubnormalArithmetic::PreservesSubnormals,
            ),
        BUFFER_BINDING_LIMIT,
    )
}

/// The source-level choices this producer selects independently of target facts.
fn emission_realization() -> MetalEmissionRealization {
    MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt)
}

/// The complete payload-production policy this producer selects.
fn build_policy() -> MetalPlanBuildPolicy {
    MetalPlanBuildPolicy::new(
        emission_realization(),
        OptimizationLevel::Default,
        NumericalRealization::strict_baseline(),
    )
}

/// Builds the bounded profile's scale-then-reduce program over one shape.
///
/// Parameterized on the reduced extent because the proof must cover an empty
/// domain, a singleton, and a nontrivial reduction, and those are three
/// *programs* rather than three operand sets: the extent is in the shape, so it
/// changes the semantic graph, the kernels, and the artifact identity.
fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Emits and offline-compiles one plan's kernels, for the cases below.
///
/// Shared at crate scope rather than duplicated per test module: `bundle`'s
/// cases must package the payload built from the *same* kernels the plan they
/// assemble dispatches, and a second fixture that recompiled independently could
/// drift from this one without failing.
///
/// It reaches `xcrun`, so it is a host-dependent fixture rather than a pure one.
#[cfg(test)]
fn emit_and_compile(
    kernels: &[&tiler_ir::kernel::VerifiedKernel],
) -> (
    tiler_metal::record::MetalTranslationUnit,
    tiler_build::CompiledMetalPayload,
) {
    let unit = emit_translation_unit(kernels, &target_facts(), emission_realization())
        .expect("the kernels emit");
    let request = metal_compile_request(
        &unit,
        OptimizationLevel::Default,
        NumericalRealization::strict_baseline(),
    )
    .expect("the emitted target and numerical realization are compilable");
    let prepared = Toolchain::system()
        .prepare(&request)
        .expect("the offline toolchain prepares");
    let payload = prepare_metal_payload(&unit, prepared)
        .expect("the emission and prepared compilation agree")
        .compile()
        .expect("the offline toolchain compiles");
    (unit, payload)
}

/// Returns the envelope path the invocation names.
///
/// Hand-parsed rather than reached for a dependency: one required flag does not
/// justify an argument crate in a `publish = false` prototype, and an unknown
/// argument is refused instead of ignored so a typo cannot look like a run that
/// simply wrote nowhere.
fn output_path() -> Result<PathBuf, ProducerError> {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(flag), Some(path), None) = (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err(ProducerError::Usage);
    };
    if flag != "--out" {
        return Err(ProducerError::Usage);
    }
    Ok(PathBuf::from(path))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("serial-sum offline producer failed: {failure}");
            ExitCode::FAILURE
        }
    }
}

struct Publication<'a> {
    base: &'a std::path::Path,
    cache: &'a ExpansionCache,
    toolchain: &'a Toolchain,
}

/// One offline pass: publish every member of the proof matrix.
///
/// Six members — three reduction classes times two plan roles — because the
/// proof compares a fused single-dispatch program against the materialized
/// two-dispatch program that computes the same function, across the reduction
/// boundaries where they could differ. Each member is a complete, independently
/// decodable envelope with its own sidecar; nothing is shared between them at
/// run time, which is what lets the runner treat each as a separate proof run.
fn run() -> Result<(), ProducerError> {
    let base = output_path()?;
    let cache_root = std::env::temp_dir().join(format!(
        "tiler-prototype-compile-cache-{}",
        std::process::id(),
    ));
    let cache = ExpansionCache::open(&cache_root);
    let toolchain = Toolchain::system();
    let publication = Publication {
        base: &base,
        cache: &cache,
        toolchain: &toolchain,
    };
    let mut published = 0_usize;
    for (class, columns) in REDUCTION_CLASSES {
        let program = serial_sum_program(ROWS, columns);
        // Stated, not defaulted. The strict contract is unhonourable on every
        // governed Apple family, so this producer says which contract its
        // program means rather than discovering that by reading a rejection.
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .map_err(ProducerError::Compile)?;
        println!(
            "{class} (reduced extent {columns}): target profile {}",
            compilation.target_profile_key(),
        );

        for role in PLAN_ROLES {
            let plan = match role {
                "selected" => compilation.selected().ok_or(ProducerError::NoSelection)?,
                // The retained alternative the portfolio did not rank first.
                // Asked for by shape rather than by position: "not fused" is
                // the property the proof needs, and an index would silently
                // follow a reordering of the portfolio.
                _ => compilation
                    .alternatives()
                    .find(|alternative| !alternative.is_fused())
                    .ok_or(ProducerError::NoMaterializedAlternative)?,
            };
            publish_member(&publication, class, role, columns, &program, plan)?;
            published += 1;
        }
    }
    println!(
        "published {published} proof member(s) under {}",
        base.display()
    );
    let _ = std::fs::remove_dir_all(cache_root);
    Ok(())
}

/// Emits, compiles, bundles, validates, and writes one member of the matrix.
///
/// The order is deliberate and is the same one a single-member producer used:
/// everything is validated before either file is written, so a member the
/// artifact layer refuses stops the publication instead of leaving an envelope
/// on disk that no consumer would accept.
fn publish_member(
    publication: &Publication<'_>,
    class: &str,
    role: &str,
    columns: u64,
    program: &SemanticProgram,
    plan: tiler_compiler::session::PlanAlternative<'_>,
) -> Result<(), ProducerError> {
    let facts = target_facts();
    let entry_count = plan.kernels().len();
    let accepted = accept_or_publish_metal_plan(
        publication.cache,
        publication.toolchain,
        program,
        plan,
        &facts,
        build_policy(),
    )
    .map_err(ProducerError::Plan)?;
    let artifact = accepted.artifact();
    let (bytes, decoded) = match accepted.resolution() {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
            (entry.envelope_bytes().to_vec(), entry.artifact())
        }
        Resolution::Uncached {
            envelope, artifact, ..
        } => (envelope.clone(), artifact),
    };
    let metallib_len = decoded
        .payload_object(0)
        .expect("the accepted singular Metal artifact carries its checked object")
        .len();

    // The cache route has already decoded and validated these bytes. Re-encoding
    // the producer-side verified view proves that no field was lost while the
    // hit/publication path crossed the decoded representation.
    if artifact.encode().map_err(ProducerError::Encode)? != bytes {
        return Err(ProducerError::UnstableEncoding);
    }

    // Built before either file is written, so a sidecar the artifact layer
    // refuses stops the publication instead of leaving an envelope on disk with
    // no record describing it.
    let sidecar_bytes =
        sidecar::encoded(artifact, program, ROWS, columns).map_err(ProducerError::Sidecar)?;

    let envelope_path = proof_member(publication.base, class, role);
    let sidecar_path = proof_sidecar(&envelope_path);
    std::fs::write(&envelope_path, &bytes)
        .map_err(|cause| ProducerError::Write(envelope_path.display().to_string(), cause))?;
    std::fs::write(&sidecar_path, &sidecar_bytes)
        .map_err(|cause| ProducerError::Write(sidecar_path.display().to_string(), cause))?;

    println!(
        "  {role}: {} entr(y/ies), {} bytes of metallib, {} envelope byte(s), {} sidecar byte(s) \
         -> {}",
        entry_count,
        metallib_len,
        bytes.len(),
        sidecar_bytes.len(),
        envelope_path.display(),
    );
    Ok(())
}

/// Returns the envelope path for one published member of the proof matrix.
///
/// Derived by appending to the base path rather than by rewriting an extension,
/// for the same reason [`proof_sidecar`] appends: the whole set stays obviously
/// one unit on disk and no two members can collide. `prototypes/serial-sum-run`
/// derives the identical names from the base path it is given, and both crates
/// pin the derivation in a test naming the other side, because no code crosses
/// between them.
fn proof_member(base: &std::path::Path, class: &str, role: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{class}.{role}"));
    PathBuf::from(name)
}

/// Returns the identity sidecar path for one envelope path.
///
/// Derived by appending rather than by replacing an extension, so the two names
/// cannot collide with each other and the pair stays obviously one unit on disk.
/// Shared with `prototypes/serial-sum-run`, which derives the same name from the
/// path it is given.
fn proof_sidecar(envelope: &std::path::Path) -> PathBuf {
    let mut name = envelope.as_os_str().to_owned();
    name.push(SIDECAR_SUFFIX);
    PathBuf::from(name)
}

/// Why one offline pass did not produce a `metallib`.
///
/// The stages are kept apart rather than collapsed into one message: a program
/// this build does not compile, a target that cannot honour the declared
/// numerics, and a host without a usable toolchain are three different things
/// to do next.
#[derive(Debug)]
enum ProducerError {
    Usage,
    Write(String, std::io::Error),
    Compile(CompileFailure),
    NoSelection,
    NoMaterializedAlternative,
    Plan(MetalPlanBuildError),
    Encode(ArtifactCodecFailure),
    UnstableEncoding,
    Sidecar(sidecar::SidecarError),
}

impl fmt::Display for ProducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(
                "usage: tiler-prototype-compile --out <path>; it writes the envelope to <path> \
                 and its artifact identity to <path>.identity",
            ),
            Self::Write(path, cause) => write!(formatter, "{path} could not be written: {cause}"),
            Self::Compile(failure) => write!(formatter, "the program did not compile: {failure:?}"),
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::NoMaterializedAlternative => formatter.write_str(
                "the portfolio retained no materialized alternative, so the proof cannot compare \
                 a fused program against the multi-dispatch program computing the same function",
            ),
            Self::Plan(cause) => write!(formatter, "the checked Metal plan failed: {cause}"),
            Self::Encode(cause) => write!(formatter, "the envelope did not encode: {cause}"),
            Self::UnstableEncoding => {
                formatter.write_str("re-encoding the decoded envelope did not reproduce its bytes")
            }
            Self::Sidecar(cause) => write!(formatter, "{cause}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        COLUMNS, PLAN_ROLES, REDUCTION_CLASSES, ROWS, build_policy, emission_realization,
        serial_sum_program, sidecar, target_facts,
    };
    use crate::{ArtifactCodecFailure, decode_artifact};
    use tiler_artifact::proof::decode_proof_sidecar;
    use tiler_cache::expansion::{ExpansionCache, Resolution};
    use tiler_compiler::capability::{
        LoweringCapabilityRegistryBuilder, install_governed_index_access,
    };
    use tiler_compiler::session::{
        CompileFailureClass, CompileRequest, InstalledCapabilities, compile,
    };
    use tiler_compiler::session::{NumericalContract, compile_governed};
    use tiler_compiler::target::{TargetProfile, TargetRequest};
    use tiler_ir::index::FrozenScalarRegistry;
    use tiler_ir::semantic::multiply_f32_op;
    use tiler_metal::emit::emit_translation_unit;
    use tiler_metal_aot::driver::Toolchain;

    fn governed_targets() -> TargetRequest {
        TargetRequest::new([TargetProfile::governed()])
            .expect("the governed singleton target request is valid")
    }

    /// The offline path composes as far as deterministic MSL, without a
    /// toolchain.
    ///
    /// This is the part of `run` that has no host dependency, so it is the part
    /// the gate can assert. Compiling the emitted source needs `xcrun` and is
    /// exercised by running the producer, not by this test.
    #[test]
    fn the_selected_program_emits_deterministic_metal_source() {
        let program = serial_sum_program(ROWS, COLUMNS);
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let selected = compilation.selected().expect("a selected alternative");
        let kernels: Vec<_> = selected.kernels().iter().collect();

        let first = emit_translation_unit(&kernels, &target_facts(), emission_realization())
            .expect("the kernels emit");
        let second = emit_translation_unit(&kernels, &target_facts(), emission_realization())
            .expect("the kernels emit");
        assert_eq!(
            first.source(),
            second.source(),
            "emission is a pure function of the kernels, target facts, and selected realization",
        );
        assert!(!first.entry_points().is_empty());
        assert!(
            first.source().contains("kernel"),
            "the emitted unit declares a Metal kernel",
        );
    }

    /// A carried payload is content-addressed over its compilation *inputs*.
    ///
    /// This is the identity decision `prototype-metal-bundle-assembly` made,
    /// asserted against a real emission rather than a fixture: relinking the
    /// same source must not change what artifact this is, and changing a
    /// compilation input must. Without the first property, artifact identity
    /// would depend on linker reproducibility, which `docs/artifact-abi.md`
    /// explicitly refuses to promise.
    #[test]
    fn the_payload_identity_follows_its_compilation_subject() {
        let (_unit, compiled) = emit_and_compile();
        let payload = compiled.content().clone();
        let identity = payload.identity().expect("the subject has an identity");

        // Same subject, different emitted object: the artifact is unchanged.
        let mut relinked = payload.clone();
        relinked.code.push(0xff);
        assert_eq!(
            relinked.identity().expect("the subject has an identity"),
            identity,
            "the object is opaque; a different link is the same artifact",
        );

        // A changed compilation input: a different artifact.
        let mut recompiled = payload.clone();
        recompiled.metadata.source.push(b' ');
        assert_ne!(
            recompiled.identity().expect("the subject has an identity"),
            identity,
            "different source is a different compilation subject",
        );

        // Flag order is meaning, not presentation.
        let mut reordered = payload;
        reordered.metadata.provenance.compile_flags.reverse();
        assert_ne!(
            reordered.identity().expect("the subject has an identity"),
            identity,
            "a compiler resolves conflicting flags positionally, so order is identity",
        );
    }

    /// No absolute path reaches the payload's portable subject.
    ///
    /// `ResolvedTool::path` and `SdkIdentity::path` are local provenance by
    /// their own documentation. A subject that folded one would give two hosts
    /// running the same toolchain two different artifact identities.
    #[test]
    fn the_payload_subject_carries_no_local_path() {
        let (_unit, compiled) = emit_and_compile();
        let payload = compiled.content();
        let provenance = &payload.metadata.provenance;
        let mut text = vec![
            provenance.toolchain.clone(),
            provenance.target.clone(),
            provenance.family.clone(),
            provenance.language.clone(),
            provenance.sdk.name.clone(),
            provenance.sdk.version.clone(),
            provenance.sdk.build.clone(),
        ];
        text.extend(
            provenance
                .components
                .iter()
                .map(|part| part.version.clone()),
        );
        text.extend(provenance.compile_flags.iter().cloned());
        text.extend(provenance.link_flags.iter().cloned());
        for value in text {
            // Any absolute path anywhere in the value, not merely one the value
            // starts with or one under `/Applications`. The narrower predicate
            // passed on a host whose toolchain reported its `InstalledDir`
            // under `/private/var/...` while failing on one that reported
            // `/Applications/Xcode_16.4.app/...`, so it was deciding by which
            // host ran it rather than by whether identity stayed portable.
            let local = value
                .split_whitespace()
                .find(|token| token.starts_with('/') && token.len() > 1);
            assert!(
                local.is_none(),
                "{value:?} carries the local path {:?} and must not be portable identity",
                local.unwrap_or_default(),
            );
        }
    }

    /// The entry mapping names the kernel identity, not the emitted symbol.
    #[test]
    fn the_entry_mapping_keys_on_the_kernel_identity() {
        let (unit, compiled) = emit_and_compile();
        let payload = compiled.content();
        let entry = &payload.metadata.entries[0];
        let emitted = &unit.entry_points()[0];
        assert_eq!(
            entry.entry_key.as_bytes(),
            emitted.kernel_identity().as_bytes()
        );
        assert_eq!(entry.symbol, emitted.symbol());
        assert_eq!(
            entry.transports,
            emitted
                .buffers()
                .iter()
                .map(|binding| binding.index())
                .collect::<Vec<_>>(),
            "the transport slots are the emitted argument-table indices",
        );
    }

    /// The published pair is consistent: the sidecar names these exact bytes.
    ///
    /// This is the check the runner makes before it trusts anything, and the
    /// reason it exists here too is that the producer/runner handoff is a pair
    /// of files read at run time, which no compilation sees. When the producer
    /// stopped writing `.identity` and the runner still read it, the complete
    /// gate stayed green over a slice that was broken end to end.
    #[test]
    fn the_published_sidecar_binds_to_the_published_envelope() {
        let (artifact, envelope, sidecar_bytes) = published();
        let sidecar = decode_proof_sidecar(&sidecar_bytes).expect("the published sidecar decodes");
        sidecar
            .bind_to_envelope(&envelope)
            .expect("the sidecar names the envelope published beside it");
        sidecar
            .bind_to_artifact(&artifact)
            .expect("and re-proves its cases against the declared interface");
        assert_eq!(
            sidecar.artifact_identity_bytes(),
            artifact.canonical_identity().as_bytes(),
            "the runner takes its expected identity from here",
        );
    }

    /// A sidecar paired with a different envelope is refused, not tolerated.
    #[test]
    fn a_perturbed_envelope_no_longer_binds_its_sidecar() {
        let (_artifact, envelope, sidecar_bytes) = published();
        let sidecar = decode_proof_sidecar(&sidecar_bytes).expect("the published sidecar decodes");
        let mut perturbed = envelope.clone();
        let last = perturbed.len() - 1;
        perturbed[last] ^= 0x01;
        sidecar
            .bind_to_envelope(&perturbed)
            .expect_err("one flipped byte is a different envelope");
        sidecar
            .bind_to_envelope(&envelope[..envelope.len() - 1])
            .expect_err("a truncated envelope is a different envelope");
    }

    /// The producer validates the real bundle without a device, on the negative
    /// paths as well as the positive one.
    ///
    /// The codec's own cases pin these against synthetic content. This is where
    /// they meet a bundle a real `xcrun` link produced, which is the only place
    /// a field the encoder writes but the decoder ignores would show up.
    #[test]
    fn the_produced_bundle_is_refused_by_the_class_each_damage_earns() {
        let (_artifact, envelope, _sidecar) = published();
        decode_artifact(&envelope).expect("the undamaged bundle decodes");

        let mut trailing = envelope.clone();
        trailing.push(0x00);

        // Each offset names the header field it lands in, read from this
        // envelope's own layout. They are positions rather than names because
        // the codec exposes no way to address a header field, so a layout change
        // is expected to land here — and should, since the refusal a damaged
        // field earns is the property under test.
        let forms: [(&str, Vec<u8>, &str); 12] = [
            ("no bytes at all", Vec::new(), "malformed"),
            (
                "half the envelope",
                envelope[..envelope.len() / 2].to_vec(),
                "malformed",
            ),
            (
                "one byte short",
                envelope[..envelope.len() - 1].to_vec(),
                "malformed",
            ),
            ("the magic alone", envelope[..8].to_vec(), "malformed"),
            ("one trailing byte", trailing, "malformed"),
            ("a damaged magic", flip(&envelope, 0), "malformed"),
            (
                "an envelope format this reader does not implement",
                flip(&envelope, 8),
                "unsupported",
            ),
            (
                "a canonical encoding this reader does not implement",
                flip(&envelope, 13),
                "unsupported",
            ),
            (
                "a digest algorithm this reader does not implement",
                flip(&envelope, 16),
                "unsupported",
            ),
            (
                "a declared total length that is not the actual one",
                flip(&envelope, 24),
                "malformed",
            ),
            (
                "a section count past the governed bound",
                flip(&envelope, 36),
                "limit",
            ),
            (
                "a damaged payload section",
                flip(&envelope, envelope.len() - 2),
                "integrity",
            ),
        ];

        for (form, bytes, expected) in forms {
            let refusal = decode_artifact(&bytes)
                .map(|_| ())
                .expect_err(&format!("{form} is refused"));
            assert_eq!(
                class(&refusal),
                expected,
                "{form} was refused, but as {refusal}",
            );
        }
    }

    /// A structural violation cannot be reached by damaging these bytes,
    /// because the manifest digest refuses first.
    ///
    /// This is the boundary of what the case above can measure, stated rather
    /// than left as apparent coverage. `ArtifactCodecFailure::Invalid` is the
    /// class carrying noncanonical order, duplicate items, and dangling or
    /// missing references, and `ArtifactIdentityMismatch` is an integrity
    /// failure over the same covered bytes. Every one of them lives inside the
    /// region the manifest digest covers, so byte surgery on a published
    /// envelope always earns `IntegrityFailure` before any structural check
    /// runs. Reaching them needs a manifest *re-encoded* around the violation,
    /// which is a codec-internal construction this producer cannot perform and
    /// should not gain a way to.
    ///
    /// They are covered there instead, against content built for the purpose:
    /// `a_forged_identity_is_rejected`, `a_repeated_interface_key_is_rejected`,
    /// `an_unreferenced_section_is_rejected`,
    /// `a_repeated_expression_node_is_rejected`, and
    /// `an_expression_reference_outside_the_arena_is_rejected` in
    /// `crates/tiler-artifact/src/program/codec/tests.rs`.
    ///
    /// To refute the precedence claim rather than the conclusion, flip any byte
    /// at or past the manifest digest at offset 37 and observe the class.
    #[test]
    fn a_structural_violation_is_unreachable_behind_the_manifest_digest() {
        let (_artifact, envelope, _sidecar) = published();
        for offset in [40, envelope.len() / 3, envelope.len() / 2] {
            let refusal = decode_artifact(&flip(&envelope, offset))
                .map(|_| ())
                .expect_err("a damaged manifest is refused");
            assert_eq!(
                class(&refusal),
                "integrity",
                "a digest-covered byte earned {refusal} rather than an integrity failure",
            );
        }
    }

    /// Returns `bytes` with the byte at `offset` inverted.
    fn flip(bytes: &[u8], offset: usize) -> Vec<u8> {
        let mut damaged = bytes.to_vec();
        damaged[offset] ^= 0xff;
        damaged
    }

    /// Names one refusal's class.
    ///
    /// The five classes are the codec's own account of *why* it refused, and a
    /// bare `expect_err` cannot tell them apart: a bundle rejected as malformed
    /// where it should have been rejected as unsupported would pass a test that
    /// only asked whether it was rejected.
    ///
    /// `ArtifactCodecFailure` is `#[non_exhaustive]`, so this match cannot be
    /// exhaustive and a sixth class would not be a build error here. The
    /// wildcard therefore returns a name no case expects, which fails the
    /// assertion carrying the refusal's own text, rather than folding an
    /// unrecognized class into one of the five and reporting agreement.
    fn class(failure: &ArtifactCodecFailure) -> &'static str {
        match failure {
            ArtifactCodecFailure::Malformed { .. } => "malformed",
            ArtifactCodecFailure::IntegrityFailure { .. } => "integrity",
            ArtifactCodecFailure::Unsupported { .. } => "unsupported",
            ArtifactCodecFailure::Invalid { .. } => "invalid",
            ArtifactCodecFailure::Limit { .. } => "limit",
            _ => "a class this test does not name",
        }
    }

    /// Produces the exact triple the producer publishes, through the real path.
    fn published() -> (
        tiler_artifact::program::VerifiedArtifactProgram,
        Vec<u8>,
        Vec<u8>,
    ) {
        let directory = std::env::temp_dir().join(format!(
            "tiler-prototype-published-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        let cache = ExpansionCache::open(directory.join("cache"));
        let program = serial_sum_program(ROWS, COLUMNS);
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let selected = compilation.selected().expect("a selected alternative");
        let accepted = tiler_build::accept_or_publish_metal_plan(
            &cache,
            &Toolchain::system(),
            &program,
            selected,
            &target_facts(),
            build_policy(),
        )
        .expect("the checked plan resolves");
        let artifact = accepted.artifact().clone();
        let envelope = match accepted.resolution() {
            Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
                entry.envelope_bytes().to_vec()
            }
            Resolution::Uncached { envelope, .. } => envelope.clone(),
        };
        let sidecar_bytes =
            sidecar::encoded(&artifact, &program, ROWS, COLUMNS).expect("the sidecar builds");
        let _ = std::fs::remove_dir_all(directory);
        (artifact, envelope, sidecar_bytes)
    }

    /// An out-of-crate caller composes its own lowering registry and compiles
    /// through it.
    ///
    /// **This is the ticket's closing condition, and it can only be asserted from
    /// here.** ADR 0078 item 4 records the asymmetry it closes: everything needed
    /// to *build* a `FrozenLoweringCapabilityRegistry` was already public, and
    /// nothing could install one, so a provider written against the public
    /// surface could never reach the compile path. The compiler's own conformance
    /// case for this composition had to live inside `tiler-compiler` because two
    /// steps of the recipe and one field assignment were `pub(crate)`.
    ///
    /// This crate takes `tiler-compiler` as an ordinary dependency and sees only
    /// its public surface, so if this compiles and passes, the path is genuinely
    /// reachable rather than reachable-looking.
    #[test]
    fn an_out_of_crate_caller_installs_its_own_capability_registry() {
        let program = serial_sum_program(ROWS, COLUMNS);
        let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority");
        let mut builder = LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        );
        install_governed_index_access(&mut builder, &[])
            .expect("the governed capabilities install onto a caller's builder");
        let installed = InstalledCapabilities::installed(builder.freeze(), scalars);

        let batch = compile(
            CompileRequest::new(
                &program,
                NumericalContract::FlushSubnormalsToZeroF32,
                governed_targets(),
            )
            .with_capabilities(installed),
        )
        .expect("a caller-installed registry compiles the governed program");
        let outcomes: Vec<_> = batch.targets().collect();
        assert_eq!(outcomes.len(), 1);
        assert!(
            outcomes[0]
                .outcome()
                .expect("the governed target compiles")
                .selected()
                .is_some(),
            "the caller's own registry resolved every occurrence",
        );
    }

    /// Omitting one family from the installed registry fails closed.
    ///
    /// The negative half, and the one that makes the positive mean anything: a
    /// `with_capabilities` that silently ignored its argument and used Tiler's
    /// governed snapshot would pass the case above and fail this one. It also
    /// shows the installed authority is genuinely what resolution runs through
    /// rather than a value the request records and never consults.
    #[test]
    fn an_installed_registry_missing_a_family_fails_closed() {
        let program = serial_sum_program(ROWS, COLUMNS);
        let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority");
        let mut builder = LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        );
        // Everything except the multiply family this program needs.
        install_governed_index_access(&mut builder, &[multiply_f32_op()])
            .expect("the remaining governed capabilities install");
        let installed = InstalledCapabilities::installed(builder.freeze(), scalars);

        // Matched rather than `expect_err`: the success value is a whole
        // compilation, and unwrapping it on failure renders megabytes of plan
        // where one sentence is wanted.
        let outcome = compile(
            CompileRequest::new(
                &program,
                NumericalContract::FlushSubnormalsToZeroF32,
                governed_targets(),
            )
            .with_capabilities(installed),
        );
        let Err(failure) = outcome else {
            panic!(
                "a registry with no multiply capability compiled the program anyway, so the \
                 installed authority was not the one resolution ran through",
            );
        };
        // The program is valid; this installed authority does not cover it. That
        // is a capability statement, never invalid compiler output.
        assert!(
            !matches!(failure.class(), CompileFailureClass::InvalidCompilerOutput),
            "an uncovered occurrence was reported as a Tiler defect: {failure:?}",
        );
    }

    /// The producer's half of the *member* filename interface, pinned.
    ///
    /// `prototypes/serial-sum-run` derives these same twelve names from the base
    /// path it is given and carries the identical assertion. Nothing mechanical
    /// compares the two crates -- they share no code -- so this pair of tests is
    /// the only thing that does, and the slice has already been broken end to
    /// end for a whole commit by one side renaming a file the other opened.
    #[test]
    fn the_member_names_are_the_ones_the_runner_opens() {
        let base = std::path::Path::new("/tmp/a.tiler");
        let names: Vec<String> = REDUCTION_CLASSES
            .iter()
            .flat_map(|(class, _)| {
                PLAN_ROLES
                    .iter()
                    .map(move |role| super::proof_member(base, class, role))
            })
            .map(|path| path.display().to_string())
            .collect();
        assert_eq!(
            names,
            [
                "/tmp/a.tiler.empty-domain.selected",
                "/tmp/a.tiler.empty-domain.materialized",
                "/tmp/a.tiler.singleton.selected",
                "/tmp/a.tiler.singleton.materialized",
                "/tmp/a.tiler.nontrivial.selected",
                "/tmp/a.tiler.nontrivial.materialized",
            ],
        );
        assert_eq!(
            super::proof_sidecar(std::path::Path::new(&names[0]))
                .display()
                .to_string(),
            "/tmp/a.tiler.empty-domain.selected.proof",
        );
    }

    /// The producer's half of the filename interface, pinned.
    ///
    /// `prototypes/serial-sum-run` carries the identical assertion. Changing
    /// one without the other fails there, which is the whole point: the two
    /// crates share no code, so this pair of tests is the only thing that
    /// compares their idea of the name.
    #[test]
    fn the_sidecar_suffix_is_the_one_the_runner_opens() {
        assert_eq!(super::SIDECAR_SUFFIX, ".proof");
        assert_eq!(
            super::proof_sidecar(std::path::Path::new("/tmp/a.tiler")),
            std::path::PathBuf::from("/tmp/a.tiler.proof"),
        );
    }

    /// Measures whether `xcrun` produced byte-identical `metallib` output twice.
    ///
    /// Recorded as evidence, never asserted. Reproducibility of the linker's
    /// bytes is a property of a toolchain this repository does not control and
    /// has not proven for any version; turning an observation on one host into
    /// a gate would make an unrelated Xcode update look like a Tiler defect.
    /// What *is* asserted is the part Tiler owns -- the emitted MSL and the
    /// artifact identity derived from it, both pinned by the determinism case
    /// above.
    ///
    /// The provenance printed here is what makes the measurement quotable: a
    /// reproducibility claim with no toolchain attached names no environment.
    #[test]
    fn metallib_byte_reproducibility_is_measured_and_recorded() {
        let program = serial_sum_program(ROWS, COLUMNS);
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let selected = compilation.selected().expect("a selected alternative");
        let kernels: Vec<_> = selected.kernels().iter().collect();

        let (_first_unit, first) = super::emit_and_compile(&kernels);
        let (_second_unit, second) = super::emit_and_compile(&kernels);

        let reproducible = first.content().code == second.content().code;
        let provenance = &first.content().metadata.provenance;
        println!(
            "metallib reproducibility: {} ({} and {} bytes)",
            if reproducible {
                "byte-identical across two links on this host"
            } else {
                "NOT byte-identical across two links on this host"
            },
            first.content().code.len(),
            second.content().code.len(),
        );
        println!("  toolchain components: {:?}", provenance.components);
        println!(
            "  sdk: {} {} (build {}), target {:?}",
            provenance.sdk.name,
            provenance.sdk.version,
            provenance.sdk.build,
            provenance.compile_flags,
        );
    }

    /// Compiles the proof program once for the payload cases above.
    fn emit_and_compile() -> (
        tiler_metal::record::MetalTranslationUnit,
        tiler_build::CompiledMetalPayload,
    ) {
        let program = serial_sum_program(ROWS, COLUMNS);
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let plan = compilation.selected().expect("a selected alternative");
        let kernels: Vec<_> = plan.kernels().iter().collect();
        super::emit_and_compile(&kernels)
    }

    /// The contract a caller states decides whether this target can honour it.
    ///
    /// This case previously asserted a *refusal*, because the strict contract
    /// was the only one the compiler registered and Apple `f32` arithmetic
    /// flushes subnormals in every math mode. It was deliberately written to
    /// break the day a contract became selectable, so that its reasoning had to
    /// be re-derived rather than silently passing under a new meaning. This is
    /// that re-derivation.
    ///
    /// Both directions are asserted, because the point is that the caller's
    /// statement is load-bearing: the strict contract is still refused on this
    /// target — it is not deliverable and must not be emitted — and the
    /// flush-accepting contract is honoured. Nothing was relaxed to reach the
    /// second; a different contract was stated.
    #[test]
    fn the_stated_contract_decides_whether_this_target_honours_it() {
        let program = serial_sum_program(ROWS, COLUMNS);

        let strict = compile_governed(&program, NumericalContract::StrictF32)
            .expect("the strict program still compiles");
        let strict_plan = strict.selected().expect("a selected alternative");
        let strict_kernels: Vec<_> = strict_plan.kernels().iter().collect();
        let strict_unit =
            emit_translation_unit(&strict_kernels, &target_facts(), emission_realization())
                .expect("the kernels emit");
        assert!(
            strict_unit.require_declared_realization().is_err(),
            "a subnormal-preserving contract is still unrealizable on a flushing target",
        );
        assert_eq!(
            strict_unit
                .numerical_gaps()
                .iter()
                .map(|gap| gap.rule())
                .collect::<Vec<_>>(),
            ["subnormal-flush-in-arithmetic"],
        );

        let flushing = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the flush-accepting program compiles");
        let flush_plan = flushing.selected().expect("a selected alternative");
        let flush_kernels: Vec<_> = flush_plan.kernels().iter().collect();
        let flush_unit =
            emit_translation_unit(&flush_kernels, &target_facts(), emission_realization())
                .expect("the kernels emit");
        flush_unit
            .require_declared_realization()
            .expect("the target honours the contract the caller stated");
        assert!(
            flush_unit.numerical_gaps().is_empty(),
            "an honoured contract leaves no gap",
        );
    }
}
