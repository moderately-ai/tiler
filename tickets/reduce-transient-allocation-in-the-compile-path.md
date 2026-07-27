---
id: reduce-transient-allocation-in-the-compile-path
title: Reduce transient allocation in the compile path
status: todo
priority: p2
dependencies: []
related: [remove-the-remaining-duplicate-work-in-the-planner]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [performance, compiler]
---
Split from `remove-the-remaining-duplicate-work-in-the-planner`, which closed because its own premise was spent: after the region-graph fix, every named item of duplicated computation measured under 2.5%, and the largest single cost turned out to be a correctness check that must stay. What is left is a different problem and needs its own title, or the same profile gets re-derived under a heading that no longer describes it.

## Fact — where the compile's time actually goes

Apple M4 Max, `samply --rate 4000 --unstable-presymbolicate` over `hot_path_profile_loop`, 20 s, 28,032 compiles, 79,649 active samples. Each sample charged to the nearest enclosing frame of ours, skipping generic `alloc::`/`core::` code (`RawVecInner::finish_grow` is monomorphized into our binary and masks its caller otherwise).

| where the active leaf lands | share |
| --- | --- |
| our own code | 47.1% |
| allocator (`malloc`/`free`/`realloc`) | 33.3% |
| `_platform_memmove` / `_platform_memcmp` | 18.0% |

**Just over half the compile is allocating and copying**, and it is diffuse. The top charged frames are `identity::push_slice` 8.9%, `region::assemble` 5.3%, `region::canonical_member_order` 5.2%, `KernelBuilder::emit` 4.6%, `enumerate_complete_plans` 3.1% — a long tail rather than a hotspot.

## Inference — this is a lifetime problem, not a loop-count problem

The parent ticket's fixes all had the shape "compute this pure function once instead of N times". That well is dry. These allocations are not recomputations; they are transient buffers built, digested or compared, and dropped, once each. Reducing them means changing how long a buffer lives and who owns it, not how often a function runs.

Two structural facts make that tractable rather than speculative:

- Canonical encodings terminate in `Arc<[u8]>` (for example `RegionContentIdentity`), which copies to an exact size at the end. Over-reserving the transient `Vec` therefore wastes nothing in the retained value — a reserve or a reused scratch buffer is free of the usual capacity-waste objection.
- `verify_portfolio` re-derives the whole downstream pipeline by design (23.3% of active time, and it must stay — see the parent). Every allocation removed from a shared building block is therefore paid back twice per alternative, once in the build and once in the verify.

## Measurement boundary

The one systemic change already made — an exact `reserve` inside `push_slice`, covering all twenty-odd encoders at once — measured **−0.40%** (12/12 interleaved pairs, M3, min-of-200). That is the calibration for this ticket: `push_slice` was 8.9% of self time and halving its growth events bought 0.4%, because most of that 8.9% is the byte copy itself and not the reallocation. **Do not assume the 51% is recoverable.** A large share of it is copying that has to happen somewhere.

That is also why this is filed rather than actioned: the cheap systemic fix is done, and everything past it needs a design (buffer reuse, arena, or borrowed encoding) whose cost is real and whose payoff is not yet bounded.

## Closes when

Either a bounded design reduces transient allocation in the shared encoders with the compile time measured before and after on the M3 and artifact identity byte-identical; or the measurement shows the remaining traffic is irreducible copying and that is recorded here with the evidence, so the 51% figure stops reading as an available win.

Do not open this without re-profiling first. The parent ticket had to be re-profiled twice because its item list was written from code reading, and both times the profile disagreed with the ordering.
