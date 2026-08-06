---
id: close-the-eps-exp-open-axis-in-the-rule-object-record
title: Close the eps_exp open axis in the rule-object record
status: done
priority: p3
dependencies: [expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate]
related: [derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, connect-certified-rounding-error-bounds-to-rewrite-permissions]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, accuracy, docs]
---
## User-visible outcome

A reader of [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md) and [the certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) learns that the numeric `eps_exp` gap is closed and what the number is, instead of reading two open axes whose owning ticket has landed.

## Why this exists

**Fact.** [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md) landed `elementary_relative_accuracy` in `crates/tiler-compiler/src/target/accuracy.rs`. Both records name the gap it closed, and neither could be edited by that ticket: its scopes were `implementation/compiler` and shared `project/tickets`, and `research/numerics` was outside them.

**Fact — three sentences are now false and one is a status claim rather than a derivation.** The rule-object record's open axes state "The numeric `eps_exp` gap is narrower than filed … → a dated note on [that ticket], which stays `todo`"; its Part 4 obligation 2 is headed "**refuses**, and the gap is narrower than it was filed as"; its Part 6 states "**Fact — prerequisite (2) is not satisfied.** `grep -m1 '^status:' tickets/expose-…` returns `status: todo` at this base." The certified-bounds record's open axes list "The target accuracy authority cannot yield the numeric `eps_exp` a bound needs" with no resolution note, where its sibling axes carry dated ones.

**Inference — the correction is not merely a status flip, and that is why it is a ticket rather than a sweep.** Part 6's readiness statement is a derivation over three prerequisites, and the third element of it moving changes the conclusion: the condition's stated prerequisites are now all satisfied. The record's own reconsideration-trigger section says "**The condition has not fired**", which stops being true. Restating it needs the same care the original had — in particular, that readiness existing and the rewrite becoming reachable are different claims, because obligation 1 still refuses on `SOFTMAX_F32_FACT_SUBNORMALS` independently of both permissions and obligation 3 still wants a merge topology no schedule type carries.

## What this ticket must produce

- Part 4 obligation 2 restated at its new verdict, with the number the query returns for each registered family read from source rather than from this ticket: `24u` for both exponentials, `2u` for the faithful reciprocal square root, `u` only for a contract that states correct rounding.
- The metric-conversion boundary recorded as what it became — a field of the returned answer naming the reference magnitude the bound holds at or above — and its dependence on obligation 1 restated as the *same* obligation rather than a second one.
- Part 6's readiness statement re-derived, and the reconsideration-trigger sentence corrected, keeping the two claims separate.
- Dated resolution notes on both records' open axes, in the shape their sibling axes already use.

## Non-goals

Deciding either permission; re-deriving any bound; editing the ADR, whose reopening condition is unchanged by this — what changes is whether its prerequisites are met, which is a fact the records report rather than a decision they make.

## Outcome — 2026-08-06

**Both records now report the closed gap, and every number in them was read at `crates/tiler-compiler/src/target/accuracy.rs` at `28aa5f0d` rather than carried from this ticket.** The three false sentences and the one status claim are corrected in their own idiom, tense-preserving, with the original derivations preserved beside each correction.

**Fact — the per-family numbers, read at source and confirmed against the module's own exact-rational tests.** `tiler::softmax-f32@1` and `tiler::silu-f32@1` both yield `12 · 2^-23 = 24u`; `tiler::rms-norm-f32@1`'s `Faithful` reciprocal square root yields `2^-23 = 2u`; `u` is returned only for `AccuracyContractForm::CorrectlyRounded { NearestTiesToEven }`, a form no registered family states. `the_registered_softmax_accuracy_is_twenty_four_unit_roundoffs` asserts `(u + 24u)/(2u) = 25/2` exactly, so the rule-object record's inferred `12.5` is now a checked value. The ticket's stated numbers and the source agree; no disagreement to report.

**Fact — the metric-conversion boundary is recorded as what it became.** `RelativeAccuracyDomain` is a field of the returned answer — `EveryAdmittedReference`, or `ReferenceMagnitudeAtOrAbove(ExactRational)` naming the reference magnitude the bound holds at or above. The softmax's is `ReferenceMagnitudeAtOrAbove(2^-126)`, because `clause_proves_a_normal_reference` reads the clause's own `ReferenceResultConstraint` magnitude and no registered contract states one. Both records now state that this is obligation 1's no-subnormal clause seen from the accuracy side — the *same* obligation, not a second one.

**Fact — Part 6 re-derived, with the two claims kept apart.** `grep -m1 '^status:' tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md` returns `status: done` at `28aa5f0d`, so all three prerequisites of ADR 0095's second reopening condition are satisfied and **the condition has fired** — stated as a fact, together with the fact that acting on it (reopening the distributivity decline) is a decision nobody has taken and this work does not take. Readiness and reachability stay separate claims: obligation 1 still refuses on `SOFTMAX_F32_FACT_SUBNORMALS`, independently of both permissions, and obligation 3 still wants a merge topology no schedule type carries, so admitting both permissions would change the refusal rather than deliver the rewrite.

**Fact — what moved, beyond the four sentences the ticket named.** The rule-object record's Outcome tally, its Part 2 parameter-provenance sentence, its Part 4 intro tally (`Five refuse. One discharges.` → four refusals, one conditional discharge, one clean one, with obligations 1, 3, 4, and 5 standing exactly as derived), its "Closes" summary, and its base-verdicts caveat all carried the superseded verdict and each is corrected in place. The certified-bounds record's Part 3 obligation 2 carried the same claim as its open axis and is corrected beside it. `elementary_relative_accuracy` is `pub(crate)` and unconstructed on the compile path, which the rule-object record now states explicitly, so "retrievable" is recorded as a fact about the authority rather than about any caller.

**Commands.** `tkt lint`; `git diff --check`; `tkt guard --base 28aa5f0d tkt/close-the-eps-exp-open-axis-in-the-rule-object-record`; `git diff --name-only`. Docs-only — no crate file was edited, and `crates/tiler-compiler/src/target/accuracy.rs` was read but not touched, its scope being held by a live worker.
