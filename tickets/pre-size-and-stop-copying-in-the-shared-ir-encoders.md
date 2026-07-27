---
id: pre-size-and-stop-copying-in-the-shared-ir-encoders
title: Pre-size and stop copying in the shared IR encoders
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [performance]
---
Compile of a 5-operation program is 882 us. A samply profile of the compile path attributes a large share of what remains to diffuse allocation and copying, with `finish_grow` (Vec reallocation by doubling), `Vec<BlockData>::clone`, and `identity::push_slice` all naming this crate.

Goal: find which encoders in `crates/tiler-ir` produce bytes on the compile path without reserving first, pre-size them against an exact precomputed length guarded by `debug_assert_eq!`, and remove deep clones of kernel model data. Every identity byte this crate produces must be unchanged.

Measurement discipline: min-of-N, never mean; before/after with the same harness.

## Outcome

**Measurement:** compile of the 5-operation program falls from ~862 us to ~822 us, about **-4.5%**, on `hot_path_compile_time_by_shape` (min-of-200, six interleaved A/B pairs; base wins 0 of 18). A confirming eight-pair run of base against the final tree favours the change in 20 of 24 paired comparisons.

**Fact — where the time actually is.** A samply profile of the base (20 s, 18 304 compiles, 4 kHz) puts **42.8% of active samples inside `tiler-ir`** — far more than its 5% self-time suggests. The largest owned entry points are `kernel::lower::lower_scheduled_region` (12.5% inclusive), `program::builder::KernelProgramBuilder::build` (3.5%), `index::scalar::FrozenScalarRegistry::revalidate_region` (2.0%), and `index::builder::IndexRegionBuilder::build` (1.9%).

**Fact — the brief's `push_slice` lead does not point here.** `push_slice` is the single largest caller of `_platform_memmove` (25.9% of it, i.e. 3.8% of active), but making the framing primitives transparent shows its callers are almost entirely `tiler-compiler` encoders — `explain.rs`, `cover.rs`, `selection.rs`, `region.rs`, `request.rs`. `tiler-ir`'s own share of that memmove is about 3.7% of it. Likewise `_platform_memcmp` (10.0% of active) is 42% `ExplainWriter::push` and 23% `VerifiedRequestSubject::eq`. Both are outside this crate; recorded for the compiler-side tickets.

**Fact — `Vec<BlockData>::clone` (1.08% of active) is 65% `KernelBuilder::assemble`**, then `KernelProgramBuilder::build` (11%), `push_stage` (10%), and compiler-side `build_artifact_plan` (13%). `assemble(&self)` deep-cloned all four kernel arenas on every build *and* on every refinement check, while the verifier only ever read the assembled copy — the original was duplicated in order to be dropped. Replaced with a take/restore pair: `build` moves the arena out, verifies, and either publishes it or puts it back, preserving the recoverable-builder contract `KernelVerificationError` documents. `derive_canonical` now consumes its throwaway builder via `into_data`. Same for `KernelProgramBuilder`. **This change is the measured win.**

**Measurement — presizing is real but below this harness's floor.** A ceiling probe (capacity large enough that the three hot `encode_identity` encoders never regrow) read ~2% on one interleave and ~1% on a second; the exact-length version for kernel identity won 7 of 15 paired comparisons, i.e. a coin flip. It is kept on structural grounds rather than a timing claim: a kernel identity is **620 bytes** (from the assert), so building it from `Vec::new()` costs 8 allocations and 7 copies per encode, and it is encoded on every kernel build. `identity_encoded_len` mirrors the encoder arm for arm and measures each integer field with `size_of_val` rather than restating a width; `debug_assert_eq!` is the backstop, and its failure path was verified reachable by injecting an off-by-one (9 tests failed, `left: 620, right: 618`).

**Defect fixed — stale documentation.** `identity.rs` claimed the framing rule is enforced "in `scripts/check_workspace.py`, over every workspace member" with a `FRAMING_SITE_CITATIONS` table. That table went in `0b31488` and the Python gate in `e197176`; `scripts/` does not exist at HEAD. Three sites in `identity.rs` and one in `semantic/identity.rs` corrected to say nothing enforces it now, keeping the history because it records what a replacement would need. The two independent hand-spelled prefix assertions the text cites (`shape/env.rs:1218`, `tiler-compiler/src/feasibility.rs:1458`) were checked and do still exist.

**Not done, deliberately.** `verify_kernel` re-derives the whole canonical body on every kernel (`derive_canonical`, **4.9% of active**) and `derive_canonical` clones the entire `ScheduledRegion` into a builder field that only `build` reads — so that clone is never read on that path. Removing the clone needs either a lifetime on the public `KernelBuilder` or an `Option` field admitting an unreachable state; both are public-boundary or invariant changes for an estimated sub-0.5% win, so they are Tom's call rather than mine. The re-derivation itself is the refinement gate and was left alone: it is what makes a producer-authored kernel proven rather than trusted.

**Identity unchanged.** All 995 workspace tests plus 289 release-profile numerical tests pass; no golden moved. `make full` green.
