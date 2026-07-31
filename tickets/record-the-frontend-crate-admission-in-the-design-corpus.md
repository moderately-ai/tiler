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

## Public boundary for Tom

Two conflicts in `docs/integration/frontends.md` (`contract_status: accepted`) are design questions, not prose drift, and neither is this ticket's to settle alone:

1. Its inline-region example spells the macro `tiler! { ... }`. The ratified path is `tiler::tensor!`. Probably just an older illustrative spelling — confirm, then align.
2. Its generated-expansion example emits `::tiler_candle::execute_or_fallback(::tiler_artifact::EmbeddedBundle::new(...))` — generated tokens naming internal crates directly. That contradicts `prototype-inline-proc-macro-frontend`'s "generate only paths reachable through the consumer's declared `tiler` dependency", and it is the exact problem `tiler::__private` was added to solve. Either the example predates the facade decision and should route through the facade, or the facade must re-export those types. Ask; do not pick.

## Closes when

Every site above states what is true; the admission ADR exists with a `decision_status` matching Tom's actual acceptance; `docs/decisions/README.md` and the affected catalog views list it; the `docs/status.md` absence block runs green as written; and the two `frontends.md` conflicts are resolved by Tom's answer rather than by an edit that picks a side.

## Graph maintenance

- This cannot land before the admission merges, or its edits become false in the other direction.
- If Tom rejects the admission diff, close this rather than reworking it — nothing to record.
