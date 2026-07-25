---
id: carry-the-metal-payload-in-an-artifact-envelope
title: Carry the Metal payload in an artifact envelope and round-trip it
status: in-progress
priority: p0
dependencies: [assemble-the-metal-payload-from-emission-and-compilation, relocate-abi-expressions-into-tiler-ir]
related: [route-the-runtime-proof-through-the-artifact-envelope, prototype-public-compiler-api]
scopes: [implementation/metal-aot, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, artifact, aot]
claimed_from: todo
assignee: agent-coordinator
lease_expires_at: 1784991074
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

## The decision above is RETRACTED — accepted ADR 0068 assigns this to the compiler

The reading landed and it overturns the decision, which is recorded rather than quietly replaced.

**Fact — `docs/decisions/0068-co-locate-abi-expressions-with-executable-program-ir.md:41-42`, `decision_status: accepted`:** "`tiler-compiler` owns lowering into and construction of `AbiExpr`; it is not the runtime expression authority."

That is directly on point and more specific than the `docs/artifact-abi.md` "construction inputs" phrasing the retracted decision rested on. The compiler is *supposed* to construct ABI expressions. So the withdrawn option was right and the recorded reasoning against it was wrong.

**But it cannot be done today, and the reason is a second divergence.** The same ADR decides: "Place the public `AbiExpr` domain type, admitted roots, validation, canonical identity, and authoritative pure checked evaluation semantics ... in `tiler-ir`." **It is not there.** `AbiRoot` is defined at `crates/tiler-artifact/src/program/expr.rs:159`, and `crates/tiler-ir/src/program/` contains no ABI expression module. The ADR's `implementation_status` is `spike-only`, which is consistent — it is accepted and unimplemented.

**Inference — the whole difficulty is downstream of that one divergence.** With `AbiExpr` where the ADR puts it, `tiler-compiler` constructs ABI expressions over `tiler-ir`, which it already depends on, and **no new dependency edge is needed at all**. The dilemma that produced the retracted decision — expose a plan record, or add `tiler-compiler → tiler-artifact` — exists only because the type is in the wrong crate. Neither option is right; the ordering is.

## Measured gap, for whoever sequences this

**What `push_variant` requires** (`crates/tiler-artifact/src/program/builder.rs:437`), taking `&VerifiedKernelProgram`: a boolean `applicability_guard`, `TargetProfileRef`, `FeasibilityRuleSetRef`, deferred predicates, and one `EntrySpec` per program stage — each with one `BindingSpec` per kernel buffer parameter (`accessible_bytes`, which must statically equal `access.view().window().length`) and a `LaunchSpec` (`grid_threads`, `threads_per_workgroup` which must statically equal `stage.kernel().requirements().threads_per_workgroup`, `zero_work_skips_dispatch`, preconditions).

**What a caller outside `tiler-compiler` can obtain today:** `PlanAlternative::kernels() -> &[VerifiedKernel]` and `Compilation::target_profile_key() -> &str`. That is the entire list. `ArtifactConstructionPlan` is `pub(crate)` in a private `mod program` with **every field private and exactly one accessor** (`lowering_providers`, `program.rs:183`). `KernelProgram::core()` — the only path to a `VerifiedKernelProgram` — is `#[cfg(test)]` (`program.rs:164`), so **no non-test path to the `push_variant` argument exists at all**.

**`grid_threads` is not on a kernel.** `VerifiedKernel::requirements()` gives `threads_per_workgroup`; total grid size lives only on `ScheduledRegion::schedule.launch.grid_threads`, behind the `pub(crate)` compiler wrapper. This is why `prototypes/serial-sum-run/src/main.rs` hardcodes its dispatch geometry and buffer indices instead of deriving them.

**No ABI derivation exists anywhere.** The only constructions of `accessible_bytes` and launch geometry as expressions are hand-written fixtures: `crates/tiler-artifact/src/program/tests.rs:321` `formulas()` and `:358` `entry()`, cloned in the `program/mod.rs:243` doctest and once more in `codec/tests.rs:278`. All assume the two-dimensional serial-sum shape, a hardcoded 4-byte element, and `grid_threads = rows`. The artifact layer *checks* these expressions and never mints one; it does not check `grid_threads` against anything at all.

The scalar truths to lift into expression form already exist and should not be re-derived: grid threads = iteration-shape product (`crates/tiler-compiler/src/physical.rs:424`, invariant at `crates/tiler-ir/src/schedule/builder.rs:286`), bytes = elements x element width (`crates/tiler-compiler/src/program.rs:490`), window length = required bytes (`crates/tiler-ir/src/program/builder.rs:276`).

**Blocked on `relocate-abi-expressions-into-tiler-ir`.** Building either option now would build on a divergence from an accepted decision.
