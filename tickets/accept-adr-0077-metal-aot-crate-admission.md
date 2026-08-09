---
id: accept-adr-0077-metal-aot-crate-admission
title: Accept or reject ADR 0077, the tiler-metal-aot crate admission
status: done
priority: p1
dependencies: [record-an-adr-for-the-metal-aot-crate-admission]
related: [admit-the-device-free-runtime-validation-crate, decide-the-expansion-cache-owner-and-digest-authority]
scopes: [contracts/decisions, contracts/navigation, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, governance, workspace]
---
**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. It is the graph node standing for a decision that has not been made, so that anything conditional on that decision is held out of the ready frontier by a dependency edge rather than by a worker noticing after being dispatched. Its permanent status is `awaiting-decision` — a `parked` category state that `tkt ready` excludes and that never satisfies a dependent.

**The four Facts below record the board as it stood before Tom decided, and every one of them was true then and false now.** They are kept because they are what the decision was taken against; the Outcome at the foot of this ticket records what the corpus says instead. The single reproduction command each carries still resolves, and now returns the accepted state.

**Fact.** [`docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md`](../docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) carries `decision_status: "proposed"`. Reproduce: `grep -n decision_status docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md`.

**Fact — this record points the opposite way from an ordinary proposal, which raises rather than lowers its urgency.** Its own status line says so: the crate, its empty dependency closure, and the development-only `tiler-metal` → `tiler-metal-aot` edge are already implemented and mechanically pinned in `scripts/check_workspace.py`, and what is missing is the decision. Until it is accepted, ADR 0056's retained clause — "MSL emission and AOT invocation remain modules in `tiler-metal`" — still stands as retained text that the workspace contradicts. `AGENTS.md` requires a durable decision to be superseded explicitly rather than silently departed from, and only acceptance of this record performs that supersession.

**Fact — the contradiction is currently disclosed rather than hidden, and that is the state acceptance ends.** [`docs/architecture.md`](../docs/architecture.md) line 350 names ADR 0077 as the *proposed* record, states that its supersession takes effect when Tom accepts it, and states that ADR 0056's retained packaging text still places AOT invocation inside `tiler-metal` until then. That paragraph is the model for how a governed contract may cite a proposed decision without asserting it; it is not a substitute for the decision.

**Fact — no ticket currently declares a dependency on this acceptance.** Reproduce: `grep -n 'dependencies:.*record-an-adr-for-the-metal-aot-crate-admission' tickets/*.md` returns nothing. Three tickets name `record-an-adr-for-the-metal-aot-crate-admission` under `related` only, and each was read in full: `correct-adr-0074-driver-vocabulary-consumers` corrects two falsified factual claims inside *accepted* ADR 0074 against measured source, `correct-artifact-crate-lockstep-ir-permission` corrects a crate doc comment against *accepted* ADRs 0056, 0070, and 0071, and `record-metal-aot-in-architecture-crate-profile` is `done`. None is conditional on ADR 0077 being accepted. This ticket therefore exists to hold the decision itself, and to be the edge target for any future ticket that would propagate ADR 0077's supersession into a contract.

## What Tom is deciding

Whether to admit `tiler-metal-aot` as a sixth reusable crate whose empty dependency closure and development-only inbound edge are decided properties rather than accidents of ordering, and thereby to supersede ADR 0056's retained AOT-invocation clause.

The record is deliberate about what it does *not* supersede, and accepting it accepts those judgements too:

- ADR 0065 is correct exactly as accepted; its "fifth reusable target-independent crate" is an ordinal about `tiler-reference`, not a cap on the profile, and it gains no superseding note.
- ADR 0070's dependency block is incomplete rather than wrong; ADR 0077 `refines` it by restating the block completely with six libraries and both development edges, instead of superseding correct edges to add missing ones.

## Closes when

`decision_status` in `docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md` moves off `proposed`, the record's status line is rewritten to match, `uv run --locked python scripts/docs.py render` regenerates `docs/decisions/README.md`, and `uv run --locked python scripts/check_repository.py` passes.

- **Accepted.** Set `decision_status: "accepted"`. `scripts/docs.py`'s graph validation then requires the accepted decision to carry `applies_to` and `evidence`, which ADR 0077 already has; ADR 0056 is already `decision_status: "superseded"` and already the target of `supersedes` edges from ADRs 0065, 0070, and 0077, so no further metadata moves. `docs/architecture.md`'s paragraph naming this record as proposed becomes stale in the same moment and must be rewritten to state the accepted packaging profile directly; that edit is `contracts/foundation`, so file it as its own ticket if the accepting change does not hold that scope.
- **Rejected.** Close with `tkt close` rather than `done`, so it does not satisfy dependents. Rejection does not restore the workspace to ADR 0056's retained clause — the crate exists and is pinned — so a rejection must be followed immediately by a ticket that either removes the crate or writes a different superseding record. Do not leave the contradiction undisclosed.

## Decision — Tom, 2026-07-25

**Accepted.** `decision_status` moves `proposed` → `accepted`.

Two consequences to carry out rather than assume: the disclosure at `docs/architecture.md:350` is no longer required by the proposed-decision gate (Check A) and may be reworded to cite an accepted decision; and this ADR's own clause that its admission must not be cited as precedent stays in force — it is the reason `admit-the-device-free-runtime-validation-crate` is a separate question rather than a corollary.

## Outcome

**Accepted by Tom, 2026-07-25.** `decision_status` moved `proposed` → `accepted` on `docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md`.

**The disclosure site was corrected rather than deleted.** `docs/architecture.md` opened its ADR 0077 paragraph with "No accepted ADR yet records that admission" and explained that the supersession of ADR 0056's retained AOT-invocation clause "takes effect when Tom accepts it; until then ADR 0056's retained packaging text still places AOT invocation inside `tiler-metal`". Both halves were true only while the record was proposed, and the second was making a live claim about where AOT invocation lives — so leaving it would have left the contract asserting the opposite of the accepted decision. Rewritten to state the supersession is in force.

The `validate_proposal_disclosure` gate check does not fire either way: it requires disclosure only for a *proposed* citation, so acceptance silently removes its obligation. That is precisely why the stale wording had to be found by reading rather than by the gate.

**Still in force:** the ADR's own clause that its admission must not be cited as precedent for admitting another crate. `admit-the-device-free-runtime-validation-crate` therefore remains a separate open question rather than a corollary of this acceptance.
## Outcome — landed 2026-07-25, awaiting Tom's close

**Status is `review`, not `done`, deliberately.** The standing rule at the head of this ticket — only Tom closes it, and no agent sets it `done` — was not lifted by the Decision section he added below it, so an agent landing the consequences is not the same act as closing the node. Everything the Decision directs is landed; the close is Tom's.

### What moved

`docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md` carries `decision_status: "accepted"`, and its status line records acceptance on 2026-07-25 with the decision unchanged from the proposed text. No Decision item was amended. `applies_to` and `evidence` were already present, which is what `scripts/docs.py`'s graph validation requires of an accepted decision, so no other frontmatter moved.

**The withheld edit is written.** ADR 0077's "Implementation boundary" named one edit as the acceptor's: ADR 0056's Decision paragraph gaining the same in-body `**Retired:**` marker its Consequences already carry from ADRs 0070 and 0071. That marker now sits beside the AOT-invocation sentence, states that ADR 0077 supersedes exactly the invocation half, and restates what is untouched — MSL emission, the IR and compiler-pass placements, and the withheld crate list, including that `tiler-metal-aot` is not an exception to the reusable Metal-*runtime* crate that list withholds. ADR 0056's status line no longer says ADR 0077 "is not accepted"; it names the supersession and its date. ADR 0077's own boundary paragraph was rewritten from announcing a pending edit to recording the one that landed.

### Every disclosure site, and what each sentence claimed

`validate_proposal_disclosure` identifies a site as an inline Markdown link, from a `kind: contract` record, resolving to a decision whose `decision_status` is `"proposed"`. Running that identification against the corpus with ADR 0077 held at `proposed` returns exactly one site — `docs/architecture.md:350` — which matches the count `gate-proposed-decision-assertions` measured at `1b75b19` and again at `b70da90`. ADR 0078 has no contract citation at all, so accepting it created no stale disclosure.

That one paragraph made three claims, each false the moment the record was accepted, and each was read before it was rewritten rather than deleted:

- *"No accepted ADR yet records that admission"* — it does now; ADR 0077 is that record.
- *"[ADR 0077] is the proposed record of it"* — the citation itself, and the word the gate looks for.
- *"That supersession takes effect when Tom accepts it; until then ADR 0056's retained packaging text still places AOT invocation inside `tiler-metal`, and this section and that proposal are together the record."* — the substantive half. It named the contract section as co-author of the packaging profile because no accepted decision held it. The replacement says the opposite and says why: ADR 0077 is the authority for the profile and this section states it.

The paragraph's last sentence — that ADR 0065 is correct exactly as accepted, its count being an ordinal about `tiler-reference` rather than a cap on the profile — is unchanged, because acceptance did not touch it and ADR 0077 restates the same judgement.

Three further citations were stale in the same direction and are outside the gate's predicate, since none is a contract citing a link:

- `docs/decisions/0077-*.md`'s own Context claimed ADR 0056's status line superseded it "only" for three things. That word was true while this record was proposed and false after. It now reads as the state before acceptance and points at the marker.
- `docs/decisions/0079-*.md`'s rejected alternative used ADR 0077 as a live contrast — "ADR 0077 is `proposed` because the workspace ran ahead of a decision Tom had not made". The contrast still holds and is now past tense with its resolution date.
- `tickets/decide-the-expansion-cache-owner-and-digest-authority.md` rested part of its conflict on ADR 0077 being a hypothesis under `AGENTS.md`. That reading is no longer available, which sharpens rather than resolves its question: two accepted authorities now assign the expansion cache to different components. Its "Also to decide" clause, which offered ADR 0077's acceptance as one possible carrier of the ownership-row correction, is corrected — acceptance did not carry it, so that decision must.

### What acceptance did not do

ADR 0077 item 5's clause that this admission must not be cited as precedent for a reusable Metal-runtime crate stays in force, and the status line now says so explicitly. [`admit-the-device-free-runtime-validation-crate`](admit-the-device-free-runtime-validation-crate.md) remains an open p0 question for Tom, not a corollary of this record, and nothing here narrows or widens it. The expansion-cache ownership conflict is likewise untouched and stays with its own decision node.

### The base commit was red, and that is why the two acceptances are one commit

**Measurement, reproducible in one line.** `git archive 63b02ec | tar -x -C <dir>` then `uv run --locked python scripts/docs.py validate --root <dir>` reports two errors at the dispatch base — this ticket and `accept-adr-0078-public-extension-seams` each depending on the ticket that drafted a still-`proposed` ADR. Commit `63b02ec`, "Unpark the acceptance nodes now Tom has decided them", moved both nodes out of `awaiting-decision`, and `validate_tickets` only exempts a *parked* ticket from that edge. Unparking them before their records were accepted is precisely the state Check B exists to report, so the gate was correct and the base was red.

**The consequence for this change.** Accepting one ADR clears one error and leaves the other, so no intermediate commit is green: the two acceptances are inseparable rather than merely convenient to land together, which is independent of their sharing one generated catalog. Splitting was attempted first and abandoned on this evidence.

**What that costs at the guard.** `tkt guard` compares a whole branch against one ticket's declared scopes, and this branch carries both acceptances. Guarded as this ticket, which declares the union, the verdict is `WARN` — declared-area overlaps only, no escape. Guarded as `accept-adr-0078-public-extension-seams`, it reports `contracts/foundation` under-declared, because this ticket's `docs/architecture.md` edit is on the same branch. That scope is not added to the other ticket, which touched no file under it; recording the asymmetry is the honest form. The integrator should guard this branch as `accept-adr-0077-metal-aot-crate-admission`.

### Gate

`uv run --locked python scripts/docs.py render` and the full `uv run --locked python scripts/check_repository.py` both pass; `git diff --check` is clean and `tkt lint` reports no problems.

## Current follow-on correction — 2026-08-09

The open-question statements above are historical. [`admit-the-device-free-runtime-validation-crate`](admit-the-device-free-runtime-validation-crate.md) is `done`: ADR 0081 admitted `tiler-runtime` as a device-free artifact loader by applying ADR 0077's own distinction, not by waiving its non-precedent clause. [`decide-the-expansion-cache-owner-and-digest-authority`](decide-the-expansion-cache-owner-and-digest-authority.md) is also `done`, and [`implement-the-expansion-cache-protocol`](implement-the-expansion-cache-protocol.md) delivered the dedicated `tiler-cache` crate under ADR 0082 while preserving `tiler-metal-aot`'s empty dependency closure. The cache ticket explicitly retains its complete-subject and process-crash/race follow-ups; this correction records ownership and admission, not closure of those narrower gaps.

## Current packaging-profile qualification — 2026-08-09

ADR 0077's six-library/two-executable restatement is an acceptance-time
ordinal, not the live complete workspace population and not a cap. The live
packaging profile in [`docs/architecture.md`](../docs/architecture.md) now
contains twelve reusable libraries, one conformance member counted separately,
and three non-published proof/integration executables. ADRs 0081, 0082, 0085,
0088, and 0104 admitted the later reusable rows; ADR 0106 admitted the
separately counted conformance member.

Those later admissions do not weaken what ADR 0077 still governs:
`tiler-metal-aot` owns offline Apple compiler invocation, keeps its empty
complete dependency closure, and receives only a development edge from
`tiler-metal`. The live architecture contract owns the current full population;
this completed acceptance ticket records the smaller profile Tom accepted on
2026-07-25 and the later records that extended it.
