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
//!
//! # The one surface here that is not Metal's
//!
//! [`assemble_plan_artifact`] is the backend-neutral build-time orchestration
//! seam [ADR 0090](../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 promotes. It names no backend and takes no Metal type; a producer
//! supplies its payload and, per stage, the binding transports, the zero-work
//! dispatch policy, and the launch preconditions, while every fact the checked
//! plan already decided is derived from the plan. The Metal path above is one
//! caller of it, and `crates/tiler-build/tests/custom_backend` is another that
//! shares no code with it.
//!
//! What is *not* neutral is stated rather than implied: cache-subject
//! composition, miss-only external compilation, and payload correspondence
//! validation are still spelled for one prepared Metal compilation, so a second
//! backend that wants them composes them itself from `tiler_cache`'s own public
//! constructors. That boundary is where the next promotion would go.
//!
//! It is also where one authoritative macOS Metal compile-time declaration is
//! assembled and bound. [`BoundMetalCompileDeclaration`] is the only place in
//! the workspace that can see the compiler's target vocabulary, the Metal
//! emitter's, and the AOT driver's at once, which is why the checked profile,
//! the emission facts, the selected realizations, and the total AOT projection
//! are one value rather than four arguments a caller pairs by hand.

mod metal_assembly;
mod metal_cache;
mod metal_declaration;
mod metal_payload;
mod metal_plan;
mod metal_profile;
mod plan_artifact;

pub use metal_assembly::{
    CompiledMetalPayload, MetalAssemblyError, PreparedMetalPayload, metal_compile_request,
    prepare_metal_payload,
};
pub use metal_cache::{
    AcceptedMetalArtifact, MetalArtifactProtocolError, MetalCacheError,
    accept_or_publish_single_payload_metal_artifact,
};
pub use metal_declaration::{
    BoundMetalCompileDeclaration, BoundMetalDeclarationError, MetalPlanProfileMismatch,
};
pub use metal_payload::{MetalPayloadFact, MetalPayloadMismatch, validate_prepared_metal_payload};
pub use metal_plan::{
    AcceptedMetalPlanArtifact, MetalPlanBuildError, accept_or_publish_metal_plan,
};
pub use metal_profile::{MetalF32TargetProfileError, declare_metal_f32_subnormal_behaviour};
pub use plan_artifact::{BackendEntryDeclaration, PlanArtifactError, assemble_plan_artifact};
