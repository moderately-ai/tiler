---
id: expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate
title: Expose the numeric elementary accuracy a parametric bound can instantiate
status: in-progress
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, accuracy, compiler]
claimed_from: todo
assignee: agent-eps-exposure
lease_expires_at: 1786026692
---
## User-visible outcome

A parametric rewrite bound whose value depends on the target's relative accuracy for an elementary function can obtain that number from the authority that owns it, instead of being written against a plausible constant.

## Why this exists

**Fact.** `request::require_elementary_accuracy` (`crates/tiler-compiler/src/request.rs`) collects a program's operation keys and calls `target::accuracy::assess_program_elementary_accuracy` (`crates/tiler-compiler/src/target/accuracy.rs`), which requires some realization the target declares to *provably refine* each registered operation's accuracy obligation. `readmit_candidate` asks it again of every semantic candidate. The answer is a refinement verdict — an admission, or a typed refusal carrying the operation, the profile key, and the refusing authority.

**Fact.** That is a yes-or-no question, and it is the right question for the obligation it answers.

**Inference, from [the certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 3.** A per-rule parametric bound instantiates from shape and target facts, and for the worked online-softmax bound one of those facts is `eps_exp`, the target's *numeric* relative accuracy for `exp`. The refinement verdict does not carry it, so a bound written today would have to hard-code a constant — precisely the failure that record's trust-boundary section lists as the one a reviewer is likeliest to wave through, because a plausible constant looks exactly like a derived one.

## What this ticket must establish before it changes anything

**Read the accuracy authority in full first; the gap may be narrower than this ticket assumes.** `AccuracyContract` and its forms (`CorrectlyRounded`, `Faithful`, `NamedElementary`, `BoundedPiecewise`) already carry exact-rational tolerances in `tiler-ir`, and `tiler_reference::accuracy` compares against them exactly. It is possible that the numeric bound is already reachable from a declared realization and that only a query is missing rather than a representation. **Establish which before proposing a surface** — this ticket is filed as a gap on the strength of one reading, and that reading should be redone rather than trusted.

Note the asymmetry that must survive whatever is added: `assess_elementary_accuracy` is conservative in one direction only, so it may reject a legal implementation and can never admit an illegal one. A numeric accessor must inherit that direction — where several realizations could satisfy an obligation, the number handed to a bound is the **weakest** admissible one, or the query refuses.

`NamedElementary` is the hard case and is probably where a refusal belongs: its result set lives in an external descriptor that the reference evaluator holds only a digest of, which is why `decide_contract` answers `NamedProfileNotInterpretable` there. A numeric query cannot do better and must say so rather than guess.

## Narrowed 2026-08-06 — one reading redone, and the gap is smaller than filed

**This does not close the ticket and does not substitute for the re-reading above.** [The rule-object record](../docs/research/numerics/online-softmax-rule-object.md)'s Part 4 obligation 2 redid part of the reading this ticket asks for and reports three things.

**Inference — the weakest admissible number is already reachable one step to the side.** `required_elementary_accuracy` (`crates/tiler-compiler/src/target/accuracy.rs:739`) returns the operation's registered contract, and an admission is a proof that the installed realization *refines* it — so the realization's error is provably no worse than the requirement's tolerance. Instantiating a bound from the **requirement** is conservative by construction, which is exactly the asymmetry this ticket says a numeric accessor must inherit. The gap is therefore not "no number is reachable".

**Inference — what still refuses is the metric, not the retrieval.** The registered requirement is a ULP bound, not a relative one: `softmax_f32_exponential_accuracy_contract` (`crates/tiler-ir/src/semantic/softmax.rs:468`) is `BoundedPiecewise` with `AccuracyPredicate::ulp(ulp_reference_gap_metric_key(), 12)`. Converting to the relative `eps_exp` a bound consumes needs `ulp(r)/|r|`, and `UlpFormat::ulp_scale` (`crates/tiler-ir/src/semantic/accuracy/metric.rs:696`) returns the fixed subnormal gap below the least normal — so the ratio is bounded by `2^-23` for a normal reference and unbounded for a subnormal one. **The conversion is valid only where the consuming bound's own no-subnormal side condition is discharged**, which makes this query and that side condition dependent rather than independent obligations.

**Inference — the number the requirement gives is 24 times weaker than the constant both bound records instantiate at.** Above the subnormal band the ceiling is `12 · 2^-23 = 3 · 2^-21 = 24u`, against the `eps_exp = u` both records label a choice about the target rather than a fact about one. At first order that multiplies the online fold's price by `(u + 24u)/(2u) = 12.5`. Whatever surface this ticket lands, the number it hands a bound should be checked against that ratio, because a query that quietly returned `u` would look right and be twelve times optimistic.

## Non-goals

Implementing any rewrite bound; changing the refinement algebra; adding a public API without Tom's approval of the boundary, which stays his under ADR 0075.

## Closes when

Either a numeric relative-accuracy query exists with its fail-closed direction tested, or the reading establishes that the number is already reachable and the record's claimed gap is corrected at its point of use.
