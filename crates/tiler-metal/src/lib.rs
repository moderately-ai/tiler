//! Pure structured-kernel-to-Metal-source lowering for Tiler.
//!
//! This crate owns deterministic source emission and target metadata, not live
//! device/runtime APIs, Apple tool discovery, offline compiler invocation,
//! artifact caching, or publication. Host-side AOT orchestration belongs in
//! `tiler-metal-aot`.
