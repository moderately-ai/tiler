---
id: fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it
title: Fail a partial path whose leading component has vanished instead of skipping it as external
status: todo
priority: p3
dependencies: []
related: [stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: [check-citations.sh]
tags: [gates, citations, correctness]
---
## User-visible outcome

Deleting a whole directory can no longer convert a Tiler citation into one the checker skips as belonging to another project.

## Why this exists — filed 2026-08-19 from the ledger work

**Fact.** `check-citations.sh` decides that an unresolvable partial path belongs to another repository by testing its leading component against every component of every tracked path: the branch is guarded by `if (sub(/\/.*$/, "", lead) && !(lead in component))` and reports `rooted outside this tree`. The rationale in the header is sound for `candle-core/src/metal_backend/device.rs` — demanding upstream paths resolve here is an unsatisfiable condition.

**Inference — the branch fails open on a directory deletion.** A live citation of `codec/encode.rs:443` fails loudly today if `encode.rs` goes away, because `codec` is still a tracked component. Delete the whole `codec` directory and the same citation is skipped as belonging to another project instead, so a genuinely broken citation reports nothing. This is a coverage hole rather than a wrong resolution — unlike the twin case the parent ticket fixed, it never points a reader at the wrong file — which is why it was left out of that change rather than folded into it.

**Measurement 2026-08-19 at `bda38064`.** 32 citations reach the branch, over 10 distinct paths, and reading them shows all 10 are genuinely upstream: eight under `candle-core/` and `candle-metal-kernels/`, two under `MacOSX26.5.sdk/`. The hole is unoccupied today, so this is prevention rather than repair.

    ./check-citations.sh --verbose | sed -n 's/^SKIP  [^:]*: `\(.*\)` (rooted.*/\1/p' | sort -u

## Required work

- Re-audit the 32 and the 10 at your own base before designing anything; an occupied hole changes what the fix may do.
- Decide how a Tiler-rooted partial path is told from an upstream one without leaning on a component that a deletion can remove. Candidates worth costing: an explicit list of upstream roots, recorded the way the retired-ambiguity ledger is; requiring an upstream citation to carry its provenance in the span; or testing the *trailing* components rather than the leading one. Derive the design from the script, and state what each option would do to the 32.
- Perturb the subject: delete a tracked directory whose name no other tracked path carries, with a live citation rooted in it, and quote the checker before and after.

## Closes when

A citation rooted in this tree fails when its directory is deleted, the upstream population still skips, and `make citations` is green.
