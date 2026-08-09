---
id: prove-partition-coverage-for-symbolic-extents
title: Prove partition coverage for symbolic extents
status: deferred
priority: p2
dependencies: []
related: [admit-sub-range-write-domains-for-unequal-partitions, lower-the-concatenate-occurrence-through-partitioned-writes]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, shapes, deferred]
---
## User-visible outcome

A partition whose boundary or member extents are symbolic is proved or refuted through the shape environment, instead of falling to an enumeration that cannot run and being reported as unproved ownership.

## Why this exists

**Fact — both partition mechanisms are literal-only, at their first line.** `decide_partition_by_interval` (`crates/tiler-ir/src/index/builder/proof.rs`) opens with `boundary_element_count(shape)` and returns `PartitionVerdict::Enumerate` on `None`; `write_partition_box` opens with `boundary_extents(shape)` and returns `None` on the same. Both resolve through `determined`, which yields a value only for a one-point extent interval. `partition_walk_elements` then needs the same `boundary_element_count`, so the enumeration route returns `None` too, and every root of the output is reported `WriteOwnershipNotProven`.

**Fact — a single-root write already has a symbolic path, and it is not this one.** `write_is_permutation` asks `extents_proved_equal`, so `y[i] = f(i)` over a symbolic `n` into a boundary sized `n` is proved from the symbol alone. A partition needs symbolic *addition* — that the members' extents sum to the boundary's — which no current predicate asks of the environment.

**Inference — the offsets close the same gap from the other side.** `coordinate_offset_dimension` admits an offset only through `to_u64`, so a member's displacement is a literal. A partition of a symbolic axis generally needs a symbolic offset (member *k* starts at the sum of the extents before it), so admitting symbolic extents without symbolic offsets would admit only partitions whose cut points are all literal.

**Fact — the case is reachable and is already spelled in a pinned occurrence.** [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) pins `[8, 0, 128]` joined with `[8, T, 128]`. If `T` reaches the emitted region as a `ShapeEnv` symbol rather than a literal, that occurrence lands exactly here and is refused; if the lowering binds `T` to a literal per occurrence, it does not.

**Fact — nothing regressed.** This gap predates [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md); that ticket changed which *literal* partitions are expressible and left every symbolic path exactly as it found it. The refusal is fail-closed in both directions: an unproved partition is reported unproved, never admitted.

## What the work is

Decide what the shape environment must be asked. The candidate obligation is that the members' spans sum to the boundary extent along the partitioned axis and agree on every other, which is an additive relation over sourced extents rather than the equality `extents_proved_equal` already answers. Establish whether `ShapeEnv` can carry it before designing around it — an additive relation the environment cannot decide makes this a refusal to state, not a proof to build.

Whatever is admitted, re-derive the two obligations rather than assuming they carry, exactly as the dependency did: per-root injectivity, and the rectangle-volume identity that reads a root's volume as its element count. The volume identity in particular is currently arithmetic over `u64`, and a symbolic volume is a different object.

## Explicit non-goals

- The literal case, which works and is not reopened.
- Widening the coordinate-expression language. A symbolic offset would be a new sourced-extent-valued displacement, not a new `IndexNode` form.

## Closes when

Either a symbolic partition is proved and its evidence names the environment facts it used, or the case is refused under a diagnostic that says the environment could not decide it — with the choice derived rather than defaulted, and the trigger log below closed.

## Graph maintenance

- `implementation/ir` alone: both mechanisms, `coordinate_offset_dimension`, and the sourced-extent resolvers are in `crates/tiler-ir/`.
- Filed at `deferred` rather than `todo` because its activation depends on a fact not yet established — whether the concatenate lowering's pinned occurrence carries `T` symbolically — and a deferral offered as dispatchable work costs a reader a full body to re-derive what the log already says.

## Trigger check log

- 2026-08-06 — **not fired.** The trigger is a live consumer emitting a partition over a symbolic boundary extent. No governed region is symbolic at all: the compiler declares every domain dimension and boundary through the literal constructors, so no emitted region can reach the symbolic path, let alone a symbolic partition. The only candidate consumer is the concatenate lowering, which is `todo` and has emitted nothing. Reproduce: `grep -n 'symbolic_dimension\|sourced_tensor' crates/tiler-compiler/src/governed.rs` — empty on this date; a non-empty result is the first thing that could fire this.
- 2026-08-09 — **not fired.** Symbolic and sourced index construction is now well exercised inside `tiler-ir`, but no compiler construction site calls `symbolic_dimension` or `sourced_tensor`, and no governed concatenate lowering emits a symbolic partition. The IR vocabulary's existence is not the trigger; the first compiler-produced symbolic partition remains the trigger.
