---
id: close-the-contributor-source-carrier-s-review-remainder
title: Close the contributor-source carrier's review remainder
status: done
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

**Fact — a test re-implements the function it is named for.** `crates/tiler-compiler/src/pipeline/tests/produced_folds.rs`, anchor `a_produced_folds_reduction_stays_resolvable_for_the_numerical_proof` (1 hit). It re-implements `record_numerical_equivalence`'s lookup rather than calling it, and feeds it the fold's own members rather than a whole-program candidate's. It pins the property, but a change to that function's lookup shape would not redden it. Its doc comment is honest about what it pins, so this is a strengthening opportunity rather than a defect — **state which it is when you close it**, and if calling the real function is not reachable from a unit test, say so rather than leaving the proxy unexplained.

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

## Worker record — 2026-08-23, base `cb6a872dc7a0b8fcf23b59e72e8c9a95d20778dd`

### Per-Fact verdict, re-audited at this base

- **The stale `physical.rs` doc comment — verified.** `derived from the same fact` → 1, `whether the recognized program has a prologue` → 1, `The two intermediates are different regions and the same role` → 1, and the trapped anchor `one without reads the declared input directly` → 0, all against `crates/tiler-compiler/src/physical.rs`. Behaviour re-derived by reading `SerialSumContributorSubject::declared_input` in `crates/tiler-compiler/src/request/subject.rs`, anchor `the fold reads directly, or`: the `Materialized` arm answers `None`, so `declared_contributor_tensor` resolves `Intermediate`. No behaviour changed.
- **The macros test comment — verified.** `the sides rule refuses it under` → 1 in `crates/tiler-macros/src/region/tests.rs`.
- **The proxy test — content verified, path citation false.** `crates/tiler-compiler/src/pipeline/tests.rs` does not exist at this base; a grep against it errors rather than returning 0. The test is at `crates/tiler-compiler/src/pipeline/tests/produced_folds.rs`, anchor `a_produced_folds_reduction_stays_resolvable_for_the_numerical_proof` → 1. Repaired above. This is the module-split hazard `AGENTS.md` names, in its louder variant: the named file was removed rather than retained behind a re-export.
- **The untested admitted population — verified, and it is real.** `inner = sum(x, [cols])`, `outer = sum(inner * 2, [rows])` with both declared as ordered named outputs compiles at this base: one retained `materialized` alternative over three cover regions, two materializations, four dispatches, one publishing copy. No in-tree assertion covered it.

### What was done

- **`physical.rs`.** `subject_contributor_tensor`'s comment now argues from whether the fold names a declared input, and carries the corrective paragraph its sibling `declared_contributor_tensor` already had. Anchor `The fact is the named ordinal, not the presence of a prologue`.
- **`tiler-macros`.** The check now observes the rule rather than the comment claiming it. The rendered refusal already names the failing check — `crates/tiler-macros/src/aot.rs`, anchor `the check that refused is` — so each case carries its expected rule and the loop asserts it.
- **Regression test.** `a_produced_folds_published_producer_compiles_and_agrees` in `crates/tiler-compiler/src/pipeline/tests/produced_folds.rs` pins the assembled structure, the declared output order, the single publishing copy over an uncovering stage, and both publications bit for bit against `ReferenceEvaluator`. The three near-identical inline builders in that file collapsed into one `produced_fold(publish_producer: bool)` fixture.
- **The proxy test — a strengthening, not a defect.** `record_numerical_equivalence` is unreachable for a produced fold: `crates/tiler-compiler/src/pipeline/planning.rs` gates the recording on `fused_prologue_constants` of the output the whole-program candidate implements, and a `Materialized` contributor has no prologue for `fused_prologue_constants` to recover. The guard's first conjunct declines earlier still — measured, `output_for_region` answers `None` for this program's four-occurrence whole-program candidate — and the call could not be assembled by hand either, because its `FusionNumericalProof` argument comes from `prove_fused_numerics` over a fused region a produced fold has no spelling for. So the proxy stays, its reason is now stated, and both guard conjuncts are asserted beside the lookup so a change that opened the path reddens here.

### Perturbations, subject not assertion

- Made `published_and_consumed_overlap` decline: `a_produced_folds_published_producer_compiles_and_agrees` failed with `called ``Result::unwrap()`` on an ``Err`` value: UnsupportedCapability { phase: "strategy", rule: "output-partition-overlap" }`.
- Resolved every contributor to `TensorRole::Input`: the same test failed on `product.targets[0].failure()`, which became `NoFeasiblePlan(Selection(Structure { rule: "no-complete-plan" }))`.
- Renamed the rule `crates/tiler-compiler/src/request/folded.rs` reports: `an_unrecognized_region_names_what_a_consumer_would_change` failed with ``case `a reduction of a reduction of a reduction` must refuse under `reduction-contributor-depth`:`` followed by the diagnostic naming `perturbed-rule-key`.

### Identity

No identity value moves. The change is three doc comments, one test-fixture extraction, one added assertion in an existing macros test, two added guard assertions in an existing compiler test, and one added test. Nothing touches an identity encoder, a governed key, a schema version, or a golden.

## Coordinator correction — 2026-08-23: I verified five anchors and missed the sixth citation's file path

This ticket's Fact 3 named `crates/tiler-compiler/src/pipeline/tests.rs`. **That file does not exist**, and has not since `split-the-compiler-pipeline-test-monolith-by-orchestration-phase` landed — a split **I merged myself** earlier the same day. The test is at `crates/tiler-compiler/src/pipeline/tests/produced_folds.rs`. Confirmed by the coordinator at `bcc63fe8`.

I checked all five *anchors* in this ticket before dispatching, including confirming that the one it warns about returns 0. I did not check the **file path** of the sixth citation. A path is a citation too, and after a module split it is the part most likely to be wrong.

**This is the louder variant of the module-split hazard**, and worth distinguishing from the quiet one AGENTS.md documents. When a split leaves the named file behind a re-export, a grep returns 0 and reads as *the item was removed*. Here the file was **deleted outright**, so a grep against it **errors** rather than returning 0. Louder, but only if someone runs it — and my pre-dispatch check ran greps against files I had assumed existed.

**The lane also found the strengthening this ticket imagined is not reachable, and proved it rather than asserting it.** `record_numerical_equivalence` cannot be reached for a produced fold on any path: the recording is gated on `fused_prologue_constants` of the output the whole-program candidate implements, and a `Materialized` contributor has no prologue to recover one from. The guard's *first* conjunct declines even earlier — measured, not inferred: the candidate covers 4 occurrences and `output_for_region` answers `None`. Nor can the call be assembled by hand, since its `FusionNumericalProof` comes from `prove_fused_numerics` over a fused region a produced fold has no spelling for. So the proxy stays, its reason is now written down, and **both guard conjuncts are asserted beside the lookup** so a change that opens the path reddens there rather than silently beginning to record a proof about a fold the test only inspects. That is a better outcome than the strengthening the ticket asked for.
