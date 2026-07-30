---
id: accept-adr-0076-numerical-realizations
title: Accept or revise ADR 0076 on target-honourable numerical realizations
status: done
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

One of those, `record-delivered-numerical-realization`, was expected to create the first public numerical surface in `tiler-artifact` and was therefore separately Tom's to approve under ADR 0075. Its outcome remained crate-private, as the current qualification below records.

## Six open questions the record leaves explicit

Read them in the ADR's own "Open questions" section before deciding; they are recorded unresolved on purpose. The two most consequential: whether a caller's ordered preference list is the right shape or one contract plus an explicit caller retry is (the record chooses the list and says the alternative was not rejected on evidence), and whether `SupportedOnlyUnderDeclaredRelaxation` earns a distinct implemented outcome or is only an explain-trace refinement.

## What closes this ticket

Either set `decision_status: accepted` with an acceptance date and unblock the four implementation tickets, or record the requested revisions here and send the record back. If accepted with modifications, amend the ADR rather than superseding it — it has never been operative.

## Outcome

Tom accepted ADR 0076 on 2026-07-24 without modification: "yes accept, i have no issue". `decision_status` is now `accepted` and the status line records that the decision is unchanged from the proposed text.

`implementation_status` deliberately stays `not-started`. Acceptance authorizes the four ordered follow-up tickets; it does not perform them, and the record's six open questions remain open on purpose rather than being closed by acceptance.

**What this unblocks, in dependency order.** `widen-numerical-vocabulary-and-complete-identity` (`tiler-ir`: grow `SubnormalMode` and `NumericalPermission`, complete `schedule::model::push_numerical`, stop `derive_requirements` collapsing the realization into one bit, and state in `docs/ir.md` where the realization sits in the identity layering) → `select-numerical-contract-and-compose-feasibility` (`tiler-compiler`: the contract becomes a required typed request input, the per-dimension honourability authority lands as a peer of `CheckedTargetProfile`, and the boolean `supports_strict_f32` axis is retired) → `declare-metal-numerical-honourability` and `record-delivered-numerical-realization` in parallel. None is independently shippable: widening the vocabulary without completing the identity encoding is a correctness defect, and widening it without the profile declaration leaves the new variant unreachable.

**One boundary remains Tom's separately.** The delivered-realization work adds the first public numerical surface to `tiler-artifact`, which ADR 0075 reserves for his approval regardless of this acceptance. Accepting ADR 0076 is not accepting that boundary's eventual shape.

**Current qualification (2026-07-29).** `record-delivered-numerical-realization` landed only a crate-private four-dimension draft; it did not add a public surface. A full-tree audit later disproved that draft's dtype-free, opaque-means, incomplete-provenance shape. The current path is serial: caller-declared profile decision → shared Metal honourability form → structured provenance → compile-checked delivered-record redesign → exact public-boundary acceptance → production wiring. This qualification preserves the historical ADR acceptance while correcting the present implementation account.

**Two measurement refinements landed before acceptance** and are recorded inline in the ADR rather than as amendments, since neither changes a conclusion: the fast-math flag spellings hold only at `-ffp-contract=fast`, and the emitted-operation-count account of the arithmetic-deletion trap is complete at `-O2` and incomplete at `-O0`, where two operations survive into the readable IR and still do not execute. The second strengthens the record's central inference.

**One claim in the ADR is still unreproduced** and is tracked rather than hidden: the re-verification asserts an additive-path input flush that the checked-in harness does not establish, because every probe kernel that adds does so after a multiply. `extend-the-numerical-probe-to-an-additive-path-kernel` owns closing it. Separately, `measure-numerics-across-apple-artifact-families` establishes whether the subnormal flush is Apple-wide or per-family — which `declare-metal-numerical-honourability` needs before it can decide whether honourability is declared once or per artifact family.
