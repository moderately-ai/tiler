// `variant_count` sizes `request::BudgetResource::ALL` from its own enum, so a
// budget added to that vocabulary and not to the list is a build error rather
// than a census that quietly stops covering the domain it reports on. Every
// other site that has to know a `BudgetResource` is an exhaustive `match`,
// which `rustc` already closes. The list exists only for the key-distinctness
// test, so the feature is gated on `test` and no normal build needs it.
#![cfg_attr(test, feature(variant_count))]
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
/// Every identity-domain spelling this crate declares, pinned to exact bytes.
///
/// Crate-level because six domains are inline literals no constant can
/// enumerate; the test-only source census reaches them without adding a public
/// compiler boundary.
#[cfg(test)]
mod domains;
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
mod measured_cost;
mod normalize;
mod physical;
pub mod physical_provider;
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
