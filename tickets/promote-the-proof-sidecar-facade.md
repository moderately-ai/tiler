---
id: promote-the-proof-sidecar-facade
title: Promote the proof-sidecar facade
status: todo
priority: p0
dependencies: [prototype-proof-case-sidecar]
related: [prototype-metal-aot-slice, prototype-metal-runtime-proof]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, api, review]
---
The proof-case evidence sidecar landed as a crate-private draft authority in `crates/tiler-artifact/src/proof/` under ADR 0074 convention 7. Its two intended consumers are out-of-crate binaries — the producer `prototypes/serial-sum-compile` and the runner `prototypes/serial-sum-run` — and neither can reach a crate-private module, so the capability exists and is unreachable until its facade is accepted.

**Fact — this is ADR 0075's always-ask category, twice over.** Making the sidecar reachable means a new `pub mod proof` in the `tiler-artifact` crate root, which is "a new publicly reachable namespace", and promoting items from `pub(crate)` to `pub`, which is its own always-ask item. Neither is the coordinator's to merge. The precedent is exact: `prototype-neutral-artifact-codec` landed the envelope codec crate-private, and `carry-the-metal-payload-in-an-artifact-envelope` promoted `decode_artifact`, `DecodedArtifact`, and the payload vocabulary on Tom's review on 2026-07-25.

**Fact — the surface to review is bounded and already written.** `crates/tiler-artifact/src/proof/mod.rs` re-exports it in one place: the builder (`ProofSidecarBuilder`, `ProofCaseSpec`, `ProofProvenance`), the product and its read views (`VerifiedProofSidecar`, `ProofCaseRef`, `ProofPayloadRef`, `CanonicalProofSidecarIdentity`), the vocabulary (`ProofCaseKey`, the three provenance subjects), the reader (`decode_proof_sidecar`, `DecodedProofSidecar`), and the four error enums plus their classification. Every item conforms to ADR 0074 conventions 1 through 6 and is tested; what remains is the interface decision, not the implementation.

**Two questions worth putting to Tom with the diff**, rather than deciding here:

- Whether the whole surface is promoted, or only the *capability* the way the envelope codec's was — bytes out, a validated view back, accessors rather than types — leaving the builder crate-private until a producer needs it. The envelope codec's promotion took the narrower shape deliberately.
- Whether `DecodedProofSidecar::bind_to_artifact` is promoted at all. It is the stronger check and it requires the caller to hold a `VerifiedArtifactProgram`, which is exactly the binding-by-identity posture that `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is deciding. Promoting it commits a second public call site to that posture before that ticket closes.

**Not closed by** widening visibility without review. A crate-private authority that is unreachable is the accepted staging state (ADR 0074 convention 7), not a defect to be worked around.
