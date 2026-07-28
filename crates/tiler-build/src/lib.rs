//! Build-time compilation, artifact, and cache orchestration for Tiler.
//!
//! This crate sits downstream of compiler, backend, artifact, AOT-driver, and
//! cache authorities. It may compare their facts and sequence their operations;
//! it does not re-derive any authority's identity or parse its private subject
//! encoding.
//!
//! The implemented Metal slice validates that carried metadata describes the
//! exact prepared compilation, derives the complete cache subject from the
//! prepared compilation and pending artifact identities, compiles only inside
//! the cache's miss closure, and re-proves correspondence before accepting
//! either a publication or a hit. Broader compiler-plan orchestration remains
//! incremental.

mod metal_assembly;
mod metal_cache;
mod metal_payload;

pub use metal_assembly::{
    CompiledMetalPayload, MetalAssemblyError, PreparedMetalPayload, metal_compile_request,
    prepare_metal_payload,
};
pub use metal_cache::{
    AcceptedMetalArtifact, MetalArtifactProtocolError, MetalCacheError,
    accept_or_publish_single_payload_metal_artifact,
};
pub use metal_payload::{MetalPayloadFact, MetalPayloadMismatch, validate_prepared_metal_payload};
