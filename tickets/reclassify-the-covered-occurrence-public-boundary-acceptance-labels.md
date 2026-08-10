---
id: reclassify-the-covered-occurrence-public-boundary-acceptance-labels
title: Reclassify CoveredOccurrence acceptance labels after the 2026-08-05 decision
status: in-progress
priority: p2
dependencies: []
related: [accept-the-proof-bound-stage-coverage-public-boundary, bind-stage-coverage-to-index-refinement-identity]
scopes: [contracts/foundation, contracts/decisions, research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, public-boundary, contracts]
claimed_from: todo
assignee: sol-covered-occurrence-labels
lease_expires_at: 1786383181
---
## User-visible outcome

Three live documents stop denying acceptance of the proof-bound stage-coverage public boundary that Tom accepted on 2026-08-05 under [`accept-the-proof-bound-stage-coverage-public-boundary`](accept-the-proof-bound-stage-coverage-public-boundary.md). Readers of the IR contract, ADR 0071, and the production-crate audit learn the surface is accepted pre-alpha vocabulary, not an open Proposal or unaccepted draft.

## Why this is a separate ticket

The acceptance ticket closed on the decision and correctly left the ADR *catalog row* alone (no dedicated ADR owns this surface). It did **not** reclassify three present-tense denials that still treat the landed surface as unaccepted:

1. [`docs/ir.md`](../docs/ir.md) — `**Proposal — program and artifact stages now consume it, and the public boundary that lands with them is not yet accepted.**`
2. [ADR 0071](../docs/decisions/0071-use-checked-builders-for-shared-compiler-ir.md) — `**Built, and not yet accepted.**`, `public boundary is not yet accepted`, and the `implementation_status stays partial` rationale that still waits on this acceptance.
3. [`docs/research/documentation/production-crate-codebase-audit.md`](../docs/research/documentation/production-crate-codebase-audit.md) — `**Resolved in the tree; the public boundary is not yet accepted.**`

Those files sit under `contracts/foundation`, `contracts/decisions`, and `research/documentation`. The acceptance node holds only `contracts/decisions` as its decision record and is `done`; this remainder owns the post-acceptance label sweep. Audit evidence: ticket-audit report at base `c99ac54950f2` (Fact 9 false as a live-completeness claim).

## What to change

Re-read each site in full at the edit base before changing it; the anchors below are distinctive phrases, not line numbers.

1. **`docs/ir.md` CoveredOccurrence paragraph.** Reclassify the Proposal lead-in to an Accepted lead-in for 2026-08-05, citing [`accept-the-proof-bound-stage-coverage-public-boundary`](accept-the-proof-bound-stage-coverage-public-boundary.md). Mirror the PublishingCopy / StagedRealization acceptance style already on the same page (`**Accepted 2026-08-06 …**` blocks for those surfaces): who, date, venue or provenance, surface inventory, acceptance-is-not-stabilization clause. Keep the technical content (private fields, `from_receipt`, builder refusals, stage-key steps) unless a sentence is false in present tense.

2. **ADR 0071.** Reclassify `Built, and not yet accepted` for the CoveredOccurrence / stage-coverage product; drop or rephrase present-tense `public boundary is not yet accepted` and the partial-status rationale that still waits on this acceptance. Do **not** invent a move of `implementation_status` to `implemented` unless the ADR's *other* stated blockers (e.g. the named verified type that does not exist) are already discharged — only remove the blocker that was this acceptance. Leave catalog rows and unrelated ADR clauses alone.

3. **`docs/research/documentation/production-crate-codebase-audit.md`.** Reclassify `Resolved in the tree; the public boundary is not yet accepted` for the proof-bound coverage finding so the audit no longer denies acceptance after 2026-08-05. Prefer a dated resolution note over silent deletion of the finding's history.

## Non-goals

- Reopening or amending Tom's 2026-08-05 acceptance decision.
- Code, rustdoc, identity domains, or crate API changes (the surface is already live).
- Filing a dedicated ADR for CoveredOccurrence, or changing the decisions catalog merely because labels moved.
- Sweeping every research doc that might still say "draft" beyond the three sites named above (optional hygiene only if a cheap grep while editing finds an obvious sibling denial of *this* surface).

## Closes when

- `docs/ir.md` presents CoveredOccurrence as accepted on 2026-08-05 with provenance to the acceptance ticket, in the same style as neighbouring Accepted program-surface paragraphs.
- ADR 0071 no longer asserts present-tense non-acceptance of this public boundary as a live blocker.
- The production-crate audit no longer asserts present-tense non-acceptance of this boundary.
- A re-run of distinctive greps (`the public boundary that lands with them is not yet accepted`, `Built, and not yet accepted` scoped to the CoveredOccurrence product, and the audit's `Resolved in the tree; the public boundary is not yet accepted`) shows those three false live claims are gone or clearly historical.
- `tkt lint` and `make citations` pass for the edited documents and this ticket's links.

## Graph

- Filed from the 2026-08-10 ticket audit repair of [`accept-the-proof-bound-stage-coverage-public-boundary`](accept-the-proof-bound-stage-coverage-public-boundary.md).
- Related to [`bind-stage-coverage-to-index-refinement-identity`](bind-stage-coverage-to-index-refinement-identity.md), which landed the surface the labels describe.
