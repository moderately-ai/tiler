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
//! set, the reached semantic subjects including the retained shape environment,
//! the ordered named interface, the selected lowering-capability providers, the
//! backend payload descriptors, the shared ABI expression arena, each plan
//! variant with its guard, declared target profile and feasibility rule set,
//! selected physical-implementation run, deferred predicates, and executable
//! entries — each entry's stage subject, proven resource requirements, declared
//! numerical realization, ABI bindings with the interface reference each
//! addresses, launch contract, and backend entry — and the delivered-realization
//! record. It also carries the *digest* of the artifact's canonical identity
//! once, which the decoder re-derives from the content and compares.
//!
//! # What it deliberately excludes
//!
//! **The frozen registry snapshot.** ADR 0072 keeps the provenance of providers
//! a plan never used out of packaged artifact identity. Carrying it here would
//! put it back into the envelope's bytes and therefore into its digest, so an
//! unused provider could invalidate a cache entry. The three reached semantic
//! subjects travel, and so does the fifth subject's lossless retained
//! environment. The snapshot stays out.
//!
//! **Presentation-only declaration order.** Providers, payloads, deferred
//! predicates, launch preconditions, entries, expression arena nodes, and
//! sections are all written in the canonical content order artifact identity
//! already uses. Two artifacts with equal identity therefore encode to equal
//! bytes, which is what makes an envelope digest usable as a cache key. Variant
//! order, interface order, and ABI binding order are meaning and are retained.
//!
//! **Backend payload semantics.** A carried payload includes canonical metadata and opaque code bytes in governed sections. Artifact identity covers the compilation subject recorded by the metadata and deliberately excludes the emitted object's bytes; section and envelope digests bind the exact transported bytes for integrity. The neutral codec validates that structure without parsing backend code.
//!
//! **A reconstructable kernel program.** The manifest carries one packaged variant's canonical kernel-program identity and its validated dispatch record, including ordered stages and dependencies; it does not serialize a `VerifiedKernelProgram`. The decoded record gives a runtime what it needs to route and execute the carried entries without optimizer or semantic-registry internals, including a multi-stage program, but cannot resurrect the compiler-side verified program object.
//!
//! # Lockstep
//!
//! The reader supports exactly the versions and features this build writes. A
//! major mismatch, a minor beyond what this build implements, an unrecognized
//! digest algorithm, or a required feature this build cannot supply is a typed
//! rejection, never a best-effort read.

mod budget;
mod decode;
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
    ArtifactEnvelope, EntryRow, NumericalFacts, VariantRow, canonical_entry_positions, position,
};
// The governed digest is re-exported rather than owned, for the reason
// `tiler-digest` states: `docs/artifact-abi.md` requires every digest use to
// name one governed algorithm, and a component that hashed under a different one
// would be a second identity authority over the same subject. It lived here
// until ADR 0104 needed it in `tiler-ir`, which sits below this crate and cannot
// be moved above it; the algorithm moved to the workspace's bottom crate and
// this path keeps resolving. `crate::proof` reaches it here, and so does
// `tiler-cache` — the expansion cache must validate the section digests of a
// stored bundle on every hit, and ADR 0050's whole argument rests on that digest
// being *the* governed one.
pub use tiler_digest::{DIGEST_BYTES, Digest, DigestAlgorithm};
// The external envelope digest stays crate-private: it names one published
// encoding of an artifact, which only `crate::proof`'s sidecar association
// and the typed `ArtifactEnvelopeDigest` mints need.
pub(crate) use encode::envelope_digest;

/// Derives the governed section digest one object would carry as payload code.
///
/// Crate-visible so a payload plan-determinism receipt can bind the exact
/// emitted object bytes under the same digest the envelope's section table
/// carries for them, rather than under a second, drifting association.
pub(crate) fn payload_code_section_digest(object_bytes: &[u8]) -> Digest {
    let kind = model::SectionKind::BackendPayloadCode;
    let schema = kind.schema();
    DigestAlgorithm::GOVERNED.digest_qualified(
        encode::SECTION_DIGEST_DOMAIN,
        &[
            &[kind.tag()],
            &schema.major().to_be_bytes(),
            &schema.minor().to_be_bytes(),
        ],
        object_bytes,
    )
}
// Every governed domain this container admits, reachable under test so
// `crate::domains` can enumerate the crate's whole set in one place. The two
// framing tags are here for the same reason the digest domains are: each opens a
// canonical byte run that is digested or compared, so a prefix relation involving
// one merges subjects exactly as a colliding digest domain would.
#[cfg(test)]
pub(crate) use encode::{
    ENVELOPE_DIGEST_DOMAIN, IDENTITY_DIGEST_DOMAIN, MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN,
    SECTION_DIGEST_DOMAIN,
};
#[cfg(test)]
pub(crate) use payload::{PAYLOAD_IDENTITY_DOMAIN, PAYLOAD_METADATA_DOMAIN};
// The carried-payload vocabulary is the one part of this module that is
// public. A backend assembler outside this crate must be able to describe what
// it compiled, and nothing else here is reachable: the envelope, the encoder,
// the decoder, the rejection vocabulary, and the governed constants all stay
// `pub(crate)` behind this private module under ADR 0074 convention 7.
//
// Promoted on Tom's review, 2026-07-25.
pub use payload::{
    PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadPlatform, PayloadProvenance,
    PayloadSdkIdentity, PayloadTargetObligation, ToolComponent,
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
    ArtifactCodecFailure, DecodedArtifact, DecodedBinding, DecodedComponent,
    DecodedDeferredPredicate, DecodedEntry, DecodedExpr, DecodedExtentOperand, DecodedInput,
    DecodedNumerical, DecodedOutput, DecodedStageDependency, DecodedVariant, SectionPurpose,
    SectionView, decode_artifact,
};

#[cfg(test)]
mod tests;
