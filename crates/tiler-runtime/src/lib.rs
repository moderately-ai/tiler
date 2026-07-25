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
//! # Public boundary status
//!
//! [`load`] is a **reviewed draft boundary** (ADR 0074 §7, ADR 0075), on the
//! same footing as [`tiler_artifact::program`]: it is `pub` so its shape can be
//! reviewed as a whole, and it is not an accepted public facade until Tom
//! accepts the exact interface.

pub mod load;
