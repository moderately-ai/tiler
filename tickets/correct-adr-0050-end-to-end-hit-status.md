---
id: correct-adr-0050-end-to-end-hit-status
title: Correct ADR 0050's end-to-end hit status
status: todo
priority: p2
dependencies: []
related: [exercise-the-expansion-cache-under-cargo-and-rust-analyzer]
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: [cache]
---
ADR 0050's traceability paragraph says a "positive end-to-end hit carrying a real compiled artifact remains the orchestrator's". That is now partly superseded by evidence and the record does not say so.

`exercise-the-expansion-cache-under-cargo-and-rust-analyzer` built an orchestrator holding both crates: `spikes/cache/build-tool-exercise/` resolves through the public `ExpansionCache::get_or_publish`, whose validator is the real `decode_artifact`. The envelope is produced by a genuine `tiler-compiler` session, encoded by `tiler-artifact`, published, re-read, and completely validated, under real `cargo` and a real `rust-analyzer` proc-macro server.

## What this ticket owes

- Replace the "remains the orchestrator's" sentence with the narrower gap that actually remains: the payload is **declared by descriptor rather than carried**, so no compiled backend object has yet travelled through a cache entry.
- Say that the concurrent multi-process behaviour is measured under the two build tools ADR 0050's context sentence names, and cite `docs/research/cache/build-tool-exercise.md` and `spikes/cache/results/build-tool-exercise-macos-27.0-2026-07-25.tsv`.
- Re-check whether `implementation_status: partial` is still the right value, and what else it is still waiting on besides `measure-expansion-cache-durability-policies`.

Split out rather than done in the exercise branch because editing `docs/decisions/` needs the `contracts/decisions` scope, which `decide-per-dtype-dispatchability-as-a-target-capability` held while that work ran.
