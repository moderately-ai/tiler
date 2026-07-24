---
id: carry-a-compatibility-contract-reference-on-the-payload-descriptor
title: Carry a compatibility-contract reference on the backend payload descriptor
status: todo
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec, prototype-metal-bundle-assembly, record-the-implemented-artifact-envelope-in-the-contract]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization]
---
**Fact — the contract lists five things that name a backend payload and the model carries four of them.** `docs/artifact-abi.md`: "The neutral layer references a backend payload through a governed backend key, representation key, payload digest, compatibility-contract reference, and an opaque backend entry key." `BackendPayloadDescriptor` in `crates/tiler-artifact/src/program/model.rs` carries the backend key, the representation key, its own payload schema version, the content digest, and an execution policy. There is no compatibility-contract reference.

**Fact — the declared target contract is carried one level away, on the plan variant.** `VariantRow` carries `TargetProfileRef` (key plus descriptor digest) and `FeasibilityRuleSetRef` (key plus revision). Those are the *plan's* declared target requirements, not the *payload's* compatibility contract.

**Inference — the two coincide only while a payload is not shared.** `MAX_ARTIFACT_PAYLOADS` is 16 and entries cross-reference payloads by index, so nothing in the model stops two variants that declare different target profiles from realizing their entries through one payload. When they do, no field in the envelope states which compatibility contract the payload's own bytes were built against, and the envelope research's rule — "a backend payload is compatible only when the installed backend provider supports its backend key, payload schema, representation, compatibility contract, and execution/translation policy" — has nothing to evaluate. A loader would have to infer the payload's contract from whichever variant it happened to route to, which is the kind of inference this layer exists to forbid.

**What closes this.** Either add the compatibility-contract reference to `BackendPayloadDescriptor` and fold it into the payload canonical key and artifact identity, or decide that a payload is per-variant by construction and state that invariant with the check that enforces it. The first is the contract as written; the second is a narrower model that must be argued rather than assumed. Whichever is chosen, record it in `docs/artifact-abi.md` beside the sentence above.

**Scope note.** Deliberately left unscoped: the fix touches `implementation/artifact` and `contracts/artifacts`. It is adjacent to `prototype-metal-bundle-assembly` but distinct from it — this is the neutral descriptor's field set, not the Metal payload's schema or its identity basis, and it must not pre-empt that ticket's decision on either.
