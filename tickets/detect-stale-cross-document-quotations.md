---
id: detect-stale-cross-document-quotations
title: Detect stale verbatim cross-document quotations
status: todo
priority: p3
dependencies: []
related: [record-distributivity-in-the-navigation-contracts, settle-contraction-chain-distributivity-permission, qualify-contraction-association-reassociation-permission]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, tooling]
---
The documentation corpus quotes itself verbatim across documents, and nothing checks that a quoted string still exists in the document it is attributed to. `scripts/docs.py` validates frontmatter, catalog freshness, and links; it does not validate quotations, so a link can resolve while the sentence it supports has been rewritten.

**Measurement — the class is real, not hypothetical.** At `412ceae`, three verbatim quotations of `docs/compiler/optimizer.md` in the Milestone 6 framing of `docs/roadmap.md` no longer matched any text in that file, all rotted by a single day of merges (`qualify-contraction-association-reassociation-permission` and `settle-contraction-chain-distributivity-permission` rewrote the logical-exploration rules):

- `"choose alternative contraction associations"` — the rule now reads "choose alternative associations of a tensor contraction only when the effective distributivity, reassociation, and operand-permutation permissions all authorize the regrouping";
- `"alternative contraction associations for future multi-input einsum"` — the equivalence-group example now reads "alternative associations of a future multi-input einsum contraction, under a numerical policy that permits the distributivity the regrouping consumes";
- `"for future multi-input einsum"` — the same rewrite.

Each was verified absent by `grep -n -F` against the current file rather than inferred. `record-distributivity-in-the-navigation-contracts` repaired all three, because they fell inside `contracts/navigation`. The point of this ticket is that they were found only because an editor happened to re-verify quoted strings while working nearby.

This is sharper than ordinary documentation drift. A stale paraphrase reads as an author’s summary; a stale quotation asserts that a named document contains exact words it does not, which is the same failure mode `AGENTS.md` treats as a research-standards violation when asserting absence. It is also silent: the quotation marks make the claim look verified.

**Bounded first step.** Extract quoted spans that immediately follow or precede a Markdown link to another corpus document, and check each as a literal substring of the linked file. Both the true-positive and false-positive shapes should be measured before deciding whether this becomes a gate phase: quotations spanning a hard-wrapped line break, quotations of a document that is not linked in the same sentence, and deliberately shortened quotations with ellipses are the expected sources of noise. If the false-positive rate makes a gate phase unusable, record that measurement and close this ticket with the finding rather than shipping a check the corpus must fight.

Note that the reverse direction is out of scope: this checks that quoted text exists, not that it still means what the quoting document claims. Only reading does that.
