---
id: define-implementation-status-for-superseded-decisions
title: Define implementation status for superseded decisions
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, governance, metadata]
claimed_from: todo
assignee: agent-define-implementation-status-for-superseded-decisions
lease_expires_at: 1784834740
---
The ADR status audit found a corpus inconsistency it correctly declined to fix, because the metadata contract does not define the case. ADR 0057 carries `decision_status: "superseded"` together with `implementation_status: "implemented"`, but the state it claims to have implemented — a workspace `rust-version = "1.89"` floor — is no longer true: ADR 0067 replaced it with the pinned nightly, and no `rust-version` is declared in the workspace or any member manifest. Verified 2026-07-23.

The underlying question is a governance one and belongs to `docs/document-metadata.md`: what does `implementation_status` mean once `decision_status` is `superseded`? Three readings are defensible and the contract currently permits all of them — that the field records what was historically built (so "implemented" stays and is read historically), that it records the present state of the codebase (so a superseded decision whose implementation was replaced should not read "implemented"), or that the field is meaningless on a superseded record and should be constrained or dropped.

Decide one reading, state it normatively in the metadata contract, enforce it in `scripts/docs.py` where it is mechanically checkable, then apply it to ADR 0057 and audit the other superseded decision (ADR 0056) for the same class of drift. Do not simply flip 0057's value without settling the rule, or the next audit will reopen it. Run the full documentation gate before completion.

## Outcome

Chose reading (a): `implementation_status` records the highest implementation maturity a record's own decided behaviour reached — a retained high-water mark, not a live mirror of the working tree. Supersession updates `decision_status` alone and never lowers `implementation_status`; on a `superseded` decision the field is read historically, and the superseding decision carries the present-state maturity of the replacing contract. This preserves the "did this ever ship?" provenance that (b) present-state and (c) drop/sentinel both destroy — under (b) a void decision's present maturity collapses toward `not-started` because the codebase implements the successor, erasing history and forcing a value the enum cannot express ("built then reverted"); (c) contradicts the required-fields table and needs a schema carve-out to discard the same information. Reading (a) is also the only coherent reading for a non-current decision (its realization is inherently a past question) and requires no value change to either superseded ADR, so a future audit re-reads the rule and stops rather than reopening it.

Normative statement added to `docs/document-metadata.md` (Kinds and status facets): "`implementation_status` names the highest implementation maturity the record's own decided behaviour has reached. It is a retained high-water mark, not a live mirror of the working tree: superseding a decision updates `decision_status` alone and never lowers `implementation_status`. On a `superseded` decision the field is therefore read historically — the maturity the work reached while the decision was in force — while the superseding decision carries the present maturity of the contract that replaced it. A superseded decision keeps `implemented` when its work was built and later replaced; it reads `not-started` only when it was superseded before any of its work was built." A companion invariant in the required-fields section requires every `superseded` decision to be the target of at least one decision `supersedes` edge, and every decision named as a `supersedes` target to be itself `superseded`, so the successor carrying present state is always reachable and the retained value stays legible.

Enforcement: `scripts/docs.py` `validate_graph` now checks that biconditional over decision records (superseded ⇔ inbound decision `supersedes` edge) and reports `superseded decision must be the target of a supersedes edge from its replacement` or `decision named as a supersedes target must be superseded`. Locked by `test_superseded_decision_and_replacement_reference_each_other` in `scripts/tests/test_docs.py`, which asserts an orphaned superseded decision fails, an unmarked supersedes target fails, and a mutually referencing pair passes.

Applied to ADR 0057: value stays `implemented`. Evidence that the Rust 1.89 workspace floor was built and then replaced: ADR 0057's own body ("The Rust 1.89 workspace floor was implemented"), ADR 0067 ("Workspace implementation removes the claim that these nightly-only crates support a stable MSRV"), and the current tree, where `[workspace.package]` and every member manifest declare no `rust-version` and `rust-toolchain.toml` pins `nightly-2026-07-19`. The successor ADR 0067 supplies the enforced inbound `supersedes` edge, so the historical `implemented` is now legible rather than contradictory. The lone `rust-version = "1.89"` in `spikes/shapes/shape-evidence/Cargo.toml` is a standalone spike workspace (its own `[workspace]` table), not a tiler-workspace member, so it does not reinstate ADR 0057's floor.

Audited ADR 0056: value stays `partial`. It is a deliberately reduced prototype crate layout whose realization was partial and then partly retired by ADRs 0065/0070/0071; `partial` is a valid historical high-water mark, and inbound `supersedes` edges from ADR 0065 and ADR 0070 satisfy the reachability invariant. No value change was warranted, so the "same class of drift" did not recur under reading (a). No ADR frontmatter changed, so the generated decision catalog is unchanged.

Gate: `uv run --locked python scripts/check_repository.py` reports `complete repository validation passed` (148 pytest cases including the new one, docs validate over 175 records, catalog fresh, Rust gate passed). `git diff --check` clean. `tkt guard` stayed within scope.
