---
id: detect-stale-cross-document-quotations
title: Detect stale verbatim cross-document quotations
status: done
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

## Outcome

**Shipped as a gate phase.** `validate_quotations` in `scripts/docs.py`, wired into `validate()` so the existing repository gate runs it, with four mutation tests in `scripts/tests/test_docs.py` and the rule and its bounds stated in `docs/document-metadata.md`. It found one live defect, repaired below. The ticket's own stop condition — close with the finding if the corpus must fight the check — was not reached; the final rule reports zero false positives on two commits.

**The rule.** A quoted span of two or more words, reached from a *preceding* inline link to a governed document within the same sentence and 220 characters, must appear in *some* governed document that paragraph links. Comparison flattens whitespace, case, and inline code/emphasis markers; an ellipsis splits the quotation into fragments required in order; fenced blocks are not mined; tickets and the quoting document are not haystacks.

**Measurement — the shipped implementation, replayed on two trees.** At `412ceae` it mines 205 spans, checks 15, and reports exactly the three quotations this ticket documents — no more. At `ab67a8d` it mines 249, checks 19, and reports one. Zero false positives on both. Coverage is 19 of 249, not a corpus-wide audit: 159 spans have no preceding governed-document link and 56 are single words. That bound is stated in the contract so a green gate is not read as "no stale quotations anywhere".

**The live defect, repaired.** `docs/roadmap.md` Milestone 6 attributed to [IR](../docs/ir.md) the words `"remain explain/search state only and cannot enter an executable ImplementationFrontier or manifest"`. That string occurred nowhere in the corpus except the quotation itself: at `ab67a8d`, `grep -rn "explain/search state only" docs/ tickets/` returned only `docs/roadmap.md:314`. IR's actual rule is narrower in a way the paraphrase erased: an "`Unknown` *feasibility* verdict keeps its candidate in explain and search state only", a rule IR explicitly declines to generalize to every `Unknown` in the corpus — while the roadmap was using it to justify inadmissibility of *unknown numerical evidence*, which is the optimizer's rule, not IR's. The paragraph now quotes IR exactly, carries IR's own scoping caveat, and attributes the numerical half to the optimizer. So the check's first catch was not a typo; it was a quotation laundering two rules into one.

**Retracted.** Mid-investigation I reported `docs/roadmap.md`'s `"the bounded P0 frontier admits only checked ScheduledKernel proposals and rejects opaque-call proposals explicitly"` as a second stale quotation, on a `grep` of `docs/compiler/fusion-and-scheduling.md` alone. It is not stale: `docs/compiler/optimizer.md:319` contains it verbatim, and the quoting sentence names both contracts. I asserted absence from one check, which is the exact failure `AGENTS.md` warns about, and the corrected finding is what forced the rule to clear a quotation against *any* document its paragraph links rather than the nearest one.

**Measurement — three attribution rules were built and compared, not one.** Nearest-preceding-link alone: 4 findings at `412ceae` (3 true, 1 the retracted composite) and 2 at `ab67a8d` (1 true). Any-linked-document, paragraph-wide: precision collapses to 3/7 and 1/6, because a document quoting *itself* — the roadmap quoting its own Milestone 6 bullet list, a research record quoting its own previous version — is indistinguishable from a stale attribution. Admitting the quoting document as a haystack makes the check vacuous, since the quotation is trivially present in the document containing it; requiring a second occurrence instead launders a stale quotation that was repeated twice, and measurably lost 2 of the 3 documented true positives. Only the shipped combination survives both trees.

**Measurement — the candidate set must exclude tickets, and this is not tidiness.** With `tickets/*.md` admitted as haystacks, the `412ceae` replay *misses* `"choose alternative contraction associations"`, because `tickets/scope-einsum-contraction-support.md` is linked in the same paragraph and quotes the same rotted string, inherited from the same rot. A stale quotation propagates into the ticket that documents it, so any haystack including tickets launders exactly the staleness the check exists to find.

**False-positive shapes, measured on the 16 findings the first prototype produced at `ab67a8d`.** Mis-attribution to a document named in prose but not linked, or to the nearest link when the sentence names two (7); a term in scare quotes rather than an attributed quotation, e.g. `"Conservative"`, `"Contraction"` (2); sentence-initial case normalization when a quotation is embedded mid-sentence or opens one (3); inline-code markers added or dropped by the quoting author (1); a quotation containing a line break defeating straight-quote pairing and yielding a nonsense span (1); markup emphasis inside the source (1). Each has a rule, and no rule names a document or a string. The ticket predicted three of these shapes; the two it did not predict — case normalization and self-quotation — were the ones that mattered most.

**Not done.** The check still says nothing about whether quoted words still *mean* what the quoting document claims; the ticket puts that out of scope and it stays out. The 159 unattributed spans are also unchecked by construction. No follow-up ticket is filed for either: widening the rule was measured and every relaxation admitted honest prose faster than defects, so the trigger for revisiting is a *new* incident whose shape the current rule declines, not an appetite for coverage.

Gate: `uv run --locked python scripts/docs.py render`, then `uv run --locked python scripts/check_repository.py`, both green.
