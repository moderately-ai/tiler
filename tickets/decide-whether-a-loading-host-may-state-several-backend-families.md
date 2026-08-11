---
id: decide-whether-a-loading-host-may-state-several-backend-families
title: Decide whether a loading host may state several backend families
status: done
priority: p1
dependencies: []
related: [expose-explicit-backend-provider-and-selection-policy-composition, select-executable-variants-across-registered-backend-families, route-a-custom-backend-through-an-independently-selected-adapter, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, express-the-typed-backend-family-selection-policy]
scopes: [contracts/decisions, implementation/runtime]
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

## Accepted third position — 2026-08-11

**Decision — one routing attempt names exactly one backend approach, explicitly.** Tom rejected automatic cross-family selection and both surviving positions above in the T3 Code orchestration conversation: "no silent fallbacks... users MUST specify the approach they want to use... this is where prechecks/preflight come in". A loading host continues to state exactly one [`ExecutionEnvironment`](../crates/tiler-runtime/src/load/host.rs) per attempt. The user or consumer explicitly chooses Metal, CUDA, CPU, or another backend family before routing; `preflight` or `prepare` validates that exact profile/backend/representation/dtype declaration and refuses a mismatched artifact before allocation, preparation, or routing commit.

This answer is intentionally stronger than Option B. No consumer facade receives an ordered family policy that silently retries another backend. A caller may inspect a refusal and make a new, explicit attempt under another environment, but that is a new application decision after the first preflight ended and before any commit. Metal bytes presented under a Linux CUDA environment refuse, CUDA bytes presented under Metal refuse, and an artifact with no route for the explicitly chosen family refuses; none falls through to a different family.

The decision does **not** remove ordinary variant selection inside the chosen backend family. The producer's stable priority may still choose among compatible plans for the one stated environment, and an ineligible variant's guard is still not evaluated. It removes only cross-family policy and fallback. One route, one environment, one live device, and one command stream remain the initial profile; multi-device execution and mixed-family variants remain unsupported.

No artifact field, artifact identity, runtime identity, or public Rust surface changes. The existing singular `ExecutionEnvironment` and `DecodedProgram::{preflight, prepare}` boundary is the accepted shape.

## Outcome

Closed by Tom's direct 2026-08-11 decision in the T3 Code orchestration conversation. ADR 0090 item 4 and the runtime host documentation now state the explicit-single-family rule. [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md) is closed `wontdo`, and the portfolio proof is rewritten around explicit per-family attempts rather than an automatic Metal-or-CPU policy. No implementation remains on this decision.

## Graph maintenance

- The rejected set-valued loader and consumer-facade retry policy remain here as alternatives considered, not as implementation reservations.
- A future request for automatic cross-family fallback must return as a new product decision with a use case that justifies overturning the explicit-selection rule.
