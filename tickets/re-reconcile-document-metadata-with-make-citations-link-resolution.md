---
id: re-reconcile-document-metadata-with-make-citations-link-resolution
title: Re-reconcile document-metadata with make citations link resolution
status: done
priority: p3
dependencies: []
related: [reconcile-the-document-metadata-validator-claim-with-its-own-validation-section, resolve-the-markdown-links-the-citation-check-cannot-see]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`docs/document-metadata.md` (accepted contract `tiler.contract.document-metadata`) agrees with AGENTS.md and the live gate about local markdown link resolution: `make citations` / `check-citations.sh` resolves path existence for path-carrying local links in open tickets and live documents and fails the gate when a target is missing, subject to the checker's stated exclusions for vendored upstream sources and pure heading anchors. For a path carrying a `#` fragment it resolves the path but not the heading. It does not check link meaning, frontmatter, supersession correctness, or catalog correctness. No sentence resurrects the retired name `validate_links`.

## Why

**Fact — 2026-08-10 audit of `reconcile-the-document-metadata-validator-claim-with-its-own-validation-section`.** That ticket correctly closed on 2026-08-05 (`f97771119f3e4a7a692ad76cfb0d694df443e707`) by removing a false promise of a deleted `validate_links` gate so the contract's two halves agreed that nothing resolved local links. On 2026-08-08, `resolve-the-markdown-links-the-citation-check-cannot-see` landed link resolution in `check-citations.sh` and put `citations` on the `make check` / `make full` path. AGENTS.md was updated (`One mechanical property is checked: make citations resolves every local markdown link…`). The contract was not: it still says `nothing in this repository resolves local links`, lists local links among purely hand-maintained items under Validation (`There is no validator and no renderer` … "local links"), and says `Reading is the only standing check`. Same defect class as the parent ticket, inverted — the contract under-claims a gate that exists.

## Fact audit — exact base `7b2e245190f9b680c7825041252030ead602e71a`

- **Verified — historical delivery.** The complete `f97771119f3e4a7a692ad76cfb0d694df443e707` diff removes the `validate_links` promise on 2026-08-05. Source-safe delivery anchor: commit subject `Stop the metadata contract promising a link gate it also says does not exist`.
- **Verified — later inversion.** `6a0184a5` and its integrated counterpart `757cb4c1` add local-link resolution on 2026-08-08. Complete `Makefile` anchors `check: citations fmt build lint test`, `citations:`, and `full: check doc` place `./check-citations.sh` on both standing gate paths. Complete `AGENTS.md` anchor `One mechanical property is checked` states the same current behavior.
- **Verified — stale contract claims.** Complete `docs/document-metadata.md` still carries the source-safe anchors `nothing in this repository resolves local links`, `There is no validator and no renderer`, and `Reading is the only standing check`; the Validation paragraph includes `local links` in the list maintained only by hand and reading.
- **Imprecise — link population.** The original outcome's unqualified `local links` omitted deliberate exclusions stated in the complete `check-citations.sh`: external targets are not resolved, pure heading anchors are skipped, a path carrying a fragment has only its path resolved, and links in vendored upstream sources under `docs/research/*/sources/` are not resolved. The outcome and residual requirements now carry those boundaries.
- **Imprecise — supersession.** The original outcome said the check does not check `supersession`. The script does read `superseded` to select the live-document population; what it does not validate is whether supersession metadata is correct. The outcome and residual requirements now say `supersession correctness`.
- **Verified — retained unchecked properties.** Complete `check-citations.sh` and the complete Validation section show that link meaning, heading-fragment correctness, frontmatter/schema rules, typed-edge semantics, catalog correctness, and quotation fidelity remain outside the standing gate. The two catalog/typed-edge scripts named under Validation remain hand-run ticket-body artifacts rather than repository tooling.
- **Verified — authority and scope.** Accepted ADR 0054's source-safe anchors `The repository checks metadata, relationship targets` and `broken paths and IDs fail checks` broadly overclaim the current tree beyond local-link path existence. Repairing that accepted decision belongs to `contracts/decisions`, not this ticket's exclusive `contracts/navigation` scope, and would be an authority expansion. This ticket therefore records observed standing-check behavior without editing or silently superseding ADR 0054.

**Reproduce.**

- Contract anchors: `nothing in this repository resolves local links`; `Reading is the only standing check`; Validation includes "local links" in the hand-maintained list.
- Gate path: Makefile `check: citations …`, `citations: ./check-citations.sh`, `full: check …`.
- AGENTS: `Documentation is manually maintained` paragraph naming `make citations` as the one mechanical property.

## Requirements

1. Rewrite the decision-cites-experiment and Validation prose so path-existence resolution via `make citations` / `check-citations.sh` is stated honestly, including the deliberate exclusions for vendored upstream sources and pure heading anchors and the path-only treatment of links carrying a `#` fragment. Keep residual honesty: frontmatter graph, catalogs, supersession correctness, quotation fidelity, link *meaning*, and heading anchors after `#` remain reading-only or hand-run ticket scripts, consistent with AGENTS' "Nothing else is validated".
2. Re-evaluate the decision-cites-experiment argument that enforcement "separates none of the options" for body links versus stored metadata edges: path existence of local body links is now gated; metadata edges and meaning remain unenforced; heading anchors still unchecked. Do not claim more than the checker does.
3. Name only `make citations` / `check-citations.sh`. Do not resurrect `validate_links`.
4. Do not reopen or re-edit the parent's close condition; parent stays `done` for the 2026-08-05 delivery.

## Closes when

Every present-tense claim in `docs/document-metadata.md` about whether local links are resolved or only hand-checked matches AGENTS and `check-citations.sh`; the decision-cites-experiment section's enforcement argument is re-evaluated against path-existence gating; residuals that remain unvalidated are still stated as such; `make citations` is green on the edited tree.

## Outcome

The accepted contract now names the standing check that exists: `make citations` runs `check-citations.sh`, and a missing path target in its checked link population fails `make check` and `make full`. The decision-cites-experiment argument now distinguishes the one property a body link gains over a stored metadata edge — path existence — from the meaning and qualification that neither representation receives mechanically.

**Retained exclusions.** External targets and pure heading anchors are not resolved; vendored upstream links under `docs/research/*/sources/` are skipped; a path carrying `#heading` has only its path resolved. A green result does not validate link meaning, frontmatter/schema rules, typed-edge semantics, supersession correctness, ticket or entrypoint references, catalogs, or quotation fidelity. The two catalog/typed-edge scripts remain historical hand-run ticket artifacts. The name `validate_links` appears only in the historical audit and requirement that forbid resurrecting it, never as live tooling or contract behavior.

**Failure demonstration.** Temporarily changing the real contract link target `../tickets/reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md` to `../tickets/reconcile-the-research-and-experiment-catalogs-with-their-frontmatter-planted-missing.md` made `make citations` fail with: `no tracked file or directory at tickets/reconcile-the-research-and-experiment-catalogs-with-their-frontmatter-planted-missing.md`. The subject perturbation was restored before the green run.

**Verification.** `make citations` resolves 6,444 local Markdown links and 1,189 pinned citations on the restored tree; `tkt lint --format json` reports `ok: true`; `git diff --check` is clean. Exact-base `tkt guard` is run on the committed branch and reported in the handoff because the guard deliberately does not treat uncommitted changes as branch diff.

**Full-gate carry.** The latest green full gate is main `0b0e6952aaa6c88f7c7be923c3158adba9d86add`. From that commit through the dispatched base, only the three claim-owned ticket files changed. This delivery adds only `docs/document-metadata.md` and this ticket. None of `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh` changes, so the full gate carries under AGENTS.md; the required `make citations` and `tkt lint --format json` checks were rerun.

**Authority residual.** ADR 0054 remains untouched. Its broader present-tense anchors about repository checks are an accepted-decision correction in `contracts/decisions`, not a licence for this `contracts/navigation` ticket to rewrite it. Recommended follow-up: **Reconcile ADR 0054's metadata-check promises with the surviving documentation gate**, exclusive scope `contracts/decisions`, shared `project/tickets`, related to this ticket and `reconcile-the-document-metadata-validator-claim-with-its-own-validation-section`.
