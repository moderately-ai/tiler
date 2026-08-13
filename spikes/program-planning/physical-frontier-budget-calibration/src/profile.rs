//! Target profiles this spike compiles against.

use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuilder, TargetProfileKey,
};
use tiler_ir::schedule::{ExceptionalValueAssumption, NumericalPermission, SubnormalMode};
use tiler_ir::semantic::F32;

/// The compiler-governed prototype target-neutral baseline profile.
#[must_use]
pub fn governed() -> TargetProfile {
    TargetProfile::governed()
}

/// A profile whose workgroup capacity is a compile-time declared fact.
///
/// Declared rather than deferred: the governed profile answers workgroup
/// capacity through a prepared-entry query, so a specialization that overran it
/// would resolve as a deferred predicate and never produce the hard infeasible
/// rejection the infeasible-proposal workload needs to observe.
#[must_use]
pub fn declared_workgroup_profile(key: &str, max_threads_per_workgroup: u32) -> TargetProfile {
    let source = TargetFactSource::external_guarantee(
        TargetFactProducerIdentity::new("test.calibrate-profile-producer.v1".to_owned(), 1)
            .expect("the producer identity is valid"),
        TargetNormativeReferenceIdentity::new("test.calibrate-profile-spec.v1".to_owned(), 1)
            .expect("the specification identity is valid"),
    );
    let mut builder =
        TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).expect("the key is valid"));
    builder
        .declare_max_threads_per_grid_axis(65_535, source.clone())
        .expect("grid axis");
    builder
        .declare_max_threads_per_workgroup(max_threads_per_workgroup, source.clone())
        .expect("workgroup");
    builder
        .declare_max_buffer_bindings_per_entry(31, source.clone())
        .expect("bindings");
    builder
        .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
        .expect("index arithmetic");
    builder
        .declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())
        .expect("address width");
    builder
        .declare_device_memory(true, source.clone())
        .expect("device memory");
    builder
        .declare_local_memory_bytes(32_768, source.clone())
        .expect("local memory");
    let subject = ScalarArithmetic::f32();
    builder
        .declare_input_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("input subnormals");
    builder
        .declare_result_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("result subnormals");
    builder
        .declare_contraction(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("contraction");
    builder
        .declare_reassociation(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("reassociation");
    builder
        .declare_permutation(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("permutation");
    builder
        .declare_signed_zero(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("signed zero");
    builder
        .declare_nan_assumptions(
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("nan");
    builder
        .declare_infinity_assumptions(
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .expect("infinity");
    builder
        .declare_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source,
        )
        .expect("dtype");
    builder.build().expect("the declared profile builds")
}
