#![doc(test(attr(forbid(unsafe_code))))]
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
//! # The two surfaces here that are not Metal's
//!
//! [`assemble_plan_artifact`] is the backend-neutral build-time *assembly* seam
//! [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 promotes. It names no backend and takes no Metal type; a producer
//! supplies its delivery-ordered payload run and, per stage, the binding
//! transports, the zero-work dispatch policy, and the launch preconditions,
//! while every fact the checked plan already decided is derived from the plan.
//!
//! [`accept_or_publish_delivered_payload_artifact`] is the *cache* half of the
//! same boundary: complete subject composition, miss-only external compilation,
//! identity agreement before publication, and re-validation of every result. A
//! backend states two things and no more — the governed payload descriptor it
//! declares at each delivery position, as data in a [`DeclaredPayload`], and how
//! a carried payload's metadata is compared against the compilation it performed
//! there, as one closure. Its own module documentation states why that split is
//! the shape and what the alternatives lose.
//!
//! A third thing a backend *may* state is debug text to retain beside a published
//! entry, in [`CompiledPayloads::retained`]. It is optional in the exact sense
//! that costs nothing: `From<Vec<PayloadContent>>` is the whole of the
//! non-retaining case, and a retention reaches neither the cache subject nor the
//! key, so one compilation resolves to one entry either way. The Metal path
//! states one run per offline stage per delivery position — `stage_retention`,
//! private to the `metal_cache` module — including the empty run of a stage that
//! said nothing,
//! which is what distinguishes a silent compiler from an entry published before
//! any of this existed.
//!
//! [`realization::translate`] is the third, and it is neither an assembly nor a
//! cache seam: it is the transcription of the compiler's borrowed
//! delivered-realization evidence into the record every artifact must carry.
//! [`assemble_plan_artifact`] calls it, so no backend states a numerical fact
//! and none can. Its own module documentation states why the transcription
//! forwards structured values rather than matching over them.
//!
//! The Metal path above is one caller of both, and
//! `crates/tiler-build/tests/custom_backend` is another that shares no code with
//! it. What remains bounded rather than neutral is stated rather than implied:
//! the cache seam admits one payload per delivery position, shared by every
//! executable entry, and an artifact whose entries are realized by different
//! objects at one position is expressible in the artifact model and is not
//! orchestrated there.
//!
//! This crate is also where one authoritative macOS Metal compile-time declaration is
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
mod metal_subgroup_declaration;
mod payload_cache;
mod plan_artifact;
pub mod realization;

pub use metal_assembly::{
    CompiledMetalPayload, MetalAssemblyError, PreparedMetalPayload, metal_compile_request,
    prepare_metal_payload,
};
pub use metal_cache::{
    MetalArtifactProtocolError, MetalCacheError, accept_or_publish_delivered_metal_artifact,
};
pub use metal_declaration::{
    BoundMetalCompileDeclaration, BoundMetalDeclarationError, MetalPlanProfileMismatch,
};
pub use metal_payload::{MetalPayloadFact, MetalPayloadMismatch, validate_prepared_metal_payload};
pub use metal_plan::{
    AcceptedMetalPlanArtifact, MetalPlanBuildError, accept_or_publish_metal_plan,
};
pub use metal_profile::{MetalF32TargetProfileError, declare_metal_f32_subnormal_behaviour};
pub use payload_cache::{
    AcceptedArtifact, CompiledPayloads, DeclaredPayload, DeliveredPayloadCacheError,
    DeliveredPayloadProtocolError, accept_or_publish_delivered_payload_artifact,
};
pub use plan_artifact::{
    BackendEntryDeclaration, PlanArtifactError, PlanDeterminismDeclaration, assemble_plan_artifact,
};
// The verdict half of [`BoundMetalCompileDeclaration::dtype_dispatchability_rows`],
// re-exported rather than restated. A consumer of that accessor must match the
// verdict to state anything about it, and the two that do — the frontend
// expansion and the Candle prototype — carry no `tiler-compiler` edge of their
// own. A second two-valued enum minted here would be a third spelling of a fact
// only the compiler profile produces, which is the drift the accessor exists to
// remove.
pub use tiler_compiler::target::DTypeDispatchability;
