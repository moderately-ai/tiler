---
id: admit-a-caller-declared-target-profile
title: Admit a caller-declared target profile
status: in-progress
priority: p1
dependencies: []
related: [express-metal-honourability-in-the-shared-form, prototype-public-compiler-api]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, feasibility, identity]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785190012
---
The compiler admits exactly one target profile and offers no way to author another. `express-metal-honourability-in-the-shared-form` needs one and is currently unreachable without it.

## Fact — declaration does not exist, and selection is not it

`crates/tiler-compiler/src/request.rs` declares `pub(crate) struct PrototypeTargetProfile` with one constructor, `governed()`. `verify_request` rejects any other profile with `UnsupportedCapability { phase: "target", rule: "prototype-target-neutral-baseline-v1" }`, and `for_target` rejects again independently, so the `Vec<PrototypeTargetProfile>` field is structurally plural and admits exactly one distinct value. Twelve sites in that file compare against `PrototypeTargetProfile::governed()`.

`prototype-public-compiler-api` landed the request half of the public boundary on 2026-07-27 — a caller can now install its own `FrozenLoweringCapabilityRegistry` — and deliberately did not land this. Selecting the governed profile and authoring a profile are different capabilities, and only the first exists.

Reproduce with `grep -n "pub(crate) struct PrototypeTargetProfile" crates/tiler-compiler/src/request.rs` and `grep -c "PrototypeTargetProfile::governed()" crates/tiler-compiler/src/request.rs`.

## Why this is filed now rather than discovered later

**The work graph is currently pointing at unreachable work.** With its dependency on `prototype-public-compiler-api` satisfied, `express-metal-honourability-in-the-shared-form` is p0, dependency-satisfied, and the top of `tkt next` — while being impossible to close. A comment on that ticket records the reason; a comment is not an edge, and `tkt next` does not read comments. This ticket exists so the graph states the blockage it already has.

## What the substance is

Not visibility. A caller-authored profile must be *validated*, and deciding what makes one well-formed is the work:

- an honourability declaration is a set of per-dimension rows, and a profile that omits a dimension must stay `Unknown` on it rather than becoming trivially satisfiable — the fail-closed direction `GOVERNED_TARGET_HONOURABILITY` already documents for `FlushToZero { AlwaysPositive }`;
- profile keys enter the request subject and therefore artifact identity, so key uniqueness and key governance are identity questions, not hygiene;
- the quantitative bounds (`max_threads_per_grid_axis`, `max_threads_per_workgroup`, `max_buffer_bindings_per_entry`, `index_bits`, `supports_device_memory`) are consumed by feasibility, and a profile declaring bounds no device has is a way to make an infeasible plan look feasible.

`PrototypeTargetProfile`'s `numerical` field is a `&'static [DeclaredBehaviour]`, and `DeclaredBehaviour` lives in the crate-private `honourability` module. Whether declaration promotes that vocabulary, or takes a different shape at the boundary, interacts directly with the three-way siting decision `express-metal-honourability-in-the-shared-form` carries. **Do not settle that siting here by implication** — if this ticket forces it, say so and record it as an accepted decision rather than leaving it implicit in a signature.

## Closes when

An out-of-crate caller can author and compile against a target profile it declared; a malformed or under-declared profile is refused with a typed diagnostic naming what was wrong; an omitted numerical dimension resolves `Unknown` rather than satisfied; the identity consequence of a caller-supplied profile key is recorded; and `make full` passes.
