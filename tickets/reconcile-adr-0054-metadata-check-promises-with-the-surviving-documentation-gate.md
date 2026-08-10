---
id: reconcile-adr-0054-metadata-check-promises-with-the-surviving-documentation-gate
title: Reconcile ADR 0054 metadata-check promises with the surviving documentation gate
status: in-progress
priority: p3
dependencies: []
related: [re-reconcile-document-metadata-with-make-citations-link-resolution, reconcile-the-document-metadata-validator-claim-with-its-own-validation-section, resolve-the-markdown-links-the-citation-check-cannot-see]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: sol-adr0054-metadata-checks
lease_expires_at: 1786411875
---

## User-visible outcome

Accepted ADR 0054 describes the documentation checks that actually survive: local Markdown link path existence and pinned citation reachability are checked by `make citations` / `check-citations.sh`, while metadata schema, typed relationship targets, entrypoints, document ticket references, IDs, and catalog derivation remain manually maintained or bounded by historical hand-run audits. The decision keeps its accepted metadata model and history; only its implementation-standing promises are corrected.

## Why

**Fact — source audit at integration base `b298862c`.** Complete [`docs/decisions/0054-use-typed-documentation-metadata.md`](../docs/decisions/0054-use-typed-documentation-metadata.md), anchor `The repository checks metadata, relationship targets`, continues by promising checks for `entrypoints, ticket references, and deterministic generated catalog sections`. Its Consequences anchor `broken paths and IDs fail checks` makes the same claim in compressed form. Those present-tense promises are false of the current tree.

**Fact — the surviving mechanical property is narrower.** Complete `check-citations.sh` resolves pinned citation reachability and path existence for path-carrying local Markdown links in open tickets/comments, live documents, and repository-root documents. It deliberately does not validate link meaning, heading fragments, frontmatter/schema, typed document IDs or relationship semantics, catalog derivation, supersession correctness, entrypoint fields, or quotation fidelity. `Makefile` places `citations` on `check` and `full`; it contains no general document-metadata validator or renderer.

**Fact — the accepted contract owns the exact observed boundary.** [`re-reconcile-document-metadata-with-make-citations-link-resolution`](re-reconcile-document-metadata-with-make-citations-link-resolution.md) corrects `docs/document-metadata.md` within `contracts/navigation` and explicitly leaves ADR 0054 untouched. The earlier done ticket [`reconcile-the-document-metadata-validator-claim-with-its-own-validation-section`](reconcile-the-document-metadata-validator-claim-with-its-own-validation-section.md) preserves the 2026-08-05 historical close before link resolution landed. This ticket is the decisions-scope remainder; it does not reopen either delivery.

## Work

- Re-read the accepted decision, complete metadata contract, complete `check-citations.sh`, `Makefile`, AGENTS.md, and the related historical tickets at the working base before editing.
- Preserve the accepted choice: strict typed metadata, stable IDs, one stored direction per relationship, and manually maintained catalogs remain the model.
- Add a dated, quotation-preserving correction beside the live repository-check paragraph and, if needed, its Consequences shorthand. State exactly which link/citation property is gated and which metadata/catalog properties remain reading-only or historically hand-run.
- Do not add tooling, resurrect `validate_links`, rewrite the historical Context, change decision or implementation status, or claim that a path existing proves a correct relationship.

## Closes when

No present-tense sentence in ADR 0054 promises metadata, relationship-ID, entrypoint, ticket-reference, or generated-catalog validation the repository does not perform; the surviving citation/link-path gate is stated with its bounded meaning; the accepted decision and historical rationale remain intact; and `make citations` passes.
