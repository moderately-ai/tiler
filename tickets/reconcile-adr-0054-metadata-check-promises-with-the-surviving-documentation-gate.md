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

Accepted ADR 0054 describes the documentation checks that actually survive within `check-citations.sh`'s named populations and exclusions: locally resolvable pinned source citations and supported local Markdown link paths are checked by `make citations`, while metadata schema, typed relationship targets, entrypoints, document ticket references, IDs, and catalog derivation remain manually maintained or bounded by historical hand-run audits. The decision keeps its accepted metadata model and history; only its implementation-standing promises are corrected.

## Why

**Fact — source audit reverified at dispatched base `fca8f4ae`.** Complete [`docs/decisions/0054-use-typed-documentation-metadata.md`](../docs/decisions/0054-use-typed-documentation-metadata.md), anchor `The repository checks metadata, relationship targets`, continues by promising checks for `entrypoints, ticket references, and deterministic generated catalog sections`. Its Consequences anchor `broken paths and IDs fail checks` makes the same claim in compressed form. Those present-tense promises are false of the current tree.

**Fact — the surviving mechanical property is narrower.** Complete `check-citations.sh`, anchors `if (status in is_terminal)`, `if (role == "doc" || role == "root")`, and `if (vendored)`, checks locally resolvable pinned citations and local Markdown path existence in non-terminal tickets and their comments, non-superseded documents under `docs/`, and repository-root Markdown documents. The script separately skips or declines external and ambiguous source citations; terminal ticket/comment files; superseded documents; external, empty, whitespace-bearing, same-document-heading, and vendored-source links; and heading fragments after resolving a path-carrying link. It does not validate link meaning, frontmatter/schema, typed document IDs or relationship semantics, catalog derivation, supersession correctness, entrypoint fields, or quotation fidelity. `Makefile`, anchors `check: citations fmt build lint test` and `full: check doc`, places `citations` on both paths and contains no general document-metadata validator or renderer.

**Fact — the accepted contract owns this boundary, but its repair is not yet landed at this base.** Complete `docs/document-metadata.md`, anchors `nothing in this repository resolves local links` and `There is no validator and no renderer`, still under-claims the surviving link-path gate. The in-progress [`re-reconcile-document-metadata-with-make-citations-link-resolution`](re-reconcile-document-metadata-with-make-citations-link-resolution.md) owns that contract correction within `contracts/navigation` and explicitly leaves ADR 0054 untouched. The earlier done ticket [`reconcile-the-document-metadata-validator-claim-with-its-own-validation-section`](reconcile-the-document-metadata-validator-claim-with-its-own-validation-section.md) preserves the 2026-08-05 historical close before link resolution landed. This ticket is the decisions-scope remainder; it does not reopen either delivery.

## Fact audit at dispatched base `fca8f4ae`

- **Verified:** the two ADR source anchors above survive unchanged and make broader present-tense checking promises than the tree implements.
- **Imprecise, repaired above:** the original checker summary omitted status, provenance, syntax, and fragment exclusions. The script's green census at this base reports 1,189 checked pinned citations and 6,445 checked local links while separately reporting skipped terminal/superseded files and unresolved external, vendored, and heading-anchor populations.
- **False in present tense, repaired above:** the sibling contract ticket is `in-progress`; the accepted contract still carries both under-claiming anchors above at this base. It owns the contract-side correction, but has not delivered it here.
- **Purpose/authority verdict:** the repairs narrow evidence to the behavior the live gate implements and distinguish planned sibling work from landed authority. They do not change this ticket's decisions-scope purpose or authorize a new model.

## Work

- Re-read the accepted decision, complete metadata contract, complete `check-citations.sh`, `Makefile`, AGENTS.md, and the related historical tickets at the working base before editing.
- Preserve the accepted choice: strict typed metadata, stable IDs, one stored direction per relationship, and manually maintained catalogs remain the model.
- Add a dated, quotation-preserving correction beside the live repository-check paragraph and, if needed, its Consequences shorthand. State exactly which link/citation property is gated and which metadata/catalog properties remain reading-only or historically hand-run.
- Do not add tooling, resurrect `validate_links`, rewrite the historical Context, change decision or implementation status, or claim that a path existing proves a correct relationship.

## Closes when

No present-tense sentence in ADR 0054 promises metadata, relationship-ID, entrypoint, ticket-reference, or generated-catalog validation the repository does not perform; the surviving citation/link-path gate is stated with its bounded meaning; the accepted decision and historical rationale remain intact; and `make citations` passes.

## Outcome — 2026-08-10

**Decision correction.** ADR 0054 retains its accepted paragraphs verbatim and adds dated corrections beside both the Decision promise and its Consequences shorthand. The correction distinguishes the accepted authority model from implementation standing: strict metadata, stable IDs, typed relationships, and one stored relationship direction remain decided; `decision_status: "accepted"`, `implementation_status: "implemented"`, Context, rationale, and traceability are unchanged. Checked-in catalogs remain hand-maintained representations of the authoritative edges rather than a second relationship authority; no generator or validator is implied.

**Surviving gate.** `make citations` runs `check-citations.sh` on non-terminal tickets and their comments, non-superseded `docs/` documents, and repository-root Markdown documents. Within its documented syntax and provenance exclusions, it checks locally resolvable pinned source citations and local path existence for supported Markdown links. The ADR now says explicitly that a green result establishes only that a checked citation or link points somewhere, not that its prose is true or its destination has the intended meaning.

**Properties still unchecked.** No standing gate validates frontmatter/schema, stable-ID uniqueness, typed relationship targets, document `ticket` references, experiment `entrypoints`, supersession correctness, heading fragments, quotation fidelity, catalog or backlink derivation, or catalog/entrypoint completeness. The historical hand-run catalog and typed-edge scripts remain commit-bounded evidence, not repository tooling.

**Source-safe residual census.** The retained historical anchors `The repository checks metadata, relationship targets`, `Catalogs and backlinks are derived and`, and `broken paths and IDs fail checks` each occur in the original accepted body. The Decision pair containing the first two is followed by `this correction retires the present-tense implementation promises`; the Consequences list containing the third is followed by `The combined present-tense guarantee` and explicitly retires it. A complete read found no other live promise of the removed checks; the Alternatives statement concerns what free-form links can support, and the Traceability statement describes the historical migration.

**Verification and gate carry.** `make citations` passes with 1,189 checked pinned citations and 6,446 checked local links; `tkt lint --format json` returns `ok: true`; and `git diff --check` is clean. The coordinator-supplied latest green full-gate commit is `0b0e6952aaa6c88f7c7be923c3158adba9d86add`. It is an ancestor of the dispatched base `fca8f4ae5dcbcb18b3b8ef62ecd01dc491a632ad`; the intervening path census contains only `docs/operation-extensions.md` and ticket files, and an explicit diff over the gate-invalidating path set is empty. This ticket adds only its own ticket file and `docs/decisions/0054-use-typed-documentation-metadata.md`, so it carries that full gate under AGENTS.md's docs/ticket-only rule while rerunning both required documentation checks. `tkt guard` against base and config ref `fca8f4ae` reports exactly those two changed files, direct effects only on the two declared scopes, no under-declaration, and non-gating `warn` severity for declared sibling collisions.
