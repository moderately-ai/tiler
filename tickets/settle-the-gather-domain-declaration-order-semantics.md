---
id: settle-the-gather-domain-declaration-order-semantics
title: Settle the gather domain declaration-order semantics
status: in-progress
priority: p1
dependencies: []
related: [admit-the-selected-data-dependent-index-representation, close-the-gather-review-findings-on-the-index-layer]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, identity, verification]
claimed_from: todo
assignee: worker-gatherorder
lease_expires_at: 1787428751
---
## User-visible outcome

A gather region whose result dimensions are declared in any order is either admitted by both validators or refused by both, with the rule stated where a caller can read it — instead of being admitted by `gather_read` and then refused by `build()`, or being unrepresentable in one of its two spellings.

## Why this exists

Filed 2026-08-22 from the post-chain multi-lens audit. **The audit executed this**, in a scratch crate outside the repository path-depending on `crates/tiler-ir`, and the coordinator re-verified the code claim at source at `56eecba1`.

**Fact — two validators read two different orders of the same set.** `crates/tiler-ir/src/index/builder.rs`, `prepare_gather_access`: the `for dimension in domain` loop pushes onto `declared` in **caller order** and compares `Shape::try_new(declared)` against `gather_result_shape(...)`; it then commits `domain: domain_set.iter().copied().collect()` — a `BTreeSet`, i.e. **ascending ordinal**, discarding caller order. `crates/tiler-ir/src/index/builder/proof.rs`, `verify_gather_access`: `for dimension in &gather.domain` rebuilds `declared` from the **stored sorted run** and compares against the same derived shape.

**Fact — the measured consequence.** For `out = gather(source=[4,5], index=[3], axis=1)` (result `[4,3]`), varying **only the order the two result dimensions are declared in**, with identical dimension set, extents, output shape, coordinates, and domain-slice contents:

```
declared=ascending domain=result-order -> Ok(())
declared=ascending domain=by-ordinal   -> Ok(())
declared=reversed  domain=result-order -> Err(build refused: GatherAccess { rule: DomainShape })
declared=reversed  domain=by-ordinal   -> Err(gather_read refused: GatherDomainShape {
                                              expected: Shape([4, 3]), actual: Shape([3, 4]) })
```

Row 3 is an **uncorrupted region admitted by `gather_read` and then refused by `build()`**. Under reversed declaration the region is **unrepresentable** — both spellings refuse, at different layers with different diagnostics.

**Fact — this refutes two stated design claims, both still in the tree.** `proof.rs`'s `#[cfg(test)] mod tests` doc — *not* the file's leading `//!` module doc, which is about diagnostic accumulation — says "Every gather that arrives through `gather_read` was already checked by `prepare_gather_access` against the same obligations, so no admitted region can make an arm fire — which is the design" (`grep -c` → 1). Row 3 is exactly that. And `verify_gather_access`'s doc says it "Revalidates one gather access against every obligation `gather_read` enforces" — it enforces a *different* obligation on the same field.

**Fact — the constraint is undocumented.** `GatherDomainShape` silently constrains the **declaration order** of result dimensions. Nothing in the builder docs, ADR 0108, or the error type says so, and no other index-layer rule behaves this way: for a direct access the domain is a pure set and positional meaning is carried by `coordinates`.

**This is not a soundness hole.** Both directions fail closed; the audit checked the fourth quadrant and found no combination where the two validators agree on a wrong pairing. It is a representability and stated-invariant defect, not a wrong answer.

**Why the suite missed it.** Every fixture in `crates/tiler-ir/tests/index_gather.rs`, `crates/tiler-reference/tests/index_region_oracle.rs`, and `proof.rs`'s `admitted_gather()` declares result dimensions in ascending order, so caller order and ordinal order coincide.

**Correction — 2026-08-22: the tag value is right, the collision framing was wrong.** I wrote that a bounds proof at `0x03` would collide with `TAG_SCALAR_BROADCAST` in the access-map space. It would not: `push_bounds_proof` and `push_logical_access` write into **disjoint frames**, and tag values already repeat across them — `TAG_LINEAR_IDENTITY = 0x01` and `TAG_COVERAGE_PADDED = 0x01` coexist in `crates/tiler-ir/src/schedule/model.rs`, which documents the overlap as deliberate (anchor `overlap deliberately`, wrapped across two `///` lines). **`0x13` remains correct**, on the `0x1X` bounds-proof family-run convention rather than on collision avoidance. Use that ground; do not restate the collision claim.

## Fact audit — `worker-gatherorder`, 2026-08-22, at base `e7b6026f`

Every Fact above re-read at source at this base, and the four-quadrant probe re-executed in a scratch crate outside the repository path-depending on `crates/tiler-ir` (pinning `rust-toolchain.toml`, without which the crate does not build on stable).

| Fact | Verdict |
| --- | --- |
| Two validators read two different orders of the same set | **Verified.** `for dimension in domain` at `builder.rs:1761`, `declared.push(extent)` at 1773, the gather commit `domain: domain_set.iter().copied().collect()` at 1809; `for dimension in &gather.domain` at `proof.rs:600`. |
| The measured four-quadrant consequence | **Verified.** Reproduced row for row, including both error spellings. |
| Refutes two stated design claims, both still in the tree | **Verified.** Both anchors `grep -c` → 1, at `proof.rs:1606` and `proof.rs:513`. Imprecise on *where*: the first is the `mod tests` doc, repaired above. |
| The constraint is undocumented | **Verified.** `gather_read`, `IndexBuildError::GatherDomainShape`, and `GatherAccessRule::DomainShape` were each read in full and none mentioned order. |
| Not a soundness hole | **Verified** for the four quadrants. |
| Why the suite missed it | **Verified**, after repairing the path: the oracle is `crates/tiler-reference/tests/index_region_oracle.rs`. Every other `gather_read` call site in the workspace passes a rank-one domain, where the two orders coincide trivially. |

**Imprecise — the second `domain_set.iter().copied().collect()`.** `builder.rs:1890` is `prepare_access`, the *direct* access commit. It is not a second instance of this defect: `prepare_access` builds only `domain_set` and never a `declared` shape, so it imposes no order rule at all. That asymmetry is evidence *for* the repair below, and the ticket's "no other index-layer rule behaves this way" is correct.

**Re-derived — the realization-law input, and why it does not settle the question.** Confirmed independently: `declare_parallel_domain` maps `shape.extents()` in order, `context.dimension` appends, and in `realize_gather` it is the first declaration, so ordinals follow result order by construction. But `gather_read` is a **public** builder method, so the defect is reachable from the public boundary whatever the law layer does. The law layer's inability to produce a non-ascending domain is why nothing caught this, not a reason the question is moot.

**New Fact the ticket did not have, and it decides the choice.** A gather's domain order is *already* excluded from identity, deliberately and in three separate places, all in `crates/tiler-ir/src/index/builder/compact.rs`: `remap_domain` sorts after remapping (anchor `remapped.sort_unstable()`), the alpha access key's gather arm sorts before hashing (anchor `tiler.index.access-gather-read.alpha.v1`), and the canonical access ordering sorts its `remapped_domain`. So the identity boundary has always treated this field as a set. That makes option (1) provably identity-preserving, and shows option (2) was never a field-type change — it would have had to edit all three encoders.

## The decision this turns on — do not pick the smaller diff

The stored `domain` is **a set at rest**, so `prepare_gather_access` is currently checking a property the record does not retain. Two coherent repairs:

1. **Order-insensitive over the extent multiset.** Keeps `domain` a set; keeps two spellings of one meaning at **one** identity. The audit recommends this and declined to decide it inside an audit.
2. **Make `GatherReadAccessData.domain` order-carrying.** Gives two spellings of one meaning **two identities** — an identity-domain question, with a step and pin consequences.

Derive the choice; state which and why. **If you conclude (2), stop and report** — that is an identity-domain change and belongs to Tom, not to this repair.

## Input from the realization-law layer — 2026-08-22, and it simplifies the decision

`worker-gather-remainder` was asked whether its layer observes this ordering. Its answer, reported with the reasoning: **it cannot, and neither repair option moves any identity above the index layer.** `declare_parallel_domain` maps over result extents in order and `push_dimension` assigns strictly increasing ordinals, so caller order and ascending order coincide **by construction** at that layer — not by fixture convention. An order-carrying `domain` would therefore store exactly the ascending run the `BTreeSet` stores today.

**What that does to the choice.** The identity-domain objection to option (2) is weaker than this ticket assumed: no realization-law, kernel-program, or artifact identity moves under either option. Re-derive that yourself rather than taking it — it is one worker's reading of one layer, and the index-layer identity question (two spellings, one or two identities) is unaffected by it. But do not carry the "option (2) necessarily steps an identity domain" framing into the decision without re-checking it.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict; re-run the four-quadrant probe yourself rather than trusting the table.
- Make the two validators agree, by the derived choice. Repair **both** stale doc claims — the module-doc invariant and `verify_gather_access`'s "every obligation" line — so neither states something the code does not do.
- If the rule survives in any form, **document it where a caller reads it**, not only where it is enforced.
- **Perturb the subject and quote the failure text**: reverse the stored domain in `prepare_gather_access` and show the admission tests redden. Add a fixture declaring result dimensions in **descending** order, since the whole suite currently coincides on ascending.
- **No identity may move** under choice (1). Recompute pins and say so; if any moves, stop and report.

## Non-goals

Anything past the index layer — the schedule relation, KIR wall, and compiler consumers belong to `admit-the-selected-data-dependent-index-representation`'s remainder. Re-opening ADR 0108's accepted surface.

## Closes when

Both validators agree on every declaration order, the choice is derived and stated, both stale doc claims are repaired, a descending-order fixture exists, the perturbation is quoted, and no identity has moved without an explicit stop-and-report.

## Outcome — `worker-gatherorder`, 2026-08-22

**Choice: option (1), order-insensitive over the extent multiset.** Not a stop-and-report; option (2) was eliminated on evidence rather than deferred.

The derivation, beyond the ticket's framing. The gather domain carries no positional meaning anywhere in the layer: the axis pairing is carried by `axis` and the two coordinate runs, coordinate scope is checked with `is_subset`, point count is a product, and the identity encoders sort the run three times over. The ordering `prepare_gather_access` enforced was therefore pure syntax — it constrained the order a caller *listed* dimensions in and nothing else, since `resolve_run` pairs coordinates to the domain by set membership and never by position. Dropping it loses no semantic content; the multiset comparison is the strongest statement a set at rest can support.

A third option the ticket did not list, and why it loses: **canonicalize both validators onto the stored ascending run**. It makes the two agree and moves no identity, but it makes a legal region *unrepresentable* — a caller who declared the extent-3 dimension first can never author that gather in either spelling — and refuses with a shape disagreement against a domain that does match the derived shape. It renames the defect rather than repairing it.

Implementation: one shared predicate, `gather_domain_carries_result_extents`, called from both `prepare_gather_access` and `verify_gather_access`. The two read the domain from opposite ends — the caller's slice and the committed sorted run — so two spellings of one comparison are two chances to disagree, which is exactly how this arose.

**No identity moved.** Nothing stored changes for any currently-admitted region; the change is only to the admission predicate. `cargo nextest run --workspace` and `cargo test --workspace --doc` both pass unchanged, including the artifact, reference-oracle, and determinism suites that carry the identity pins.

### Perturbations, each on the subject

| # | Subject broken | Result |
| --- | --- | --- |
| 1 | Authoring check restored to `declared_shape != derived` | `every_declaration_order_...` fails on **`reversed=true by-ordinal=true`** |
| 2 | Revalidation check restored to `declared_shape != derived` | same test fails on **`reversed=true by-ordinal=false`** |
| 3 | Predicate made vacuous | all three refusal tests fail, in both validators |
| 4 | Draft commit `collect()` → `rev().collect()` | **nothing reddens, workspace-wide** — see below |
| 5 | `remap_domain`'s `sort_unstable()` → `reverse()` | `a_gather_commits_its_domain_in_ascending_ordinal_order` fails |

Perturbations 1 and 2 redden *different rows*, which is what shows each half of the repair is independently load-bearing rather than one masking the other.

Perturbation 4 is a finding, not a pass. Reversing the draft's stored gather domain left all 3894 workspace tests green, because the three downstream sorts absorb it — the draft's order is genuinely unobservable. The invariant that *is* load-bearing is compaction's sort, since `encode_gather_bounds_identity` frames `subject.domain` in stored order; perturbation 5 isolates it, and `a_gather_commits_its_domain_in_ascending_ordinal_order` was added to hold it.

### Out of scope, and worth its own ticket

**A gather's coordinate expressions get no bounds obligation against their axes.** `verify_accesses` routes gathers away from `cheap_index_domain_predicates` (anchor `Only a direct access reaches the machinery below`), and `derive_gather_index_bounds` reasons only about the loaded U32 index values against the source-axis extent. Nothing appears to check that a *source* or *index* coordinate expression stays within the axis it addresses — so a coordinate spelled as a linear combination over two domain dimensions could exceed its axis. This predates the repair and is untouched by it; the ordered domain rule never constrained it either, since coordinates are paired to the domain by set membership. **Not verified by execution** — it is a reading of the routing, and the first thing a follow-up should do is try to build such a region and see whether something else refuses it.
