---
id: propagate-extension-seam-classification-into-governed-contracts
title: Propagate the extension-seam classification into governed contracts
status: done
priority: p2
dependencies: []
related: [accept-adr-0078-public-extension-seams, draft-public-extension-seam-ownership-adr]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions, public-api]
---
Conditional on Tom accepting ADR 0078. That record classifies which surfaces Tiler intends as public extension seams, which are permanently internal, the maturity rung each has reached, and — most of its content — what a seam is *not*. Until it is propagated, the classification lives only in a decision record and no governed contract states it, which is the state `implementation_status: "partial"` reports.

On acceptance, represent the classification in the contracts that own the affected areas, without creating a second authority over what ADR 0078 already decides:

- `docs/operation-extensions.md` owns the public capability surface and the trust, identity, registration, and diagnostic obligations of a provider. It should gain the seam classification and the negative-space rules that constrain a provider surface — offering nothing is a legitimate local result, a resolved provider's claim is re-derived rather than inherited, an unenumerated capability fails closed as `Unknown`, an absent capability and a contended one are different findings, a reservation is not a capability, and a provider revision is provenance rather than a version negotiation.
- `docs/architecture.md` owns component ownership and the packaging profile. It should record which authorities are permanently internal, and the qualification ADR 0078 makes about explain (internal authority, public obligation) and feasibility (internal procedure, with the target-profile data left explicitly undecided).

Do not restate ADR 0078's reasoning or its open questions in either contract; cite the record. Do not propagate anything ADR 0078 leaves unassigned — the physical-implementation provider and the mature fusion numerical capability are recorded as open questions and must not acquire an intent by propagation.

Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` before completion.

## Released 2026-07-25 — Tom accepted ADR 0078

**Fact.** `docs/decisions/0078-name-the-intended-public-extension-seams.md` is `decision_status: "accepted"`. Reproduce: `grep -n decision_status docs/decisions/0078-name-the-intended-public-extension-seams.md`. It was accepted unamended, so this ticket's body needs no revision — the instruction in the paragraph before this section still holds exactly as written, including its refusal to propagate the two surfaces ADR 0078 leaves as open questions.

**The dependency edge is retired rather than satisfied.** [`accept-adr-0078-public-extension-seams`](accept-adr-0078-public-extension-seams.md) is a node only Tom closes, and Tom's decision on it directs this ticket to be repointed off it rather than to wait for that closure. It moves to `related` so the reason this work was parked stays legible; nothing schedules against it any more.

**Why no part of this ticket landed before now.** Every deliverable it names is conditional — "On acceptance, represent the classification in the contracts that own the affected areas". Writing the classification into `docs/operation-extensions.md` and `docs/architecture.md` while the record was proposed would have converted a proposal into fact inside two governed contracts, which `AGENTS.md` forbids directly ("do not silently convert a proposal into fact") and which is the specific failure the record itself guards against. There was no unconditional remainder to split out: the ticket has no deliverable that would have survived the ADR being rejected or amended.

**The precedent is explicit.** `propagate-accepted-api-conventions-into-governed-contracts` is the same shape one ADR earlier, and it propagated only after ADR 0074 reached `decision_status: "accepted"` — its title names the accepted state. ADRs 0072, 0074, and 0075, which 0078 depends on, are all `accepted`, and 0078 now is too, so this ticket is in the same posture that one was when it propagated.

**This ticket was scheduled as ready, and that was wrong.** Its one dependency was `draft-public-extension-seam-ownership-adr`, which is `done` — correctly, since drafting a *proposed* ADR was its whole outcome. The dependency graph had no node representing Tom's acceptance, so nothing separated "the ADR has been written" from "the ADR has been decided", and the ticket surfaced in `tkt ready`. It reached a worker's queue that way, and the worker parked it by hand.

**How that was prevented while the decision was outstanding.** [`make-adr-acceptance-visible-to-the-work-graph`](make-adr-acceptance-visible-to-the-work-graph.md) replaced the drafting dependency with [`accept-adr-0078-public-extension-seams`](accept-adr-0078-public-extension-seams.md), a node that only Tom closes and that sat in `awaiting-decision`. A parked dependency never satisfies a dependent, so this ticket stayed out of the ready frontier structurally rather than by its own status, and `tkt rollup` named the acceptance node as the reason it was blocked. That mechanism did its job: the ticket was held until the decision existed, and it is released by the decision rather than by a worker's judgement.

**The trigger fired on 2026-07-25.** It was ADR 0078 reaching `decision_status: "accepted"`, and it did, unamended. Items 4 and 5 were re-read against the accepted text before the edge was retired: nothing in them moved, so the instruction above still forbids propagating the physical-implementation provider and the mature per-operation fusion numerical capability, both of which the accepted record still carries as open questions rather than classifications.

## Outcome

The classification is now stated in both governed contracts. ADR 0078's `decision_status` and its body agree — frontmatter reads `accepted`, and the body's status line reads "accepted on 2026-07-25 … unchanged from the proposed text" — so the acceptance was carried out rather than only recorded, and this is a propagation of an accepted decision.

### The propagation

`docs/operation-extensions.md` gains a **"Public extension seams"** section stating what a seam is (the propose-then-re-verify admission test, as four properties a surface must hold), the five surfaces intended as third-party seams, and the two ADR 0078 deliberately leaves unassigned — named explicitly as unassigned, so standing beside five classified rows confers no intent on them. A **"What a seam is not"** subsection under *Capability coherence* carries the negative space. `docs/architecture.md` gains a **"Permanently internal authorities"** subsection under *Component ownership* carrying item 6 and both qualifications. Each contract cites the record for the reasoning and states neither its derivation nor its open questions.

### Which edits corrected a wrong governed statement, and which only recorded an already-true thing better

Three governed statements were **wrong** and are corrected:

1. **`docs/operation-extensions.md` — "the ordinary compiler API does not exist yet … `tiler-compiler` exports no compile entry point".** False at this base. `pub mod session` exposes `compile_governed(&SemanticProgram, NumericalContract)`, which Tom approved on 2026-07-25 under [`prototype-public-compiler-api`](prototype-public-compiler-api.md). What is *still* true, and is what the paragraph exists to establish, is that no out-of-crate caller supplies providers: `compile_governed` names the governed profile, and `CompilationRequest` and its `capabilities` field are both `pub(crate)`. The paragraph now separates three claims that were being carried as two — composing a registry, installing one, and reaching the compiler.
2. **`docs/architecture.md` — "*installing* one is not, because the request path is crate-private and no public compile entry point is exported".** The conclusion is right and half its stated reason is false. Corrected to the reason that actually holds.
3. **The status line — "remaining compiler capabilities proposed".** Scalar-lowering and reference capabilities are implemented, registered, resolvable, and tested; they are not proposed. Calling them proposed understates two rows of the accepted table by a whole rung.

Genuinely **new** normative content, absent from both contracts before: the seam inventory and the intended participation model; the four-property admission test; that offering nothing is a legitimate local result; that an unenumerated capability fails closed as `Unknown` where "conservative" means a refusal for lowering and an `Unknown` class for fusion legality; that an exhausted budget is a gap rather than a rejection; that a reservation is not a capability; that a provider revision is provenance rather than a version negotiation; that contention at one seam is not contention at another; and the permanently-internal list with the explain and feasibility qualifications.

Two of ADR 0078's negative-space rules were **deliberately not restated**, because the contract already states them and a second statement would be a second authority inside one document. *An absent capability and a contended one are different findings* is stated twice already, in *Registry lifecycle and coherence* and in *Capability coherence*; the new subsection says so where a reader would expect to find it. *A resolved provider's claim is re-derived, never inherited* is stated for its lowering realization in the same two places; it landed instead as the admission test's first property, which is the general form and does not repeat the specific one.

### A count was replaced by an invariant

ADR 0078's rung table is pinned to `412ceae`, which is correct in a decision record and would go stale in a contract. Rather than copy the rungs, `docs/operation-extensions.md` states the invariant that decides one, labelled **Inference**: a surface has reached a tested guarantee only when a provider written outside the defining crate's own governed set has driven it through the ordinary compile path and the resulting plan names that provider as its authority. The two rows below that bar are named with the reproducible reasons they sit there — no compile-path caller resolves the scalar family, and `tiler-reference` is a `[dev-dependencies]` edge of `tiler-compiler`.

### Verified rather than inherited

Every source fact ADR 0078 cites that this propagation repeats was re-checked at `c173ed3` rather than taken from the record. `resolve_scalar_lowering` has exactly two non-test sites, its own definition and the `scalar_provider` accessor; every caller is inside a `#[cfg(test)]` module in `capability.rs` or `legality.rs`. `crates/tiler-compiler/Cargo.toml` declares `tiler-ir` under `[dependencies]` and `tiler-reference` under `[dev-dependencies]` and nothing else. `LoweringSelector` carries exactly `{family, operation, signature}`, so resolution cannot see a provider revision. All five seam traits exist at the paths named. That check is what found the stale entry-point claims, which the record's own text would have propagated.

### Split

[`correct-adr-0078-public-module-and-entry-point-facts`](correct-adr-0078-public-module-and-entry-point-facts.md). Four of ADR 0078's item-3 and item-4 source facts were true at `412ceae` and are false at `c173ed3` for the same reason the contract statements were: `pub mod session` landed in between. The record is `contracts/decisions` and this ticket declares only `contracts/foundation`, so the correction is split rather than reached for. Its classification is untouched by the drift; only the evidence cited for item 4 and the trigger on its fourth open question are.

### No stale proposed-status disclosure

`validate_proposal_disclosure`'s obligation disappears on acceptance rather than inverting, so a disclosure that becomes wrong is invisible to the gate. Checked by reading, both directions: no `kind: contract` record cites ADR 0078 at all — `grep -rn "0078" docs/` outside `docs/decisions/0078-*` returns only the two generated `docs/decisions/README.md` catalog lines and ticket bodies — so accepting it created no disclosure to correct, which is what [`accept-adr-0078-public-extension-seams`](accept-adr-0078-public-extension-seams.md) predicted. The one contract citation of a formerly-proposed record, `docs/architecture.md`'s ADR 0077 citation, already reads "is the accepted record"; it was corrected when 0077 was accepted and is not stale.

### No open question closed

`docs/open-questions.md` tracks no entry for the seam-classification question, so nothing there is answered by this propagation and nothing was removed. Checked with `grep -n -i "seam" docs/open-questions.md`, which returns nothing, and by reading the four entries that name the operation-extension contract as owner — Q-SEM-007, Q-SEM-009, Q-SEM-011, Q-PKG-002 — none of which is about which surfaces are intended public.

`uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
