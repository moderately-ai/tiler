//! Read-only reproduction fixture for preserved draft 54e272ba.
//!
//! Copy this file into that revision's `crates/tiler-compiler/tests/` in a
//! disposable worktree. It deliberately uses only the draft's public surface.

use tiler_compiler::session::{CompileRequest, NumericalContract, compile};
use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuilder, TargetProfileKey, TargetRequest,
};
use tiler_ir::schedule::{ExceptionalValueAssumption, NumericalPermission, SubnormalMode};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

fn five_op_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

fn strict_profile(key: &str) -> TargetProfile {
    let source = TargetFactSource::external_guarantee(
        TargetFactProducerIdentity::new("test.draft-budget-profile.v1".to_owned(), 1).unwrap(),
        TargetNormativeReferenceIdentity::new("test.draft-budget-spec.v1".to_owned(), 1).unwrap(),
    );
    let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
    builder
        .declare_max_threads_per_grid_axis(65_535, source.clone())
        .unwrap();
    builder
        .declare_max_threads_per_workgroup(64, source.clone())
        .unwrap();
    builder
        .declare_max_buffer_bindings_per_entry(31, source.clone())
        .unwrap();
    builder
        .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
        .unwrap();
    builder
        .declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())
        .unwrap();
    builder
        .declare_device_memory(true, source.clone())
        .unwrap();
    builder
        .declare_local_memory_bytes(32_768, source.clone())
        .unwrap();
    let subject = ScalarArithmetic::f32();
    builder
        .declare_input_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_result_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    for declare in [
        TargetProfileBuilder::declare_contraction,
        TargetProfileBuilder::declare_reassociation,
        TargetProfileBuilder::declare_permutation,
        TargetProfileBuilder::declare_signed_zero,
    ] {
        declare(
            &mut builder,
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    }
    builder
        .declare_nan_assumptions(
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_infinity_assumptions(
            subject,
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source,
        )
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn request_scoped_256_returns_a_compiled_prefix() {
    let program = five_op_program();
    let profiles = (0..16)
        .map(|index| strict_profile(&format!("test.draft-budget-{index}.v1")))
        .collect::<Vec<_>>();
    let batch = compile(CompileRequest::new(
        &program,
        NumericalContract::STRICT_F32,
        TargetRequest::new(profiles).unwrap(),
    ))
    .unwrap();
    let mut succeeded = Vec::new();
    let mut refused = Vec::new();
    for (index, target) in batch.into_targets().into_iter().enumerate() {
        let (_, outcome) = target.into_parts();
        match outcome {
            Ok(_) => succeeded.push(index),
            Err(failure) => refused.push((index, format!("{failure:?}"))),
        }
    }
    println!("MEASURE draft succeeded={succeeded:?} refused={refused:?}");
    assert_eq!(succeeded, (0..13).collect::<Vec<_>>());
    assert_eq!(refused.len(), 3);
    assert_eq!(
        refused.iter().map(|(index, _)| *index).collect::<Vec<_>>(),
        vec![13, 14, 15],
    );
    for (_, failure) in refused {
        assert!(failure.contains("BudgetExhausted"), "{failure}");
        assert!(failure.contains("PhysicalFrontierOutcomes"), "{failure}");
        assert!(failure.contains("limit: 256"), "{failure}");
        assert!(failure.contains("reported: 257"), "{failure}");
    }
}
