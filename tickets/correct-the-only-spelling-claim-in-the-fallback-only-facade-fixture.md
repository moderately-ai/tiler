---
id: correct-the-only-spelling-claim-in-the-fallback-only-facade-fixture
title: Correct the stale "only spelling" claim in the fallback-only facade fixture
status: todo
priority: p3
dependencies: []
related: [correct-two-stale-delivery-spans-in-the-frontends-contract]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, frontend, inline-dx, staleness]
---
## Why this exists

Found while correcting the frontend contract's stale refusal paragraph under [`correct-two-stale-delivery-spans-in-the-frontends-contract`](correct-two-stale-delivery-spans-in-the-frontends-contract.md), which held `contracts/integrations` and so could not reach `crates/`.

`crates/tiler/tests/facade/pass/deliver_states_fallback_only.rs:5` says of `deliver fallback-only;`: "it is the only spelling of the statement that reaches an expansion today: every other profile and every family list selects an artifact family, and nothing compiles one yet, so those are refusals rather than pass cases."

**That is false at `561dfe0b`**, and the fixture directory beside it refutes it: `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` states `deliver macos;` at lines 215, 249, and 282, and its compilation runs the offline driver, embeds the artifact, and routes it. This is the same stale claim the contract carried — `stated_delivery`'s own doc comment at `crates/tiler-macros/src/delivery.rs:751` records that it "no longer refuses a selection that invokes the backend compiler" — and the contract's half was corrected under the parent ticket while this one stayed out of scope.

A doc comment is load-bearing. This one tells the next reader that a pass fixture for a delivering region cannot exist, which makes the fixture beside it look like a mistake.

## Closes when

The module doc states what the fixture actually proves at the commit that lands it — the equivalence of `deliver fallback-only;` with the statement's absence — without asserting that no other spelling reaches an expansion, and the corrected sentence is checkable against `deliver_compiles_embeds_and_routes.rs`. `grep -rn "only spelling of the statement" crates/` reports no match.
