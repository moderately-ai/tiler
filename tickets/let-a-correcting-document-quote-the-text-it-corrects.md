---
id: let-a-correcting-document-quote-the-text-it-corrects
title: The quotation validator rejects a document that quotes the staleness it is fixing
status: done
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, gate, navigation]
---
**Fact — observed on `main`, not hypothesised.** Merging the new `validate_quotations` phase together with ADR 0080 turned the repository gate red:

```
docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md:
  quoted text attributed to docs/compiler/optimizer.md appears in no document
  this paragraph links: 'remains the only numerical contract the compiler registers'
```

**Fact — the ADR was right and the check was right.** ADR 0080's paragraph exists to record that two contracts carried a claim that went stale, and it quotes the stale wording in order to say so. The wording genuinely no longer appears in `optimizer.md`, because the same change corrected it. So the check fired on a true statement about the past, phrased as an attributed quotation of the present.

**Inference — this is a known shape, met a second time.** The validator's own outcome already measured the sibling case and rejected the obvious relaxation: admitting `tickets/` as haystacks made the historical replay *miss* a true positive, because the ticket documenting the rot quotes the rotted string. Correcting documents have the same need and the same hazard, so widening the haystack is not the fix here either.

**What was done to unblock `main`, and why it is not the answer.** The ADR was rephrased into indirect speech — it now states what the two documents said rather than quoting them. That restored a green gate without weakening the check, but it is a workaround: the corpus should be able to quote text *exactly* when the whole point of the sentence is which words were wrong, and indirect speech is strictly weaker evidence than a quotation for that purpose.

## Scope

Give a paragraph a way to mark a quotation as historical — the text a document is correcting or superseding, rather than citing as current. Options, none chosen:

- an explicit inline marker the miner recognises and skips;
- resolving an attributed quotation against the linked document *at a named commit*, so a correcting document cites what was true then and the check verifies that rather than the present;
- scoping the exemption to `docs/decisions/`, on the ground that superseding is what an ADR is for.

Prefer whichever keeps the check able to catch the defect it already caught on the live tree — an erased caveat in `docs/roadmap.md` — since a broad exemption would have let that through. Say which you judged and why the others were rejected.

## Closes when

A document can quote the exact text it is correcting without the gate refusing it, the `412ceae` historical replay still reports its three true positives and no more, the live-tree defect the phase originally caught would still be caught, and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

**Chosen: the explicit marker, upgraded from a skip into an inverted obligation.** `superseded-quotation` — spelled `<!-- superseded-quotation -->` and written directly after the closing quotation mark — declares that the span is *deliberately* absent. The check does not then stop looking; it demands exactly that, requiring the marked quotation to appear in none of the documents its paragraph links. That is the negation of the predicate the unmarked case asserts, computed by the same code, so the marker adds an obligation to one named span rather than removing one. Three consequences follow that a plain skip would not have bought: a marker cannot silence a quotation whose attribution still resolves; restoring the wording later, or reverting the correction, makes the record's own claim false and the gate says so; and a marker qualifying nothing — following no quotation, or a span the rule does not check — is itself an error, so writing a marker without acquiring its obligation is unreachable. A marker inside a code span is mentioned rather than used, which is how `docs/document-metadata.md` names it.

**Rejected — resolving an attributed quotation against the linked document at a named commit.** This is the strongest evidence on offer and it was the closest call. It fails on what it would cost to buy it: documentation validation would depend on the repository's history being present, on a commit-reference syntax the corpus would have to learn and the validator to parse, and on `git` from a phase that today reads only files. What it buys is narrower than it looks — it proves the words were once there, which is a claim a reader can already settle from the correcting commit the paragraph ought to cite, and which no reader disputes in the case that motivated this. It does not prove they are gone now, which is the half that actually rots. The chosen rule checks the half that rots and `docs/document-metadata.md` says plainly that it establishes absence and never prior presence, so the weaker guarantee is stated rather than implied.

**Rejected — scoping the exemption to `docs/decisions/`.** This is the measured hazard a second time. Admitting `tickets/` as haystacks made the `412ceae` replay miss `"choose alternative contraction associations"`, because the ticket documenting the rot quoted the rotted string; exempting a directory is the same mistake in the other direction, and worse here, because quoting a current contract is most of what an ADR does — the exemption would blind the check where attributed quotations are densest. It is also unnecessary: superseding is what an ADR is for, but a *correcting quotation* is a property of one sentence, not of a directory, and the roadmap defect proves the same sentence shape occurs outside `docs/decisions/`.

**Measurement — both required regressions, replayed against exported trees with the shipped rule.** At `412ceae`: 205 spans mined, 15 checked, 0 marked, exactly the three documented rotted quotations reported and nothing else. At `ab67a8d`: 249 mined, 19 checked, 0 marked, and the one live-tree defect still reported — `docs/roadmap.md`'s `"remain explain/search state only and cannot enter an executable ImplementationFrontier or manifest"` attributed to `docs/ir.md`. Both are byte-identical to the rule before this change, which is expected, because no document on either tree uses the marker. Reproduce with `git archive <commit> | tar -x -C <dir>` and this branch's `validate_quotations` against `<dir>`.

**Measurement — the restored quotation, verified on a copy.** `docs/decisions/` belongs to another scope, so the ADR was not edited here. Applying the exact substitution `ecbe12b` reversed, plus the marker, to an export of this working tree gives a green `scripts/docs.py validate` over 183 records and a quotation replay of `mined=324 checked=25 marked=1 findings=0`. Split as `restore-adr-0080-verbatim-quotation`, which carries the exact strings and this measurement.

**Measurement — the inverted polarity fires on the real corpus, not only in unit tests.** Marking a currently-passing corpus quotation as superseded was rejected with a located message: `docs/backends/metal.md: quotation marked superseded still appears in docs/decisions/0002-aot-metal-artifacts.md: 'creates and caches pipeline objects from compiled artifacts but does not compile MSL source'`.

**Coverage, stated rather than improved.** This change adds no coverage. The bound in `docs/document-metadata.md` is unchanged and still honest: 19 of 249 spans checked at `ab67a8d`, the remainder being 159 with no preceding governed-document link, 56 single words, and 15 beyond a sentence boundary or the 220-character reach. That 15 was implied by the earlier arithmetic but never written down; it is now named, and the earlier 159 and 56 were re-measured and reproduce exactly.

**Retracted.** A first mutation run reported the code-span test as vacuous. That was a defect in the throwaway harness, not the test: `scripts/tests/test_docs.py` loads its own module instance under the name `tiler_docs`, and the harness had mutated a second, unrelated `docs` import. Re-run against `test_docs.docs`, each of the four new tests fails under a mutation that removes the behaviour it names, and passes unmutated.

**Not done.** The marker establishes absence and never prior presence; that limit is written into the contract rather than left for a reader to infer, and closing it needs the commit-resolution design rejected above. No follow-up ticket is filed: the trigger for revisiting is a case where a reader cannot settle prior presence from the correcting commit, which the motivating case is not.

Gate: `uv run --locked python scripts/docs.py render`, then `uv run --locked python scripts/check_repository.py`.
