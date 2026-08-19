---
id: close-the-contributor-source-carrier-s-review-remainder
title: Close the contributor-source carrier's review remainder
status: todo
priority: p2
dependencies: []
related: [replace-the-serial-sum-contributor-fields-with-the-exhaustive-source, re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement]
scopes: [implementation/compiler, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, documentation, test-coverage]
---
## User-visible outcome

The comments and tests left behind by the exhaustive contributor-source carrier describe what the code now does: no doc comment states a fact the migration retired, and the newly admitted producer-and-produced-fold population has a regression test rather than only a probe result.

## Why this exists

Filed 2026-08-19 from the independent review of `replace-the-serial-sum-contributor-fields-with-the-exhaustive-source` at `e2e25b44`. That review found **no correctness defects** at any rank and independently derived the identity claim, the exemption narrowing, the admission routing, and the fixture resize. These are its minor and informational findings, verified by the coordinator at `fac004c6` before filing. **Nothing here is a behavioural bug** — every item is a claim-about-behaviour or a coverage gap.

**Fact — a doc comment states the retired fact, while its sibling was corrected.** `crates/tiler-compiler/src/physical.rs`, on `subject_contributor_tensor`, anchor `derived from the same fact` (1 hit). The comment says the projection is derived from `whether the recognized program has a prologue` (1 hit) and then that a fold "with a prologue reads the intermediate that prologue region materialized; one without reads the declared input directly". Under the contributor source the governing fact is whether the fold **names a declared input**, and the second clause is now **false for the materialized arm**, which reads an intermediate while having no prologue. The recognized-side sibling `declared_contributor_tensor` did receive the corrective paragraph — anchor `The two intermediates are different regions and the same role` (1 hit) — and is the model to match. Behaviour is correct: `declared_input()` answers `None` and the tensor resolves `Intermediate`.

*Citation note: the review supplied `one without reads the declared input directly` as an anchor and it returns **0**, because the source wraps it across two `///` lines. The two anchors given above were each grepped against `physical.rs` at this base and each return 1. This is the rendered-view anchor trap AGENTS.md documents; prefer the short single-line fragments.*

**Fact — a test comment asserts a rule the test cannot observe.** `crates/tiler-macros/src/region/tests.rs`, anchor `the sides rule refuses it under` (1 hit). The claim is **true** — the review probed the compiler directly and got `Err(UnsupportedCapability { rule: "reduction-contributor-depth" })` for the triple-nested subject — but this test's only discriminators are `expect_err` plus three diagnostic substrings and the absence of `"UnsupportedCapability {"`. It never observes the rule, so the comment asserts something its own file cannot check. The rule *is* pinned elsewhere (`contraction_direct_path`, the epilogue wall test, `request/tests.rs`). Either soften the comment to cite those pins or make the check observe the rule; do not leave a comment claiming a check the file does not perform.

**Fact — a test re-implements the function it is named for.** `crates/tiler-compiler/src/pipeline/tests.rs`, anchor `a_produced_folds_reduction_stays_resolvable_for_the_numerical_proof` (1 hit). It re-implements `record_numerical_equivalence`'s lookup rather than calling it, and feeds it the fold's own members rather than a whole-program candidate's. It pins the property, but a change to that function's lookup shape would not redden it. Its doc comment is honest about what it pins, so this is a strengthening opportunity rather than a defect — **state which it is when you close it**, and if calling the real function is not reachable from a unit test, say so rather than leaving the proxy unexplained.

**Fact — a newly admitted population has no regression test.** A program declaring **both** the producer and the produced fold as ordered named outputs — `inner = sum(x, [cols])`, `outer = sum(inner * 2, [rows])` — was refused before this carrier and is now admitted through `published_and_consumed_overlap`. The review compiled it successfully with an out-of-tree probe. The carrier's ticket did not enumerate it, so it is currently a real admitted population resting on no in-tree assertion.

## Required work

- Re-audit every Fact above at your actual base before editing, per the stale-Facts rule, and report a per-Fact verdict. Re-grep each anchor against the file its citation names first.
- Repair the `physical.rs` comment so it argues from the fact the code uses, matching the corrective paragraph its sibling already carries. Do not change behaviour.
- Resolve the `tiler-macros` comment one way or the other, and say which you chose and why.
- Add the missing regression test for the producer-and-produced-fold population, asserting what it compiles to rather than only that it compiles — a bare `Ok` would restate the probe rather than pin the behaviour.
- Decide the proxy test's disposition explicitly and record the reasoning.

## Non-goals

Re-litigating the carrier's design, the fixture resize (the review proved the bound is the governed profile's grid axis applied to an already-admitted control, not a regression), or the `From<ElementwiseRefusal>` dead arm, which the review confirmed is unreachable by construction and cannot silently become the thing under test. The stale `reassociation-permitted: false` reasoning in `fusion_legality.rs` belongs to its own ticket.

**Deliberately not repaired, recorded so it is not rediscovered:** `tickets/name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md` and `tickets/drive-staged-materialization-boundary-tests-past-elementary-accuracy.md` both name `reduction-contributor-materialization` as the live rule. Both are `done`, so they are history by repository convention, and `docs/compiler/optimizer.md` carries the retirement in the contract.

## Closes when

Every comment above states what the code does, the producer-and-produced-fold population has an in-tree regression test asserting more than compilation, the proxy test's disposition is decided and recorded, and the touched packages' `cargo nextest`, Clippy-with-warnings-denied, and rustdoc gates are green.
