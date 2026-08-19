---
id: keep-a-module-size-and-complexity-census-with-a-split-queue
title: Keep a module size and complexity census with a split queue
status: todo
priority: p3
dependencies: []
related: [split-the-schedule-builder-into-cohesive-submodules, split-the-compiler-target-module-into-cohesive-submodules, split-the-artifact-program-test-monoliths-into-focused-modules, split-the-compiler-request-module-before-the-contributor-source-carrier, split-the-index-refinement-module-into-cohesive-submodules]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [maintainability, census]
---
## User-visible outcome

Module size stops regressing silently: a recorded census names the workspace's oversized files, a prioritized split queue orders the repairs, and the closing state records either a mechanical floor check or the reasoned decision not to add one.

## Census — 2026-08-19, base `e53eb65f` working tree

Command: `find crates -name '*.rs' | xargs wc -l | sort -rn`. Population: **60 files over 2,000 lines** across 469,689 workspace lines. Top of the census (lines, file):

- 13,746 `tiler-compiler/src/request.rs` — split ticket filed (sequenced behind the contraction merge, before the contributor-source carrier)
- 10,554 `tiler-ir/src/schedule/builder.rs` — split ticket filed, dispatched
- 9,438 `tiler-compiler/src/pipeline/tests.rs` — queue below
- 8,633 `tiler-compiler/src/target.rs` — split ticket filed, dispatched
- 7,936 `tiler-artifact/src/program/tests.rs` — split ticket filed, dispatched (with codec/tests.rs 5,795)
- 7,434 `tiler-ir/src/kernel/tests.rs`; 7,363 `tiler-compiler/src/frontier.rs`; 7,101 `tiler-ir/src/index/refinement.rs` (ticket filed); 5,848 `tiler-compiler/src/physical.rs`; 5,781 `tiler-compiler/src/explain.rs`; 5,661 `tiler-compiler/src/target/feasibility.rs`; 5,392 `tiler-ir/src/semantic/registry.rs`; 5,315 `tiler-ir/src/index/law.rs`; 5,266 `tiler-ir/src/program/tests.rs`; 5,192 `tiler-compiler/src/session.rs`; 5,109 `tiler-compiler/src/region.rs`; 4,807 `tiler-compiler/src/governed.rs`.

Per-crate concentration: tiler-ir 150k lines / 155 files; tiler-compiler 132k / 65 files (the highest lines-per-file ratio — the split queue's centre of gravity).

## Required work

- After the five filed splits land, re-run the census, extend the queue over the remaining hotspots above (test monoliths first — they carry the read-in-full burden with the least seam risk; then `frontier.rs`, `physical.rs`, `explain.rs`, `session.rs`, `region.rs`, `feasibility.rs`, `registry.rs`, `law.rs`), and file the next tranche as bounded tickets with the same pure-code-motion discipline.
- Decide, with a recorded rationale, whether a mechanical check is worth its maintenance: a size floor can only make regressions loud (e.g. a census script asserting no file exceeds a stated ceiling without a recorded exemption), and per repository rules a check that cannot fail is worse than none. If added, perturb it and quote the failure text; if declined, record why.
- Complexity between modules is part of the census, not just line counts: where a split reveals a dependency tangle (two modules that cannot separate without a shared-internals module larger than either), record it here as a restructuring candidate rather than forcing a bad seam.

## Closes when

The first tranche of splits is landed and recorded here, the second tranche is filed, and the mechanical-check decision is recorded with its rationale (and failure text, if adopted).
