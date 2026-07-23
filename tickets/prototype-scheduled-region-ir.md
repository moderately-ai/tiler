---
id: prototype-scheduled-region-ir
title: Implement checked scheduled-region IR
status: done
priority: p0
dependencies: [prototype-semantic-index-refinement]
related: [scheduled-region-model]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, verifier]
---
Implement reviewed target-neutral ScheduledRegion and KernelSchedule builders,
canonical identities, and intrinsic verifier. Validate axes, work ownership,
loops, vector/tail organization, staging, reduction topology, synchronization,
launch expressions, and specialization before target feasibility is queried.
No cost or provider callback can repair malformed schedule intent.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Module-placement note (coordinator, 2026-07-23)

Per ADR 0070 the target-neutral scheduled-region IR belongs in `tiler-ir` as its own
module, `tiler_ir::schedule`, alongside the existing `tiler_ir::index`. Build it there
rather than growing `tiler-compiler/src/physical.rs`, which currently holds the
bounded serial-Sum prototype's schedule/kernel/program types in one ~1,300-line
file. Extract only your layer's concern, keep the serial-Sum path green, and
leave the shared `physical.rs` no larger than you found it (ideally smaller).
This keeps the crate's public surface modular so later layer work can proceed
without one monolith as a shared merge point; it is architecture ADR 0070
already mandates, not extra scope.

## Outcome

- Added the target-neutral scheduled-region IR as its own module
  `tiler_ir::schedule` (`crates/tiler-ir/src/schedule/{mod,handles,numerics,model,builder,error}.rs`),
  following the ADR 0071 checked-builder discipline: a public transactional
  `ScheduledRegionBuilder` with private storage, a consuming `build()` that runs
  whole-region intrinsic verification, and an opaque `VerifiedScheduledRegion`
  exposing read-only meaning plus a canonical identity. It mirrors the structure
  and error boundaries of `tiler_ir::index` (insertion-time `ScheduleBuildError`;
  consuming `ScheduledRegionBuildError` carrying `ScheduledRegionDiagnostic`s and
  the recoverable builder).
- **Intrinsic verifier (Fact, tested):** proves launch/domain coverage, tail
  policy, exactly one read + one owning write, output-ownership uniqueness
  (race-freedom), bounds-proof refinement of each access, scalar-program /
  reduction-topology / access-map agreement, reduction contributor/order
  legality, and zero-domain behaviour, and derives `ResourceRequirements`. It
  runs before any feasibility query; no cost or provider callback participates.
- **Placement / line delta (Fact):** extracted the schedule concern out of
  `crates/tiler-compiler/src/physical.rs`, which shrank 1932 -> 1640 lines
  (-292). `physical.rs` now delegates intrinsic verification to the new module
  and retains only compiler-owned refinements: semantic-occurrence binding,
  request-subject binding, target feasibility, and structured-kernel lowering.
- **Serial-Sum path stays green / feasibility seam preserved (Fact):** the
  compiler `VerifiedScheduledRegion` now wraps the tiler-ir verified region and
  adds `semantic_members` + `target_profile_key` + `request_subject`. The merged
  `tiler_compiler::feasibility` authority is unchanged and still consumes the
  `ResourceRequirements` derived by the schedule verifier via `assess_region`.
  Full repository gate and `tkt guard` pass.
- **Canonical identity (Fact, tested):** `CanonicalScheduledRegionIdentity` is a
  deterministic big-endian byte encoding of the normalized region (no HashMap;
  transient `RegionId` excluded). Equivalent normalized regions built from
  different planning ordinals share identity; distinct content differs.
- **Boundaries flagged for Tom's review** (see report): the new public
  `tiler_ir::schedule` surface; descriptor structs carry public read fields
  (opacity is enforced at `VerifiedScheduledRegion`, not the leaf descriptors);
  numerical vocabulary (`NumericalPermission`/`SubnormalMode`) moved from
  `tiler-compiler::request` to `tiler-ir::schedule` and is re-exported.
- **Deferred:** `VerifiedScheduledRegion` carries its own bounded index-region
  description rather than referencing a `tiler_ir::index::VerifiedIndexRegion`;
  unifying the two index-region representations is future work. ADR 0071's
  implementation-boundary note ("Schedule ... builders remain unimplemented") is
  now partially superseded and should be updated by a documentation ticket
  (docs are outside this ticket's scopes).
