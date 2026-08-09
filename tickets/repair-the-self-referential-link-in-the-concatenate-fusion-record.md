---
id: repair-the-self-referential-link-in-the-concatenate-fusion-record
title: Repair the self-referential link in the concatenate fusion record
status: done
priority: p2
dependencies: []
related: []
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## What is broken

`docs/research/indexing/concatenate-fusion-role-and-lowering.md:146` links to itself with a target written from `docs/`:

```
[Concatenate fusion role and lowering](research/indexing/concatenate-fusion-role-and-lowering.md)
```

It resolves to `docs/research/indexing/research/indexing/concatenate-fusion-role-and-lowering.md`, which does not exist.

Surfaced by the first run of the markdown-link resolution added to `check-citations.sh` under `resolve-the-markdown-links-the-citation-check-cannot-see`.

## The judgement this needs

The link sits inside a blockquoted trigger-check bullet that reads "[Concatenate fusion role and lowering] ran the elimination on 2026-08-05 at `d5960e81`". A document citing itself in the third person is usually text that was **moved** here from a document that legitimately linked to it — most likely the deferred-question record whose trigger this bullet answers. Before repairing the path, check whether the bullet belongs here at all; if it does, the self-reference should probably become plain text rather than a link to the page the reader is already on.

## Fact audit — 2026-08-08, at `db3f4d077bf8bd680cacd7a36986f39fec6294f8`

Every claim above re-read at that base. **The two sections above are retained as written and are wrong in one shared way: there is no copy-paste mistake here, and the link is not a self-reference the author failed to notice.** It is a destination-relative link inside a verbatim transfer record, and the sentence immediately above it in the document already says so.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The link resolves to `docs/research/indexing/research/indexing/concatenate-fusion-role-and-lowering.md`, which does not exist | **Verified** | `./check-citations.sh` at this base printed exactly that target in its one `FAIL` for this file. |
| The target is "written from `docs/`" | **Verified, and it is deliberate** | The paragraph that introduces the span reads "The paths below are written relative to `docs/open-questions.md`, which is where this text lands; they do not resolve from this record." The link is spelled for its destination, not mis-copied. |
| The link "links to itself" | **Imprecise** | True only of where the stripped path lands. `docs/open-questions.md`'s Q-SHAPE-006 section still carries the identical link and it resolves correctly from there — so the span is a faithful record of live text elsewhere, not a page citing itself. |
| It sits inside "a blockquoted trigger-check bullet" | **False** | It is the record's verbatim-landable replacement for Q-SHAPE-006's live-pressure bullet, under the heading "Q-SHAPE-006's firing condition, restated". A `## Trigger check log` is the deferred-ticket convention and this record has none. |
| The text "was **moved** here from a document that legitimately linked to it — most likely the deferred-question record" | **False, and backwards** | The text was drafted *here* and moved *out* to `docs/open-questions.md` by [`carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus`](carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus.md); the record states the transfer executed byte-identically. `docs/open-questions.md` is the open-questions register, not a deferred-question ticket record. |
| "The self-reference should probably become plain text rather than a link" | **Rejected** | Delinking edits a retained transfer record so that it stops being what was transferred, spending the byte-identity the paragraph above it asserts. Repointing is worse: it would produce a link to the page the reader is already on. |
| "Check whether the bullet belongs here at all" | **Checked; it belongs** | The record retains it as the account of what the carrier transferred, and says in the same paragraph that the landed bullet has since been corrected in place, so the span is history rather than a duplicate authority. |

## Repair — fenced, not delinked or repointed

The span is now a `text` fence instead of a blockquote, and the transferred bullet is byte-identical to its previous content less the `> ` blockquote marker (`cmp` against the base blob's line: no difference). Fencing is this corpus's declared spelling for content whose links belong to another file — `check-citations.sh "fenced block is content proposed for somewhere else"` — and `scope-transformer-nonlinear-normalization-and-reductions`, `derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound`, `catalog-the-cache-hot-path-efficiency-records`, and `catalog-the-kani-verification-research-and-spike` all already fence a destination-relative block for exactly this reason. The introducing paragraph gained a sentence stating why the fence is there and what it costs.

**The check was shown able to say no.** Perturbing the subject — restoring the blockquote and changing no assertion — reproduced the original failure verbatim:

```text
FAIL  docs/research/indexing/concatenate-fusion-role-and-lowering.md
        link: [...](research/indexing/concatenate-fusion-role-and-lowering.md)
        no tracked file or directory at docs/research/indexing/research/indexing/concatenate-fusion-role-and-lowering.md
```

**The cost is measured, not asserted.** Fencing removes the span's four retired line pins from the citation matcher's reach. Three were being checked and one (`program/verify.rs`, matching two tracked files) was already skipped as ambiguous, and the introducing paragraph gained one anchor citation, so `./check-citations.sh` moved its `docs` population from 702 checked citations to 700. All four pins name line numbers the record's own dated correction discharges. (The whole-run total reads 944 rather than 943 because this ticket's audit above adds an anchor citation of its own; the `docs` line is the file-scoped number.)

**Not repaired here, and reported rather than filed:** [`repair-the-eight-dangling-links-in-the-runtime-route-answer-record`](repair-the-eight-dangling-links-in-the-runtime-route-answer-record.md) directs a worker to repoint links inside a drafted-ADR span, but [`land-the-backend-scoped-route-requirement-answer-adr`](land-the-backend-scoped-route-requirement-answer-adr.md) records that those same eight were left broken *deliberately*, because repointing spends the byte-identity that makes the span quotable. Those two tickets disagree, and the fence is a third answer neither considered. That is the coordinator's call, not this ticket's.

## Closes when

`make citations` reports no link failure in this file.

## Outcome — delivered

Commit `e96e6aaa` fenced the complete destination-relative Q-SHAPE-006 transfer
as `text`, preserving the transferred bytes while removing its link from the
wrong source-relative resolution context. The opening-fence perturbation
restored the exact doubled-path failure, and the measured citation cost is
recorded above. Commit `c4c05e5e` closed this repair and routed the analogous
retained-ADR spans through their shared decision ticket; that decision later
selected the same whole-span-fence convention. The live navigation copy was not
delinked or repointed.
