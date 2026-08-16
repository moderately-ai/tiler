---
id: refuse-mixed-pointwise-live-row-major-access-relations-before-lowering
title: Refuse mixed pointwise live-row-major access relations before lowering
status: in-progress
priority: p0
dependencies: []
related: [decide-the-source-bound-live-row-major-access-surface, admit-symbolic-extents-through-schedule-formation]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [defect, correctness, schedule, kernel]
claimed_from: todo
assignee: fix_mixed_live_maps
lease_expires_at: 1786921403
---
## User-visible outcome

A hand-built pointwise schedule cannot mix static `LinearIdentity` accesses with
live-loop `LiveRowMajor` accesses and then receive a verified kernel that applies
the live offset to every buffer. The current one-read live fixture and all-static
pointwise schedules remain valid; mixed addressing refuses during intrinsic
schedule verification, before lowering can mint a wrong or out-of-bounds load.

## Exact-base Fact audit — 2026-08-16, `e2522345d571d5088ce47039e4399b7247e7bc47`

1. **Fact — intrinsic pointwise verification admits the mixed population.**
   `crates/tiler-ir/src/schedule/builder.rs`, anchor
   `LogicalAccess::LinearIdentity | LogicalAccess::LiveRowMajor { .. }`, admits
   either map independently for every read and the final write. It never proves
   that all accesses share the live loop's address relation. The shared
   `verify_pointwise_region` gate covers both F32 and BF16.
2. **Fact — one live read switches the whole pointwise body to live emission.**
   `crates/tiler-ir/src/kernel/lower.rs`, anchor
   `.any(|addressing| matches!(addressing, ReadAddressing::LiveRowMajor { .. }))`,
   chooses `emit_live_row_major` when any read is live. That emitter derives one
   `offset = row * columns + col`, then at anchor
   `for (buffer, read) in read_buffers.iter().zip(plan.reads)` loads every read
   buffer at that offset and stores the result at the same offset. It does not
   consult each read's `ReadAddressing` inside the loop.
3. **Fact — canonical kernel verification repeats rather than detects the wrong
   derivation.** `crates/tiler-ir/src/kernel/verify.rs`, anchor
   `if data != &canonical`, compares against the body re-derived by the same
   lowering. `verify_effects`, anchor `EffectKind::Load { bounds, ordinary }`,
   checks that each load carries some declared bounds witness but does not prove
   its dynamic offset realizes that access's map. `verify_reduction`, anchor
   `let live_row_major = reads.iter().any`, permits the one loop on the same
   any-read predicate. A final `LiveRowMajor` write is explicitly allowed at
   loop depth one.
4. **Fact — the high-severity population can silently address outside a
   verified static read.** Start from a two-input pointwise region with outer
   shape `[2]`; give read 0 `LinearIdentity` with a two-element linear-range
   proof, read 1 and the final write `LiveRowMajor { inner_axis: Axis(1) }` with
   the existing zero-element live proof convention. Intrinsic construction
   accepts it and canonical lowering uses the live offset for both reads. At
   runtime `N = 14`, row 1 / column 0 loads read 0 at element 14 although its
   verified static range ends at element 1. The bounds witness survives while
   its promised access relation is not what the kernel executes.
5. **Fact — the current positive fixture already states the safe local
   boundary.** `crates/tiler-ir/src/kernel/tests.rs`, anchor
   `fn live_row_major_region`, gives its only read and final write the same live
   relation and proves both element accesses are inside the live loop at anchor
   `every_live_row_major_element_access_is_inside_its_live_range`. This repair
   need not decide which input supplies a future multi-input symbolic extent.

Reproduce the source derivation:

```sh
rg -n -C 12 'LogicalAccess::LinearIdentity \| LogicalAccess::LiveRowMajor|fn pointwise_read_map_is_admissible' crates/tiler-ir/src/schedule/builder.rs
rg -n -C 14 'any\(\|addressing\| matches!\(addressing, ReadAddressing::LiveRowMajor|fn emit_live_row_major|for \(buffer, read\) in read_buffers' crates/tiler-ir/src/kernel/lower.rs
rg -n -C 12 'if data != &canonical|EffectKind::Load|let live_row_major = reads.iter\(\).any' crates/tiler-ir/src/kernel/verify.rs
rg -n 'fn live_row_major_region|every_live_row_major_element_access_is_inside_its_live_range' crates/tiler-ir/src/kernel/tests.rs
```

## Required work

- In the shared intrinsic pointwise verifier, require the complete access list
  to choose one addressing regime: all reads and the final write are static, or
  every read and the final write carry `LiveRowMajor` with the same inner axis.
  Refuse the mixed list before a `VerifiedScheduledRegion` exists. Do not repair
  it in lowering by treating `LinearIdentity` as live or by choosing one access
  as authority.
- This fail-closed repair uses the existing
  `ScheduledRegionDiagnostic::NumericalOrAccessRefinement` access-map
  disagreement path while the accepted public map has no source relation. That
  is a temporary truthful statement that one current pointwise access map and
  the region-wide live execution topology disagree; it must not invent the
  source access or add the unaccepted source-bound public surface. After Tom
  accepts a survivor in
  [`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md),
  its implementation replaces this temporary mixed-regime refusal with the
  selected dedicated `LiveRowMajorSourceRule::ConsumerMissingRelation` rule.
- Keep all currently verified schedules' canonical bytes unchanged. This is a
  narrowing of an invalid population, not an encoder reinterpretation or an
  identity migration; capture the all-static and existing all-live bytes before
  and after to prove that claim.
- Do not widen kernel verification to trust the canonical lowering as a bounds
  proof. The schedule relation must make the emitted offset legal before kernel
  construction.

## Evidence and negative controls

- Add the exact two-read reproducer from Fact 4 and first prove it reaches a
  verified kernel on the parent commit. After the repair, perturb only read 0
  from live to `LinearIdentity`; schedule construction must fail before
  lowering, and the test must quote the owning stable diagnostic.
- Independently perturb only the final write from live to `LinearIdentity` and
  require the same fail-closed schedule boundary. This proves output coverage
  cannot be the only check stopping the mixed population.
- Keep all accesses live on the same axis and require the existing live fixture
  to verify and lower. Keep all accesses `LinearIdentity` and require its exact
  schedule and kernel identity bytes to remain unchanged.
- Independently change one live relation's axis. It must refuse rather than let
  `emit_live_row_major` select the first extent and apply that stride to every
  buffer.
- Count the F32 and BF16 pointwise verifier entry points that reach the shared
  gate. Perturb one example of each width so a width-specific bypass cannot look
  green.

## Worker evidence — 2026-08-16, exact base `e2522345d571d5088ce47039e4399b7247e7bc47`

- The source-first audit above was rerun against the claimed base. All five
  Facts remain verified; only the audit heading's earlier base hash was stale,
  and this ticket now names the exact claimed base. The purpose did not change.
- Before the repair, the exact two-read mixed subject built and lowered to a
  verified F32 kernel and a verified BF16 kernel. Their buffer element counts
  were `[2, 0, 0]`: the first read retained its two-element static proof while
  the body selected the live-row-major loop.
- With the final refusal assertions present but the shared predicate absent,
  the read, write, and axis tests each failed with
  `a mixed live-row-major access list must fail intrinsically: VerifiedScheduledRegion`.
  Independently weakening the final predicate's read, write, and axis clause
  reproduced that same red failure for the corresponding exact test alone.
- `static_and_same_axis_live_pointwise_identities_remain_exact` pins the complete
  pre-repair one-read all-live schedule and kernel bytes and reuses the existing
  complete all-static kernel pin. The schedule builder's existing
  `the_strict_f32_region_has_its_recorded_canonical_identity` test pins the
  all-static schedule bytes. All four comparisons pass after the repair.
- The two width-specific dispatch arms remain exactly the F32 and BF16 arms that
  reach `verify_pointwise_region`; the mixed-read test perturbs and checks both.

## Graph and closing conditions

This ticket is an independent existing-correctness prerequisite of
[`admit-symbolic-extents-through-schedule-formation`](admit-symbolic-extents-through-schedule-formation.md).
It does not depend on the source-surface decision and therefore creates no
cycle. Its current-surface all-static/all-live refusal neither introduces a
source marker/reference nor ranks the decision ticket's marker, total-local,
and hybrid candidates. The symbolic implementation remains blocked on both:
this ticket first removes current silent wrong code; the decision then
authorizes the exact source relation and dedicated refusal that replaces the
temporary narrow gate.

Close only when the mixed relation refuses intrinsically, every subject
perturbation shows its failure text, all existing valid schedule/kernel bytes
are unchanged, targeted IR tests plus rustdoc and Clippy pass, and `tkt lint`,
`make citations`, `git diff --check`, and exact-base guard are green.
