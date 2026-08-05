---
id: expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate
title: Expose the numeric elementary accuracy a parametric bound can instantiate
status: todo
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions]
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

## Non-goals

Implementing any rewrite bound; changing the refinement algebra; adding a public API without Tom's approval of the boundary, which stays his under ADR 0075.

## Closes when

Either a numeric relative-accuracy query exists with its fail-closed direction tested, or the reading establishes that the number is already reachable and the record's claimed gap is corrected at its point of use.
