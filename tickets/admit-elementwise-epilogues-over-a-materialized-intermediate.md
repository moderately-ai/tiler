---
id: admit-elementwise-epilogues-over-a-materialized-intermediate
title: Admit an elementwise epilogue over a materialized intermediate
status: done
priority: p2
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary, admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary]
related: [admit-a-strict-serial-fold-that-writes-a-materialized-intermediate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
---
## User-visible outcome

A program whose output is an elementwise expression over a *contraction's* or a *reduction's* result — `matmul(a, b) * 2.0`, or `sum(x * x) * scale` — compiles, instead of refusing at the request boundary under `operation-set`.

## Why this exists

**Fact — the recognizer refuses it deliberately.** `normalize_contraction` requires the contraction occurrence to cover the program exactly (`CONTRACTION_OPERATIONS`), and `recognize_elementwise` classifies every operand as a declared input, a constant, or another elementwise occurrence — so an operand produced by a fold or a contraction refuses. `select_supported_strategy`'s documentation names this ticket for the widening.

**~~Fact~~ — refuted 2026-08-06; see the falsification section below. The wall is the physical layer's, not the schedule IR's.** `TensorRole::Intermediate` is a *per-region* role, so nothing in `tiler-ir` forbids a chain that stages a second temporary; `partial_reduction_region` already writes an intermediate that `final_reduction_region` reads. What is missing in `tiler-compiler` is: an elementwise region that reads an intermediate (`pointwise_region` builds every read as `TensorRole::Input { ordinal }`), a contraction region that writes one (`contraction_region` hard-codes `TensorRole::Output`), the program assembly for the resulting chain, and the request-subject binding arms for both.

The three `tiler-compiler` citations in that paragraph are all still accurate at `fd1716c4` and were re-read rather than trusted. What does not follow is the conclusion: the role is per-region, and the thing that forbids the chain is the access contract each scalar-program family declares around the role.

**Inference — this is what makes the recognizer's generality reach the families it already knows.** The elementwise walk is general over its own vocabulary; what it cannot yet do is compose *across* a materialization boundary, which is the composition every fused-epilogue workload needs.

## Boundaries

- The recognized program must still be a bounded chain: the `regions` and `buffers` budgets bound it, and `verify_program` derives both from the declared arity, so a chain needing more must refuse by name rather than be assembled.
- Every stage's request-subject binding must re-derive its accesses from the recognized program, as the existing arms do. A stage admitted on its scalar program alone would let a provider bind the wrong tensor.
- ~~`ProgramAlternativeKind::of` classifies a plan by its cover, and `build_plan_program` matches on `(kind, region count)`. A three-stage non-split chain is a shape neither currently expresses; widening them is part of this ticket, not a follow-on.~~ **Discharged elsewhere — corrected 2026-08-06.** `assemble-a-kernel-program-from-an-arbitrary-cover` landed: `build_plan_program` no longer takes a kind and derives the assembly from the cover for *any* region count, and `CoverAssembly::from_plan` already mints one internal value per materialization edge, orders stages so producers precede consumers, and binds an intermediate read to the edge the cover hands the region. A cover it cannot express is a typed missing-capability refusal naming the region. `ProgramAlternativeKind::of` survives as a presentation and equivalence-evidence discriminator (`Fused` exactly when one region covers every operation) and needs no widening. Nothing in the program-assembly layer is this ticket's work any more.

### Corrected budget facts, 2026-08-06

The first bullet's "derives both from the declared arity" is half right, and the halves matter to the chain this ticket builds. Read from `check_program_budgets` at `fd1716c4`:

- **`buffers` is derived**: `program.input_count() + 3`, documented as covering every declared input, the prologue's materialized temporary, a split's staged partial tensor, and the output. A three-stage `prologue -> fold -> epilogue` chain needs inputs + two temporaries + the output, which is *exactly* that bound — it fits, with nothing left for a split on top.
- **`regions` is not derived, and it does not read the submitted program at all**: the call is `check_budget("regions", budgets.regions, 3)`, where the *actual* is the literal `3` — spelled, and justified in place as "the split program's own stage count — pointwise, partial, and final", because a region count belongs to a plan and the request is admitted before any plan is chosen. With the governed `budgets.regions` also `3`, the check asserts that the budget admits the widest plan this profile assembles; it is not a bound on the caller's graph, and the first bullet's "so a chain needing more must refuse by name" does not describe it. Two consequences for the chain this ticket builds. A `prologue -> fold -> epilogue` chain is *also* three regions, so nothing has to move for the unsplit case. But the same chain with the fold split is four, so admitting the epilogue while leaving the literal at three would drop the split alternative for every reduction-epilogue program — and drop it *silently*, because no program is ever measured against this number. Move the literal and `budgets.regions` together, or record the lost alternative deliberately.
- **`CONTRACTION_OPERATIONS` no longer exists.** The "Why this exists" section's citation of it is stale: `normalize_contraction`'s whole-program exact-cover obligation moved to `check_output_cover`, which requires the per-output walks to partition the program's occurrences. Verified absent by `grep -rn CONTRACTION_OPERATIONS crates/`, which returns nothing.

## Closes when

A contraction feeding an elementwise epilogue compiles through `tiler_compiler::session` to an emitted region; a chain the budgets cannot admit refuses by name, observed failing; and the request-subject binding refuses a forged stage for each new region kind, observed failing.

## Corrected 2026-08-05 by the coordinator's pre-resume sweep — the wall this ticket owns doubled, and the recognition layer it cites was rebuilt

The Fact above describes the single-output recognition as it stood before `recognize-several-ordered-named-outputs-at-the-compiler-request-boundary` and `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` landed (both 2026-08-05). Recognition is now one walk per declared output (`recognize_program_outputs`), and `select_supported_strategy`'s documentation moved with it — re-read it at dispatch rather than trusting the paragraph above. What is unchanged: an operand produced by a fold or contraction still refuses inside a single output's elementwise walk, which is this ticket's original wall.

**What is new: this ticket now owns a second, measured wall.** A program that publishes an intermediate *and* consumes it — the conformance suite's own multi-output fixture, publishing `scaled` and reducing it into `reduced` — refuses at `phase: "strategy", rule: "output-partition-overlap"`, because the published value's occurrence sits inside the reducing output's walk and two outputs may not claim one occurrence. The gate row's paragraph in `docs/correctness-and-testing.md` records the derivation: one region's owning write would have to serve both a materialization edge and a publication, `ValueRole` is exclusive (`fills` refuses an `Output` value for an `Intermediate` buffer), so the shape needs a copy stage reading `Intermediate` and writing `Output` — and building that copy stage is exactly this ticket's outcome. The pinned test is `a_published_and_consumed_intermediate_refuses_by_name`. Discharging this ticket therefore flips one of the compiler-facade gate's two named open bounds (the other is `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`), which is worth stating in the dispatch brief because it raises the ticket's effective priority above its filing-time framing.

## Outcome — premise falsified 2026-08-06; the wall is a `tiler-ir` one and this ticket is now its dependent

**Why this stopped rather than shipped.** This ticket's own "Why this exists" derived, from `TensorRole::Intermediate` being a per-region role, that nothing in `tiler-ir` forbids an elementwise region reading a materialized intermediate. The premise is true and the conclusion is false: the role is per-region, and what forbids the chain is the *access contract* each scalar-program family declares around it. Neither the epilogue's region nor the copy stage the published-and-consumed case needs is expressible in the current schedule vocabulary, and both refusing rules live in `crates/tiler-ir/src/schedule/builder.rs` — outside this ticket's `implementation/compiler` scope. Discovery stop, per AGENTS.md: recorded, edged, parked, reachable remainder delivered.

**Measurement — worktree at base `fd1716c4`, 2026-08-06, the pinned nightly, governed baseline profile.** Three regions built by hand and submitted to `ScheduledRegionBuilder::build`, each with a control differing only in one tensor role:

| Region | Role under test | Verdict |
| --- | --- | --- |
| `ScalarProgram::PointwiseF32`, one read | read `TensorRole::Intermediate` | refused, `NumericalOrAccessRefinement`; control at `Input { ordinal: 0 }` verifies |
| `ScalarProgram::StrictSerialSum`, `ReductionTopology::Serial` | write `TensorRole::Intermediate` | refused, `NumericalOrAccessRefinement`; control writing `Output` verifies |
| `ScalarProgram::StrictTensorContraction` | write `TensorRole::Intermediate` | **admitted** — no widening needed |

**Fact — the two refusing rules, read from the verifier rather than inferred.** `verify_pointwise_region` computes `ordinals_bind_in_order` and requires read access `i` to be `TensorRole::Input { ordinal: i }` at *every* position, shared by the `f32` and `bf16` widths. `verify_access_and_semantics` admits a `StrictSerialSum` under a `ReductionTopology::Serial` only in the arm guarded by `read.tensor == TensorRole::Intermediate && write.tensor == TensorRole::Output`.

**Fact — the compiler cannot route around it by binding differently.** Declaring `TensorRole::Input { ordinal }` on the region and binding a temporary at assembly time is refused independently, a second time, in `tiler-ir`: `ValueRole::fills` lists `(Temporary, Input { .. })` as false (`crates/tiler-ir/src/program/model.rs:182`), enforced by `KernelProgramBuilder::push_stage`'s `check_stage_accesses`. Already pinned by `a_published_output_value_cannot_fill_an_intermediate_buffer`.

**Inference — the multi-input precedent, repeated exactly.** `admit-multi-input-elementwise-programs-at-the-compiler-boundary` hit this structure: the recognizer was where the refusal was *observed*, the vocabulary that made the shape inexpressible was a crate down, and the resolution was to file the `tiler-ir` widening and make the compiler-side admission its dependent. [`admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`](admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md) is that ticket and is now in `dependencies`.

**What both walls this ticket owns now depend on.** The reduction-then-scale shape needs the pointwise read *and* the serial-sum write. The contraction-then-scale shape needs only the pointwise read, because the contraction producer already composes — so it is the smaller first slice once the blocker lands. The published-and-consumed refusal (`a_published_and_consumed_intermediate_refuses_by_name`) needs the pointwise read alone, since its copy stage reads `Intermediate` and writes `Output`.

### What landed on this branch

Nothing that changes behaviour. No recognizer, encoder, region builder, or budget moved, and the full `tiler-compiler` suite is green at 660 tests.

1. **Evidence.** `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs`, five tests: both epilogue programs (`contract(a, b) * 2.0` and `sum(x * x, axis 1) * 2.0`) refuse at `phase: "strategy", rule: "operation-set"` under all five contracts, each beside its bare-producer control which compiles; and the three region-level verdicts in the table above. Reproduce with `cargo nextest run -p tiler-compiler --test materialized_intermediate_epilogue_wall`.
2. **Perturbation.** All five assertions were perturbed and observed failing in one run before the file was kept — the two expected rules swapped to `elementwise-shape`, the refused pointwise read swapped to its control's `Input { 0 }`, the refused fold write swapped to `Output`, and the admitted contraction write swapped to an inadmissible `Input { 0 }` (which reported `AccessContract`). The file was then restored byte-identically and re-run green.
3. **Corrected claims.** Five doc sites carried the falsified derivation and now name the schedule-vocabulary wall and its owning ticket: `select_supported_strategy`, the multi-output paragraph and `check_output_cover` in `request.rs`, `normalize_contraction`'s inline comment, `the_two_ordered_outputs_one_region_would_publish...`'s doc, `a_published_and_consumed_intermediate_refuses_by_name` in `pipeline/conformance.rs`, and two sites in `tests/multi_output_boundary.rs`. A doc comment is a claim, and this one made unreachable work look reachable.
4. **Ticket corrections.** The stale `CONTRACTION_OPERATIONS` citation, the discharged `build_plan_program` boundary bullet, and the half-wrong budget derivation are corrected above with the exact checks that reproduce each.

**Identity — no pin moved, determined at the owning site.** The dispatch brief required deciding whether an epilogue admission steps a request-subject sub-tag. It does not here, and the derivation is the forced-not-chosen standard the `pointwise-f32.v4` and `serial-sum-f32.v3` comments in `request.rs` state: a sub-tag steps when an arm's *bytes* move and injectivity against the previous framing cannot be argued. This branch touches no `Normalized*` type, no encoder arm, no recognized program shape, and no registry, so no arm's bytes move and no sub-tag steps. Confirmed empirically rather than only argued: the explain qualifier `request=6dd42be71c6745fe` pinned at `crates/tiler-compiler/src/explain.rs:4149` is unchanged and its test passes.

**Scope.** `crates/tiler-compiler/**` (exclusive `implementation/compiler`) and `tickets/**` (shared `project/tickets`) only. No `crates/tiler-ir/**` file was edited — which is the finding, not an omission.

**Recommended board move after integration.** This ticket is set to `review` so the branch can be integrated, but `review` misrepresents it: the stated outcome is not delivered and the ticket now carries an unmet dependency. Once the diff is merged, move it to `blocked` — the parked category the unmet `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary` edge actually describes — so the scheduler stops treating it as work in flight. Do not mark it `done`; nothing it promised compiles yet.

## Unblocking note, 2026-08-06 — two of this ticket's three shapes are reachable and the third is not

Written by the worker on [`admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`](admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md), which is the dependency above and is now delivered. Read it before dispatching this one; what follows is the delta a brief needs.

**Fact — the pointwise read is admitted, and the shape of the admission matters to the region builder this ticket writes.** `verify_pointwise_region` no longer requires read access `i` to be `TensorRole::Input { ordinal: i }`. It requires the reads to name strictly ascending *declared inputs* and at most one `TensorRole::Intermediate`, refusing a second intermediate read (nothing attributes two reads to two materialization edges) and a read of `TensorRole::Output` by name. The consequence for `pointwise_region` in `crates/tiler-compiler/src/physical.rs`, which today builds every read as `Input { ordinal }` with the ordinal equal to its position: an epilogue's expression leaf ordinal is the *access position* — `emit_pointwise` looks a leaf up among the values loaded in access order — while the `TensorRole::Input` ordinal names the declared input that read binds, which `CoverAssembly::from_plan` resolves against the program's declared interface. An epilogue reading a staged value and the program's third input therefore carries leaves `0` and `1` with roles `Intermediate` and `Input { ordinal: 2 }`, and a builder that kept them equal would bind the wrong buffer.

**Fact — `ValueRole::fills` was re-derived and deliberately not changed.** An epilogue's intermediate read binds a `Temporary` value, and `(Temporary, Intermediate)` is already `true`. The escape hatch this ticket's own Outcome named — declaring `Input { ordinal }` and binding a temporary there — stays closed, and the widening is what makes opening it unnecessary: the region now *says* `Intermediate` where it means one. `a_published_output_value_cannot_fill_an_intermediate_buffer` still passes unchanged.

**Fact — the assembly layer already binds the shape.** `CoverAssembly::from_plan` maps `Input { ordinal }` to `AssemblyBinding::Input(ordinal)` and `Intermediate` to the consumed edge, and refuses a second intermediate read under `cover-intermediate-read-attribution`. Nothing in it needs widening for a mixed read list.

**Fact — the reduction shape is still walled, one layer down, and it is not this ticket's scope.** `verify_access_and_semantics` admits a serial `StrictSerialSum` only when its owning write targets `TensorRole::Output`, so `sum(x * x) * scale` has no producer region. [`admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`](admit-a-strict-serial-fold-that-writes-a-materialized-intermediate.md) owns it and is filed at `todo`; it is `related` rather than a dependency because this ticket's other two shapes do not need it. **So this ticket can deliver `contract(a, b) * 2.0` and the published-and-consumed copy stage now, and cannot deliver `sum(x * x) * scale` until that one lands** — a dispatch should either narrow this ticket's outcome to the two reachable shapes and split the reduction half, or land the serial-fold widening first. Marking this ticket done while its own user-visible outcome still names `sum(x * x) * scale` would overstate it.

**Fact — `materialized_intermediate_epilogue_wall.rs` was updated rather than deleted.** Its pointwise row now reads `yes` and `a_pointwise_region_may_read_a_materialized_intermediate` asserts the admission, the mixed read list, and the three refusals. Its two request-boundary tests are unchanged and still green: this ticket's `operation-set` refusals are exactly what this ticket lifts.

## Unblocking note, 2026-08-06 (second) — the narrowing question above is resolved; all three shapes are reachable

Written by the worker on [`admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`](admit-a-strict-serial-fold-that-writes-a-materialized-intermediate.md), which has landed. **This supersedes the note above's closing recommendation.** That note said a dispatch should "either narrow this ticket's outcome to the two reachable shapes and split the reduction half, or land the serial-fold widening first". The widening is landed, so **no narrowing and no split is needed** — this ticket's user-visible outcome can be delivered whole, including `sum(x * x) * scale`, and marking it done will not overstate it.

**Fact — the write rule, and its exact width.** `verify_access_and_semantics` no longer requires a fold's owning write to target `TensorRole::Output`. Every *committing* fold pass now carries `CommittedTensor::CoverAssigned`, which admits `TensorRole::Intermediate` or `TensorRole::Output` and refuses `TensorRole::Input { .. }` by name: the four serial arms (`StrictSerialSum`, `SquaredSerialSum`, `FusedMultiplyAddSerialSum`, `StrictSerialMaximum`), the multi-pass split's **final** pass, and the cooperative tile. The split's **partial** pass keeps `CommittedTensor::Exactly(TensorRole::Intermediate)` — a partial is an unfolded fragment and is no cover's declared output. So both the fused (`SquaredSerialSum`) and the materialized (`prologue -> StrictSerialSum`) spellings of `sum(x * x)` can produce a value an epilogue reads, under all three topologies, and no alternative is silently lost.

**Fact — the compiler-side work this exposes, read from `physical.rs` rather than inferred.** `RegionWrite` already exists and already answers the question correctly (`ProgramOutput => Output`, `Materialized => Intermediate`), but only `RegionSpellingKind::Pointwise` carries it. `reduction_region`, `final_reduction_region`, and `single_workgroup_tree_region` each hard-code `TensorRole::Output` for the owning write — and each writes it in three places, the access, its bounds proof, and the ownership proof, which `verify_proof_records` requires to name one tensor. `partial_reduction_region` correctly keeps `Intermediate` and must not be threaded. Threading `RegionWrite` into the reduction spellings is this ticket's work; nothing in `tiler-ir` refuses the result.

**Fact — the budget consequence is now live rather than hypothetical.** This ticket's own "Corrected budget facts" section derived that a `prologue -> fold -> epilogue` chain is three regions and fits, while the same chain *with the fold split* is four and would be dropped silently by the `check_budget("regions", budgets.regions, 3)` literal. That is no longer a hypothetical: the split's final pass can now stage its result, so the four-region alternative is genuinely spellable and the literal is what would drop it. Move the literal and `budgets.regions` together, or record the lost alternative deliberately — the choice that section already framed, with the vocabulary half now settled.

**Fact — `ValueRole::fills` is unchanged and needs no change.** A fold's staged result binds a `Temporary` value, and `(Temporary, Intermediate)` is already `true`. `a_published_output_value_cannot_fill_an_intermediate_buffer` passes unchanged.

## Unparked whole — 2026-08-06

Both discovered prerequisites are done: the Intermediate-read widening and the cover-assigned fold commit. The superseding note above stands — no narrowing, all three shapes deliverable, with the compiler-side threading map (RegionWrite into the three reduction region builders, partial pass excluded) and the now-live regions-budget concern recorded there for the dispatch brief.

## Outcome — delivered whole, 2026-08-06

**Why this landed the way it did.** The two prerequisites the first dispatch discovered were both in place, so the wall was where the superseding note said: entirely compiler-side. The change has one shape behind it — **an elementwise walk that reaches a folding family has found a materialization boundary, not an unrecognizable program** — and everything else follows from naming the value it stopped at instead of discarding it.

### What the caller can now compile

**Fact — all three shapes the user-visible outcome names, each observed compiling through `tiler_compiler::session::compile` against the governed profile under all five statable numerical contracts.**

| Shape | Dispatches | Evidence |
| --- | --- | --- |
| `matmul(a, b) * 2.0` | 2 | `materialized_intermediate_epilogue_wall::an_elementwise_epilogue_over_a_contraction_compiles_as_a_chain`, `contraction_direct_path::a_contraction_with_an_elementwise_epilogue_compiles_as_a_chain` |
| `sum(x * x, axis 1) * 2.0` | 3 | `materialized_intermediate_epilogue_wall::an_elementwise_epilogue_over_a_reduction_compiles_as_a_chain` |
| published-and-consumed copy stage | — | **not delivered; the wall is one crate down and is now filed.** See below. |

**Measurement — bit agreement against `tiler-reference`, on the pinned nightly, governed baseline profile.** Each chain is interpreted stage by stage on the KIR machine and compared to `ReferenceEvaluator::standard().evaluate` bit for bit, and each also asserts an independently hand-derived value so a reference evaluator agreeing with a wrong compiler would still be caught:

- `pipeline::tests::a_contraction_epilogue_chain_matches_the_reference_evaluator` — `contract(a,b) * 2.0`, one materialization edge, `[160, 1152, 640, 4608]`.
- `pipeline::tests::a_reduction_epilogue_chain_matches_the_reference_evaluator` — `sum(x*x) * 2.0`, two materialization edges, `[170]`.
- `pipeline::tests::an_epilogue_reading_a_staged_value_and_a_declared_input_matches_the_reference` — `contract(a,b) * a`, the one shape that exercises the access-position/declared-ordinal separation, `[80, 1152, 1280, 18432]`.
- `pipeline::tests::an_epilogue_reaching_declared_inputs_out_of_order_still_compiles` — `contract(a,b) * b * a` beside `contract(a,b) * a * b`, which is what makes the canonical read order a requirement rather than a convention.
- `pipeline::tests::a_chain_over_a_fused_prologue_and_fold_retains_both_producer_spellings` — `sum(a*2 + 1) * 3.0`, whose retained stage counts are `[2, 3]` because the fused producer may stage its result too, `[102]`.

### What moved, by layer

**`request.rs` — recognition.** `recognize_elementwise` split into `plan_elementwise` (all validation, linearized in mint order) and `mint_elementwise` (numbering only), because two callers need the same validation under different leaf numbering and a second walk would be a second classifier. `plan_elementwise` reports `ElementwiseRefusal::Folded(value)` naming the boundary; `recognize_epilogue` walks the epilogue against it and `recognize_epilogue_producer` recognizes the producer as its own shape. `NormalizedOutput::Epilogue` carries the producer as a whole `NormalizedOutput`, which is what lets every existing region builder, cost, and subject binding apply to it unchanged.

**`physical.rs` — spelling and building.** `spell_region` recurses into a chain's producer through `spell_output`, so every producer part keeps the spelling it would have as a standalone output. `pointwise_region`'s access scaffolding factored into `elementwise_region`, parameterized by the read list; `epilogue_region` is the second caller. **`RegionWrite` threaded into five builders, not the three the unblocking note named**: `reduction_region`, `final_reduction_region`, `single_workgroup_tree_region` as recorded, and additionally `contraction_region` and `fused_region`, each at the access, its bounds proof, and its ownership proof. `partial_reduction_region` is untouched, as required.

**Measurement — four of the five threaded sites are observable, and the fifth is not.** Hard-coding `TensorRole::Output` back into `contraction_region`, `reduction_region`, `fused_region`, or `final_reduction_region` fails a test (see the perturbation table). Doing it to `single_workgroup_tree_region` fails none: the tree is *offered* for a chain's fold exactly as for a standalone one, but the portfolio's structural cost prunes it before assembly for every shape this profile admits, so no retained plan places a cooperative tile under a materialized write. It is threaded anyway — a region built for a write the cover did not assign is refused at assembly and the alternative disappears with no diagnostic, which is the failure mode worth pre-empting — and the site says so in place. `calibrate-and-activate-parallel-reduction-selection` is what would make it observable.

**Everything below is unchanged and was verified to need nothing**: cover enumeration, materialization-edge derivation, execution order, `CoverAssembly::from_plan`, named-output attribution, and `ProgramAlternativeKind`. The chain is placed by the ordinary cover search.

### The regions budget — the answer, executed

**Measurement — the widest plan this profile assembles is now four dispatches.** `pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue` compiles `sum(x*x, axis 1) * 2.0` at `[1, 4]` and reports retained stage counts `[3, 4]` under a reassociating contract and `[3]` under one that forbids reassociation — so the fourth stage is attributable to the split rather than to the epilogue. The four-region alternative the note called "spellable" is genuinely retained.

**Fact — nothing silently dropped it, and that is why the literal had to move anyway.** `check_budget("regions", budgets.regions, 3)` measures no submitted program: the actual is a constant asserting that the budget admits the widest plan the profile assembles. Leaving it at three would not have lost the alternative; it would have made the assertion false, which is worse, because the next widening derives from it. Moved to `4`, with `DeterministicBudgets::governed().regions` moved with it and the derivation rewritten to cite the measurement.

**Identity — one pin moved, recomputed on this tree.** Every budget is written into the request subject, so the budget value change moved every governed compilation's qualifier. `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately` request qualifier moved `6dd42be71c6745fe` → `689c3aefc30f48d3`, recomputed by observing the failing value here and never copied. It is the only pin the request subject reaches; `grep -rnE '"[0-9a-f]{16}"|request=[0-9a-f]{16}' crates/ --include='*.rs'` returns that one site, and `crates/tiler-compiler/tests/` holds no hex literal at all. A ledger paragraph records the move beside the four earlier budget steps. `cargo nextest run --workspace` is green at 2801 tests, so no other crate pinned a subject-derived value.

### Sub-tag determination — one added, none stepped

**Determined at the owning site under the forced-not-chosen standard the `pointwise-f32.v4` and `serial-sum-f32.v3` comments state.** A fourth per-output arm was added, `epilogue-f32.v1`, and the enclosing `tiler.compiler.request-subject.v5` domain did **not** step — the contraction arm's argument, verbatim: no existing arm's bytes move, so a subject encoded before this variant existed still encodes to exactly what it did, and a reader reaching this tag is reading a subject the earlier vocabulary could not express. Neither `pointwise-f32` nor `serial-sum-f32` nor `contraction-f32` moved: the epilogue is a new arm, not a widening of an old one, and the nested producer is encoded through the same function so a chain's fold encodes exactly as a standalone fold does.

One byte was added to the relation encoder — `LogicalAccess::LinearIdentity` gained tag `0x03` where it previously fell through the wildcard `0x00`. It moves no already-encodable subject's bytes, because `encode_access_maps`'s run records only the ordinals a structural occurrence interposed and an identity read was never written there. An epilogue read that interposes none reaches it directly, which is why it needed a name rather than the refusal tag.

### A silent-wrongness defect found and closed fail-closed

**Measurement, and it is not this ticket's shape.** At base `912b6058`, `out = a * permute(a)` over `[[1, 2], [4, 8]]` compiled and returned `[1, 16, 4, 64]` — `permute(a) * permute(a)` — where the reference evaluator gives `[1, 8, 8, 64]`. The region binds one read per declared input and the expression's two `Input { ordinal: 0 }` nodes share it, so the mapped relation served both leaves. Refused by name under `structural-access-conflict`, which is the rule that already refuses two *different* relations on one leaf; this is the same conflict with `LinearIdentity` as one of the two. Fixed here rather than filed-and-left because it returns an incorrect tensor, which the architectural contract does not admit deferring. The *widening* that would admit the program instead is filed as [`admit-two-reads-of-one-declared-input-in-an-elementwise-region`](admit-two-reads-of-one-declared-input-in-an-elementwise-region.md), and the regression row in `every_refusal_names_its_unrecognized_property` carries the measured wrong values.

### The third shape: not delivered, and the wall is verified

**Fact — every *region* the published-and-consumed copy stage needs is now built**; what it lacks is a program-scope *account*. A stage publishing a value another region computed claims no occurrence, and `verify_partial_reductions` in `crates/tiler-ir/src/program/verify.rs` refuses every empty-coverage stage that is not a declared split's combiner. The two compiler-side routes around it are both closed and were checked by reading: `attribute_named_outputs` refuses a region that materializes and publishes, and a duplicating cover is refused by `CoverPolicy::governed`. The widening is therefore `crates/tiler-ir/**`, outside this ticket's scope, and is filed as [`admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary`](admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary.md). `a_published_and_consumed_intermediate_refuses_by_name` is unchanged and still green; its doc now names the program-scope wall rather than the schedule-vocabulary one it had already outlived.

**For the coordinator — the facade-gate row did not flip and its prose needs no edit from this branch.** `docs/correctness-and-testing.md` maps to `contracts/numerics`, which this ticket does not hold. The gate's two named open bounds both stand: this one for the reason above, and `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` untouched. What *is* worth recording there when someone holds that scope is that the row's stated cause moved — it was "no elementwise region this profile builds reads a materialized intermediate", and that is no longer true.

### Renamed tests, each perturbed and observed failing

Four assertions pinned refusals this landing lifts, and each was rewritten rather than deleted, keeping the measured history and gaining a still-refusing neighbour:

- `an_elementwise_epilogue_over_a_contraction_refuses_at_the_request_boundary` → `..._compiles_as_a_chain`, with `nested_contraction_chain` refusing one boundary deeper.
- `an_elementwise_epilogue_over_a_reduction_refuses_at_the_request_boundary` → `..._compiles_as_a_chain`, with `nested_reduction_chain`.
- `a_contraction_with_an_extra_operation_refuses_with_a_typed_reason` → `a_contraction_with_an_elementwise_epilogue_compiles_as_a_chain`.
- `every_refusal_names_its_unrecognized_property`'s contraction-epilogue row now asserts the chain, and a prologue reading a folded value replaces it as the `operation-set` row.

**Six deliberate perturbations, applied one at a time and each observed failing before the file was restored** — and two of the first six did not fail, which is why there are six rather than four: perturbing `EPILOGUE_REGION` moved the builder and the binding together (vacuous), and reversing the read order was invisible because every recognized binary family is commutative. Both were replaced by perturbations that fire, and the read-order one is what motivated `an_epilogue_reaching_declared_inputs_out_of_order_still_compiles`.

| Perturbation | Observed failing |
| --- | --- |
| the staged read binds `Input { 0 }` instead of `Intermediate` | both chain tests |
| `epilogue_region` emits `RegionId::new(0)` while the binding expects `EPILOGUE_REGION` | both chain tests |
| the read list left in walk order rather than canonical order | the out-of-order test |
| `DeterministicBudgets::governed().regions` left at `3` | the widest-plan test |
| the dense-and-mapped conflict guard removed | the refusal inventory |
| the epilogue member match inverted, so the binding falls through to the producer | both chain tests |
| `fused_region` hard-codes the output tensor | the fused-producer chain test |
| `final_reduction_region` hard-codes the output tensor | the widest-plan test |
| `single_workgroup_tree_region` hard-codes the output tensor | **nothing** — recorded above as a measurement boundary, not repaired by a weaker test |

### Commands run

`cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `cargo nextest run --workspace`; `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

### Scope

`crates/tiler-compiler/**` (exclusive `implementation/compiler`) and `tickets/**` (shared `project/tickets`) only. No `crates/tiler-ir/**` file was edited, and the two tickets filed above are why.

## Current-boundary correction, 2026-08-09

The Outcome's historical statement that the published-and-consumed copy stage remained undelivered was true at this ticket's landing and is no longer current. [`lift-the-four-published-and-consumed-walls-together`](lift-the-four-published-and-consumed-walls-together.md) subsequently admitted the exact overlap, and `pipeline::conformance::a_published_and_consumed_intermediate_compiles_and_agrees` now compiles it and bit-compares both outputs. Preserve the refusal name above as measurement history; it is not the current test or boundary.
