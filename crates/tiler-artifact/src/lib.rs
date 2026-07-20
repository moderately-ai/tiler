//! Target-neutral artifact, ABI, validation, and routing contracts for Tiler.
//!
//! This crate may depend on lockstep prototype IR types, but it must never call
//! compiler passes. The initial shell establishes that runtime-facing boundary.
