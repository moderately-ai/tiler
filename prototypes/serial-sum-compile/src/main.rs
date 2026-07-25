//! The non-published offline producer for the serial-Sum vertical slice.
//!
//! It drives the already-implemented component capabilities — semantic
//! construction, compilation, MSL emission, and offline Metal compilation —
//! through one path, and implements no component capability of its own. The one
//! thing it owns is the orchestration those components cannot own individually:
//! [`target`], the translation between the emitter's and the driver's target
//! vocabularies, which exists here because neither backend crate may depend on
//! the other.
//!
//! # What this proves and what it does not
//!
//! Running it proves the offline path composes end to end: a semantic program
//! reaches a verified kernel through the public compiler boundary, that kernel
//! emits deterministic MSL, `xcrun` turns that MSL into a real `metallib` on this
//! host, and [`bundle`] carries both in a neutral artifact envelope that survives
//! an encode, a decode, and a byte-identical re-encode.
//!
//! It proves nothing about execution — no device is created, no kernel is
//! dispatched, and no output bits are compared. An artifact that assembles,
//! encodes, and re-validates from its own bytes is not an artifact that has run.

mod bundle;
mod payload;
mod target;

use std::fmt;
use std::process::ExitCode;

use tiler_artifact::program::{ArtifactCodecFailure, decode_artifact};
use tiler_compiler::session::{CompileFailure, NumericalContract, compile_governed};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalFlushedZeroSign, MetalPlatform,
    MetalSubnormalArithmetic, MetalTargetFacts, MslLanguageVersion,
};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{CompileRequest, NumericalRealization, OptimizationLevel};

/// The buffer argument-table capacity Apple's feature tables state per compute
/// function for every current family.
///
/// Stated by the caller rather than derived, so a signature needing more
/// bindings is rejected instead of emitted with an unaddressable attribute.
const BUFFER_BINDING_LIMIT: u32 = 31;

/// The target facts this producer emits for.
///
/// macOS 13.0 under MSL 3.1, declaring the measured Apple subnormal behaviour:
/// `f32` arithmetic flushes subnormal operands and results to zero on every
/// governed family. Declaring it is what lets emission reject a kernel whose
/// numerical contract the target cannot honour, rather than emitting one that
/// silently computes something else.
fn target_facts() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(13, 0),
        LaunchIndexRealization::ThreadPositionInGridUInt,
        MetalSubnormalArithmetic::FlushesToZero {
            zero_sign: MetalFlushedZeroSign::PreservesSign,
        },
        BUFFER_BINDING_LIMIT,
    )
}

/// Builds the bounded profile's scale-then-reduce program.
fn serial_sum_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([4, 1]),
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
    tiler_metal_aot::record::CompiledArtifact,
) {
    let unit = emit_translation_unit(kernels, &target_facts()).expect("the kernels emit");
    let request = CompileRequest::new(
        unit.source(),
        target::compile_target(target_facts()),
        OptimizationLevel::Default,
        NumericalRealization::strict_baseline(),
    );
    let artifact = Toolchain::system()
        .compile(&request)
        .expect("the offline toolchain compiles");
    (unit, artifact)
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

/// One offline pass: compile, emit, and translate to a `metallib`.
fn run() -> Result<(), ProducerError> {
    let program = serial_sum_program();
    // Stated, not defaulted. The strict contract is unhonourable on every
    // governed Apple family, so this producer says which contract its program
    // means rather than discovering that by reading a rejection.
    let compilations = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
        .map_err(ProducerError::Compile)?;
    let compilation = compilations.first().ok_or(ProducerError::NoTarget)?;
    println!("target profile: {}", compilation.target_profile_key());

    let selected = compilation.selected().ok_or(ProducerError::NoSelection)?;
    println!(
        "selected alternative: {} ({})",
        selected.stable_id(),
        if selected.is_fused() {
            "fused"
        } else {
            "materialized"
        },
    );

    let facts = target_facts();
    let kernels: Vec<_> = selected.kernels().iter().collect();
    let unit = emit_translation_unit(&kernels, &facts).map_err(|_| ProducerError::Emit)?;
    // Emission succeeds even when the target cannot honour the declared
    // numerical contract, so the conformance question is asked explicitly here
    // rather than inferred from a successful emission.
    unit.require_declared_realization()
        .map_err(|_| ProducerError::UnrealizableNumerics {
            gaps: unit
                .numerical_gaps()
                .iter()
                .map(|gap| gap.rule().to_owned())
                .collect(),
        })?;
    println!(
        "emitted {} entry point(s), {} bytes of MSL",
        unit.entry_points().len(),
        unit.source().len(),
    );

    let request = CompileRequest::new(
        unit.source(),
        target::compile_target(facts),
        OptimizationLevel::Default,
        NumericalRealization::strict_baseline(),
    );
    let artifact = Toolchain::system()
        .compile(&request)
        .map_err(|_| ProducerError::Toolchain)?;
    println!(
        "compiled {} bytes of metallib for {}",
        artifact.metallib.len(),
        request.target.triple(),
    );

    // Fill the neutral carried payload from what was just emitted and compiled.
    // The subject is the compilation *inputs*; the object travels beside it.
    let payload = payload::carried_payload(&unit, &artifact.provenance, &artifact.metallib)
        .map_err(ProducerError::Payload)?;
    let identity = payload
        .identity()
        .map_err(|_| ProducerError::PayloadIdentity)?;
    println!(
        "payload subject: {} entr(y/ies), {} obligation(s), identity {} bytes",
        payload.metadata.entries.len(),
        payload.metadata.obligations.len(),
        identity.as_bytes().len(),
    );

    // Carry it in the neutral envelope, then read the bytes back. A payload that
    // assembles but does not survive a round trip is not carried.
    //
    // A successful decode is itself the identity proof: `decode_artifact`
    // re-derives the identity from the decoded content and refuses when it does
    // not equal the one the manifest carries. The byte-identical re-encode is
    // the complement — a field the decoder silently dropped could not be written
    // back — and, because the encoder *derives* the identity rather than copying
    // it, byte equality also pins the manifest's stored identity to that
    // re-derivation.
    let artifact = bundle::assemble(&program, compilation, selected, payload)
        .map_err(ProducerError::Bundle)?;
    let bytes = artifact.encode().map_err(ProducerError::Encode)?;
    let decoded = decode_artifact(&bytes).map_err(ProducerError::Decode)?;
    if decoded.re_encode().map_err(ProducerError::Encode)? != bytes {
        return Err(ProducerError::UnstableEncoding);
    }
    println!(
        "artifact envelope: {} bytes, {} section(s), {} variant(s), identity {} bytes",
        bytes.len(),
        decoded.sections().len(),
        decoded.variant_count(),
        decoded.identity().as_bytes().len(),
    );
    Ok(())
}

/// Why one offline pass did not produce a `metallib`.
///
/// The stages are kept apart rather than collapsed into one message: a program
/// this build does not compile, a target that cannot honour the declared
/// numerics, and a host without a usable toolchain are three different things
/// to do next.
#[derive(Debug)]
enum ProducerError {
    Compile(CompileFailure),
    NoTarget,
    NoSelection,
    Emit,
    UnrealizableNumerics { gaps: Vec<String> },
    Payload(String),
    PayloadIdentity,
    Toolchain,
    Bundle(bundle::BundleError),
    Encode(ArtifactCodecFailure),
    Decode(ArtifactCodecFailure),
    UnstableEncoding,
}

impl fmt::Display for ProducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(failure) => write!(formatter, "the program did not compile: {failure:?}"),
            Self::NoTarget => formatter.write_str("the compilation returned no target profile"),
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::Emit => formatter.write_str("the selected kernels have no Metal realization"),
            Self::UnrealizableNumerics { gaps } => write!(
                formatter,
                "the target cannot honour the kernels' declared numerical contract: {}",
                gaps.join(", "),
            ),
            Self::Toolchain => {
                formatter.write_str("the offline Metal toolchain did not produce a metallib")
            }
            Self::Payload(cause) => write!(formatter, "the carried payload is malformed: {cause}"),
            Self::PayloadIdentity => {
                formatter.write_str("the carried payload's compilation subject has no identity")
            }
            Self::Bundle(cause) => write!(formatter, "the artifact did not assemble: {cause}"),
            Self::Encode(cause) => write!(formatter, "the envelope did not encode: {cause}"),
            Self::Decode(cause) => write!(formatter, "the envelope did not decode: {cause}"),
            Self::UnstableEncoding => {
                formatter.write_str("re-encoding the decoded envelope did not reproduce its bytes")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{payload, serial_sum_program, target_facts};
    use tiler_compiler::session::{NumericalContract, compile_governed};
    use tiler_metal::emit::emit_translation_unit;

    /// The offline path composes as far as deterministic MSL, without a
    /// toolchain.
    ///
    /// This is the part of `run` that has no host dependency, so it is the part
    /// the gate can assert. Compiling the emitted source needs `xcrun` and is
    /// exercised by running the producer, not by this test.
    #[test]
    fn the_selected_program_emits_deterministic_metal_source() {
        let program = serial_sum_program();
        let compilations = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let selected = compilations[0].selected().expect("a selected alternative");
        let kernels: Vec<_> = selected.kernels().iter().collect();

        let first = emit_translation_unit(&kernels, &target_facts()).expect("the kernels emit");
        let second = emit_translation_unit(&kernels, &target_facts()).expect("the kernels emit");
        assert_eq!(
            first.source(),
            second.source(),
            "emission is a pure function of the kernels and the target facts",
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
        let (unit, artifact) = emit_and_compile();
        let payload = payload::carried_payload(&unit, &artifact.provenance, &artifact.metallib)
            .expect("the payload assembles");
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
        let (unit, artifact) = emit_and_compile();
        let payload = payload::carried_payload(&unit, &artifact.provenance, &artifact.metallib)
            .expect("the payload assembles");
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
            assert!(
                !value.starts_with('/') && !value.contains("/Applications"),
                "{value:?} looks like a local path and must not be portable identity",
            );
        }
    }

    /// The entry mapping names the kernel identity, not the emitted symbol.
    #[test]
    fn the_entry_mapping_keys_on_the_kernel_identity() {
        let (unit, artifact) = emit_and_compile();
        let payload = payload::carried_payload(&unit, &artifact.provenance, &artifact.metallib)
            .expect("the payload assembles");
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

    /// Compiles the proof program once for the payload cases above.
    fn emit_and_compile() -> (
        tiler_metal::record::MetalTranslationUnit,
        tiler_metal_aot::record::CompiledArtifact,
    ) {
        let program = serial_sum_program();
        let compilations = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let plan = compilations[0].selected().expect("a selected alternative");
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
        let program = serial_sum_program();

        let strict = compile_governed(&program, NumericalContract::StrictF32)
            .expect("the strict program still compiles");
        let strict_plan = strict[0].selected().expect("a selected alternative");
        let strict_kernels: Vec<_> = strict_plan.kernels().iter().collect();
        let strict_unit =
            emit_translation_unit(&strict_kernels, &target_facts()).expect("the kernels emit");
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
        let flush_plan = flushing[0].selected().expect("a selected alternative");
        let flush_kernels: Vec<_> = flush_plan.kernels().iter().collect();
        let flush_unit =
            emit_translation_unit(&flush_kernels, &target_facts()).expect("the kernels emit");
        flush_unit
            .require_declared_realization()
            .expect("the target honours the contract the caller stated");
        assert!(
            flush_unit.numerical_gaps().is_empty(),
            "an honoured contract leaves no gap",
        );
    }
}
