---
id: re-reconcile-document-metadata-with-make-citations-link-resolution
title: Re-reconcile document-metadata with make citations link resolution
status: in-progress
priority: p3
dependencies: []
related: [reconcile-the-document-metadata-validator-claim-with-its-own-validation-section, resolve-the-markdown-links-the-citation-check-cannot-see]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: sol-document-metadata-links
lease_expires_at: 1786411317
---
## User-visible outcome

`docs/document-metadata.md` (accepted contract `tiler.contract.document-metadata`) agrees with AGENTS.md and the live gate about local markdown link resolution: `make citations` / `check-citations.sh` resolves path existence of local links in open tickets and live documents and fails the gate when a target is missing; it does not check link meaning, heading anchors after `#`, frontmatter, supersession, or catalog correctness. No sentence resurrects the retired name `validate_links`.

## Why

**Fact — 2026-08-10 audit of `reconcile-the-document-metadata-validator-claim-with-its-own-validation-section`.** That ticket correctly closed on 2026-08-05 (`f97771119f3e4a7a692ad76cfb0d694df443e707`) by removing a false promise of a deleted `validate_links` gate so the contract's two halves agreed that nothing resolved local links. On 2026-08-08, `resolve-the-markdown-links-the-citation-check-cannot-see` landed link resolution in `check-citations.sh` and put `citations` on the `make check` / `make full` path. AGENTS.md was updated (`One mechanical property is checked: make citations resolves every local markdown link…`). The contract was not: it still says `nothing in this repository resolves local links`, lists local links among purely hand-maintained items under Validation (`There is no validator and no renderer` … "local links"), and says `Reading is the only standing check`. Same defect class as the parent ticket, inverted — the contract under-claims a gate that exists.

**Reproduce.**

- Contract anchors: `nothing in this repository resolves local links`; `Reading is the only standing check`; Validation includes "local links" in the hand-maintained list.
- Gate path: Makefile `check: citations …`, `citations: ./check-citations.sh`, `full: check …`.
- AGENTS: `Documentation is manually maintained` paragraph naming `make citations` as the one mechanical property.

## Requirements

1. Rewrite the decision-cites-experiment and Validation prose so path-existence resolution via `make citations` / `check-citations.sh` is stated honestly. Keep residual honesty: frontmatter graph, catalogs, supersession, quotation fidelity, link *meaning*, and heading anchors after `#` remain reading-only or hand-run ticket scripts, consistent with AGENTS' "Nothing else is validated".
2. Re-evaluate the decision-cites-experiment argument that enforcement "separates none of the options" for body links versus stored metadata edges: path existence of local body links is now gated; metadata edges and meaning remain unenforced; heading anchors still unchecked. Do not claim more than the checker does.
3. Name only `make citations` / `check-citations.sh`. Do not resurrect `validate_links`.
4. Do not reopen or re-edit the parent's close condition; parent stays `done` for the 2026-08-05 delivery.

## Closes when

Every present-tense claim in `docs/document-metadata.md` about whether local links are resolved or only hand-checked matches AGENTS and `check-citations.sh`; the decision-cites-experiment section's enforcement argument is re-evaluated against path-existence gating; residuals that remain unvalidated are still stated as such; `make citations` is green on the edited tree.
