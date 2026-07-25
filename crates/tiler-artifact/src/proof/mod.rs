#![allow(
    dead_code,
    unused_imports,
    reason = "the proof-case evidence sidecar is a crate-private draft authority (ADR 0074 convention 7). It reserves the sidecar's framing, its canonical manifest, the case vocabulary, the three recorded provenance subjects, and the two envelope-association checks a consumer runs before it treats any case as data. Its two intended consumers are out-of-crate binaries — the producer `prototypes/serial-sum-compile` under `prototype-metal-aot-slice`, and the runner `prototypes/serial-sum-run` under `prototype-metal-runtime-proof` — and neither can reach a crate-private module. `unused_imports` covers the same reservation one level up: the re-exports below name the surface a reviewer reads as a whole, and until a consumer exists none of them has a non-test user. Promoting the surface to `pub` is Tom's call under ADR 0075 and has not been made."
)]

//! The separate, versioned proof-case evidence sidecar.
//!
//! A producer that compiles an artifact also knows what the artifact is
//! *supposed to compute*, because it can evaluate the same semantic program
//! through the target-independent reference evaluator. This module is the
//! bounded container that carries that knowledge beside an artifact: stable
//! case keys, bit-preserving input bytes, normative expected output bytes, the
//! identities of the three authorities that make the expectation meaningful,
//! content digests over every payload, and an exact association with the one
//! envelope the cases are about.
//!
//! # This is not artifact semantics, and the separation is structural
//!
//! `tiler.research.workspace.prototype-crate-layout-and-msrv` records the rule
//! this module implements: "The sidecar is not part of artifact semantics."
//! Nothing in [`crate::program`] references this module, no envelope section
//! carries a proof case, and an artifact decodes, validates, and dispatches
//! with no sidecar present. The dependency runs one way — a sidecar names an
//! artifact, an artifact never names a sidecar — which is what makes proof data
//! deletable without changing what a program means.
//!
//! The separation is also why the two containers share exactly two things and
//! no more: the governed digest algorithm, which `docs/artifact-abi.md`
//! requires every digest use in this crate to name explicitly rather than
//! choose locally, and the envelope digest function, which *is* the
//! association. Framing, schema, vocabulary, limits, and failure classification
//! are this module's own.
//!
//! # What a consumer may conclude, and what it may not
//!
//! A validated sidecar is evidence of **integrity and association**: these are
//! the exact bytes a producer wrote, and they name exactly one artifact.
//!
//! It is not evidence of **authenticity**. Every digest and identity in the
//! container is derived from the container's own content, so a forger that
//! rewrites a case recomputes them all and the result validates. What protects
//! a proof run is that the expected bytes are compared against a device
//! readback: a forged expectation makes a correct device fail the comparison,
//! which is a loud result rather than a silent one. A consumer must therefore
//! treat sidecar payloads as *test data* — the runtime-proof ticket says
//! exactly this — and never as a semantic authority, a fallback value, or an
//! input to routing.
//!
//! # The two association strengths
//!
//! [`DecodedProofSidecar::bind_to_envelope`] is available to a consumer that
//! holds only bytes. It re-derives the envelope digest over the exact bytes
//! supplied, decodes them, and compares the re-derived artifact identity with
//! the one the sidecar recorded.
//!
//! [`DecodedProofSidecar::bind_to_artifact`] is available to a consumer that
//! holds the verified artifact program. It additionally re-proves every
//! structural obligation the builder proved — that the sidecar binds exactly
//! the artifact's declared inputs and outputs in the artifact's own interface
//! order, that every case's payload length is a whole number of elements of the
//! declared shape, and that all cases agree on each interface entry's length.
//!
//! The weaker check is not a weaker *association*: both prove the same artifact
//! identity, and the artifact identity already folds the ordered named
//! interface. The difference is that the stronger one re-proves the obligations
//! locally instead of inheriting them through an identity comparison, which is
//! what a consumer wants when the sidecar was written by an older producer.
//!
//! # Limits
//!
//! Every bound below is checked before any allocation proportional to it, in
//! both directions: the encoder refuses to write a container a reader would
//! not admit, and the reader refuses a declared count before reserving for it.

mod builder;
mod codec;
mod model;

pub(crate) use builder::{
    ProofBuildError, ProofCaseSpec, ProofDirection, ProofInterfaceError, ProofProvenance,
    ProofSidecarBuilder,
};
pub(crate) use codec::{
    DecodedProofSidecar, ProofAssociationError, ProofCodecError, ProofFailureClass,
    ProofLimitExceeded, ProofLimitKind, ProofOrderedSubject, decode_proof_sidecar,
};
pub(crate) use model::{
    CanonicalProofSidecarIdentity, ProofCaseKey, ProofCaseKeyError, ProofCaseRef,
    ProofNumericalIdentity, ProofPayloadRef, ProofReferenceIdentity, ProofSemanticSubject,
    ProofSubjectError, VerifiedProofSidecar,
};

/// Maximum proof cases admitted by one sidecar.
pub(crate) const MAX_PROOF_CASES: usize = 256;
/// Maximum UTF-8 byte length of one stable proof-case key.
pub(crate) const MAX_PROOF_CASE_KEY_BYTES: usize = 256;
/// Maximum named interface entries one sidecar binds per direction.
///
/// Deliberately equal to the artifact model's own interface bound: a sidecar
/// binds one payload per declared entry, so a looser bound here would admit a
/// container no artifact could ever associate with.
pub(crate) const MAX_PROOF_INTERFACE_ENTRIES: usize = 4_096;
/// Maximum bytes of one case payload — one input or one expected output.
pub(crate) const MAX_PROOF_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
/// Maximum bytes of one received opaque provenance subject.
pub(crate) const MAX_PROOF_SUBJECT_BYTES: usize = 1_024;
/// Maximum bytes of the sidecar's canonical manifest.
pub(crate) const MAX_PROOF_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
/// Maximum bytes of one complete encoded sidecar.
pub(crate) const MAX_PROOF_SIDECAR_BYTES: usize = 256 * 1024 * 1024;
/// Maximum bytes of the derived canonical sidecar identity.
pub(crate) const MAX_PROOF_IDENTITY_BYTES: usize = 8 * 1024 * 1024;

#[cfg(test)]
mod tests;
