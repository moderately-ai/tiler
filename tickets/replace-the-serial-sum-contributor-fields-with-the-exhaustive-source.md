---
id: replace-the-serial-sum-contributor-fields-with-the-exhaustive-source
title: Replace the serial-sum contributor fields with the exhaustive source
status: in-progress
priority: p3
dependencies: [admit-a-materialized-producer-in-a-serial-reduction-contributor]
related: [match-the-declared-input-contributor-in-the-fused-proof-exemption, admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler, contracts/optimizer, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, identity, numerics]
claimed_from: todo
assignee: worker-contributor-source
lease_expires_at: 1787159586
---
## User-visible outcome

`sum(sum(x) * 2)`, `sum(contract(a, b) * 2)`, and `sum(rms_norm(x, w))` compile: a strict serial reduction consumes a contributor computed across one materialization boundary, with the producer's numerical identity preserved, while the declared-input and pointwise-prologue neighbours keep their exact `serial-sum-f32.v3` bytes. Subjects one edge deeper report `reduction-contributor-depth` by name.

## Authority

Tom accepted carrier (4) on 2026-08-18 — see `## Accepted decision — 2026-08-18` in [`admit-a-materialized-producer-in-a-serial-reduction-contributor`](admit-a-materialized-producer-in-a-serial-reduction-contributor.md). That ticket's re-gated packet (measurements at `7c90391d`-era base `1957227c`, independently reviewed at `90714048`) is the complete specification; this ticket lands it and must not re-litigate the carrier shape. Its Facts were audited at `1957227c`/`236753a3`; re-verify each at this ticket's actual base before editing, per the stale-Facts rule.

## Required work

- Replace `NormalizedSerialSum`'s `prologue`/`prologue_reads`/`contributor_input` trio with the exhaustive contributor source (`DeclaredInput(ordinal)` | `PointwisePrologue { expr, reads }` | `Materialized(Box<MaterializedContributor>)` where `MaterializedContributor { producer: NormalizedOutput, continuation: Option<ContributorContinuation> }`), and mirror it in `NormalizedSerialSumSubject`. The packet's 31-error lib census is the migration map; the sites it does **not** force are the named hand-work list: the member partition (continuation members are a third part, never `RecognizedSerialSumMembers::pointwise`), `output_region_role` (a continuation region must not answer whole-program), and the `check_output_cover` members extension.
- Recognition per the packet: `plan_elementwise` with `staged: None`; on `Folded` under `ReductionContributorAdmission::OneEdge`, re-plan with the staged leaf, mint the continuation, call `recognize_epilogue_producer`; under `NoEdge`, refuse `reduction-contributor-depth`; never map `Folded` through `From<ElementwiseRefusal>` after a retain is attempted. `recognize_output` passes `OneEdge`; `recognize_epilogue_producer` passes `NoEdge`.
- Subject encoding: the `Materialized` arm writes the new `serial-sum-produced-f32.v1` framed tag, the producer through `encode_output_subject` recursion, and a continuation presence byte then the epilogue arm's `BoundaryRead`-tagged read vocabulary. The `DeclaredInput` and `PointwisePrologue` arms keep writing `serial-sum-f32.v3` byte-for-byte; the `domains.rs` request-subject pin row stays `(1, 0)`.
- Spelling: continuation members → `RegionSpellingKind::Epilogue` (staged-read builder), prologue → `Pointwise`, reduction → `SerialSum`, fused affine candidate unchanged (prologue ∪ fold only); continuation ∪ fold is not a part. `contributor_tensor` stays `Intermediate` when the fold names no declared input.
- The verifier-exemption repair rides this migration: the compile-forced `serial.prologue.is_none()` site is repaired under [`match-the-declared-input-contributor-in-the-fused-proof-exemption`](match-the-declared-input-contributor-in-the-fused-proof-exemption.md)'s statement (match `DeclaredInput` explicitly; no `is_none_or` vacuity), which keeps its own perturbation obligation and reviewer.
- `reduction-contributor-depth` regression population, complete: `contraction_direct_path`'s `sum(sum(contract)*2)`, a triple nested sum `sum(sum(sum(x)*2)*2)`, and an epilogue over a produced sum `(sum(sum(x)*2))*3`; plus the optimizer-contract sentence beside the retiring `reduction-contributor-materialization`, which becomes unreachable for one-edge subjects — a test still expecting it on such a subject is the admission check.
- Repair the two doc sentences the carrier makes false: `StagedOperandAdmission`'s `declared inputs by construction` and `encode_output_subject`'s `admits a folding family as a chain's producer and nothing else`.
- Run the packet's full perturbation list (2026-08-18 section, superseding 2026-08-13 where they overlap), perturbing subjects and quoting failure text: byte-identical `folded_prologue(false)`; producer + one-`BoundaryRead::Staged` continuation on `folded_prologue(true)`; `Staged` producer with no synthesized continuation on `sum(rms_norm(x, w))`; forged-bytes subject under `serial-sum-f32.v3` and omitted producer presence fail binding (forge the bytes, not the marker-run accident); `fused_prologue_constants` stays `None` on `staged * 2 + 1`; width and depth refusals unchanged; `record_numerical_equivalence` resolves a produced sum's fold.
- Regression: existing pinned request qualifiers (`deterministic_trace_is_sealed_and_rendered_separately`, the tiler-build Metal goldens) recompute unchanged — new programs mint new content identities within existing domains; no domain steps, no pin movement.

## Non-goals

The worklist producer walk (recorded reversal path; arbitrary chain depth stays refused), lifting `StagedOperandAdmission::NoEdge` or two-edge regions, backend emission, performance selection, producer-family-specific diagnostic keys.

## Coordination

Exclusive `implementation/compiler`; queues behind whatever lane holds that scope (the contraction-key replacement at dispatch time). Not a pin-moving identity migration — it does not enter the solo migration slot, but the pinned-qualifier recompute check above is mandatory evidence at review.

## Closes when

The three admitted subjects compile through the shared carrier with the packet's perturbations shown failing on perturbed subjects, the depth population refuses by name, the declared-input neighbour's bytes are proven unmoved, and the verifier-exemption ticket's repair is landed and reviewed alongside.

## Per-Fact audit at this ticket's base — 2026-08-19, base `441f321583ee08856b2b8f87e056ebabf487277b`

Every Fact below was re-read in full at this base before any edit, per the stale-Facts rule. The packet's Facts were audited at `1957227c`/`236753a3`; the tree has since moved substantially — `crates/tiler-compiler/src/request.rs` is now a directory module — so the *locations* moved even where the claims held.

### Verdicts on the packet's Facts

- **Facts 1–4 (recognition finds and discards the boundary; this is not the staged depth guard; several producer families expose one missing relation; the accepted neighbour is shallower): verified.** Anchors re-run at this base: `leaves.staged.is_none() && materializes_its_result` raises `Folded` (`request/elementwise.rs`); `Flattens a discovered materialization boundary into the rule a caller` was the one flattening and reported `reduction-contributor-materialization` (`request/elementwise.rs`); `recognize_elementwise` had exactly one production caller, in `recognize_reduction` (`request/folded.rs`); `recognize_reduction` had exactly the two callers `recognize_output` and `recognize_epilogue_producer`; `StagedOperandAdmission::NoEdge` was constructed only in `recognize_epilogue_producer`; `NormalizedSerialSum` carried `prologue`, `prologue_reads`, `contributor_input` and no producer.
- **The verifier finding: verified, and compile-forced.** `serial.prologue.is_none()` sat at `crates/tiler-compiler/src/pipeline/verify.rs:324` under the guard `output.try_serial_sum().is_none_or(|serial| serial.prologue.is_none())`, with the arm comment `The condition is the prologue, not the family` present. The census below reproduces it as one of the 31 forced errors.
- **The identity claims: verified.** The `domains.rs` request-subject row is `PinnedDomain::new(b"tiler.compiler.request-subject.v6\0", 1, 0)`; the serial-sum sub-tag holds at `serial-sum-f32.v3`; `UNREAD_DECLARED_INPUT_TAG` is `0x04`; the staged arm's producer presence byte and `encode_output_subject` recursion are as cited.
- **The two doc sentences: verified present, and repaired.** `declared inputs by construction` greps as its own line in `request/folded.rs` (the sentence wraps, exactly as the census warns); `admits a folding family as a chain's producer and nothing else` greps whole in `request/subject.rs`.

### Facts that are **imprecise** at this base, repaired here

- **The module layout.** The packet's site map says these fields live in `crates/tiler-compiler/src/request.rs`. That file is now a 181-line module spine; the fields, arms, recognizer, and encoder live in `request/normal_form.rs`, `request/folded.rs`, `request/subject.rs`, `request/elementwise.rs`, and `request/recognize.rs`, with the test module at `request/tests.rs`. Every Fact transfers; only the paths move.
- **The contraction key.** The 2026-08-13 audit records `materializes_its_result` as `strict_serial_sum_f32_op()`, `strict_tensor_contraction_f32_op()`, or `laws.family_realizes_region_sequence`. The second is now `tensor_contraction_f32_op()` (`tiler::tensor-contraction-f32@1`), the ADR 0112 successor accepted 2026-08-18. The function's *shape* and the three-family set are unchanged, so no consequence follows; the constructor name in the packet is stale.
- **The `--all-targets` census parenthetical.** Not re-derived; the independent review already recorded it as method-sensitive (42 unique diagnostics against the table's 75). The lib counts, which carry the whole forced-site argument, reproduce exactly — see below.

No Fact was **false** at this base.

### Re-derived forced-site census — 2026-08-19, this base

Method: the exact field replacement (the three fields → the contributor source, plus the `NormalizedSerialSumSubject` mirror), no consumer repairs, `cargo check -p tiler-compiler --lib`. Pinned toolchain `nightly-2026-07-19` (`rustc 1.99.0-nightly eff8269f7`) — the same toolchain the packet measured on.

**31 lib errors**, distributed `request/` 23 (`subject.rs` 12, `normal_form.rs` 8, `folded.rs` 3), `physical.rs` 7, `pipeline/verify.rs` 1. Exact match with the packet's table on every row once `request.rs` is read as *the `request` module*. The one verifier error is the exemption guard itself: `pipeline/verify.rs:324: no field 'prologue' on type '&NormalizedSerialSum'`.

**The three named hand-work sites are confirmed unforced**, which is what makes them the places a silent defect would hide: `pipeline.rs` (`output_region_role`), `pipeline/trace.rs` (`record_numerical_equivalence`), and `request/recognize.rs` (`check_output_cover`) each report **zero** errors. Each is given an explicit check in this landing.

## Outcome — 2026-08-19

Landed at `<FINAL>` on `tkt/replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`.

### Identity, proven rather than argued

The declared-input and pointwise-prologue arms' `serial-sum-f32.v3` encodings are pinned as exact byte literals in `the_declared_input_and_pointwise_prologue_arms_keep_their_exact_bytes`, and **the same literals were confirmed by running that test in a detached worktree at base `441f3215`** — before the contributor source existed. The two arms therefore encode byte-for-byte what they already did. `tiler.compiler.request-subject.v6` does not step, the `domains.rs` row stays `(1, 0)`, and no pin file is touched by this branch. `deterministic_trace_is_sealed_and_rendered_separately` and the tiler-build `ARTIFACT_IDENTITY` / `CACHE_SUBJECT` / `FIXED_CONTENT_BYTES` pins recompute unchanged.

The materialized arm writes `serial-sum-produced-f32.v1`: framed tag, declaration, shapes, axes, the reduction's members, element counts, the producer through `encode_output_subject` recursion, then a continuation presence byte and — when present — the expression, the `BoundaryRead`-tagged read run, and the continuation's members.

### Two findings the ticket did not anticipate

1. **`sum(sum(x * x) * 2)` at `[2, 4]` reports `NoFeasiblePlan`, and the cause is the profile rather than the carrier.** The wall file's `nested_reduction_chain` was sized for a program that was only ever *recognized* and refused, so no region of it was costed. Once it compiles, its `x * x` prologue — one invocation per element — is assessed against the governed profile's grid axis, which admits four: `target.grid-axis` `threads=8:4`. The fixture is resized onto the accepted control's `reduction_domain()`; the carrier admits the shape at every size. `sum(sum(x) * 2)` — the ticket's own named subject — compiles at `[2, 4]` unchanged, because it has no prologue region to cost.
2. **`crates/tiler-macros` is edited, outside this ticket's `implementation/compiler` scope.** `region::tests::an_unrecognized_region_names_what_a_consumer_would_change` enumerates the grammar-expressible shapes this build does not recognize, and its last remaining case — a `tensor!` region whose reduction's operand is itself a reduction — now compiles. The population would otherwise be **empty**, which is a check that cannot say no. The case moves to the compiling population and a reduction-of-a-reduction-of-a-reduction replaces it, refusing `reduction-contributor-depth`. Nothing else in that crate is touched. Reported rather than landed silently.

### Perturbations, with the failure each produced

Every one breaks the subject, never the assertion.

| Perturbation | Check | Quoted failure |
| --- | --- | --- |
| Route the `Materialized` arm through `encode_serial_sum` | `a_produced_fold_cannot_encode_under_the_old_serial_sum_tag` | `the materialized arm must open with its own framed tag` |
| Append a presence byte to the `serial-sum-f32.v3` arm | the byte pin | `left: …0000000000000002000000000000000000` / `right: …00000000000000020000000000000000` |
| Widen `SerialSumContributor::prologue()` alone to answer for a continuation | the fused gate | **passed** — `prologue_reads()` is the load-bearing half, so the two accessors are perturbed separately |
| Widen `prologue()` **and** report the continuation's staged read as declared ordinal 0 | `a_continuation_over_a_staged_value_has_no_fused_spelling` | `left: Some((1073741824, 1065353216))` / `right: None` — the fused constants recovered, which is the silent mis-binding |
| Make `merges_nothing` answer `true` for `Materialized` (the retired absence form) | `a_produced_folds_fused_receipt_takes_the_ordinary_proof_path` | `a produced fold is not exempt from the numerical replay: ()` |
| Drop `output_region_role`'s continuation arm | `a_produced_folds_region_roles_name_the_part_rather_than_the_program` | `left: "unrecognized"` / `right: "epilogue"` |
| Hand `recognize_epilogue_producer` `ReductionContributorAdmission::OneEdge` | the depth population | four reds; `left: Ok(())` / `right: Err(UnsupportedCapability { rule: "reduction-contributor-depth" })` in both the wall file and `contraction_direct_path` |
| Spell the continuation `RegionSpellingKind::Pointwise` | the parts and prologue-partition checks | `left: Pointwise(Materialized)` / `right: Epilogue(Materialized)` |
| Fold the continuation's occurrences into `RecognizedSerialSumMembers::pointwise` | the partition checks | two reds; `left: [member 0, 1, 3, 4, 5]` / `right: [member 5]` |

The third row is recorded rather than dropped because it is the one that says which assertion is load-bearing: a perturbation that reddens nothing is evidence about the check's shape.

### Unsupported populations, unchanged

`reduction-contributor-depth` for a fold whose producer is itself across an edge; `staged-operand-depth`; `contraction-operands`; `operation-set` for the width case `sum(sum(a) * sum(b))`, which the re-planned walk reports naturally once a retain is attempted. The worklist producer walk stays the recorded reversal path and is unfiled.

### Scopes added during the work — 2026-08-19

Two scopes beyond the claimed `implementation/compiler`, both required by authorized work rather than expansions of it, so they are added as scheduling metadata and explained here.

- **`contracts/optimizer`.** The Required work above names "the optimizer-contract sentence beside the retiring `reduction-contributor-materialization`". That sentence is in `docs/compiler/optimizer.md`, which this scope owns. The edit retires the unreachable key, states `reduction-contributor-depth` in its place, and records what the predecessor meant so a reader of an older trace can still resolve it.
- **`implementation/frontend`.** `crates/tiler-macros`. Not anticipated by the ticket and not discretionary: `region::tests::an_unrecognized_region_names_what_a_consumer_would_change` enumerates the grammar-expressible shapes this build does not recognize, and the carrier empties that population — its last case, a `tensor!` region whose reduction's operand is itself a reduction, now compiles. A population of zero is a check that cannot say no, so the case moves to the compiling population and a reduction-of-a-reduction-of-a-reduction replaces it under `reduction-contributor-depth`. Nothing else in the crate is touched, and no lane held this scope at dispatch.

`tkt guard` additionally reports `implementation/build` and `implementation/conformance` as affected *transitively via reverse-deps* of `implementation/compiler`. No file in either crate is edited on this branch.
