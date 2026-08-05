---
id: make-stage-retention-reachable-from-a-test
title: Make stage_retention reachable from a test
status: todo
priority: p3
dependencies: []
related: [retain-succeeding-metal-stage-tool-output, carry-a-producer-stated-total-into-a-retained-run]
scopes: [implementation/metal-aot, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, testing]
---
## User-visible outcome

`stage_retention` — the function that assembles the only retention this product actually publishes — is exercised by a test, so its labelling, its per-stage attribution, and the total it now states are checked rather than merely compiled.

## Why this exists

**Fact — nothing reaches it.** `crates/tiler-build/src/metal_cache.rs`'s `stage_retention` is private, takes `&[StageOutputs]`, and has no test in the workspace. Reproduce with `grep -rn "stage_retention" crates/ --include='*.rs'`: three hits, all in that file, none in a test.

**Fact — it cannot be constructed out of crate.** `tiler_metal_aot::record::StageOutputs` is `#[non_exhaustive]` with public fields and no constructor (`crates/tiler-metal-aot/src/record.rs`), and its only construction site is `crates/tiler-metal-aot/src/driver.rs:416`. A `tiler-build` test therefore cannot build one, and the alternative — a real Metal compilation — self-skips on a host without a qualified toolchain, so it would not be evidence the gate carries.

**Fact — the untested surface just grew.** `carry-a-producer-stated-total-into-a-retained-run` (commit `c39cb814`) changed `stage_retention` to state `ToolOutput::total_bytes()` through `DebugRetention::retaining_with_stated_total`. The retention side of that change is tested in `tiler-cache`; the producer side is carried by `cargo check` alone. The pairing of a stage's bytes with *that same stage's* total is exactly the kind of mistake a type does not catch.

**Inference — the remedy crosses a public boundary.** Making `StageOutputs` constructible out of crate is a `tiler-metal-aot` public-surface addition, which is Tom's to accept. The narrow alternatives are a `#[cfg(test)]`-only constructor (which does not help `tiler-build`, a different crate) or moving the assembly into `tiler-metal-aot`, which would put Metal cache labelling in the driver. Enumerate these before implementing; do not self-accept the surface.

## Closes when

A test observes `stage_retention` producing one run per stage per delivery position, under the governed labels, each carrying its own stage's bytes and its own stage's stated total — and a deliberate swap of the two stages fails it. Any new public item is accepted by Tom before the ticket closes.
