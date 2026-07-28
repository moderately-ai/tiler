//! Target-independent optimization, scheduling, and structured lowering.
//!
//! This crate owns compiler decisions and may construct artifact plans. It must
//! not depend on Metal emission, live runtime APIs, Candle, or frontend syntax.

mod boundary;
pub mod capability;
mod component_cost;
mod cover;
mod effects;
mod estimate;
mod explain;
mod feasibility;
mod frontier;
mod fusion;
mod fusion_legality;
mod governed;
mod honourability;
#[cfg(test)]
mod hot_path;
pub mod legality;
mod lowering;
mod normalize;
mod physical;
mod pipeline;
mod program;
mod region;
mod request;
mod rewrite;
mod selection;
pub mod session;
#[cfg(test)]
mod workcount;

// Keep the bounded compiler path in the ordinary library target while its
// reviewed public facade is introduced by the capability and conformance
// slices. This is a compile-time reachability assertion, not a public entry
// point and not a second compilation path.
const _: for<'a> fn(
    request::CompilationRequest<'a>,
) -> Result<pipeline::CompilationProduct, pipeline::CompileError> = pipeline::compile;
