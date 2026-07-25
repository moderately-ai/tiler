---
id: carry-the-metal-payload-in-an-artifact-envelope
title: Carry the Metal payload in an artifact envelope and round-trip it
status: in-progress
priority: p0
dependencies: [assemble-the-metal-payload-from-emission-and-compilation]
related: [route-the-runtime-proof-through-the-artifact-envelope, prototype-public-compiler-api]
scopes: [implementation/metal-aot, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
claimed_from: todo
assignee: agent-coordinator
lease_expires_at: 1784988717
---
`assemble-the-metal-payload-from-emission-and-compilation` produced a real `PayloadContent` from a real emission and compilation, and stopped there. This ticket carries it into an envelope and proves the envelope round-trips.

**Fact — why the parent could not finish it.** `ArtifactProgramBuilder` needs, per variant, the `VerifiedKernelProgram` the plan packages, its selected capability providers, its ABI expression arena, its guard, its per-entry bindings and launch contract, and its declared target profile and feasibility rule set. Every one of those reaches a caller only through `ProgramAlternative::artifact_plan`, which is `pub(crate)` in `tiler-compiler` and is not exposed on the `session` boundary. `PlanAlternative` today exposes `stable_id`, `is_fused`, and `kernels` — enough to emit, not enough to package.

**The surface decision this ticket cannot avoid.** Exposing the artifact plan is an ADR 0075 promotion and is Tom's. It is also a genuine design choice rather than a mechanical one: the plan is a large internal record, and handing it out whole would commit the public boundary to its exact shape while the compiler is still moving. The alternative is a narrower seam — `PlanAlternative::build_artifact(&mut ArtifactProgramBuilder)` or similar — that lets the compiler populate a builder the caller owns, exposing the *capability* without the record. Put both to Tom with the trade-off before implementing either.

## The work, once that is settled

Push the carried payload through `push_carried_payload`, build the artifact, encode the envelope, and then **decode the bytes back** and verify. The round trip is the point: identity re-derived from decoded content must equal the identity the manifest carries, the re-encode must be byte-identical, and every closure check must pass. A payload that assembles but does not survive a round trip is not carried.

Assert specifically that the two payload sections appear with the right purposes — `BackendPayloadMetadata` and `BackendPayloadCode` — that the descriptor digest equals `payload_identity` of the metadata bytes, and that the derived feature set contains `tiler.artifact.feature.embedded-payload-code`. Those are the properties the codec already tests against synthetic content; this is where they meet a real compilation.

## Do not

Do not weaken any codec check to make a real artifact fit. If a real bundle is refused, that is either a defect in the assembly or a gap in the model, and both are worth finding — the codec's adversarial cases exist precisely so a well-formed-looking forgery cannot pass, and a real payload should not need an exception.

## Decision — the orchestrator assembles; the compiler exposes a typed view of its construction inputs

Two options were weighed and one was **recommended and then withdrawn**, so the reasoning is recorded rather than the conclusion alone.

**Withdrawn: `PlanAlternative::build_artifact(&mut ArtifactProgramBuilder, …)`**, letting the compiler populate a builder the caller owns. It was recommended first on the grounds that it "matches how this codebase works — checked builders that populate rather than records handed out." That was taste, not evidence, and two facts defeat it.

**Fact — `crates/tiler-compiler/Cargo.toml` declares `tiler-ir` as its only dependency.** So this is not a visibility change at all; it is a **new dependency edge** `tiler-compiler → tiler-artifact`, and a durable one. Edges are far harder to remove than functions: once it exists every artifact concept is reachable from the compiler and later work will reach for it.

**Fact — `docs/artifact-abi.md` records the division the other way**: the private compiler proof "constructs provisional program portfolios and artifact-**construction inputs**". The type is named `ArtifactConstructionPlan`. Inputs, not artifacts. Making the compiler own packaging contradicts an accepted contract.

**Decision: expose a typed read view over the construction inputs and assemble in the orchestrator**, which is the only component that sees both the plan and the backend payload — structurally the same position as the emitter/driver target-vocabulary translation already in `prototypes/serial-sum-compile/src/target.rs`.

**Correctness is what settles it, not preference.** `ArtifactConstructionPlan` carries no ABI expressions: `routing_guard: HostExprId` is an index into the *compiler's* arena and there is no expression arena, binding, or launch contract in it. So whoever assembles must derive accessible byte ranges and launch geometry. Under the withdrawn option that derivation would live in `tiler-compiler` and mint expressions into `tiler-artifact`'s arena, creating a **second** place that knows how to derive an accessible byte range beside the artifact layer's own checks. Two derivations of one fact is the drift hazard, and it is the exact class of defect this session already hit twice — the hand-written mirrors in `governed_scalar_reference.rs`, and a `tiler-ir` doc comment that contradicted `docs/numerical-semantics.md`.

**The view must not leak compiler-internal identifiers.** A caller holding a raw `HostExprId` would be re-deriving cross-references the artifact builder already owns, which is what that layer exists to prevent. The exposed view hands out resolved values; the caller mints its own expressions on its own builder.

**Performance does not decide it.** Assembly runs once per compilation, not per element; a view copy is noise against `xcrun`.

**Fact — no artifact assembler exists anywhere.** `ArtifactProgramBuilder` has no consumer outside `crates/tiler-artifact/src`. So this ticket writes the first one, and the ABI-expression derivation it needs is the same work `complete-program-identity-with-abi-guards-and-routing` describes; the two meet here and should not produce two derivations.

**Sizing is an argument, not a measurement, until the gap between what `push_variant` requires and what the compiler can supply is read field by field.** That reading is in progress; do not commit to an approach in code before it lands.
