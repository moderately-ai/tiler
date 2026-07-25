#![allow(
    dead_code,
    unused_imports,
    reason = "the neutral artifact codec is a crate-private draft authority (ADR 0074 convention 7). It reserves the envelope framing, canonical manifest encoding, section digests, governed feature and schema compatibility, the decoder's re-proven obligations, and the carried-payload compilation subject. `unused_imports` covers the same reservation one level up: the payload vocabulary is re-exported to the crate so a backend assembler can name it, and until that assembler exists the re-export has no non-test consumer. Promoting the surface to `pub` is Tom's call under ADR 0075 and has not been made."
)]

//! The bounded canonical lockstep codec for the target-neutral artifact envelope.
//!
//! This module owns what `docs/artifact-abi.md` assigns to the artifact
//! contract: envelope framing, the canonical manifest's wire encoding, section
//! digests, component schema and required-feature compatibility, and failure
//! classification. It does not own program or portfolio meaning, canonical
//! identity semantics, or ABI-expression semantics; those stay with the model
//! and the shared IR, and this module reaches them rather than restating them.
//!
//! # What the encoding commits to
//!
//! An encoded envelope is a fixed-width framing header, one canonical manifest,
//! and a stream of length-delimited sections. The header bounds total length,
//! manifest length, and section count before anything is allocated, names the
//! governed digest algorithm explicitly, and carries the digest of the exact
//! manifest bytes. Each section descriptor in the manifest carries that
//! section's exact byte length and the digest of its exact bytes. The digest
//! over the complete envelope is derived externally and is never stored inside
//! the bytes it covers.
//!
//! The manifest carries every fact the artifact layer owns: the governed
//! component schema versions, the routing policy, the derived required-feature
//! set, the three reached semantic subjects, the ordered named interface, the
//! selected capability providers, the backend payload descriptors, the shared
//! ABI expression arena, and each plan variant with its guard, declared target
//! profile and feasibility rule set, deferred predicates, and executable
//! entries — each entry's stage subject, proven resource requirements, declared
//! numerical realization, ABI bindings with the interface reference each
//! addresses, launch contract, and backend entry. It
//! also carries the artifact's canonical identity once, which the decoder
//! re-derives from the content and compares.
//!
//! # What it deliberately excludes
//!
//! **The frozen registry snapshot.** ADR 0072 keeps the provenance of providers
//! a plan never used out of packaged artifact identity. Carrying it here would
//! put it back into the envelope's bytes and therefore into its digest, so an
//! unused provider could invalidate a cache entry. Only the three reached
//! subjects travel.
//!
//! **Presentation-only declaration order.** Providers, payloads, deferred
//! predicates, launch preconditions, entries, expression arena nodes, and
//! sections are all written in the canonical content order artifact identity
//! already uses. Two artifacts with equal identity therefore encode to equal
//! bytes, which is what makes an envelope digest usable as a cache key. Variant
//! order, interface order, and ABI binding order are meaning and are retained.
//!
//! **Backend payload bytes.** A payload is named by governed backend and
//! representation keys, its own schema version, its opaque content digest, and
//! its execution policy — exactly as the artifact model names it. Whether a
//! bundle's identity is content-addressed over its compilation inputs or over
//! the emitted payload bytes is `prototype-metal-bundle-assembly`'s decision,
//! and this codec does not pre-empt it. The section machinery it will need
//! exists and is exercised; the governed section purposes it will add are its
//! own versioned extension.
//!
//! **A reconstructable kernel program.** A section carries one packaged
//! variant's *canonical kernel-program identity*, not the program. A decoder
//! cannot rebuild a `VerifiedKernelProgram`: `KernelProgramBuilder::new` needs a
//! `SemanticProgram`, which needs a frozen registry holding live inferencer
//! implementations, and neither is representable as bytes. The consequence is
//! stated rather than approximated — a decoded envelope proves *which* program
//! an artifact names and cannot resurrect it, and the stage execution order of
//! a multi-stage program is not recoverable, which is why an envelope that
//! needs one declares a required feature this reader refuses.
//!
//! # Lockstep
//!
//! The reader supports exactly the versions and features this build writes. A
//! major mismatch, a minor beyond what this build implements, an unrecognized
//! digest algorithm, or a required feature this build cannot supply is a typed
//! rejection, never a best-effort read.

mod budget;
mod decode;
mod digest;
mod encode;
mod error;
mod model;
mod payload;
mod validate;
mod view;

// Only the identity encoder in `super::model` and the builder's terminal reach
// into this module today; everything else stays behind its own module path so
// the crate-private surface stays exactly as wide as its use.
pub(crate) use model::{
    ArtifactEnvelope, EntryRow, NumericalFacts, VariantRow, expression_keys, position,
};
// Re-exported one level up by `super` for `crate::proof`, which needs the
// governed digest algorithm and the external envelope digest and nothing else
// from this module. The sidecar is deliberately not artifact semantics, so it
// owns its own framing, schema, vocabulary, and limits; what it must *not* own
// is a second answer to "which hash function", because `docs/artifact-abi.md`
// requires every digest use to name one governed algorithm, and a sidecar that
// digested under a different one would be unverifiable by a reader that knows
// only the governed tag.
pub(crate) use digest::{DIGEST_BYTES, Digest, DigestAlgorithm};
pub(crate) use encode::envelope_digest;
#[cfg(test)]
pub(crate) use encode::{ENVELOPE_DIGEST_DOMAIN, MANIFEST_DIGEST_DOMAIN, SECTION_DIGEST_DOMAIN};
// The carried-payload vocabulary is the one part of this module that is
// public. A backend assembler outside this crate must be able to describe what
// it compiled, and nothing else here is reachable: the envelope, the encoder,
// the decoder, the rejection vocabulary, and the governed constants all stay
// `pub(crate)` behind this private module under ADR 0074 convention 7.
//
// Promoted on Tom's review, 2026-07-25.
pub use payload::{
    PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadProvenance, PayloadSdkIdentity,
    PayloadTargetObligation, ToolComponent,
};
// The codec's *capability* — encode an artifact, decode bytes back — is public
// as of `carry-the-metal-payload-in-an-artifact-envelope`, so an out-of-crate
// assembler can prove what it packaged survives a round trip. The envelope, the
// encoder, the decoder, and the section vocabulary all stay `pub(crate)`:
// `view` exposes accessors over them rather than the types themselves.
//
// Promoted on Tom's review, 2026-07-25.
// The dispatch-record projection landed with
// `expose-the-dispatch-record-on-a-decoded-artifact`, which implements Tom's
// decision that a decoded envelope *is* a dispatch record. These are accessors
// over rows the decoder already validated, plus one encoded fact the record
// needed and did not have — see `view`'s module documentation for which facts a
// decoder re-derives and which it takes on the producer's derivation.
pub use view::{
    ArtifactCodecFailure, DecodedArtifact, DecodedBinding, DecodedDeferredPredicate, DecodedEntry,
    DecodedExpr, DecodedInput, DecodedNumerical, DecodedOutput, DecodedVariant, SectionPurpose,
    SectionView, decode_artifact,
};

#[cfg(test)]
mod tests;
