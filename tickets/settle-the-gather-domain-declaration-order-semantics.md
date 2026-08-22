---
id: settle-the-gather-domain-declaration-order-semantics
title: Settle the gather domain declaration-order semantics
status: todo
priority: p1
dependencies: []
related: [admit-the-selected-data-dependent-index-representation, close-the-gather-review-findings-on-the-index-layer]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, identity, verification]
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

**Fact — this refutes two stated design claims, both still in the tree.** `proof.rs`'s module doc says "Every gather that arrives through `gather_read` was already checked by `prepare_gather_access` against the same obligations, so no admitted region can make an arm fire — which is the design" (`grep -c` → 1). Row 3 is exactly that. And `verify_gather_access`'s doc says it "Revalidates one gather access against every obligation `gather_read` enforces" — it enforces a *different* obligation on the same field.

**Fact — the constraint is undocumented.** `GatherDomainShape` silently constrains the **declaration order** of result dimensions. Nothing in the builder docs, ADR 0108, or the error type says so, and no other index-layer rule behaves this way: for a direct access the domain is a pure set and positional meaning is carried by `coordinates`.

**This is not a soundness hole.** Both directions fail closed; the audit checked the fourth quadrant and found no combination where the two validators agree on a wrong pairing. It is a representability and stated-invariant defect, not a wrong answer.

**Why the suite missed it.** Every fixture in `tests/index_gather.rs`, `tests/index_region_oracle.rs`, and `proof.rs`'s `admitted_gather()` declares result dimensions in ascending order, so caller order and ordinal order coincide.

**Correction — 2026-08-22: the tag value is right, the collision framing was wrong.** I wrote that a bounds proof at `0x03` would collide with `TAG_SCALAR_BROADCAST` in the access-map space. It would not: `push_bounds_proof` and `push_logical_access` write into **disjoint frames**, and tag values already repeat across them — `TAG_LINEAR_IDENTITY = 0x01` and `TAG_COVERAGE_PADDED = 0x01` coexist in `crates/tiler-ir/src/schedule/model.rs`, which documents the overlap as deliberate (anchor `overlap deliberately`, wrapped across two `///` lines). **`0x13` remains correct**, on the `0x1X` bounds-proof family-run convention rather than on collision avoidance. Use that ground; do not restate the collision claim.

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
