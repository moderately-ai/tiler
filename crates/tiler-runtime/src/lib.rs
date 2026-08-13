#![doc(test(attr(forbid(unsafe_code))))]
//! Device-free artifact loading and validation for Tiler runtimes.
//!
//! A Tiler runtime has two halves that fail for unrelated reasons. One half
//! decides whether a sequence of bytes is an artifact this host may execute;
//! the other creates device objects, encodes commands, and submits them. This
//! crate is the first half and only the first half. It touches no device, no
//! `MTLDevice`, no pipeline state, and no platform binding, and it depends on
//! [`tiler_artifact`] alone.
//!
//! That boundary is why the crate exists rather than being folded into a
//! backend runner. Everything here is decidable from bytes plus a host's stated
//! execution environment, so it is testable without hardware, portable across
//! backends, and — where it refuses — able to say exactly which obligation was
//! not met before anything has been allocated.
//!
//! # It is not a second authority over artifacts
//!
//! [`tiler_artifact::program::decode_artifact`] owns framing, integrity,
//! schema, canonical order, arena closure, required-feature negotiation, and
//! identity re-derivation. This crate calls it and classifies its rejection; it
//! re-implements none of those checks and cannot weaken them. Holding a
//! [`load::DecodedProgram`] therefore already means the bytes passed every
//! check the artifact layer performs.
//!
//! What this crate adds is the part the artifact layer cannot know, because it
//! is about the *host*: whether the declared target profile is the one this
//! machine offers, whether the loaded artifact is the one the caller expects,
//! which carried object realizes it, and the one-way commitment that separates
//! "still deciding" from "executing".
//!
//! # The device half is a seam, not a dependency
//!
//! [`adapter`] names the other half without linking it. A consumer implements
//! [`adapter::RuntimeAdapter`] for the one backend and representation family it
//! executes, selects it itself — there is no registry, no discovery, and no
//! adapter identity that travels in an artifact — and hands it to
//! [`adapter::route_with_adapter`], which sequences the loader's comparisons and
//! the adapter's reports in the order their facts become decidable. Every
//! comparison stays here; the adapter reports and never adjudicates.
//!
//! That seam does not weaken the boundary above it. The trait names no device,
//! queue, pipeline, or buffer type, this crate still depends on
//! [`tiler_artifact`] alone, and every device object lives in the implementor.
//!
//! # Public boundary status
//!
//! [`load`] and [`adapter`] are **reviewed draft boundaries** (ADR 0074 §7,
//! ADR 0075), on the same footing as [`tiler_artifact::program`]: they are `pub`
//! so their shape can be reviewed as a whole, and neither module as a whole is
//! an accepted public facade. The variant-eligibility vocabulary inside
//! [`load`] is the exception: Tom accepted that included and excluded set on
//! 2026-08-11 under [`accept-the-loader-variant-eligibility-vocabulary`].
//!
//! [`accept-the-loader-variant-eligibility-vocabulary`]: ../../../../tickets/accept-the-loader-variant-eligibility-vocabulary.md

pub mod adapter;
pub mod load;
