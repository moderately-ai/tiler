---
id: correct-the-capped-tree-partition-s-false-declared-workgroup-width-claim
title: Correct the capped tree partition s false declared-workgroup-width claim
status: todo
priority: p2
dependencies: []
related: [carry-the-tree-participant-cap-as-a-target-profile-row, bound-the-tree-cap-s-unmeasured-downward-direction]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, profiles]
---

`capped_tree_partition`'s doc comment claims something about every profile in the repository that is false, and a sibling constant a few lines away states the same fact correctly. The two disagree in the same file.

## Facts, coordinator-verified at `df5d23fc`

**Fact.** The doc comment asserts that the widest workgroup any profile in this repository **declares** is the qualified Apple9 entry's 1,024.

**Fact — no production profile declares it.** The only `declare_max_threads_per_workgroup(1_024, …)` call in `crates/tiler-build/src/metal_declaration.rs` sits inside `#[cfg(test)] mod tests`, which opens at the `#[cfg(test)]` attribute immediately preceding it. `FIRST_MACOS_APPLE9` declares workgroup threads as a `PreparedKernelPreflight` **query**, not a fact — and `declare_max_threads_per_workgroup_query` rejects a coexisting fact, so the two cannot both be present.

**Fact.** `MEASURED_TREE_PARTICIPANT_CAP`'s doc, a few lines away in the same file, states this correctly. So the file contains both the right and the wrong version of one claim.

**Inference — why it matters beyond tidiness.** The false version is load-bearing in an argument: it is offered as the reason a widened participant count stays inside the workgroup width. If the 1,024 is a test-only fixture rather than a declared profile fact, the bound rests on a query resolved at preflight, which is a different and later authority. The conclusion may well survive; the stated reason does not.

## What closes this

The claim restated to distinguish a **declared fact** from a **preflight query**, so a reader can tell which authority bounds the width and when it resolves. Prefer the phrasing `MEASURED_TREE_PARTICIPANT_CAP` already uses over inventing a third — two spellings in one file is what produced this.

**Cite by searchable anchor, not line number.** Note the failure mode `AGENTS.md` records and that bit this ticket's predecessor: an anchor spanning a line break greps as **absent**, and doc comments here wrap at 80 columns. The predecessor's durable fragment was `second target profile should carry its own row`; find an equivalent and **run its grep before committing to it**.

**Check the rest of this doc comment's inventory claims.** The worker that found this reported the comment leaks **three** downstream claims, of which this is the one it verified false — so the other two are unexamined, not clean. **Name the count you checked**, so a clean result is distinguishable from an unchecked one.

Do not change the rule, the constant, or any assertion — `bound-the-tree-cap-s-unmeasured-downward-direction` landed the selection logic and its evidence rungs, and this is a documentation repair on top of it. Do not edit `crates/tiler-build/**` (`implementation/build`, not this scope); read it to describe it correctly.
