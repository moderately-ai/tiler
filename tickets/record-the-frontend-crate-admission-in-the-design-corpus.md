---
id: record-the-frontend-crate-admission-in-the-design-corpus
title: Record the frontend crate admission in the design corpus
status: todo
priority: p1
dependencies: [admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [prototype-inline-proc-macro-frontend, define-inline-symbol-binding-and-runtime-value-adaptation]
scopes: [contracts/decisions, contracts/navigation, contracts/foundation, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A reader of `docs/` sees the workspace that exists. Today the corpus asserts the two frontend crates are absent, and one of those assertions is an executable check that inverts the moment they land.

## Implementation keys

`admit-the-tiler-facade-and-proc-macro-crate-boundary` added `crates/tiler` and `crates/tiler-macros` under `implementation/frontend` and `implementation/workspace`, which do not reach `docs/`. Every site below was read directly in the admission worktree at base `7b1e3a7`, and each is **Fact**:

- `docs/status.md`, the "workspace-member absence claims ... are reproducible" block: `test ! -d crates/tiler-macros` and `! rg -n 'proc-macro\s*=\s*true' crates --glob Cargo.toml` both **fail** once the admission merges. This is the acute one — a documented reproducible check that now reports the opposite of what the prose around it claims.
- `docs/status.md`, "Not yet delivered": "the inline proc-macro frontend remains awaiting decision". Narrower now: Tom ratified the two-crate topology and the `tiler::tensor!` path on 2026-07-30, while the grammar, expansion, and the cold/warm inline AOT workflow remain open. Say which half moved.
- `docs/architecture.md`, "Accepted prototype packaging profile": "The workspace carries nine reusable libraries and two non-published proof executables", followed by an intra-workspace edge block. Eleven now, and the block needs `tiler -> [tiler-macros] + development [trybuild]` and `tiler-macros -> []`. Note that the surrounding text already calls these edges "a description maintained by reading, not a checked contract" — `crates/tiler/tests/dependency_direction.rs` now checks one specific edge class of it (nothing may depend on the frontend) against `Cargo.lock`, which is worth recording as the first checked slice.
- `docs/architecture.md`, same section: "It deliberately omits frontend, proc-macro, Candle, and reusable Metal-*runtime* crates until the proof reaches those boundaries." Follow the ADR 0081/0082 precedent — decide whether this is an admission on the clause's own terms or an amendment of it, and say which.
- `docs/research/cache/build-tool-exercise.md`: "`crates/tiler-macros/**` is a mapped path with no crate behind it." That was the stated reason the cache-root chooser had no owner. The crate now exists; the chooser is still unowned, so correct the premise without inventing an owner.

Then add the admission record itself. Every other member carries one — ADR 0077 (`tiler-metal-aot`), 0081 (`tiler-runtime`), 0082 (`tiler-cache`), 0085 (`tiler-build`) — and these two carry none. Model the new ADR on those: admit both members, decide the dependency direction as a property rather than an ordering accident, and amend the packaging profile. **Its `decision_status` is whatever Tom's acceptance of the admission diff makes it, not `accepted` by assumption.**

## Public boundary for Tom — both questions answered (2026-07-31)

Two conflicts in `docs/integration/frontends.md` (`contract_status: accepted`) were design questions, and Tom decided both:

1. Its inline-region example spells the macro `tiler! { ... }`. **Tom confirmed the ratified `tiler::tensor!` path stands; the contract example is an older illustrative spelling and is aligned, not preserved.**
2. Its generated-expansion example emits `::tiler_candle::execute_or_fallback(::tiler_artifact::EmbeddedBundle::new(...))` — generated tokens naming internal crates directly. **Tom decided generated tokens route through facade-owned paths: rewrite the example to show facade-owned paths, add no re-exports now, and let the exact re-exports arrive with their owning tickets (`define-inline-symbol-binding-and-runtime-value-adaptation`, `promote-artifact-family-selection-for-the-frontend`) where they are reviewed.**

Separately settled: Tom accepted the exact facade surface on 2026-07-31, so the admission ADR this ticket writes records an accepted decision — `decision_status: accepted` is correct, not an assumption.

## The macro crate's edge is no longer empty (2026-07-31)

The third bullet above says the block needs `tiler-macros -> []`. That was true when it was written and is now conditional. `promote-artifact-family-selection-for-the-frontend` gives `tiler-macros` a normal dependency on `tiler-metal-aot`, so the frontend rows read:

```text
tiler-macros -> [tiler-metal-aot]
tiler        -> [tiler-macros]      + development [trybuild]
```

Record the reasoning with them, because the placement is the decision rather than the edge: a `proc-macro` crate and its dependencies are built for the host and never enter a consumer's target build graph, which is why the macro crate may hold an edge to a process-spawning Apple toolchain driver and the facade may not — the same cost ADR 0077 item 4 refused for `tiler-metal`. `crates/tiler/tests/dependency_direction.rs` checks both halves. `tiler-metal-aot`'s empty closure is untouched: the edge points at the driver, not out of it. ADR 0077 item 3's own restatement of the block has the same omission and the same fix.

**Not yet settled.** That promotion is presented to Tom under ADR 0075 and was unaccepted when this note was written. Record the edge only once it is accepted; if it is rejected or moved to the facade, the rows change accordingly.

## Closes when

Every site above states what is true; the admission ADR exists with a `decision_status` matching Tom's actual acceptance; `docs/decisions/README.md` and the affected catalog views list it; the `docs/status.md` absence block runs green as written; and the two `frontends.md` conflicts are resolved by Tom's answer rather than by an edit that picks a side.

## Graph maintenance

- This cannot land before the admission merges, or its edits become false in the other direction.
- If Tom rejects the admission diff, close this rather than reworking it — nothing to record.
