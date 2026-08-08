---
id: correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix
title: Correct the coverage graph digest domain s eight-count and hyphenated artifact prefix
status: todo
priority: p1
dependencies: []
related: [repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check, cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, digest, documentation]
---

A doc comment in `tiler-ir` argues that no prefix relation exists between its coverage-graph domain and `tiler-artifact`'s. **The argument is wrong twice**, and it is a cross-crate correctness argument rather than decoration — which is why this is p1 despite being a comment.

## Facts, coordinator-verified at `2a7a6f08`

**Fact.** `crates/tiler-ir/src/index/refinement.rs`, on `COVERAGE_GRAPH_DIGEST_DOMAIN`, contains the phrase `of \`tiler-artifact\`'s open \`tiler.artifact-\``, in a sentence claiming this domain opens `tiler.ir.` and that all **eight** of `tiler-artifact`'s open `tiler.artifact-`, so no prefix relation exists.

**Fact — first error, the count.** The population is **eighteen**, not eight: `crates/tiler-artifact/src/domains.rs` declares `ENVELOPE: usize = 7`, `PROOF_SIDECAR: usize = 4`, `PROGRAM_IDENTITY: usize = 7`, summing to a `variant_count` over `GovernedDomain`. The "eight" is the pre-repair figure from a check that covered 8 of 18 while reading as complete.

**Fact — second error, and the one that matters more.** The prefix is `tiler.artifact` **without** the hyphen. `domains.rs` asserts `bytes.starts_with(b"tiler.artifact") || bytes.starts_with(b"tiler.proof-sidecar.")`. The hyphen is wrong because `tiler.artifact.route-requirement.v1` separates with a `.`, so a claim quantified over `tiler.artifact-` does not cover every domain it needs to. `docs/artifact-abi.md` already records the hyphen correction; this comment is reported to be the last site carrying the narrower spelling.

**Inference — why the second error is the live one.** The count being stale weakens the argument's authority. The **spelling** being narrow means the argument does not range over the domains it claims to, so the no-prefix conclusion is asserted over the wrong set. Whether the conclusion still holds is a separate question — I verified independently that all 153 pairs across the eighteen domains have zero prefix relations, so the *conclusion* is true today — but the comment's reasoning does not establish it.

## What closes this

The comment restated so its quantifier ranges over the real population with the real prefix, and so a reader can tell which side is authoritative when the numbers disagree. The sibling repair in `tiler-digest` took the useful shape: it named `tiler_artifact::domains::GovernedDomain` as the thing that *sizes* the population, so prose disagreeing with the type is settled at the type. **Prose cannot size itself from a type — a bare "eighteen" here will rot on exactly the schedule "eight" did.** Prefer naming the enumeration over restating its cardinality.

Cite by **searchable anchor**, not line number, and note the failure mode `AGENTS.md` records: an anchor copied from rendered output can be unsearchable in source when a line break or emphasis marker splits it. Doc comments here wrap at 80 columns, so that case is live for exactly the text you are editing — **run your anchor's grep before committing to it**.

**Do not edit `crates/tiler-artifact/**`** (`implementation/artifact`, not this scope) — read it to describe it correctly.

**Check the rest of this file's cross-crate claims while you are in it.** A comment that went stale on both a count and a spelling is unlikely to be the only one, and the sibling worker found exactly this pattern by reading rather than grepping. **Name the count you checked**, so a clean result is distinguishable from an unexamined one.
