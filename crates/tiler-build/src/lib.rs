//! Build-time compilation, artifact, and cache orchestration for Tiler.
//!
//! This crate sits downstream of compiler, backend, artifact, AOT-driver, and
//! cache authorities. It may compare their facts and sequence their operations;
//! it does not re-derive any authority's identity or parse its private subject
//! encoding.
//!
//! The implemented Metal slice validates that carried metadata describes the
//! exact prepared compilation. Its checked-plan facade carries an owner-linked
//! compiler alternative through Metal emission, AOT preparation, neutral
//! artifact assembly, complete cache-subject construction, miss-only
//! compilation, and correspondence validation before either publication or hit
//! acceptance.

mod metal_assembly;
mod metal_cache;
mod metal_payload;
mod metal_plan;
mod metal_profile;

pub use metal_assembly::{
    CompiledMetalPayload, MetalAssemblyError, PreparedMetalPayload, metal_compile_request,
    prepare_metal_payload,
};
pub use metal_cache::{
    AcceptedMetalArtifact, MetalArtifactProtocolError, MetalCacheError,
    accept_or_publish_single_payload_metal_artifact,
};
pub use metal_payload::{MetalPayloadFact, MetalPayloadMismatch, validate_prepared_metal_payload};
pub use metal_plan::{
    AcceptedMetalPlanArtifact, MetalPlanBuildError, MetalPlanBuildPolicy,
    accept_or_publish_metal_plan,
};
pub use metal_profile::{MetalF32TargetProfileError, declare_metal_f32_subnormal_behaviour};
