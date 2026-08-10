---
id: reconcile-the-document-metadata-validator-claim-with-its-own-validation-section
title: Reconcile the document-metadata validator claim with its own validation section
status: done
priority: p3
dependencies: []
related: [resolve-the-markdown-links-the-citation-check-cannot-see, specify-the-consumer-neutral-backend-provider-composition-contract, re-reconcile-document-metadata-with-make-citations-link-resolution]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`docs/document-metadata.md` stops asserting a link-validation gate that does not exist, so a reader planning to rely on it learns the truth the corpus elsewhere states plainly: nothing validates the documentation corpus, and a reader is the only check.

**Note (2026-08-10).** The User-visible outcome above is the *filing* target (2026-07-31 / delivery 2026-08-05). After `resolve-the-markdown-links-the-citation-check-cannot-see` landed, `make citations` / `check-citations.sh` *does* resolve local markdown links in open tickets and live documents and is on the `make check` / `make full` path. That later inversion is owned by `re-reconcile-document-metadata-with-make-citations-link-resolution`, not by reopening this ticket's close condition.

## Why

**Fact — found by `specify-the-consumer-neutral-backend-provider-composition-contract` while checking its own links, 2026-07-31 (filing-era; delivery 2026-08-05).** At filing, `docs/document-metadata.md` asserted that `validate_links` "fails the repository gate", while the same document's validation section stated there is no validator. `grep -rn "validate_links" .` found the name only in that prose — no script, target, or test carried it, and `make full`'s stages then (fmt, check, clippy, nextest, doc-tests, rustdoc, release numerical tests, tkt lint, shellcheck) contained no link step. AGENTS.md's docs-maintenance section then stated the true position: local link targets are ordinary hand-maintained prose and a broken link costs a reader rather than a gate.

**Correction — 2026-08-10.** Do not treat the Why paragraph as a live description of the current tree.

- The line-114 `validate_links` / "fails the repository gate" claim is **not** true of the post-delivery file: delivery `f97771119f3e4a7a692ad76cfb0d694df443e707` rewrote that sentence to the conventional form. At this base, the searchable anchor at that site is `nothing in this repository resolves local links, so a decision citing a harness that has moved or been deleted rots silently` — the *fixed* text, not the defect.
- The tooling-identity claim about the name `validate_links` remains true: nothing named that exists under non-ticket paths. The claim that the tree has *no* link resolver is **false** after `6a0184a5` / `757cb4c1` (2026-08-08): `Makefile` has `citations: ./check-citations.sh`, `check: citations …`, and `full: check …`.
- The AGENTS claim that a broken link costs a reader rather than a gate is **false** at this base for path existence of local markdown links in the checked populations. AGENTS now says: `One mechanical property is checked: make citations resolves every local markdown link in an open ticket or a live document, so a catalog row or cross-reference that points at nothing fails the gate.` Residual properties (frontmatter, supersession, link meaning, heading anchors after `#`) remain reading-only, matching AGENTS' "Nothing else is validated".

## Closes when

The false sentence is corrected or removed, the document's two halves agree, and any other sentence in it that promises tooling the tree does not contain is swept in the same change.

## Outcome

Landed at **`f97771119f3e4a7a692ad76cfb0d694df443e707`** (2026-08-05), subject `Stop the metadata contract promising a link gate it also says does not exist`. Status flipped to `done` at `55dda5c8` without an Outcome body at the time; this section records the delivery for auditors.

**What closed.** The decision-cites-experiment paragraph no longer names `validate_links` or claims a harness body link "fails the repository gate". Enforcement was rewritten as conventional / unenforced for that era's tree. The same change swept other present-tense promises of deleted Python docs tooling (locked CommonMark parser, generated catalog, derived backlink / rendered catalog claims). The Validation section was expanded with the hand-run ticket-body checks and typed-edge measurement that actually exist, while retaining "There is no validator and no renderer" as true of a dedicated docs validator.

**Close matches the 2026-08-05 tree.** At delivery, both halves of `docs/document-metadata.md` agreed that no link gate existed; `make full` had no link step; AGENTS said a reader is the only check. That was the original close condition and remains satisfied historically. This ticket stays `done`; it is not reopened for later corpus drift.

**Later re-rot (not this close condition).** On 2026-08-08, `resolve-the-markdown-links-the-citation-check-cannot-see` taught `check-citations.sh` / `make citations` to resolve local markdown links and put that step on every `make check` / `make full` path. AGENTS was updated to state that one mechanical documentation property. `docs/document-metadata.md` was **not** amended, so the contract now *under-claims* a gate that exists (`nothing in this repository resolves local links`; "local links" listed among purely hand-maintained items; `Reading is the only standing check`). That inverted consistency defect is owned by the remainder ticket `re-reconcile-document-metadata-with-make-citations-link-resolution`.
