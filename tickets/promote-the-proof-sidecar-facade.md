---
id: promote-the-proof-sidecar-facade
title: Promote the proof-sidecar facade
status: todo
priority: p0
dependencies: [prototype-proof-case-sidecar]
related: [prototype-metal-aot-slice, prototype-metal-runtime-proof]
scopes: [implementation/artifact, contracts/artifacts]
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

## Also closes the contract gap the sidecar deliberately left open

**Fact — no governed contract records the sidecar's format.** `docs/artifact-abi.md` should not be the one: the sidecar is not artifact semantics, and its "The governed digest" section correctly describes the envelope's three domain separators while the sidecar adds four of its own (`tiler.proof-sidecar.manifest.v1`, `…manifest-digest.v1`, `…payload-digest.v1`, `…identity.v1`). Writing the format down while nothing outside `tiler-artifact` can reach it would document an unreachable format, which is why `prototype-proof-case-sidecar` recorded the gap with this promotion as its trigger rather than filing a ticket that would sit ready and unactionable.

**Two things the contract must state, whichever document takes it.** That the sidecar is producer evidence and never artifact semantics — a sidecar names an artifact, an artifact never names a sidecar, and an artifact validates and dispatches with none present. And the measurement boundary the implementation already pins as a test: a validated sidecar is evidence of *integrity and association* and not of *authenticity*, because every digest and identity in it is derived from its own content, so a re-sealed forgery validates and binds. A consumer that treats sidecar payloads as anything but test data has read a guarantee the container does not make.

**Fact — one union property crosses the two containers and is checked in only one place.** `crate::proof::tests::no_governed_domain_of_either_container_prefixes_another` checks all seven governed domains together, because one algorithm hashes both containers in one process; the envelope codec's own three-domain test carries no note pointing at it. Whoever takes this ticket should decide whether that cross-reference belongs in the codec's test or in the contract.

## Decision — Tom, 2026-07-25

**Approved: promote the proof-sidecar facade.** The container is crate-private today and the producer and runner are different crates by construction, so a public facade is the only shape that lets a case written by one be verified by the other. This unblocks `prototype-metal-aot-slice`, which was otherwise dispatchable straight into an unreachable API.

The container's stated limit does not change with promotion: authenticity is not claimed, a re-sealed forgery validates and binds, and `a_forged_case_is_indistinguishable_from_a_real_one_by_the_container_alone` exists so a reader cannot infer a stronger guarantee from a public API than the private one offered.
