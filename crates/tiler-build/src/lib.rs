//! Build-time compilation, artifact, and cache orchestration for Tiler.
//!
//! This crate sits downstream of compiler, backend, artifact, AOT-driver, and
//! cache authorities. It may compare their facts and sequence their operations;
//! it does not re-derive any authority's identity or parse its private subject
//! encoding.
//!
//! The first implemented slice validates that a Metal payload's carried
//! metadata describes the exact prepared compilation used for cache lookup or
//! execution. The broader compilation-to-publication pipeline authorized for
//! this crate remains incremental.

mod metal_assembly;
mod metal_payload;

pub use metal_assembly::{
    CompiledMetalPayload, MetalAssemblyError, PreparedMetalPayload, metal_compile_request,
    prepare_metal_payload,
};
pub use metal_payload::{MetalPayloadFact, MetalPayloadMismatch, validate_prepared_metal_payload};
