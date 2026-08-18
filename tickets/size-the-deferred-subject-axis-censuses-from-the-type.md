---
id: size-the-deferred-subject-axis-censuses-from-the-type
title: Size the deferred-subject axis censuses from the type
status: in-progress
priority: p2
dependencies: [carry-subgroup-width-through-exact-prepared-entry-equality]
related: [generalize-deferred-target-provenance-beyond-capability-axes]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-axis-census
lease_expires_at: 1787066399
---
## User-visible outcome

A widened `CapabilityAxis` vocabulary cannot silently shrink the test populations that guard the deferred-subject canonical ordering and the append-only proposal-identity encoding.

## Why this exists — filed 2026-08-18 from the multi-lens audit of the deferred-provenance landing (merge `3dc10348`)

**Fact.** `crates/tiler-compiler/src/target/feasibility.rs` (test module) and `crates/tiler-compiler/src/frontier.rs` (test module `deferred_subject_identity`) each carry a hand-written `CANONICAL_AXES: [CapabilityAxis; 7]`, and the frontier module additionally hand-writes an axis→relation map (`capability_relation`). The properties they guard are load-bearing for identity: `capability_axis_descriptor_tags_ascend_with_the_derived_order` proves the canonical-key sort preserves the pre-enum `(phase, axis)` order, and `capability_records_keep_their_pre_enum_bytes_exactly` / `atomic_deferred_subjects_are_structurally_disjoint_from_every_axis` prove byte preservation and escape disjointness — all feeding `encode_proposal_identity`'s iteration order and bytes.

**Fact.** An eighth `CapabilityAxis` variant added to the enum but omitted from these arrays passes every one of those checks silently — the AGENTS.md "population that silently shrinks" failure mode. Mitigation exists but is indirect: `tag()` and `key()` are exhaustive matches, so a new variant forces *some* edit, but nothing forces the census arrays to grow.

**Fact.** `tiler-compiler` already uses `core::mem::variant_count` (its `lib.rs` sizes `request::BudgetResource::ALL` from the enum), so the sizing pattern and any required feature gate are already present in this crate. (The audit report claimed the crate had no `variant_count` use; that claim was false — verified 2026-08-18 by `grep -rn variant_count crates/tiler-compiler/`.)

## Required content

- Assert `CANONICAL_AXES.len() == core::mem::variant_count::<CapabilityAxis>()` (or size the arrays from a single shared, type-sized source) in both test modules, so a widened vocabulary is a red test at the census rather than a silently shrunk population. Deduplicate the two hand-written arrays and the hand-written relation map if one shared spelling is cleanly reachable from both modules.
- Perturb the subject, not the assertion: demonstrate with a scratch eighth variant that the hardened census goes red, and quote the failure text.
- Secondary, same files: `capability_records_keep_their_pre_enum_bytes_exactly` builds its "legacy" bytes from the new encoder's own expression (`predicate.requirement().required()`), while the pre-enum encoder wrote `predicate.required().value()`; the two agree only because `Quantity::value()` is the identity unwrap for every axis. Either derive the legacy bytes independently of the new accessor chain or document at the test that it is a regression pin against future structural change, not an independent pre-enum control.

## Closes when

Both censuses are type-sized (or floor-asserted with a printed census), the scratch-variant perturbation's failure text is recorded, and the byte-control's independence status is either repaired or truthfully documented.
