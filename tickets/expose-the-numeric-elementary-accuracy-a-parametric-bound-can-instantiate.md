---
id: expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate
title: Expose the numeric elementary accuracy a parametric bound can instantiate
status: done
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, convert-the-remaining-accuracy-predicate-shapes-to-a-relative-bound, derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause, decide-whether-to-admit-an-elementary-identity-permission, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, accuracy, compiler]
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

## Outcome — 2026-08-06

**A parametric bound can now obtain the number from the authority that owns it, and the reading the ticket demanded was redone rather than trusted.** `elementary_relative_accuracy` (`crates/tiler-compiler/src/target/accuracy.rs`) answers the numeric question beside `assess_elementary_accuracy`'s verdict, in the same module and against the same requirement table. Both branches of the "closes when" are satisfied: the query exists with its fail-closed direction watched failing, *and* the re-reading confirms the narrowing note above — the number was reachable from the requirement, and what was missing was the metric conversion, the admission gate, and a place to state the conversion's own region.

**Fact — the number is the requirement's, and that is the elimination.** Two sources were tested. The *declaration* side (`ElementaryRealization::contract`) states what one target promises; the *requirement* side (`required_elementary_accuracy`) states what every admitted target is held to. An admission is a proof that the realization refines the requirement, so the requirement's tolerance is an upper bound on every admitted realization's error — the weakest admissible number, which is the direction the ticket requires. The declaration side is eliminated on correctness, not preference: it would price a rewrite against one profile's promise, and a second profile admitted under the same requirement would then be priced by a number nobody checked it against.

**Fact — the number is gated on the admission, not on the requirement's existence.** A requirement no installed realization refines describes a target that declared nothing about the operation; quoting its tolerance would attribute an accuracy to a declaration nobody made. The query re-establishes the refinement through `assess_elementary_accuracy` and returns its `RefinementBasis` beside the number. `an_unrefined_realization_yields_no_number` is the perturbation that separates this from a requirement *lookup*: with the cross-metric row stripped, the requirement is untouched and would still convert, and the query refuses.

**Fact — the metric-conversion boundary is the returned object's own field.** `RelativeAccuracyDomain` is not an `Option` and not a doc note. `EveryAdmittedReference` is reached only when no clause needed the metric step, or when every clause that did carries an operation-specific proof bounding its own reference magnitude at or above the least normal — read from `ReferenceResultConstraint::magnitude`, which ADR 0042 admits only through such a proof. Otherwise the answer is `ReferenceMagnitudeAtOrAbove(2^-126)`, because below the least normal the metric's scale is the fixed subnormal gap while `|r|` keeps shrinking and *no* finite relative bound follows from any ULP bound. The obligation this ticket's note called conditional is therefore represented rather than hidden, and it is the same obligation the fold bound's no-subnormal side condition states — [`derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause`](derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause.md) owns discharging it.

**Measurement — the number, checked in exact rational arithmetic.** `tiler::softmax-f32@1` and `tiler::silu-f32@1` both yield `12 · 2^-23 = 24u`; `tiler::rms-norm-f32@1`'s `Faithful` requirement yields `2^-23 = 2u`; a `CorrectlyRounded { NearestTiesToEven }` contract yields `2^-24 = u`. The last is the instantiation both bound records use, and pinning it to the form that actually states correct rounding is what makes quoting `u` for a twelve-ULP requirement detectable. `the_registered_softmax_accuracy_is_twenty_four_unit_roundoffs` asserts the first-order ratio `(u + 24u)/(2u) = 25/2` exactly, so the narrowing note's `12.5` is now a checked value rather than a stated one.

**Fact — `NamedElementary` refuses, as this ticket required.** So do a bound under a foreign metric (the implication registry crosses two ULP definitions; it says nothing about the ratio against `|r|`), a result dtype with no derivable adjacent-value behaviour, and the four predicate shapes with no sound conversion — `Absolute` and `AbsoluteRelative`, which need a proved lower bound on `|r|` no registered contract states, and `AllOf`/`AnyOf`, whose tightness-versus-precondition trade is an unforced choice with no caller. Those four are filed rather than absorbed: [`convert-the-remaining-accuracy-predicate-shapes-to-a-relative-bound`](convert-the-remaining-accuracy-predicate-shapes-to-a-relative-bound.md), `deferred` with its trigger check log.

**Fact — the surface is `pub(crate)` and no boundary was self-accepted.** The whole module is crate-internal; the first consumer would be a rewrite rule, and whether any of this reaches a public boundary is that work's question under ADR 0075. No identity moved: nothing here registers a definition, changes an encoder, or touches a target profile declaration, and `grep -rnE '"[0-9a-f]{16}"|request=[0-9a-f]{16}' crates/ --include='*.rs'` returns the one request-subject pin (`crates/tiler-compiler/src/explain.rs:4174`, `689c3aefc30f48d3`) unmoved and green.

**Inference — ADR 0095's second reopening condition now has all three prerequisites, and that is a fact for Tom's queue rather than a decision.** The condition's "ready to consume" is concrete: (1) a rule in the certified-bounds admission shape, satisfied by [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md) in the sense the condition states; (2) a retrievable `eps_exp`, satisfied here; (3) the bound derived at a parallel fold shape, satisfied by [the tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md). **Two things must be said in the same breath, because acting on either alone would be wrong.** The condition asks for readiness and readiness now exists. It does *not* follow that admitting both permissions delivers the rewrite: that record's obligation 1 refuses on `SOFTMAX_F32_FACT_SUBNORMALS`, a registered measurement independent of both permissions, and its obligation 3 still wants a merge topology no schedule type carries. Nothing here recommends either outcome.
