---
id: lower-the-concatenate-occurrence-through-partitioned-writes
title: Lower the concatenate occurrence through partitioned writes
status: review
priority: p1
dependencies: [admit-a-partitioned-write-ownership-contract, admit-sub-range-write-domains-for-unequal-partitions]
related: [scope-the-concatenate-fusion-role-and-lowering, lower-a-two-region-occurrence-through-one-index-access-capability, admit-the-structural-families-into-the-scheduled-region-vocabulary, reach-a-verified-kernel-through-the-structural-families, evaluate-write-roots-over-their-own-domains-in-the-oracle, accept-the-partitioned-concatenate-realization-law]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, lowering, indexing, concatenate]
claimed_from: todo
assignee: agent-concat-lower
lease_expires_at: 1786084358
---
## User-visible outcome

A program containing `tiler::concatenate-f32@1` resolves an index-access lowering capability and emits a verified region, so the family stops being a registered identity no plan can consume.

## Why this exists

**Fact — the family has no lowering and no realization law.** `governed_index_access_capabilities` (`crates/tiler-compiler/src/governed.rs:222-334`) registers nine capabilities and none covers a concatenation, so `resolve_index_access` (`crates/tiler-compiler/src/capability.rs:1115-1144`) fails with `MissingCapability`. The semantic registry's index-realization law table (`crates/tiler-ir/src/semantic/registry.rs:2387-2437`) registers twelve laws and none is a concatenation, so refinement fails with `MissingRealizationLaw`. (Nine and `2386-2420` until `admit-a-bf16-index-realization-law-and-refinement-contract` added the three `bf16` rows; what this fact claims — that no registered row is a concatenation — is unchanged by that step.)

**Fact — the fork is decided.** [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) eliminated the piecewise read and selected the partitioned write. The piecewise read is insufficient rather than merely expensive: the case selects a different operand *tensor* per coordinate, which `AccessData`'s single `tensor` field does not express and ADR 0046's map-level piecewise reservation does not reserve, and the read-both-and-select spelling is refused by the bounds proof and needs a predicate dtype `RQ-OP-03` owns. Q-SHAPE-006 therefore does not fire on this family.

**Fact — the coordinate arithmetic already exists.** Operand *k*'s write coordinate on the concatenated axis is `t + offset_k` for a literal `offset_k`, and `IndexNode::LinearCombination` (`crates/tiler-ir/src/index/model.rs:97-100`) carries a literal exact-integer constant. The expression stays `Affine`; nothing widens the coordinate-expression language.

**Fact — the arity forces seven registrations.** `resolve_index_access` keys on the exact `(family, operation, signature)` triple and `LoweringSignature` carries the exact operand and result type lists, while the family admits two through eight operands (`crates/tiler-ir/src/semantic/concatenate.rs:67`, `:79`). Each admitted arity needs its own registered capability, exactly as `MAX_CONCATENATE_OPERANDS`'s own doc comment explains for the reference provider.

## What the work is

Register the index-access capabilities and the matching `IndexRealizationLaw` variant, so the compiler-side emission and the semantic-side law produce the identical region and the refinement comparison is meaningful rather than one-sided.

Emit one write root per operand over the single output, each total over its own contiguous partition of the concatenated axis, with the read being the identity over that operand. The `emitted` scalar-operation list is deliberately empty for the same reason the reindex row's is (`governed.rs:293-298`): a concatenation applies no scalar operation, so declaring one would make refinement's containment check pass over an operation the region never emits.

Decide, and record, whether the region carries one iteration domain partitioned by coordinate or several — the dependency's contract fixes which of these is admitted, and this ticket must not invent a second answer.

Cover the zero-extent operand explicitly. `concatenate_result_shape` admits an operand empty on the concatenated axis and it contributes no coordinate, which is the pinned prefill occurrence (`[8, 0, 128]` joined with `[8, T, 128]`), so the partition set must handle an empty partition without that being a coverage hole.

Confirm whether the pinned explain digest moves, and if it does, execute the identity step completely.

## Explicit non-goals

- The fusion role, which is [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) and is independent of this chain.
- The request-boundary spelling that would let a program containing a concatenate be recognized at all. `request.rs`'s recognizer admits three elementwise keys, a reduction, and a contraction, and the structural families are refused under `operation-set` — that wall is [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md)'s and this ticket does not move it.
- The copy-free windowed realization. That is an M6/M7 physical candidate composing with this lowering, not part of it.
- Any second semantic family for an inner-axis concatenate. The record checked and confirmed the region is axis-uniform; the contiguous-window difference lives in the storage half.

## Closes when

Seven capabilities and one realization law are registered, a concatenate occurrence at each admitted arity emits a region that verifies and refines against the law, the zero-extent operand case is exercised, and a deliberate perturbation of one partition's offset is shown to fail the ownership proof.

## Graph maintenance

- `implementation/ir` is declared alongside `implementation/compiler` because the `IndexRealizationLaw` variant and its registration in the semantic registry live in `crates/tiler-ir/`, and a compiler-side emission with no matching law would make refinement fail closed on every occurrence it lowered.
- Depends on [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) because there is no proof form for the region this ticket emits until that contract exists.

## Outcome — 2026-08-07, at `a86fddc2`

*Every measurement below was taken on `a86fddc2`, the commit carrying the whole source change; this record and the acceptance node land on top of it so the hash they cite is the one they describe.*

**The family resolves a lowering and refines against a law, so the two-sided comparison is meaningful rather than one-sided.** Seven index-access capabilities and one `IndexRealizationLaw::PartitionedConcatenate` variant (encoding tag **12**) are registered; a `tiler::concatenate-f32@1` occurrence at every admitted arity emits a region that verifies and refines. The refinement comparison is exact canonical identity, so the compiler emission and the law realization producing the same bytes is what every refinement assertion below *is*, not a separate claim.

**Fact — the counts this ticket's premise cited, re-verified on the base `007a1ef9` rather than inherited.** `governed_index_access_capabilities` registered **ten** capabilities (not nine — the strict-affine U4 decode landed since) and the semantic registry's law table registered **fourteen** rows (not twelve — the three `bf16` rows plus the softmax). What the premise claimed is unchanged by either: none was a concatenation, `resolve_index_access` failed `MissingCapability`, and refinement failed `MissingRealizationLaw`. The profile now ships **17** capabilities and **15** law rows.

**The iteration-domain decision, with its contract citation.** The region carries **several iteration domains, one per write root** — not one domain partitioned by coordinate. This is [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md)'s Outcome verbatim: *"The subset-of-parallel-dimensions construct is admitted; the sub-range annotation is eliminated. A write's domain may be any subset of the region's parallel dimensions, so each root carries its own iteration space and the region's parallel set is their union. This is the answer [this ticket] is told not to invent a second one to: several iteration domains, one per root."* No second answer was invented, and the reason the first was forced is the pinned occurrence itself: under one shared domain every root owns the same element count, so `[8, 0, 128]` joined with `[8, T, 128]` has no spelling at all.

Within that construct one shape decision remained, and it is recorded rather than assumed: **the non-concatenated axes share one parallel dimension each, and only the concatenated axis gets a private dimension per root.** Root *k*'s domain is therefore `{d_a : a ≠ axis} ∪ {own_k}`. The family admits an occurrence only when every operand agrees on the non-concatenated axes, so one dimension per such axis is the region's own statement of that agreement; a private copy per root would put `n · (rank − 1)` dimensions into the canonical identity that are pairwise equal by construction. The counter-argument — that "one domain per root" could be read as demanding the roots' dimension sets be *disjoint* — is stated in the acceptance node as the thing to object to; subsets of one parallel set may share members, which is what `write_partition_box` quantifies over (`crates/tiler-ir/src/index/builder/proof.rs`: every quantifier ranges over `access.domain`, none over the region's parallel set).

**Fact — the emitted region, per root.** Read the operand at `(d_0, …, own_k, …, d_{r−1})` unchanged; write at the same coordinates with the concatenated axis replaced by `own_k + offset_k`, `offset_k` the prefix sum of the preceding operands' extents there. `linear_combination` normalizes `1 · d` back to `d`, so root zero's coordinate is a bare `IndexNode::Dimension` and the rest are one-term `LinearCombination`s with a literal constant — the expression stays `Affine` and nothing widens the coordinate-expression language, exactly as the record predicted. The partition is keyed by **operand**, not by distinct input: `concat(x, x)` is one input boundary and two members at two offsets, because operand order is semantic.

**Fact — seven capabilities, and seven *provider identities*, which the ticket did not anticipate.** `resolve_index_access` keys on the exact `(family, operation, signature)` triple, so arity 2..=8 is seven registrations. But `LoweringCapabilityRegistryBuilder::register` also refuses a second signature under one `(family, operation, provider)` triple as `ConflatedCapabilityKey` (`crates/tiler-compiler/src/capability.rs:1036-1049`), so seven arities under one provider identity do not register at all. They are registered as `tiler::governed-index-access.concatenate-f32.arity-N@1` for N in 2..=8. Those strings are durable — they enter the lowering-registry identity and therefore the explain request subject — and are flagged in the acceptance node.

**Fact — the emitted scalar list is empty, and the region is observed reaching nothing.** Declared `Vec::new()` for the reason the reindex row's is (`governed.rs`): a concatenation applies no scalar operation, so declaring one would make refinement's containment check pass over an operation the region never emits. Verified rather than declared: `the_governed_concatenate_lowering_refines_at_every_admitted_arity` asserts `scalar_authority().reached_operations().is_empty()` at every arity, and `the_concatenate_law_realizes_one_root_per_operand_over_one_output` asserts the law's own region has zero scalar operations.

**Measurement — arity evidence, all seven walked rather than sampled.** `the_governed_concatenate_lowering_refines_at_every_admitted_arity` builds an occurrence at each arity 2..=8 with operand extents `1, 2, 3, …` on the concatenated axis, so the partition is **unequal at every arity** — an equal-share partition is the case the shared-domain contract could already express and would not exercise the relaxation this lowering rests on. At each arity: one stage, one result binding per member, one operand binding per operand, an empty reached scalar set, and every member carrying `WriteOwnershipProofView::PartitionMember { joint: JointPartitionProofView::Interval }`. Interval reasoning is asserted rather than merely the presence of a proof: a fallback to the joint enumeration would mean the displaced-dimension placement vocabulary stopped recognizing `t + offset`.

**Measurement — the zero-extent operand, at the pinned prefill shape.** `the_pinned_prefill_concatenation_admits_its_zero_extent_operand` joins `[8, 0, 128]` with `[8, 5, 128]` on axis 1 at a literal `T = 5`. Two result bindings, both `PartitionMember { Interval }`, and exactly one zero-extent dimension in the region — the empty operand's own. An empty partition is not a coverage hole: the root's rectangle is empty, its volume is zero, and the disjointness test separates it from every sibling under the guard `admit-sub-range-write-domains-for-unequal-partitions` added for exactly this. The law-side analogue puts the empty operand in the *middle* of three (`&[[2,3,4], [2,0,4], [2,5,4]]`), which is the strictly-interior placement that guard exists for. `T` is literal because a semantic occurrence carries static extents only; the symbolic analogue stays fail-closed under [`prove-partition-coverage-for-symbolic-extents`](prove-partition-coverage-for-symbolic-extents.md).

**Measurement — the perturbation, watched failing in both directions the obligation can break while every rectangle stays inside the boundary.** `DisplacedConcatenatePartition` emits byte-for-byte what the governed provider emits except for the second root's offset. Over extents `3` and `5` into a boundary of `8`:

| Offset | Observed |
|---|---|
| `2` (governed: `3`) | `RefinementError::Build { stage: 0, .. }` carrying `IndexRegionDiagnostic::OutputPartitionRangesOverlap` — the members share `[2, 3)` |
| `4` (governed: `3`) | `RefinementError::Build` — `[3, 4)` is left bare and the member runs one element past the boundary |
| `3` (the control) | refines, two result bindings |

The control is what makes the two refusals claims about the displacement rather than about the fixture. A second, independent perturbation lives in `crates/tiler-ir/tests/index_region.rs` from the contract landing (`overlapping_unequal_partitions_refuse_despite_an_exact_volume_sum`), which is the same refusal reached without a lowering.

**Measurement — every existing law identity is unchanged, byte for byte.** `cargo nextest run --workspace` → **2935 passed, 0 failed, 7 skipped**, which exercises `the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` (three pinned chain identities with their exact lengths), `an_existing_law_payload_is_unchanged_by_the_appended_tag` (the multiply row's exact bytes), the four earlier append-only tag tests, the 20 pinned explain digests, and the `tiler-metal` shader identity goldens. `the_concatenate_law_tag_is_append_only_and_distinct` asserts tag 12 against all fourteen prior rows and asserts each of theirs is in `1..=11`; it also asserts the coincidence that `REINDEX_MAPPING_ATTRIBUTE` and `CONCATENATE_AXIS_ATTRIBUTE` are the same number, so the reader sees that the *tag* is what separates those two rows and not the payload.

**Measurement — the pin movement, enumerated old→new and recomputed on this tree.** Exactly one pinned literal moved:

| Pin | Old | New |
|---|---|---|
| `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` request qualifier (`crates/tiler-compiler/src/explain.rs`) | `7bba54bcb59ec2cc` | `0aa252e0bfa16451` |

Recomputed by that pin's own documented mechanics — running the named test on this tree and taking the reported `left` — never copied. It moves because the request subject folds both the semantic-realization authority (whose count-prefixed law sidecar gains a row) and the lowering-capability registry identity (which gains seven capabilities). Surveyed rather than assumed: the distinct maximal `[0-9a-f]` runs of length exactly 16 and exactly 64 over `crates/**/*.rs`, taken on base `007a1ef9` (`git archive 007a1ef9 crates | tar -x` into a scratch tree) and on this branch and diffed, are **7 and 36 on both, with the sixty-four-hex set byte-identical and the sixteen-hex set differing in exactly the one line above**. So no region identity, chain identity, artifact digest, cache subject, or Metal golden moved; the semantic snapshot identity is computed without the sidecar, which is what keeps them all.

**One comment corrected in place.** `UNPLANNED_OPERATIONS`'s doc (`crates/tiler-compiler/src/policy.rs`) said the concatenate is unplanned because "the family has no index-access lowering and no kernel construct". The first clause is now false. Rewritten to current truth: it is unplanned because no kernel construct writes a partitioned output and the request boundary refuses the family under `operation-set`; it now holds a fusion role *and* a lowering, and neither makes it plannable because both are answerable for a family performing no arithmetic without any target being asked anything. The family stays in the list and the two guarding tests pass unchanged.

**Non-goals, respected and confirmed.** No fusion role was touched (it already landed). **The request-boundary wall does not move and nothing landed here moves it**: `LogicalAccess` (`crates/tiler-ir/src/schedule/model.rs:244-`) has no variant expressing a partitioned write, so a concatenate occurrence still has no scheduled region to be built into and is still refused under `operation-set` — that wall remains [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md)'s. `crates/tiler-compiler/src/request.rs` and `frontier.rs` contain no reference to the family and are untouched by this diff. No windowed realization, no second family.

**One thing worth recording that this ticket did not have to build.** The region this lowering emits is independently evaluable: `evaluate-write-roots-over-their-own-domains-in-the-oracle` landed, and `crates/tiler-reference/tests/index_region_oracle.rs`'s `a_zero_extent_write_root_contributes_nothing_and_empties_no_sibling` is written against "the one the concatenate lowering emits at its pinned occurrence". `crates/tiler-reference` is outside this ticket's scopes so nothing was added there, but the oracle path exists rather than being owed.

## Public boundary — draft, for Tom

Not self-accepted. One public item changed in `crates/tiler-ir/src/index/`, tested but not approved: the additive `IndexRealizationLaw::PartitionedConcatenate { axis_attribute }` variant on the `#[non_exhaustive]` enum, its `const fn concatenate_f32()` constructor, encoding tag 12, and the standard registration for `tiler::concatenate-f32@1`. Filed with its exact surface, exclusions, and the choices worth objecting to as [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md), parked at `awaiting-decision`. Nothing releases on it; the variant is labelled a draft at its definition.

## Commands run

`cargo fmt --all --check`; `cargo check -p tiler-ir -p tiler-compiler --all-targets`; `cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir -p tiler-compiler --no-deps`; `cargo nextest run --workspace` (2935 passed, 0 failed, 7 skipped); `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard tkt/lower-the-concatenate-occurrence-through-partitioned-writes --format json`; `make full`.

## Scope

Every edit is under `crates/tiler-ir/**` (`implementation/ir`), `crates/tiler-compiler/**` (`implementation/compiler`), or `tickets/**` (`project/tickets`). Five source files and two tickets; nothing under `crates/tiler-reference/`, `docs/`, or any other crate.
