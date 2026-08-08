---
id: correct-the-roadmap-s-milestone-0b-inline-composition-claim
title: Correct the roadmap's Milestone 0B inline-composition claim
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-roadmap0b
lease_expires_at: 1786163600
---
## The roadmap asserts an absence the tree refutes

`docs/roadmap.md`'s Milestone 0B section states: "The offline Metal producer, expansion cache, neutral artifact path, and bounded device proof now exist; **inline composition and consumer integration do not**."

The first half of that conjunction is false. `repair-the-status-record-s-grammar-claim-and-its-failing-reproduction-line` established at base `0132c0c3` that a `tensor!` region stating `deliver macos;` runs the whole inline flow inside `rustc` — parse, construct, verify, optimize, emit Metal, identify, look up, compile through `xcrun`, publish, read back, embed — and that `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs "The complete inline AOT workflow, in an ordinary consumer crate"` is the out-of-tree crate that exercises it, with `crates/tiler-macros/src/aot/tests.rs "the_second_expansion_of_one_subject_compiles_nothing"` carrying the warm half. `docs/status.md` now records that composition with its evidence and its three bounds.

The second half — consumer integration — remains true and must not be swept along with the first.

## What this ticket must not do

Decide whether the Milestone 0B **exit** is met. The exit criterion also names a rust-analyzer cold/warm measurement and the native macOS and non-Apple fallback paths, and this ticket has gathered no evidence about those. Correct the stale factual assertion; if the exit accounting genuinely moves, say so separately and with its own evidence, or file it.

Note also that the section's opening still reads "The actual Tiler macro-to-dispatch vertical remains implementation work", which is a second sentence in the same family and should be assessed rather than assumed stale — the dispatch half is bounded by what `crates/tiler/tests/facade/pass/inline_region_dispatches.rs "drives the artifact the macro embedded through the loader's own comparisons"` can prove, which is that the seam ran and not that a device computed anything.

## Closes when

`docs/roadmap.md`'s Milestone 0B text states the inline-composition position the tree supports, at its correct maturity, with the consumer-integration absence preserved; the exit accounting is either evidenced or explicitly left to a named successor; and every path-with-line or path-with-anchor citation added resolves under `make citations`.

## Outcome — 2026-08-07

Four dated corrections landed in `docs/roadmap.md`'s Milestone 0B section, all citing by searchable anchor.

**The conjunction.** "inline composition and consumer integration do not [exist]" was half false, as this ticket said. Inline composition exists; consumer integration does not and is preserved as an absence.

**The second sentence in the same family was also false, and this ticket underestimated it.** "The actual Tiler macro-to-dispatch vertical remains implementation work" is refuted not by `inline_region_dispatches.rs` — which does only prove the seam ran, exactly as this ticket said — but by `spikes/runtime/inline-dispatch`, which this ticket had not found. It takes one `deliver macos;` region to a completed hardware dispatch bit-compared against the consumer's own `f32`, and a second binary does the same for a two-entry bundle while watching a reversed encode return a wrong answer. Bounded to one Apple M4 Max, out-of-tree, hand-run, and settled on producer-declared equality under ADR 0086.

**A third sentence in the section was stale in the same way** and was corrected: "The inline proc macro, complete inline cache/AOT/embedding orchestration, artifact-family delivery, and Candle adapter remain explicit downstream tickets". Three of the four have moved.

**The exit accounting was evidenced rather than deferred, because the evidence existed.** This ticket's "What this ticket must not do" section asserted that no evidence about the rust-analyzer measurement or the fallback paths had been gathered. That was true of this ticket and false of the tree: `docs/integration/frontends.md` carries a maintained Landed/Withdrawn/Outstanding/Parked sweep over exactly those items. Warm rust-analyzer was measured 2026-08-01 over a real LSP session and the "blocked, component unavailable" clause is explicitly void; cold is parked and unmeasured; the macOS fallback path is implemented and tested but constructs the declared result without evaluating the region's arithmetic; the non-Apple fallback path is check-level source compilation from a macOS host in an `#[ignore]`d test, and no expansion has ever run where `xcrun` is absent. The milestone-exit judgement is explicitly not made.

**Out-of-scope defect filed.** `correct-the-stale-fallbackonly-claims-in-tiler-macros-family-cfg` — two comments in `crates/tiler-macros/src/family_cfg.rs` still assert that no expansion compiles a selected family, refuted by `aot::deliver` and the facade fixture. Not fixable here; `crates/**` is outside this ticket's scope.
