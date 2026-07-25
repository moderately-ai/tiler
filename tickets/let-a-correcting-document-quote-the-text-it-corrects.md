---
id: let-a-correcting-document-quote-the-text-it-corrects
title: The quotation validator rejects a document that quotes the staleness it is fixing
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [documentation, gate, navigation]
claimed_from: todo
assignee: agent-nav2
lease_expires_at: 1784999640
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
