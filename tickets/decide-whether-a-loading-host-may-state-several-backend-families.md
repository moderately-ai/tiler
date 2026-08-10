---
id: decide-whether-a-loading-host-may-state-several-backend-families
title: Decide whether a loading host may state several backend families
status: awaiting-decision
priority: p1
dependencies: []
related: [expose-explicit-backend-provider-and-selection-policy-composition, select-executable-variants-across-registered-backend-families, route-a-custom-backend-through-an-independently-selected-adapter, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, express-the-typed-backend-family-selection-policy]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, public-boundary, runtime, decision, needs-tom]
---
## User-visible outcome

Tom fixes which layer owns a consumer's "I will run Metal **or** CPU" statement, so the typed backend-family selection policy [`expose-explicit-backend-provider-and-selection-policy-composition`](expose-explicit-backend-provider-and-selection-policy-composition.md) owes can be built against an accepted host model instead of inventing one.

## Why this is a decision rather than research

**Fact — a loading host states exactly one backend family, and a family policy therefore has nothing to range over.** `ExecutionEnvironment` (`crates/tiler-runtime/src/load/host.rs`) carries one `target_profile: TargetProfileRef`, one `backend: BackendKey`, one `representation: RepresentationKey`, and one `dtype_dispatch` map. `DecodedProgram::variant_eligibility` compares the backend and representation *as a pair* against those single fields and classifies the profile against that single profile, so every variant of any other family is filtered before its guard.

**Fact — the multi-family portfolio exists on the producing side and is already exercised.** [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) landed `assemble_portfolio`, and its own Outcome records that the Metal member's "carried object is this fixture's own scalar image and never decoded, because a variant of another family is filtered before its guard". The portfolio is packaged and filtered; nothing can yet *prefer* among two families it can both run.

**Inference — so a policy built on today's host model would be a guard that cannot fail.** With one stated family, `allowed {tiler.metal, tiler.cpu.scalar}` and `required {tiler.metal}` and `fallback-only {tiler.cpu.scalar}` all select exactly what the host already stated. A typed vocabulary that cannot change an outcome is worse than none, because it reads as coverage.

**Fact — the graph already intends the capability.** [`exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`](exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md) depends on the composition ticket, [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md), and [`join-build-time-producers-to-runtime-adapters-through-artifact-identity`](join-build-time-producers-to-runtime-adapters-through-artifact-identity.md), and requires testing "standard-Metal-only, custom-Metal-preferred, CPU-only, Metal-or-CPU, missing-adapter, incompatible-profile, and no-valid-route policies". Metal-or-CPU is not expressible under either option below without this decision.

## The two surviving positions

Both are correctness-capable and neither is eliminated by an accepted record, which is why this is Tom's rather than a worker's. [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4's single-environment sentence is a **Fact about the tree**, not a normative clause, so neither option contradicts it; item 13's initial device profile constrains *devices*, and a Metal-or-CPU consumer still commits to one route and one live context.

**Option A — the loader takes a set.** `DecodedProgram` routing accepts an ordered, family-unique set of `ExecutionEnvironment` plus a typed policy, and discloses which environment won so the consumer can pick the matching adapter. *Enables:* one refusal that sees the whole portfolio, so a Metal-or-CPU diagnosis is one filtered list rather than N; a construction-time refusal when the policy permits no family the host stated, which is the "reject before work" obligation the composition ticket names; and one derivation of "can I execute family X", because a policy may only restrict families the host already stated and is refused if it names another. *Prevents:* nothing measured. *Cost:* a public call-site boundary change on `preflight`/`prepare` — additive sibling methods keep at least the composition-named consumers (`tiler`, `tiler-conformance`, candle-metal-adapter) plus other direct `ExecutionEnvironment` call sites (serial-sum-run, adapter_route tests, and similar) compiling, but there are then two spellings of one path.

**Option B — the loader stays single-family and the policy lives one level up**, in `crates/tiler`'s route facade, which tries environments in the consumer's tier order and reports which won. *Enables:* `tiler-runtime`'s accepted public surface is untouched; "a host states what it is" stays exactly as accepted; the deployment preference sits in the consumer layer where it arises. *Prevents:* a single whole-portfolio diagnosis — each attempt yields its own `filtered` list — and it puts routing policy in the consumer facade that workspace libraries must not depend on (distinct from `tiler-conformance`, the member nothing may depend on). *Cost:* it lands in `implementation/frontend`.

**The strongest counterpoint to the recommendation.** `ExecutionEnvironment` bundles profile, backend, representation, and dtype map precisely because they co-vary, so a host that "can do Metal or CPU" is arguably **two hosts**, and the honest model may be two load attempts rather than one loader taking a set. Against that stands `CompileRequest::preferring`, whose documentation makes the opposite argument on the compile side — a caller that tried the strictest option, saw a refusal, and retried "would get the same answer only by accident: the compiler would never have seen the alternatives, so it could not record which were acceptable". That precedent is weaker here than it is there, and the weakness should be stated rather than papered over: the compiler *binds* the stated list into the request subject, and the loader binds nothing, so no artifact or identity records which families a host was willing to run.

**Recommendation: Option A**, on the ground that the refusal quality is the whole point of the feature. The composition ticket's obligation is to reject "policy that permits no executable route **before work**", and only the layer holding both the stated set and the packaged portfolio can decide that in one place.

## Closes when

Tom picks A or B, or names a third shape; the answer is recorded where the host model is defined rather than only here — ADR 0090 item 4's host-statement inventory and `crates/tiler-runtime/src/load/host.rs` docs, plus any new decision fragment Tom requires; and [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md) Implementation keys are rewritten against the chosen option. Do not reopen the terminal composition parent to rewrite its family-policy key again — that key was already split into express on 2026-08-09. Acceptance provenance — who, date, venue, relay source — is recorded.

## Graph maintenance

- Only Tom closes this. It is not research: both options were compared on correctness, maintainability, and refusal quality, and both survive.
- Option B requires `implementation/frontend`, which the composition ticket does not declare; whichever option is chosen, add the scopes it needs to the implementing ticket rather than editing out of scope.
- Do not build the policy vocabulary against either option before this closes. A vocabulary that cannot change an outcome is the failure this node exists to prevent.
