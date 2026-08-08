---
id: admit-the-indirect-access-class-into-the-index-layer
title: Decide whether the index layer admits a data-dependent access class
status: in-progress
priority: p2
dependencies: []
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, emit-the-indirect-gather-on-metal, implement-index-domain-predicates]
scopes: [implementation/ir, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, indexing, gather, class-generic-capability, needs-tom]
claimed_from: todo
assignee: coord
lease_expires_at: 1786184079
---
## User-visible outcome

The question of whether an index region may name a second tensor as a coordinate source is answered, with its consequence for the direct-access verifier stated, rather than left as the boundary `tiler::gather-f32@1` currently fails closed at.

## Why this exists, and why it is a decision rather than an implementation

**Fact — corrected 2026-08-08 at `cb62784c`, see the audit below.** `tiler::gather-f32@1` is registered and reference-evaluated under [ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md), and no program containing one reaches a plan. `grep -n 'gather-f32' crates/tiler-compiler/src/policy.rs` returns **two** hits, not one: line 1147 opens the doc paragraph explaining the entry, and line 1183 is the `UNPLANNED_OPERATIONS` literal. That constant sits inside `#[cfg(test)] mod tests`, which opens at `policy.rs:1039`, so it is a **test-only inventory that documents the boundary rather than the boundary an occurrence fails closed at**. The boundary itself is the absence of a capability row plus `request.rs`'s `mismatch("operation-set")` in `recognize_epilogue_producer`, reached because `family_realizes_region_sequence` is false for a key with no registered `IndexRealizationLaw`. `grep -c 'gather' crates/tiler-compiler/src/fusion_legality.rs` returns `0` as stated — verified.

**Fact — the obstacle is the access record's shape, not a missing expression form.** Verified, with one stale citation. `crates/tiler-ir/src/index/model.rs:138`'s `AccessData` carries `tensor: u32`, a single tensor ordinal, so an access has nowhere to name a second tensor as a coordinate source. `IndexNode` at `model.rs:105` has five variants and every operand of every one is a literal, a domain-dimension ordinal, or one declared shape symbol. `IndexExprClass` at `model.rs:58` has three variants and no data-dependent member, and its `join` — at **`:90`**, not `:83`; line 83 is the opening `impl IndexExprClass {` — is an exhaustive match written so that adding a class is a build error. **This Fact is a correct diagnosis of inexpressibility and is not a direction for the remedy**, which is what ADR 0108 decides: see the audit below.

**Inference — this cannot be answered by implementing it. Half false.** [ADR 0046](../docs/decisions/0046-separate-logical-access-from-storage-addressing.md)'s consequences do admit indirect operations *on the condition* that the verifier for the initial direct-access language is not weakened, and the first clause is verified: every bounds proof, interval propagation, and totality argument in `crates/tiler-ir/src/index/builder/proof.rs` is written over expressions whose operands are literals, dimensions, and symbols. The second clause is false. `verify_accesses` is **already** unable to decide bounds for an admitted population and does not fail: it records `PendingIndexDomainDisposition::Unknown(IndexDomainUnknownReason::InsufficientFacts)` and the region builds, carrying the obligation out through `unknown_index_domain_predicates`. Undecidability is the verifier's designed fallback, not the obstacle. The real obstacle is one layer in: all three unknown reasons mean *dischargeable in principle by supplying more*, and a data-dependent bound is closable by none of them — so admitting the form without a fourth reason makes one type mean two incompatible things. That, not undecidability, is the ADR 0046 weakening.

## What this ticket must answer

- Whether a data-dependent coordinate enters as a *second tensor on the access* or as an *expression variant*, and what each does to `verify_accesses`' three routes (interval, cheap-predicate, exhaustive).
- What the bounds obligation becomes when it cannot be discharged: a retained `IndexDomainUnknownReason`, a required host-side pre-dispatch validation with a named publication boundary, or a refusal.
- Whether `LogicalAccess` gains a variant, and what a `FusionOperationRole` for the family would then discharge — today it deliberately has none, because `CoordinateRelation`'s contract asserts a discharge the index verifier cannot perform for a coordinate it cannot see.
- Whether ADR 0046 is amended, extended by a second subordinate record, or left untouched.

## Non-goals

Scatter, and any data-dependent output shape. Backend emission, which `emit-the-indirect-gather-on-metal` owns and which depends on this.

## Closes when

The question is answered with its verifier consequence stated and its ADR consequence landed, or it is deliberately deferred with a reconsideration trigger and a `## Trigger check log`.

## Outcome — 2026-08-08, answered in shape and deferred in timing

[ADR 0108](../docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md) (`proposed`) answers the four questions above. Acceptance is [`accept-adr-0108-data-dependent-index-coordinate-siting`](accept-adr-0108-data-dependent-index-coordinate-siting.md).

**Second tensor on the access, or expression variant? Expression variant, and the ticket's own framing pointed the wrong way.** All three costs run against the access-record route. The canonical encoder dispatches on an explicit per-form tag — `encode_index_node` and `structural_index_key` both write `1` for a constant through `5` for a modulo — so a sixth form changes the bytes of no region that lacks one, and `IndexExprData.class` is not encoded at all; `encode_region`'s access block, by contrast, writes `mode | tensor | domain | coordinates` with no optional slot and `encoded_region_len` charges a fixed five bytes per access to match, so any presence discriminator moves every region identity ever derived and forces `tiler.index-region.v11` to `v12`. `IndexDomainPredicate` names a `VerifiedIndexExprId` in both variants and `validate_index_domain_predicate` requires it to be one of the access's coordinates, so an expression states the gather's bound with no new predicate kind while an indirect *axis* has no handle to name. And `access.coordinates.iter().zip(shape.extents())` and its siblings appear in seven functions of `proof.rs` — `cheap_index_domain_predicates`, `interval_verdict`, `coordinates_are_bounded_dimensions`, `verify_access_exhaustively`, `write_is_permutation`, `write_partition_box`, `verify_partition_exhaustively` — each encoding "every axis has exactly one coordinate expression", which the access route falsifies at all seven at once and silently because `zip` truncates.

**What it does to the three routes.** It declines in every one and cannot contribute to a refutation either, so no direct-access coordinate's answer changes. Both branches of `interval_verdict` require `Some((min, max))`, and a data-dependent expression has no interval, so neither `interval_proved` nor `definitely_outside` is set. `coordinates_are_bounded_dimensions` and `write_is_permutation` require `IndexNode::Dimension`. `coordinates_are_evaluable` declines, withholding the finite walk before any budget is charged. `coordinate_offset_dimension` declines, so no partition rectangle is placed.

**What the bounds obligation becomes when it cannot be discharged.** A retained unknown — but **not** `InsufficientFacts`, which is the finding. All three existing reasons mean dischargeable in principle by supplying more; a data-dependent bound is closable by none of them in any environment. The narrowest admission that does not weaken the guarantee is a fourth `IndexDomainUnknownReason` naming undecidability in principle, and it arrives **with** the form rather than after it. Whether a pre-dispatch validation may discharge through the reserved-and-unfired `IndexDomainEvidence::Empirical` is left to Tom: it decides whether a runtime check enters a region's retained evidence and therefore its canonical identity, which is a claim about the program rather than about a run.

**Does `LogicalAccess` gain a variant, and what would a `FusionOperationRole` discharge?** Neither is answerable yet and answering either now would harden a shape ahead of its consumer. `LogicalAccess` (`crates/tiler-ir/src/schedule/model.rs`) is the *scheduled* layer's realized map, `#[non_exhaustive]`, so a variant lands additively — but every existing variant is the map of a region a physical plan selected, and no gather occurrence reaches a region. The ticket's premise about the fusion role is verified: `CoordinateRelation`'s contract says the one property it introduces is "an index-verifier concern, where the alias contract already admits aliasing reads and constrains writes", which is false of a coordinate the verifier cannot see.

**Is ADR 0046 amended?** No. It stays `accepted` with its rejection of tensor-data-derived indices intact, and ADR 0108 is a second subordinate record beside ADR 0107 — the third option the ticket named.

**Left to Tom, and why.** Whether to accept the shape and the deferral at all; whether a runtime validation may enter retained evidence; and the four public widenings the record shapes, which are a decided shape and an **undrafted** surface under ADR 0075 — none is written, so none is yet a labelled draft.

**Delivered in `implementation/ir`.** `crates/tiler-ir/src/index/builder/tests.rs` now pins the vocabulary counts from their types, so ADR 0107's negative is enforced rather than asserted: a sixth `IndexNode` form, a fourth `IndexExprClass` member, a fourth `IndexDomainUnknownReason`, a fourth `IndexDomainSoundProof`, a fifth `IndexDomainEvidence`, and a third `IndexDomainFactSource` each redden their own guard. The same commit stops three identity inventories in that file sizing their expectations from the lists they check.

**Out-of-scope defect found and filed.** ADR 0107's acceptance note claims the fail-closed boundary "is tested rather than asserted, with `classify` returning `None`". `grep -rn 'gather_f32_op' crates/tiler-compiler/` returns nothing and `crates/tiler-compiler/tests/` mentions gather zero times, so no test compiles a gather program or names the key against `classify`. A dated correction is on the record; [`pin-the-gather-request-boundary-refusal-with-a-test`](pin-the-gather-request-boundary-refusal-with-a-test.md) owns the repair, in `implementation/compiler`, which this ticket does not hold.

**Do not let this ticket's closure unblock `emit-the-indirect-gather-on-metal`.** That ticket is `blocked` on this one and is still correctly blocked: nothing admitted an access class, so there is still nothing for a backend to emit. Its dependency is re-pointed at the ADR 0108 acceptance ticket.
