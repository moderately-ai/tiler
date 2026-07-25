---
id: carry-the-metal-payload-in-an-artifact-envelope
title: Carry the Metal payload in an artifact envelope and round-trip it
status: todo
priority: p0
dependencies: [assemble-the-metal-payload-from-emission-and-compilation]
related: [route-the-runtime-proof-through-the-artifact-envelope, prototype-public-compiler-api]
scopes: [implementation/metal-aot, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
---
`assemble-the-metal-payload-from-emission-and-compilation` produced a real `PayloadContent` from a real emission and compilation, and stopped there. This ticket carries it into an envelope and proves the envelope round-trips.

**Fact — why the parent could not finish it.** `ArtifactProgramBuilder` needs, per variant, the `VerifiedKernelProgram` the plan packages, its selected capability providers, its ABI expression arena, its guard, its per-entry bindings and launch contract, and its declared target profile and feasibility rule set. Every one of those reaches a caller only through `ProgramAlternative::artifact_plan`, which is `pub(crate)` in `tiler-compiler` and is not exposed on the `session` boundary. `PlanAlternative` today exposes `stable_id`, `is_fused`, and `kernels` — enough to emit, not enough to package.

**The surface decision this ticket cannot avoid.** Exposing the artifact plan is an ADR 0075 promotion and is Tom's. It is also a genuine design choice rather than a mechanical one: the plan is a large internal record, and handing it out whole would commit the public boundary to its exact shape while the compiler is still moving. The alternative is a narrower seam — `PlanAlternative::build_artifact(&mut ArtifactProgramBuilder)` or similar — that lets the compiler populate a builder the caller owns, exposing the *capability* without the record. Put both to Tom with the trade-off before implementing either.

## The work, once that is settled

Push the carried payload through `push_carried_payload`, build the artifact, encode the envelope, and then **decode the bytes back** and verify. The round trip is the point: identity re-derived from decoded content must equal the identity the manifest carries, the re-encode must be byte-identical, and every closure check must pass. A payload that assembles but does not survive a round trip is not carried.

Assert specifically that the two payload sections appear with the right purposes — `BackendPayloadMetadata` and `BackendPayloadCode` — that the descriptor digest equals `payload_identity` of the metadata bytes, and that the derived feature set contains `tiler.artifact.feature.embedded-payload-code`. Those are the properties the codec already tests against synthetic content; this is where they meet a real compilation.

## Do not

Do not weaken any codec check to make a real artifact fit. If a real bundle is refused, that is either a defect in the assembly or a gap in the model, and both are worth finding — the codec's adversarial cases exist precisely so a well-formed-looking forgery cannot pass, and a real payload should not need an exception.
