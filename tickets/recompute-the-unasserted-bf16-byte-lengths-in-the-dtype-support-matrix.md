---
id: recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix
title: Recompute the unasserted BF16 byte lengths in the dtype support matrix
status: todo
priority: p2
dependencies: []
related: [carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [dtype, artifact, identity, documentation]
---

`docs/dtype-support.md` states five BF16 artifact byte lengths that **no test asserts**, so nothing goes red when they drift. The derived index-arithmetic step grew the artifact envelope, which makes them stale in a way the gate cannot see.

## Facts, verified 2026-08-08 by the coordinator at the merge that grew the envelope

**Fact.** All five numbers live in a single paragraph of `docs/dtype-support.md`, the one whose first sentence is the searchable anchor *"BF16's physical-carrier, ABI, and kernel-vocabulary cells moved on 2026-08-05"*. They are `97,060` (the carrier-only forged pair's round-trip), and `90,806` / `45,457` / `73,556` from the pure-BF16 producer clause anchored at *"carried a pure-BF16 program from semantic construction through verified coverage"*. A fifth, `36,832`, appears in the same region.

**Fact.** `grep -rn "97060\|90806\|45457\|73556\|36832" crates/ prototypes/` returns **nothing**. No test, golden, or pin asserts any of them. This is the whole reason the ticket exists: a documented measurement with no assertion behind it is a claim that decays silently.

**Fact.** The envelope grew by exactly five bytes in `carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`: `FIXED_CONTENT_BYTES` moved `65_308 -> 65_313`, and that worker decomposed the +5 as five insertions of the literal `0x01` — one entry-row `resources` plus four embedded kernel identities. `tiler.artifact-program` moved `v15 -> v16` and `MANIFEST_SCHEMA` `(15,0) -> (16,0)` in the same step.

**Inference, and the trap this ticket exists to stop.** It is tempting to add five to each number and call it recomputed. **Do not.** The +5 was measured on one specific envelope; these five figures describe different artifacts (a forged carrier-only pair, a pure-BF16 producer artifact, its identity, and an F32 twin), and an identity length in particular has no reason to track a content-byte delta at all. Each number must be regenerated from the construction that produced it.

## What closes this

Either each figure recomputed on the merged tree from its own construction and restated with the date it moved, **or** — better, and the reason this is `contracts/navigation` rather than a doc typo — a decision that prose should not carry unasserted byte counts at all. If a number is worth stating it is worth pinning; if it is not worth pinning, stating it invites exactly this drift. A worker choosing the second path should say what replaces the figures and confirm nothing else in the document cites them.

Check the surrounding paragraphs for the same shape before closing: this is unlikely to be the only place a measurement was written into prose without an assertion behind it. Report the census either way, so "none found" is distinguishable from "did not look".
