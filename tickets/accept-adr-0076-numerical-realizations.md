---
id: accept-adr-0076-numerical-realizations
title: Accept or revise ADR 0076 on target-honourable numerical realizations
status: todo
priority: p1
dependencies: []
related: [draft-target-honourable-numerical-contract-adr, widen-numerical-vocabulary-and-complete-identity]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, numerics, needs-tom]
---
**This ticket is Tom's decision, not an agent's work item.** It exists so the four implementation tickets that follow ADR 0076 have something to depend on, rather than being schedulable while the record they implement is still proposed. `AGENTS.md` is explicit that a proposed ADR is a coherent hypothesis and not a commitment, so starting the implementation before acceptance would cross the implementation boundary on an unaccepted design.

`docs/decisions/0076-declare-target-honourable-numerical-realizations.md` is `decision_status: proposed`, `implementation_status: not-started`. Nothing operative changed when it merged.

## What the record decides

A caller states the resolved numerical contract as a required, typed compile input with no default. A target profile declares, per contract dimension, which behaviours it honours and by what means — honoured exactly, honoured by exact emulation the backend emits, honoured only under a relaxation the caller already authorized, or unhonourable. Feasibility *assesses* that declaration and never chooses the contract, because a planner that picked the contract would let a target's limitation redefine what the program means. When nothing the caller stated is honourable, compilation rejects with a typed error naming the dimension, the required behaviour, the target's declared behaviour, and the declaring profile's identity — never a silent downgrade.

## The two places the record contradicts the ticket that commissioned it

Both are worth attention, because both were corrections rather than elaborations.

**The vocabulary was already decided.** The commissioning ticket framed this as a vocabulary gap. Accepted ADR 0019 already resolves subnormal input and result handling independently with preservation or explicit flush-to-zero on each, `docs/numerical-semantics.md` already spells `SubnormalContract { inputs, results }`, and the conformance matrix already requires all four combinations as coverage. The gap is in the *implemented enums*, not the design. Inventing a vocabulary would have created a second authority over the same terms.

**"Feasibility selects a conformant contract" is the wrong verb**, by the commissioning ticket's own architectural line. `docs/artifact-abi.md` already forbids the neighbouring case: routing never chooses between different accuracy meanings. The caller states the contract; feasibility only assesses it.

## What acceptance commits to

Four crates change and none of the changes is independently shippable. Widening `SubnormalMode` without completing the identity encoding is a correctness defect; widening it without the profile declaration leaves the new variant unreachable. The follow-up tickets are ordered, not parallel: `widen-numerical-vocabulary-and-complete-identity` → `select-numerical-contract-and-compose-feasibility` → `declare-metal-numerical-honourability` and `record-delivered-numerical-realization`.

One of those, `record-delivered-numerical-realization`, creates the first public numerical surface in `tiler-artifact` and is therefore separately Tom's to approve under ADR 0075.

## Six open questions the record leaves explicit

Read them in the ADR's own "Open questions" section before deciding; they are recorded unresolved on purpose. The two most consequential: whether a caller's ordered preference list is the right shape or one contract plus an explicit caller retry is (the record chooses the list and says the alternative was not rejected on evidence), and whether `SupportedOnlyUnderDeclaredRelaxation` earns a distinct implemented outcome or is only an explain-trace refinement.

## What closes this ticket

Either set `decision_status: accepted` with an acceptance date and unblock the four implementation tickets, or record the requested revisions here and send the record back. If accepted with modifications, amend the ADR rather than superseding it — it has never been operative.
