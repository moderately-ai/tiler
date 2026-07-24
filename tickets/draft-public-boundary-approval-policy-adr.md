---
id: draft-public-boundary-approval-policy-adr
title: Draft a proposed ADR for the public-boundary approval policy
status: in-progress
priority: p1
dependencies: [draft-public-api-conventions-adr]
related: [draft-public-extension-seam-ownership-adr]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, governance, process]
claimed_from: todo
assignee: agent-draft-public-boundary-approval-policy-adr
lease_expires_at: 1784908598
---
Record, as a **proposed** ADR, which changes require Tom's explicit review before
merge and which a coordinator may merge autonomously. `AGENTS.md` today says Tom
must review "key public crate, module, trait, type, and call-site boundaries"
without defining the boundary of *key*, so every ambiguous case costs a
round-trip and the ambiguity is resolved by judgement rather than by rule.

Depends on `draft-public-api-conventions-adr` because the autonomous half of the
policy is meaningless until "conforms to the conventions" names something
checkable.

## Why now (evidence)

Across the authorities landed so far, three changes went to Tom (a new crate, a
new public module, one public method plus its error type) and all three were
approved essentially as designed; four others never needed him because they were
`pub(crate)` private drafts. The single substantive review catch was a missing
`#[non_exhaustive]` — a convention gap. So per-case approval had a low catch rate
relative to its latency, and its one catch is better served by a written
convention. That argues for a policy that spends Tom's attention on genuine
compatibility commitments instead of on conformance.

## Policy to record — **Tom decided this boundary on 2026-07-24**

The exact split below is no longer a proposal to survey: Tom was asked the
boundary directly and chose it, over a tighter variant (also bring him every new
public *type*) and a looser one (let the coordinator promote `pub(crate)` to
`pub` unaided). Record it as his decision with that context, and do not re-open
the alternatives as though undecided — note only that the looser variant was
declined because promotion is the moment a surface becomes externally
load-bearing, and may be revisited once the optimizer conformance gate has
actually exercised a seam end to end.

This ticket's dependency is satisfied: ADR 0074 (the conventions) was accepted on
2026-07-24, so "conforms to the conventions" now names something checkable, which
is what makes the no-approval half of this policy safe rather than open-ended.

- **No approval required:** a new compiler-internal authority introduced as a
  `pub(crate)` draft; additive `#[non_exhaustive]` growth; a new public error,
  provenance, or identity *record* that conforms to the conventions ADR; tests;
  documentation.
- **Always requires Tom:** a new **publicly reachable namespace** — a new crate,
  or a new `pub mod` in a crate root or in an already-public module; a new public
  **trait** (an extension seam something else implements); any breaking change to
  an existing public signature; promoting a module or type from `pub(crate)` to
  `pub`.

### Amendment 2026-07-24: the namespace reformulation

The first item originally read only "a new crate", which was **narrower than the
practice it was calibrated against**: `tiler_ir::schedule` and `tiler_ir::kernel`
were both new public *modules* with large surfaces (the latter ~4,600 lines) and
both went to Tom under the prior judgement-based rule, yet the literal wording
would not have required it. `AGENTS.md` already lists "module" alongside crate,
trait, and type, so the omission was a drafting artifact rather than a considered
narrowing.

Tom was asked directly and accepted the reformulation above over the alternative
"a new public module with a *substantial* surface" — declined because
"substantial" reintroduces exactly the judgement term that makes `AGENTS.md`'s
existing "key public … boundaries" ambiguous, which is the ambiguity this policy
exists to remove. Record the accepted cost explicitly: a trivial two-item
`pub mod` now also requires review, judged acceptable because such modules are
expected to be rare.

Record explicitly that a coordinator's terminal-merge authority is conditional on
the objective gates that already exist — a green `scripts/check_repository.py`, a
`ticketsplease guard` with no scope escape, scope conformance, and a full review
of the actual diff rather than an agent's summary — and that any of those failing
returns the change to Tom regardless of category.

## Deliverable and boundaries

Create the ADR at the next free number with `decision_status: "proposed"`, its
`ticket` field pointing at this ticket, the evidence above in Context, and every
unsettled boundary listed as an explicit open question. Do **not** mark it
accepted, and do **not** edit `AGENTS.md` here: if the policy is accepted,
propagating it into the working contract is a follow-up so that a *proposed* ADR
never silently becomes the operative rule.

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.

## Outcome

**Fact — deliverable.** `docs/decisions/0075-scope-public-boundary-approval-by-change-category.md` records the policy as `decision_status: "proposed"` with `ticket: "draft-public-boundary-approval-policy-adr"`, plus the regenerated `docs/decisions/README.md` catalog blocks. Frontmatter: `catalog_group: "documentation-governance"` (the corpus's only governance bucket; ADR 0054 is its sole other member), `implementation_status: "not-started"`, `applies_to: ["tiler.contract.architecture"]`, `evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv", "tiler.research.extensions.operation-extension-surface"]`, and `depends_on: ["ADR-0074"]`. `AGENTS.md` was deliberately not edited; propagation remains a follow-up conditional on acceptance.

**Fact — the compatibility premise this ticket used was rejected and is recorded as rejected.** The ADR does not justify any always-ask item by compatibility, downstream breakage, or a stability commitment. The supporting evidence is mechanical: `[workspace.package]` sets `version = "0.0.0"` and `publish = false`, and all six library crates plus both prototype binaries inherit both via `publish.workspace = true`, so no crate here is publishable and every consumer of every public item is in-workspace. The surviving justification is design visibility — seeing the surface as it forms, while reshaping is cheap — plus crates and traits being the largest-grained architectural commitments even when reversible. The ADR states explicitly that a future reader who finds a compatibility argument attached to this policy should treat it as reintroduced rather than inherited.

**Fact — a ticket premise that did not survive checking.** This ticket's "four others never needed him because they were `pub(crate)` private drafts" is not verifiable from the tree, and the checkable artifact at `dc9990d` is six crate-private compiler authority modules (`cover`, `explain`, `feasibility`, `frontier`, `fusion_legality`, `selection`), each with zero bare `pub` items, alongside exactly two `pub` modules (`capability`, `legality`). The ADR records the ticket's counts as the ticket's claim, notes that six modules and four changes are not comparable, and does not depend on either number.

**Fact — ADR 0074's convention 7 is accurate, verified by reading the files.** All six crate-private compiler authority modules open with the module-level `#![allow(dead_code, reason = "…")]` the convention prescribes, each reason naming what the surface reserves and why it is not yet reachable: `explain.rs:1`, `feasibility.rs:1`, `fusion_legality.rs:40`, `cover.rs:44`, `frontier.rs:53`, and `selection.rs:60`, identical at `dc9990d` and on `main` (`40dcc70`). The attribute is written across multiple lines, so the literal substring `allow(dead_code` never occurs in the tree and any `grep` or `git log -S` on that pattern returns a false negative. An earlier draft of this outcome recorded exactly that false negative as a defect in ADR 0074; it was wrong and has been deleted rather than softened. ADR 0075 now defines its `pub(crate)`-draft category by deferring to convention 7 outright instead of restating a parallel definition of the same shape — a better formulation independently of how the error arose, because the category can no longer drift away from the rule it depends on.

**Fact — the six modules conform uniformly; there is no non-conforming sibling.** `selection` was suggested as an exception carrying only an item-level allowance. It is not: `selection.rs:60` is the module-level `#![allow(dead_code, reason = "…")]` and is that file's sole `dead_code` occurrence, while its only item-level `#[allow(...)]` is `clippy::too_many_arguments` at `selection.rs:967` on an enumerator function — an unrelated lint. Counted mechanically, `#![allow(` appears exactly once in each of the six modules. No conformance gap is recorded here, because none exists.

**Measurement — the gap that prompted the reformulation.** The first wording of the always-ask list opened with "a new crate" and did not cover *introducing* a new `pub mod`, which is not a new crate, not a new public trait, not a signature change, and not a promotion. Two such surfaces exist: `crates/tiler-ir/src/lib.rs` declares `pub mod schedule` over six files totalling 1,785 lines at `dc9990d`, and on `tkt/prototype-structured-kir-slice` it gains `pub mod kernel` over eight files totalling 4,616 lines (3,436 excluding its 1,180-line test module), carrying a public builder, verified product, identity type, read views, and a large typed vocabulary. That both went to Tom under the prior judgement-based rule is reported, not verifiable from repository artifacts, and is labelled as such in the ADR. The literal first wording would not have required either review, so it was looser than the practice it was calibrated against — the one direction a rule replacing judgement must not drift.

**Fact — Tom decided the gap; the ADR records it as decision, not as an open question.** The always-ask list now reads: a new publicly reachable namespace (a crate, or a `pub mod` in a crate root or an already-public module); a new public trait; a breaking change to an existing public signature; promoting `pub(crate)` to `pub`. The namespace wording is judgement-free and subsumes the crate category Tom originally chose, so his decision is extended along its own grain rather than overruled. He declined "a new public module with a *substantial* surface" because "substantial" reintroduces the judgement term that makes `AGENTS.md`'s existing "key public … boundaries" ambiguous. The ADR records the accepted cost explicitly: a two-item `pub mod` now requires review exactly as a crate does, judged acceptable because such modules are expected to be rare and the ambiguity the alternative reintroduces would be paid on every change rather than on the rare small one.

**Open questions left explicit.** Three, down from five: Tom's namespace decision resolved both the module-introduction gap and the commitment-versus-keyword formulation, so neither survives as a question. (1) Where the policy becomes normative — `applies_to` can only name contracts, but the real destination is `AGENTS.md`, which `scripts/docs.py` does not govern, and `docs/work-tracking.md`, which is a portal and an illegal `applies_to` target; the schema has no "governs the working contract" relation and `documentation-governance` is the closest catalog bucket rather than an exact fit. (2) Whether adding a variant to a *recognized* `#[non_exhaustive]` enum is the additive growth a coordinator may merge, given that it compiles at every cross-crate consumer while silently flipping their reject-unknown arm from accept to reject; a dedicated follow-up ticket owns this and the ADR references it as unresolved rather than settling it. (3) Whether adding a defaulted method, or a satisfied supertrait bound, to an existing public trait is covered.

**Fact — verification.** `uv run --locked python scripts/docs.py render` passed (177 records); `uv run --locked python scripts/check_repository.py` passed end to end, including the Rust sub-gate; `git diff --check` clean; `ticketsplease guard tkt/draft-public-boundary-approval-policy-adr` reported no scope escape. Changes stayed inside `contracts/decisions` (the new ADR), `contracts/navigation` (the regenerated catalog), and `project/tickets` (this outcome).

**Remaining gate.** The ADR is `proposed`. Acceptance is Tom's separate step, and propagation into `AGENTS.md` is a further deliberate follow-up that has no ticket yet.
