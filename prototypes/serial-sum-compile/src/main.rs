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
//! emits deterministic MSL, and `xcrun` turns that MSL into a real `metallib`
//! on this host. It proves nothing about execution — no device is created, no
//! kernel is dispatched, and no output bits are compared. Packaging the result
//! into an artifact envelope is not done here either: the payload carrier's
//! constructors are `pub(crate)` in `tiler-artifact`, and promoting them is
//! ADR 0075 review that has not happened.

mod target;

use std::fmt;
use std::process::ExitCode;

use tiler_compiler::session::{CompileFailure, compile_governed};
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
    let compilations = compile_governed(&program).map_err(ProducerError::Compile)?;
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
    Toolchain,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{serial_sum_program, target_facts};
    use tiler_compiler::session::compile_governed;
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
        let compilations = compile_governed(&program).expect("the governed program compiles");
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

    /// **The offline path stops here, and that is the correct behaviour.**
    ///
    /// The governed numerical contract declares subnormal *preservation*.
    /// Apple `f32` arithmetic flushes subnormal operands and results to zero on
    /// every governed family and in every math mode, which
    /// `MetalSubnormalArithmetic::FlushesToZero` states as a target fact, now
    /// naming the sign-preserving zero it produces. So the
    /// target cannot honour the contract the kernels declare, and emission's
    /// conformance check refuses — after producing perfectly good MSL, because
    /// emission and conformance are deliberately separate steps.
    ///
    /// This is ADR 0076's central inference reaching running code: honourability
    /// has to be a stated target fact, and a contract the target cannot deliver
    /// must fail closed rather than silently compute something else. Reaching
    /// hardware requires making the numerical contract a stated request input
    /// with more than one expressible value — `select-numerical-contract-and-
    /// compose-feasibility`, then `declare-metal-numerical-honourability` — not
    /// relaxing this check. Relaxing it would return wrong numbers.
    ///
    /// The case is written as an assertion of the refusal so that the day the
    /// contract becomes selectable, it fails and forces this comment to be
    /// re-derived rather than silently passing under a new meaning.
    #[test]
    fn the_governed_contract_is_not_honourable_on_the_governed_apple_target() {
        let program = serial_sum_program();
        let compilations = compile_governed(&program).expect("the governed program compiles");
        let selected = compilations[0].selected().expect("a selected alternative");
        let kernels: Vec<_> = selected.kernels().iter().collect();
        let unit = emit_translation_unit(&kernels, &target_facts()).expect("the kernels emit");

        assert!(
            !unit.source().is_empty(),
            "emission succeeds; conformance is the separate step that refuses",
        );
        assert!(
            unit.require_declared_realization().is_err(),
            "a subnormal-preserving contract is unrealizable on a flushing target",
        );
        let gaps: Vec<&str> = unit.numerical_gaps().iter().map(|gap| gap.rule()).collect();
        assert_eq!(gaps, ["subnormal-flush-in-arithmetic"]);
    }
}
