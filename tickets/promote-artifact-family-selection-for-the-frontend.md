---
id: promote-artifact-family-selection-for-the-frontend
title: Promote artifact-family selection for the frontend
status: done
priority: p1
dependencies: [prototype-artifact-family-delivery, admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [prototype-inline-proc-macro-frontend, generate-cfg-gated-artifact-family-delivery]
scopes: [implementation/frontend, implementation/metal-aot, implementation/cargo-lock, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The frontend can state the accepted artifact-family delivery policy through one reviewed typed request without duplicating the crate-private `ArtifactFamilySelection` or teaching the proc macro Apple tool-discovery logic.

## Implementation keys

Review the construction sites and promote the smallest existing `ArtifactFamilySelection`, `ArtifactDeliveryPolicy`, `SelectedFamily`, `FamilyRequirement`, and validation/error surface needed by the frontend. Preserve canonical ordering, explicit `FallbackOnly`, duplicate/empty refusal, per-family deployment minimum and MSL standard, and the accepted one-envelope/N-payload identity. Do not move generated consumer-`cfg` mapping into `tiler-metal-aot`; `generate-cfg-gated-artifact-family-delivery` remains frontend-owned.

If exposing the type from `tiler-metal-aot` would make the facade depend on driver/tool discovery or leak backend-only policy into a consumer-neutral frontend boundary, move the dependency-neutral request vocabulary to the lowest existing owner that both sides may depend on rather than copying it. Preserve `tiler-metal-aot`'s empty dependency closure and reject any second canonical encoder.

## Public boundary for Tom

Present the exact owner/module/type/constructor/reader/error path and frontend call site before acceptance. This review does not reopen the already accepted family selection semantics or one-envelope/N-payload decision.

## Accepted (2026-07-31)

Tom accepted the boundary as merged: `tiler_metal_aot::family` promoted in place as the single canonical encoder, the frontend edge held by host-built `tiler-macros`, no facade re-export (consumers never name the type), and the fail-closed `tensor!` delivery statement. The draft disclosures in the module doc, the dependency-direction test, and the frontends contract were flipped to the accepted state in the closing change, and `record-the-frontend-crate-admission-in-the-design-corpus` documents the ratified edge.

## Closes when

The frontend can construct and inspect one canonical selection, the AOT driver consumes the same value, no duplicate encoding or Apple host inference exists, public dependency direction is reviewed, mutation tests prove empty/duplicate/order/version checks can fail, and targeted tests/Clippy plus `make full` pass.

## Graph maintenance

- Follow facade admission explicitly because the reviewed packet includes the frontend call site; do not rely on frontend-scope collision for prerequisite order.
- Keep generated consumer-`cfg` mapping in `generate-cfg-gated-artifact-family-delivery` and Apple tool discovery in the AOT owner.
- Release the proc-macro proof only after one dependency-neutral canonical request is available without duplicating its encoder.

## Implementation drafted — awaiting Tom's acceptance

Delivered on branch `tkt/promote-artifact-family-selection-for-the-frontend` from base `6187d006add4d4eba648e1ba20dbc157beacee68`. Everything below is a concrete draft under ADR 0075's "promoting a module or type from `pub(crate)` to `pub`" and "a new publicly reachable namespace" categories, both of which always require Tom's review. Nothing here is self-accepted.

`implementation/cargo-lock` was added to this ticket's scopes: placing the frontend's dependency edge necessarily edits `Cargo.lock`, and `crates/tiler/tests/dependency_direction.rs` reads that file as its authority.

### The elimination that chose the owner

The brief's conditional fires — a facade edge would carry both costs it names — and the constraint that resolves it is that ADR 0077 item 2 decides `tiler-metal-aot`'s dependency closure *empty*. Four candidates, three eliminated:

- **Copy the vocabulary into the frontend.** Rejected: two canonical encoders over one identity subject, which is exactly what ADR 0074 convention 2 exists to prevent, and what the brief forbids outright. A selection's bytes are folded into artifact identity, so two encoders that drift produce two artifacts for one request.
- **Move it to a new crate below both sides.** Rejected: `tiler-metal-aot` would then need an edge to that crate, which spends the closure ADR 0077 item 2 decides. Admitting a crate is separately Tom's decision and requires scopes this ticket does not hold.
- **Move it to `tiler-ir`, the lowest existing crate.** Rejected twice: same closure cost, plus `tiler-ir` would acquire Apple artifact families, deployment minimums, and MSL standards — backend vocabulary the architecture contract keeps out of the consumer-agnostic core.
- **Promote it in place, in `tiler-metal-aot`.** Survives. `tiler-metal-aot` *is* the lowest existing owner both sides may depend on: nothing sits beneath it, so a frontend edge to it costs the closure nothing, and there is exactly one encoder.

The remaining question is which frontend crate holds the edge, and it is not a preference. A `proc-macro` crate and its dependencies are built for the **host** and never enter a consumer's target build graph, so `tiler-macros` holds it for free; the accepted packaging profile already charges the "Frontend proc-macro crate" row with invoking the AOT pipeline. The same edge on `tiler` would compile a process-spawning Apple toolchain driver into every consumer on every platform and publish Apple backend policy on a consumer-neutral boundary — the cost ADR 0077 item 4 already refused for `tiler-metal`. Nothing a consumer writes needs the type: a policy is stated in region syntax, and generated tokens name `#[cfg]` predicates and byte literals.

**The facade therefore re-exports nothing for this ticket**, and `crates/tiler/src/lib.rs`'s forward-looking claim that this ticket selects re-exports is corrected to say so with its derivation.

That is an answer to a question Tom left open rather than a departure from a decision. `record-the-frontend-crate-admission-in-the-design-corpus` records his 2026-07-31 decision that generated tokens route through facade-owned paths and that "the exact re-exports arrive with their owning tickets … where they are reviewed", naming this one. Reviewed here, the answer is *none*: no generated token and no consumer-written expression names a selection type, so a re-export would publish Apple backend policy on the facade for no call site. If Tom prefers the facade edge anyway, the change is mechanical — move the `[dependencies]` entry, re-export `tiler_metal_aot::family`, and delete `the_facade_does_not_carry_the_offline_apple_driver` — and the surface table below is unchanged by it.

### The exact public surface

Owner `tiler-metal-aot`, module `family` (`mod` → `pub mod`; documented as a reviewed *draft* boundary per ADR 0074 convention 7, which is why the `#![allow(dead_code, reason = …)]` is gone rather than reworded).

| Item | Form |
| --- | --- |
| `tiler_metal_aot::family::SelectedFamily` | `pub struct`; `pub` fields `family: ApplePlatform`, `deployment_minimum: DeploymentMinimum`, `msl_version: MslVersion` (leaf value record, convention 6) |
| `…::ArtifactDeliveryPolicy` | `pub enum` `SelectedFamilies { families: Vec<SelectedFamily>, requirement: FamilyRequirement }` \| `FallbackOnly`; deliberately **not** `#[non_exhaustive]` (convention 5b — the canonical encoder matches it totally) |
| `…::FamilyRequirement` | `pub enum` `RequiredWhenTargetMatches`; 5b for the same reason |
| `…::ArtifactFamilySelection` | `pub struct`, private field; **constructor** `new(ArtifactDeliveryPolicy) -> Result<Self, FamilySelectionError>` |
| readers | `policy() -> &ArtifactDeliveryPolicy`, `families() -> &[SelectedFamily]`, `invokes_backend_compiler() -> bool`, `compile_targets() -> Result<Vec<MetalTarget>, FamilySelectionError>`, `canonical_bytes() -> Vec<u8>` |
| **error path** | `…::FamilySelectionError`, `#[non_exhaustive]` (convention 5a), variants `EmptySelection`, `DuplicateFamily { family }`, `InvalidTarget { source: MetalTargetError }`, with `Display` and `std::error::Error` |

No signature changed; only visibility, plus `#[must_use]` where Clippy pedantic requires it on the newly public readers. `canonical_bytes` stays bytes rather than a digest, so no second identity authority appears.

### The frontend call site

`crates/tiler-macros/src/delivery.rs` (crate-private module, `mod delivery;`):

- `stated_policy() -> ArtifactDeliveryPolicy` — the policy this expansion states. `FallbackOnly` today, because `tensor!` has no grammar, an invocation names no family, and ADR 0053 makes "performs no backend compiler work" an explicit policy rather than an absence. `prototype-inline-proc-macro-frontend` replaces it with the policy an invocation's tokens resolve to; it is a function of a *policy* precisely so that ticket changes what is stated without changing what validates it.
- `stated_delivery(ArtifactDeliveryPolicy) -> Result<ArtifactFamilySelection, DeliveryRefusal>` — validates through the one canonical constructor and inspects the result.
- `DeliveryRefusal::InvalidSelection(FamilySelectionError)` carries the driver's rejection unflattened; `DeliveryRefusal::BackendCompilationUnavailable { families }` refuses a selection this expansion cannot build, because ADR 0053 forbids a selected-family build failure becoming silent fallback on the matching target.

`tensor!`'s empty-input arm now routes through `expand_region()`: it states the policy, validates it, and emits the facade anchor only when the selection invokes no backend compiler; a refusal becomes a spanned `compile_error!`. Nothing here discovers Apple tools, reads the host, or decides a `cfg` predicate.

The AOT driver consumes the same value: `compile_targets()` is the fan-out to one `MetalTarget` per selected family, each becoming one `CompileRequest` that `Toolchain::prepare` binds. A compiled `no_run` module example shows that whole chain, and `every_selected_family_becomes_its_own_compile_request` asserts the fan-out survives into the exact `-target` flags rather than only into the target list.

`crates/tiler/tests/dependency_direction.rs` gains `the_facade_does_not_carry_the_offline_apple_driver`, which asserts both halves of the placement (the facade has no edge, the macro crate does) so a later "simplification" fails a test rather than passing a manifest review.

### Mutation evidence

Each check perturbed once, run, restored. Commands: `cargo nextest run -p tiler-metal-aot -p tiler-macros --no-fail-fast --status-level fail`, and `-p tiler -E 'test(the_facade_does_not_carry_the_offline_apple_driver)'` for the last two.

| Perturbation | Result |
| --- | --- |
| `EmptySelection` refusal removed | FAIL ×2: `family::tests::an_empty_selected_family_list_is_not_fallback_only`, `delivery::tests::an_empty_family_list_is_refused_as_an_invalid_selection` |
| duplicate-family loop removed | FAIL ×2: `family::tests::a_repeated_family_is_rejected_rather_than_deduplicated`, `delivery::tests::a_repeated_family_is_refused_as_an_invalid_selection` |
| `sort_unstable_by_key(canonical_key)` removed | FAIL ×3: `family::tests::declaration_order_does_not_change_the_canonical_bytes`, `family::tests::every_selected_family_becomes_its_own_compile_request`, `delivery::tests::declaration_order_does_not_change_what_the_frontend_states` |
| per-family `compile_target()?` validation removed | FAIL ×2 (see the gap below) |
| `tiler` given a normal edge to `tiler-metal-aot` | FAIL: `the_facade_does_not_carry_the_offline_apple_driver` |
| `tiler-macros` edge removed (population guard) | Build error `E0433` in `delivery.rs` — the edge cannot vanish silently, so the guard's real failure mode is a compile error and the assertion is the residual check for a future where `delivery.rs` stops importing the crate |

**A gap the mutation run found and this ticket closed.** On its first run, removing the constructor's per-family target validation was caught *only* by the new frontend test: no existing `tiler-metal-aot` test covered the constructor's version check, because the existing ones exercised `MetalTarget::new` directly and `compile_targets` re-validates lazily. `family::tests::a_family_below_its_language_floor_is_rejected_at_construction` was added and the perturbation re-run; it now fails on both sides. The distinction matters beyond coverage: a selection validated lazily would let an unbuildable request reach identity, explain output, and a cache subject looking exactly like a valid one.

### Out of scope, filed as follow-ups rather than absorbed

- `docs/architecture.md`'s packaging profile block lists no frontend crate at all and still says the profile "deliberately omits frontend, proc-macro … crates". That was already stale when `tiler` and `tiler-macros` were admitted on 2026-07-31; the new `tiler-macros -> [tiler-metal-aot]` edge is one more line it will need. `record-the-frontend-crate-admission-in-the-design-corpus` owns that file and is `todo`; a note has been added there. `contracts/foundation` is not in this ticket's scopes.
- ADR 0077's item 3 block has the same omission for the same reason and is owned by the same follow-up; this ticket adds no ADR and supersedes none. The empty closure it decides is untouched: `tiler-metal-aot` still declares neither `[dependencies]` nor `[dev-dependencies]`.
