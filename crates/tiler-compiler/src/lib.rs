//! Target-independent optimization, scheduling, and structured lowering.
//!
//! This crate owns compiler decisions and may construct artifact plans. It must
//! not depend on Metal emission, live runtime APIs, Candle, or frontend syntax.
//!
//! [`session`] is the one boundary over which a caller compiles; every public
//! module beside it supplies something a request is composed from, never a
//! second compilation path. The private `pipeline` module is that one path, and
//! `session` is its only caller outside this crate's own tests.

mod boundary;
mod call_abi;
mod call_declaration;
mod call_placement;
mod call_registry;
pub mod capability;
mod component_cost;
mod cover;
mod effects;
mod elementary;
mod estimate;
mod explain;
mod failure_stage;
mod frontier;
mod fusion;
mod fusion_legality;
mod governed;
#[cfg(test)]
mod hot_path;
mod index_discharge;
pub mod legality;
mod lowering;
mod normalize;
mod physical;
mod pipeline;
mod policy;
mod program;
mod region;
mod request;
mod rewrite;
mod selection;
pub mod session;
pub mod target;
#[cfg(test)]
mod workcount;
