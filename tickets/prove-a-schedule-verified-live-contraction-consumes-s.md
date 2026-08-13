---
id: prove-a-schedule-verified-live-contraction-consumes-s
title: Prove a schedule-verified live contraction consumes S
status: review
priority: p1
dependencies: [accept-the-live-extent-operand-public-surface]
related: [admit-live-extent-operands-to-payload-indexing]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, contraction, extents, identity]
claimed_from: todo
assignee: worker-prove-live-contraction
lease_expires_at: 1786662274
---
## User-visible outcome

A bounded direct contraction consumes the live input-axis extent `S` as its contributor-loop bound and performs exactly `S` loads, without baking `S` into the schedule or kernel identity.

## Exact gap

**Fact at `6ea5de7cd866edd296e39310cdb94163ca5c1a4c` — repaired.** The draft-`9a8f53c9` sentence that `ReductionTopology::LiveContraction` and `LogicalAccess::LiveRowMajor` "exist as labelled draft variants" is false at this base. Tom accepted both on 2026-08-13; the rustdoc now says `**Accepted public surface.** Tom accepted this exact spelling on` in `crates/tiler-ir/src/schedule/model.rs` for each, under tags `TAG_LIVE_ROW_MAJOR` `0x09` and `TAG_REDUCTION_LIVE_CONTRACTION` `0x38`. `0x08` remains parametric broadcast, `0x37` remains `CooperativeContraction`, `0x36` stays reserved. Reproduce: `rg -n "Accepted public surface|TAG_LIVE_ROW_MAJOR|TAG_REDUCTION_LIVE_CONTRACTION|0x36 stays reserved" crates/tiler-ir/src/schedule/model.rs`.

**Verified at the same base, before this work.** The parent ticket's second required evidence — a schedule-verified contraction that consumes `S` and changes the oracle when the bound is replaced — was absent. `verify_contraction` admitted only `ReductionTopology::Contraction`; a `LiveContraction` region failed `NumericalOrAccessRefinement`. No live-contraction fixture existed in `crates/tiler-ir/src/kernel/tests.rs`. Reproduce: `rg -n "fn verify_contraction|LiveContraction" crates/tiler-ir/src/schedule/builder.rs crates/tiler-ir/src/kernel/tests.rs`.

**Verified.** The working construction path is `ScheduledRegionBuilder` + `lower_scheduled_region`, not `compile()`.

## Required work

- After [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md) closes, use the accepted `LiveContraction` / `LiveRowMajor` spelling.
- A bounded direct contraction consumes `S` as its contributor-loop bound and performs exactly `S` loads.
- Replacing the bound value by the neighbouring extent changes the oracle. Baking either value changes identity and fails the no-specialization assertion.
- Omitted, swapped-symbol, wrong-axis, late-phase, overflowing, and unused live operands fail at the named layer. Remove each new check and watch its negative fail.

## Required evidence

- Schedule verification plus lowered kernel for at least two neighbouring `S` values, with load-count oracles that move and identities that do not.
- Subject perturbations for the named refusal classes, each with quoted failure text.
- Targeted IR and compiler tests, identity blast radius, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Artifact envelope, payload, and pipeline execution. `compile()` through strategy selection. Widening past bounded unsigned input-axis extents available by `LiveDevicePreflight`.

## Closes when

The accepted contraction spelling is schedule-verified, the neighbouring-extent oracle moves, baked neighbours change identity, and every named negative is fail-capable.

## Outcome

Schedule-verified `ReductionTopology::LiveContraction` + `LogicalAccess::ContractionOperand` (accepted spelling; no second carrier). Free indices and the output stay static; the scalar program's contracted shape is empty rather than a specialized `S`; the named input axis is the inner trip count. `lower_scheduled_region` emits `serial_loop_range(1, S)` seeded at the first product. Addressing is `free * S + contributor`.

### Two-S oracles and identity

`kernel::tests::a_live_contraction_consumes_s_as_the_contributor_bound_without_baking_it` and `crates/tiler-compiler/tests/live_contraction_consumes_s.rs`:

- `S = 14` performs exactly 14 loads of the named live input; `S = 15` performs exactly 15. The oracle is `seed + (S-1) * body` over that buffer (one seed load, one load in the `1..S` loop).
- Re-lowering the same live region keeps kernel identity. Two live scheduled regions share identity. Baking `S = 14` or `S = 15` via `ReductionTopology::Contraction` changes identity; the two baked neighbours differ from each other.
- Kernel domain stays `tiler.kernel.v7`. Schedule domain pin `the_staging_relation_step_moves_only_the_domain_separator` still holds (`tiler.schedule.v5`). Costing a live contraction declines rather than baking `S` (`measured_cost::tests::a_live_contraction_work_span_declines_rather_than_baking_s`).

### Subject perturbations

Each new check was removed and its negative failed:

| Class | Layer | With the check | Check removed |
| --- | --- | --- | --- |
| swapped-symbol | schedule `verify_live_contraction` live-axis equality | `numerical-or-access-refinement` | `naming the free axis as the live bound must fail: VerifiedScheduledRegion { ... reduction: LiveContraction { live_input: InputOrdinal(0), live_axis: Axis(0), ... } }` |
| wrong-axis (schedule) | same | `numerical-or-access-refinement` | `axis 5 is outside the live input's rank: VerifiedScheduledRegion { ... live_axis: Axis(5) ... }` |
| wrong-axis (kernel) | `KernelBuilder::declare_input_extent` | `InputExtentWrongAxis` | `axis 5 is outside the live input's rank: KernelInputExtentId { owner: KernelBuilderId(1), index: 0 }` |
| late-phase | `verify_live_contributor_loop` requires the `InputExtent` in block 0 | `input-extent-contract` | `late-phase live operand: [BodyRefinement]` |
| unused | `verify_input_extents` unused slot | `unused-input-extent` | `unused live operand: [OutputCoverage]` |
| omitted | `verify_input_extents` declared == scheduled | `input-extent-contract` | `omitted live operand: [ReductionContract]` / `left: ReductionContract` `right: InputExtentContract` |
| overflowing | `element_count` → `ShapeProductOverflow` | `shape-product-overflow` | `a [u64::MAX, 2] output product must overflow: VerifiedScheduledRegion { ... iteration_shape: Shape([Extent(18446744073709551615), Extent(2)]) ... }` |

### Checks

- `cargo test -p tiler-ir --lib -- live_contraction`
- `cargo test -p tiler-ir --lib -- kernel::tests`
- `cargo test -p tiler-compiler --lib -- a_live_contraction_work_span`
- `cargo test -p tiler-compiler --test live_contraction_consumes_s`
- `cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir -p tiler-compiler --no-deps`
- `tkt lint`, `git diff --check`, `tkt guard tkt/prove-a-schedule-verified-live-contraction-consumes-s --format json`, `make full`

Parent `admit-live-extent-operands-to-payload-indexing` stays `review`. Not merged.
