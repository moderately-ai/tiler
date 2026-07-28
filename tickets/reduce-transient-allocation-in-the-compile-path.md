---
id: reduce-transient-allocation-in-the-compile-path
title: Reduce transient allocation in the compile path
status: done
priority: p2
dependencies: []
related: [remove-the-remaining-duplicate-work-in-the-planner]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
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

## Re-profile 2026-07-27 — the composition held, and one new target found

**This ticket's own instruction is "do not open this without re-profiling first." Done.** Apple M4 Max, `samply --rate 4000 --unstable-presymbolicate` over `hot_path_profile_loop`, 20 s, 26,880 compiles, 80,032 active samples, charged to the nearest enclosing non-generic frame of ours.

| where the active leaf lands | share |
| --- | --- |
| our own code | 48.2% |
| allocator | 31.9% |
| `_platform_memmove` / `_platform_memcmp` | 18.6% |

**The diffuse picture is unchanged**, which is itself the useful result: `identity::push_slice` 9.06%, `region::assemble` 5.48%, `region::canonical_member_order` 5.32%, `KernelBuilder::emit` 4.40%. Still a long tail, still no hotspot. The `push_slice` exact-`reserve` that measured −0.40% did not move its share, consistent with what that measurement already said — most of that 9% is the byte copy itself, not the reallocation. **Do not expect the remaining 50% to be recoverable**; a large part of it is copying that has to happen somewhere.

### One target that is not diffuse, and is not an allocation

`SelectedPlanIdentity::label` at 2.11% and `SelectedPlanIdentity::is_labelled` at 2.18% — **4.3% combined, and they digest the same bytes twice per alternative.**

**Fact.** `pipeline/planning.rs:482` builds `stable_id: plan.identity().label()`, and `pipeline/verify.rs:90` checks `plan.identity().is_labelled(&alternative.stable_id)`. Both compute `digest(&self.0)` over the *same* `SelectedPlanIdentity` bytes held in the same `ProgramAlternative`.

**Inference — caching the digest on the identity would not weaken the check, and this is worth stating because it looks like it would.** The check's power is that `stable_id` is a `String` that could have been tampered with; it is compared against a value derived from `self.0`, which the check does not read from `stable_id`. A digest cached at construction is still derived from `self.0`, so the comparison still refuses a forged `stable_id`. This is *not* the vacuity trap that `verify_portfolio` must avoid — that one re-derives a plan and must not be handed the plan, whereas this one re-hashes bytes it already holds and gains nothing from hashing them twice.

**Why it was not done here.** `SelectedPlanIdentity` derives `Eq`, `Ord`, `PartialEq`, and `PartialOrd`, and it is used as a map key and a sort key. A cache field must be excluded from every one of those or two identities with equal bytes could compare unequal — an identity defect far worse than 4% of a compile. That exclusion is mechanical but it is exactly the kind of change that should not be made with the profile still warm and the reasoning unwritten, so it is recorded rather than attempted.

**Next step is therefore specific:** cache the digest on `SelectedPlanIdentity` with the cache excluded from `Eq`/`Ord`/`Hash`, add a test that two identities built from equal bytes compare equal and hash equal, and measure on the M3. Expected ceiling is ~4%, which is larger than anything else this ticket has found and is a single contained change rather than a sweep.

## Measured 2026-07-27 — the digest cache lands, and a 4.3% profile share bought 0.3%

**Landed.** `SelectedPlanIdentity` now folds `digest(bytes)` once at construction. `label` and `is_labelled` read it instead of re-hashing, so the digest is computed once per plan rather than twice per alternative.

**Measured on the M3, ten interleaved pairs, min-of-200 per reading.** In the seven rounds that landed in the host's stable clock state, the candidate is faster in six: mean **681.9 µs → 680.0 µs, about −0.3%**.

**That is the finding, not the change.** The profile charged `label` 2.11% and `is_labelled` 2.18% of active self time — 4.3% combined — and removing one of the two digest computations returned **less than a tenth of that**. So a self-time share is not an available saving, and this is the second instance in this ticket: the `push_slice` exact-`reserve` was 9% of self time and bought 0.4%. Both times the share was real and the recoverable part was the small remainder — the byte copy in one case, the formatting, parsing, and comparison in the other.

**Anyone reading the 48/32/19 split above as headroom should read this paragraph first.** Two targeted changes against the two largest identified shares have together bought under 1%. The remaining diffuse allocation is very unlikely to behave differently, and a design that traded structure for it would be paying a real cost against a measured-small return.

**Kept anyway**, on two grounds that are not performance: computing one digest twice over identical bytes held in one struct is work with no purpose, and the change is guarded by a test for the hazard it introduces.

**The hazard, and the guard.** `SelectedPlanIdentity` is a map key and a sort key, so a cached field entering `Eq` or `Ord` would let two identities over equal bytes compare unequal — an identity defect far worse than the microseconds saved. Every comparison is therefore written out over the bytes alone rather than derived over a struct that now has two fields, and `the_cached_digest_stays_out_of_identity_comparison` constructs a deliberately corrupted cache and asserts the two still compare equal and order equal — while asserting their *labels* differ, so the field is proven live rather than dead weight. Verified to bite: letting the digest into `eq` fails it with "equality consulted the cached digest".
