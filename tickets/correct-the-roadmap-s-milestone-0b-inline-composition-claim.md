---
id: correct-the-roadmap-s-milestone-0b-inline-composition-claim
title: Correct the roadmap's Milestone 0B inline-composition claim
status: todo
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
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
