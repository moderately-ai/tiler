---
id: split-the-compiler-pipeline-by-orchestration-phase
title: Split the compiler pipeline by orchestration phase
status: done
priority: p2
dependencies: [prototype-public-compiler-api]
related: [prototype-optimizer-conformance-gate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, compiler, progressive-disclosure]
---
Make the compiler's top-level orchestration read as the compilation story rather
than as one file containing orchestration, mechanisms, conformance, and tests.

## Outcome

Keep `pipeline` as one compiler concept and public/internal path. Let its root
show request verification, transactional planning, alternative construction,
selection, and result formation in order. Move transactional state,
alternative enumeration, trace production, verification, conformance support,
and large tests into shallow phase-owned files.

The public request/provider boundary lands first so this refactor preserves the
reviewed call site rather than organizing around a temporary facade.

## Closes when

The ordinary compilation path can be followed from one module root, mechanisms
are separated by invariant rather than line count, public behavior and explain
identity are unchanged, and the full gate passes.

## Outcome — 5,226-line module split by invariant (2026-07-27)

`crates/tiler-compiler/src/pipeline.rs` went from **5,226 lines to 1,183**, and its root now states the compilation story in order: verify the request, plan transactionally, build one alternative per retained plan, select, re-derive, form the product.

| file | lines | invariant it owns |
| --- | --- | --- |
| `pipeline.rs` | 1,183 | the orchestration itself, plus the error vocabulary and failure attribution |
| `pipeline/planning.rs` | 577 | the **transaction** — nothing it produces is observable until the root accepts the portfolio |
| `pipeline/trace.rs` | 766 | **decides nothing** — observes a decision the root already made and records it |
| `pipeline/verify.rs` | 288 | **reuses no planning intermediate** — see below |
| `pipeline/conformance.rs` | 862 | drives the public `compile()` only, reaching no stage-local constructor |
| `pipeline/tests.rs` | 1,591 | everything else |

**The split is by invariant, not line count, and the files differ fivefold because of it.** `verify` is the smallest and carries the sharpest rule: it re-derives the retained portfolio and may not be handed a planning intermediate, because a verifier given the value it is checking compares that value to itself and can never say no. That is recorded at the module head rather than left as folklore — a profile put it at 23% of a compile's active self time, which is exactly the cost someone would try to "optimize" away without it.

**`conformance` is a sibling of `tests`, not part of it.** It drives the public entry point and reaches no stage-local constructor; merging them would blur the line that makes it a conformance gate rather than a unit test.

**Public behaviour and explain identity are unchanged** — 244 `tiler-compiler` tests pass, including the explain-trace identity assertions, and `pipeline` remains one concept at one internal path.

**One deliberate lint exception, with its reasoning at each site.** The three phase modules glob-import their parent (`use super::*`). `clippy::wildcard_imports` is denied workspace-wide, so each carries an `#![allow]` with a `reason`: these are private children of one module rather than separate concepts, every name they use is defined in that parent, and enumerating fifty parent items per file would have to be restated on every change for no reader benefit. The imports in the *other* direction — root reaching into the children — are enumerated explicitly, because those are a real interface and a short one.
