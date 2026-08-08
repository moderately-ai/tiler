---
id: correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places
title: Correct the every-ir-domain-opens-tiler-ir premise in two places
status: in-progress
priority: p2
dependencies: []
related: [correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [identity, digest, documentation]
claimed_from: todo
assignee: coord
lease_expires_at: 1786182999
---

A no-prefix argument rests on the premise that every domain the shared IR admits opens `tiler.ir.`. **Most do not.** The conclusion survives; the reasoning does not.

## Facts

**Reported by the worker that repaired the sibling comment, not coordinator-verified — check each before editing.** `crates/tiler-artifact/src/domains.rs` states the premise, and the same sentence appears in `docs/artifact-abi.md` (scope `contracts/artifacts`, **not this one** — report it, do not reach). **24 of 38** `tiler-ir` domain strings do not open `tiler.ir.`, including `EXPR_DOMAIN = b"tiler.artifact-program.abi-expr.v1\0"`, which opens the **same prefix** as `tiler-artifact`'s program-identity container.

**Reported: no collision results.** All 684 cross-crate pairs are clean, so this is a reasoning defect rather than a correctness one. **Verify that independently** — the sibling re-derived it from source literals rather than inheriting it, and found the cross-crate set is the one the argument actually ranges over.

## Why it matters despite no live collision

An argument that reaches the right answer by a false route will keep reaching it only by luck. The premise says the two crates' namespaces are disjoint by construction; they are not, and `EXPR_DOMAIN` is the counterexample sitting inside the very prefix the argument claims separates them. The next domain added under that prefix has nothing but coincidence keeping it clean.

## What closes this

The premise restated so it says what actually separates the domains — the sibling's repair rests on the **first differing byte after `tiler.`** rather than on a prefix quantifier, which is the shape to follow.

**Do not restate a count.** Prose cannot size itself from a type; delegate to `GovernedDomain` as the sibling did. A bare "eighteen" or "thirty-eight" here rots on exactly the schedule the retired "eight" did — and note that "8 of 18", which several tickets have repeated, **appears in no source**: the module header says the retired check covered **8 of 11**. Take counts from source, not from other tickets.

**Establish whether this was ever true** with `git log -S` before choosing a treatment: a claim true when written is dated beside, one never true is substituted with the retired wording quoted. That is repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim stays greppable, so say inline that a later hit lands inside your note.

**Cite by searchable anchor and run its grep before committing to it.** The sibling had an anchor fail because it spanned an 80-column break and caught it before shipping; doc comments here wrap.

Check the neighbouring claims and **name the count** — every sweep of these files this week found more than it was sent for.
