---
id: lift-the-four-published-and-consumed-walls-together
title: Lift the four published-and-consumed walls together
status: review
priority: p2
dependencies: []
related: [admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary, admit-elementwise-epilogues-over-a-materialized-intermediate, accept-the-kernel-program-publishing-copy-surface]
scopes: [implementation/compiler, implementation/ir, implementation/build, implementation/artifact, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, compiler-api]
claimed_from: todo
assignee: agent-four-walls
lease_expires_at: 1786017023
---
## User-visible outcome

A program that both publishes a value and consumes it — the conformance suite's own multi-output fixture, publishing `scaled` and reducing it into `reduced` — compiles and both published outputs bit-agree with the reference evaluator, instead of refusing at the request boundary under `output-partition-overlap`.

## Why this exists, and why it is one ticket rather than four

`admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary` was filed on the premise that "every *region* the shape needs is now built — only the program-scope account is missing", so that lifting the row was a `tiler-ir` widening and "not a recognizer change". **That premise was measured wrong.** The `tiler-ir` account is real and is still needed, but it is the *fourth and last* of four walls, and the first three are all in `tiler-compiler`.

### Measurement — the wall stack, worktree at base `2ebe90cb`, 2026-08-06, pinned nightly, governed baseline profile

Each wall was disabled in turn (`if false && …`) and the next refusal read from `compile()`. The fixture is `pipeline::conformance::a_published_and_consumed_intermediate_refuses_by_name`'s program — `input`, `scale = 2.0`, `product = input * scale`, `sum = strict_serial_sum(product, axis 1)`, publishing `reduced = sum` then `scaled = product` — **respelled with `SemanticProgramBuilder::try_standard`**, for the reason in the next section.

| # | Wall | Site | Reported as |
| --- | --- | --- | --- |
| 1 | recognition walks overlap | `check_output_cover`, `crates/tiler-compiler/src/request.rs` | `phase: "strategy", rule: "output-partition-overlap"` |
| 2 | one region materializes *and* publishes | `attribute_named_outputs`, `crates/tiler-compiler/src/program.rs` | `phase: "program-assembly", rule: "cover-named-output-attribution"` |
| 3 | nothing writes the published value | `derive_dependencies`, `crates/tiler-compiler/src/program.rs` | `phase: "program-assembly", rule: "internal-unwritten"` |
| 4 | the publishing stage covers no occurrence | `verify_partial_reductions`, `crates/tiler-ir/src/program/verify.rs` | `KernelProgramDiagnostic::UncoveringStage` |

Wall 4 was **not** reached in this run and is inferred from the verifier's text, not measured — walls 1–3 stop the program first, and no stage that would trigger it can be minted yet.

**Fact — walls 2 and 3 are the finding.** Between wall 1 and wall 2, *recognition, region formation, cover enumeration, legality, selection and planning all succeed.* The cover legally places the scaling region `{constant, multiply}` as both the producer of the materialization edge the fold reads and the retainer of the named result `scaled`, and it places `{sum}` as the fold publishing `reduced`. Nothing in the region or cover vocabulary had to move to get there. Wall 3 is what the copy stage's absence actually looks like: with the attribution admitted, the scaling region's one owning write goes to the materialization edge, and `scaled` is an internal value no stage writes.

### Fact — the existing fixture cannot become the compiling assertion

`a_published_and_consumed_intermediate_refuses_by_name` builds its program from `ExternalSemantics`, and `externally_registered_operations_require_their_own_realization_authority` pins that such a program refuses under `phase: "capability", rule: "semantic-authority-pairing"`. Disabling wall 1 alone reports exactly that — wall 1 was masking it. Any published-and-consumed program that is meant to *compile* must be spelled with `SemanticProgramBuilder::try_standard`, like `governed_program`. The refusing fixture may keep its external spelling; the compiling one is a new program.

### Inference — the surviving design, with the alternative eliminated

The copy is **a second dispatch of the region that computed the value**, structurally identical to a split reduction's final pass: pass 1 covers `{constant, multiply}` and writes the materialization edge the fold reads; pass 2 covers nothing, reads that edge, and writes the `ValueRole::Output` value `scaled`. Both passes are individually expressible in the schedule vocabulary already — `materialized_intermediate_epilogue_wall.rs` measures the pointwise-writes-Intermediate and pointwise-reads-Intermediate-writes-Output rows — which is the true content of the "every region the shape needs is built" claim.

**The alternative — the copy as a cover region of its own — is eliminated, not deferred.** A region covering no operation is not expressible anywhere in the search: `form_candidate` refuses an empty member set under `member-multiset` and `verify_candidate` under `membership` (`crates/tiler-compiler/src/region.rs`); a candidate's occurrence *identity* is derived from its members, so a coverless region has none; the anchored partitioner only ever chooses candidates from `containing[anchor]`, i.e. candidates containing an operation; and `augment`, the only other way a region enters a cover, returns immediately under the governed non-duplicating policy. Admitting it would mean inventing a second region-identity scheme and weakening `verify_candidate`'s "every placed region is rederived from its member set". The two-pass design instead reuses the already-accepted subprogram concept in which a later pass legitimately covers nothing — which is precisely what `verify_partial_reductions` already encodes.

## What has to move

1. **`check_output_cover`** admits the published-and-consumed overlap and nothing else. The predicate is not "any overlap": it is that the shorter walk's member set is exactly a *part* of the longer walk's recognized partition and its output value is the value that part produces. Two keys naming one value must keep refusing.
2. **The disjointness invariant three authorities rest on.** `spell_region`, `NormalizedProgram::output_for_region` and `check_output_cover` all document "the walks are proved disjoint, so at most one output owns any member set and the first match is the only match". Under the widening, `{constant, multiply}` is owned by *both* outputs and the declaration-order tie-break becomes load-bearing. Decide it explicitly and rewrite all three claims; do not let it stay an undocumented first-match.
3. **`RegionWrite`** must express a region that materializes an edge *and* publishes. It is a single choice per region today (`ProgramOutput` | `Materialized`), read from the cover.
4. **A publishing-copy pass spelling** in `crates/tiler-compiler/src/physical.rs` and its proposal in `crates/tiler-compiler/src/frontier.rs`, with a cost — a full-tensor copy is a real cost the planner must see, which is the reason it is a planned dispatch rather than something the assembler synthesizes.
5. **`attribute_named_outputs`** widens `MaterializesAndPublishes` to admit the two-pass shape, and `CoverAssembly::from_plan`'s write-role match gains the case of a non-last pass writing the *materialization edge* rather than a pass value.
6. **`tiler_ir::program`** gains the program-scope account: a typed declaration naming the publishing stage, the value it copies from and the value it publishes, admitted by `verify_partial_reductions`' uncovering-stage arm exactly as a split's combiner is. Obligations to prove: the source is defined by a *different* stage, the publisher reads it, the publisher writes the published value, the published value is `ValueRole::Output`, and the two extents agree. `UncoveringStage` must keep refusing everything else — the point is a declaration, not a relaxation.

## Identity

**A program-scope declaration section steps `PROGRAM_DOMAIN` to `tiler.kernel-program.v10`**, on the `v6` precedent recorded in `CanonicalKernelProgramIdentity`'s own comment for exactly this kind of change (a new declaration section, encoded unconditionally, so every program's bytes move and a cache or artifact holding a `v9` identity must miss rather than match). An *appended-only* conditional section that keeps zero-copy programs byte-identical is injective today but leaves the section's presence positionally ambiguous, which constrains every future appended section; that is the cheaper option and the cost it saves is grammar determinacy.

**The step is why the `tiler-ir` half must land with its producer rather than ahead of it.** Landing the declaration alone pays a global identity step — invalidating every cached artifact identity — for a vocabulary no producer can reach, and risks a second step if the minting site turns out to need a field the declaration does not carry.

**Pins to recompute on the tree the step lands into, enumerated:** `ARTIFACT_IDENTITY` and `CACHE_SUBJECT` in `the_standard_metal_path_publishes_its_recorded_identities`, `crates/tiler-build/src/metal_plan.rs` — which is why `implementation/build` is a declared scope here. That test's own doc records the regeneration procedure and the superseded-value ledger to append to. Re-survey `grep -rEn '\b[0-9a-f]{64}\b' crates/ docs/` on the branch before committing; the schedule-level goldens under `crates/tiler-metal/goldens/` sit one fold *below* the program and are not expected to move, but confirm rather than assume.

## Boundaries

- The other `output-partition-overlap` shape — two output keys naming one value — is **not** in scope and must keep refusing.
- Duplication (two regions each computing the value, one materializing and one publishing) is **not** the route: `CoverPolicy::governed` forbids duplication outright and `DuplicationRefusal::NamedResultProducer` refuses a named-result producer even under a permitting policy. That is a policy decision with its own owner.
- The `tiler_ir::program` surface additions are a **public boundary**: land them labelled a draft and park an acceptance node at `awaiting-decision` carrying the exact surface. Do not self-accept.

## A defensible split, if this is too large for one dispatch

The four walls are one user-visible outcome and any split leaves an intermediate state that is either unmeasurable or a *diagnostic regression* — lifting wall 1 alone moves the refusal from `strategy` to `program-assembly`, which is later and less specific. If it is split anyway, split by gate rather than by wall: (a) walls 1–2 plus the disjointness decision, landing with the fixture asserting `internal-unwritten` by name; (b) walls 3–4 plus the identity step, landing with the compiling assertion and bit agreement. Both halves touch `crates/tiler-compiler/src/program.rs`, so they cannot run in parallel.

## Closes when

The governed published-and-consumed program compiles; both published outputs bit-agree with `ReferenceEvaluator::standard().evaluate`; a stage with empty coverage and no declaration still refuses by name, observed failing; every new admitting and refusing path is watched failing; the identity step is executed completely with every moved pin enumerated; and the compiler-facade gate row in `docs/correctness-and-testing.md` records the flip (that file is `contracts/numerics` — declare the scope or hand the wording to the integrator).

## Outcome

**All four walls came down together, and the surviving design was the one the ticket derived.** The governed published-and-consumed program compiles through the ordinary `compile()` path and both published outputs bit-agree with `ReferenceEvaluator::standard().evaluate`. The copy is a **second dispatch of the producing region**: pass 1 covers `{constant, multiply}` and writes the materialization edge the fold reads, pass 2 covers nothing, reads that edge, and writes the `ValueRole::Output` value `scaled`. The coverless-region alternative was not revisited; nothing in the implementation needed it.

### Fact — the per-wall lift decisions, each at its owning site

1. **`check_output_cover`** (`crates/tiler-compiler/src/request.rs`) admits exactly one overlap, decided by the new `published_and_consumed_overlap`. Four conjuncts, each load-bearing: exactly one pair of walks overlaps; one walk's member set is a strict subset of the other's; the shorter walk is one whole **part** of the longer walk's recognized partition, asked through `NormalizedOutput::owns_region_members` — the same authority `spell_region` resolves a region against; and the published value is the one crossing the part boundary, i.e. some occurrence of the longer walk *outside* the part reads it. Two output keys naming one value fails the second conjunct and still refuses. Two independent published-and-consumed values, or one consumed by two downstream outputs, fail the first and reject explicitly rather than being approximated.

2. **The disjointness invariant** is decided, not left as an undocumented first-match. Under the widening both outputs own the shared member set, so `NormalizedProgram::output_for_region`'s scan finds two claimants and returns the declaration-order-first. That is correct rather than arbitrary because the admitted overlap requires the two claimants to be recognitions of *one value over one occurrence set*, so they spell the same expression over the same domain. All three authorities were rewritten (`physical::spell_region`, `NormalizedProgram::output_for_region`, `check_output_cover`), and `both_claimants_of_a_published_and_consumed_part_spell_one_region` is the check that says no if the equivalence ever stops holding — reached through two different recognizer arms (the fold's prologue part and the pointwise output's own walk), so an agreement is about the recognitions rather than one code path called twice.

3. **`RegionWrite`** gained `MaterializedAndPublished`. `tensor()` answers for the region's *first* dispatch and the doc says so; `publishes_a_copy()` is the predicate every consumer reads. `pipeline/planning.rs` decides it from the cover — materializes an edge **and** `!named_results().is_empty()`, which `verify_cover` already proved names exactly one region per declared output.

4. **The publishing-copy pass** is `physical::publishing_copy_region`: `RegionId(6)`, one `TensorRole::Intermediate` linear read, the identity `PointwiseF32` expression, one `TensorRole::Output` linear write, empty members. The frontier proposes it as a `KernelSubprogram` beside the staging pass with a **two-dispatch structural cost** — a full-tensor copy is a real cost the planner must see, which is why it is a planned dispatch rather than assembler-synthesized. Both parallel reduction strategies are **declined by name** under `region-publishes-a-copy` for such a region: their final pass writes the cover-assigned tensor and the publication is a further dispatch after it, so composing them would be a three-pass shape nothing assembles.

5. **`attribute_named_outputs`** — `AttributionFailure::MaterializesAndPublishes` was **removed**, not left unreachable: its premise ("one owning write would have to serve both") was true of one dispatch and is false of two. `CoverAssembly::from_plan` gained the non-last-pass-writes-the-edge case, the second-pass-reads-the-edge case, and a `publishing-copy-pass-count` refusal for a publishing region with other than two dispatches — a longer subprogram would be a split *and* a publication, whose middle passes have no declared account.

6. **`tiler_ir::program`** gained the program-scope declaration: `PublishingCopy`, `PublishingCopyRef`, `push_publishing_copy`, `publishing_copies()`, `MAX_PROGRAM_PUBLISHING_COPIES`, one build error and five diagnostics. `verify_publishing_copies` proves the source is defined by a *different* stage, the publisher reads it, the published value carries `ValueRole::Output`, the publisher writes it, and the two extents agree. `UncoveringStage` keeps refusing everything else — it now admits two *declared accounts* rather than relaxing.

**Inference — one derivation the ticket did not foresee.** `derive_subprogram_boundary_contract` documented "only the last stage's write leaves the subprogram", which is false for a publishing copy: the staged value is also the materialization edge another cover region reads. Left alone, the contract omitted a buffer the cover joins on and every plan was dropped at boundary reconciliation with no complete plan — observed as `NoFeasiblePlan(Selection(no-complete-plan))` before the fix. The non-final `Intermediate` write now emits a guarantee as well as the handoff when the chain ends in a copy.

### Fact — the identity step, executed completely

`PROGRAM_DOMAIN` steps to **`tiler.kernel-program.v10`** on the `v6` precedent: a new declaration section, encoded unconditionally, so a zero-copy program grows an eight-byte zero count and every program's bytes move. The appended-only conditional alternative was eliminated in the source comment and in the ledger: injective today, but it leaves the section's presence positionally ambiguous and constrains every future appended section.

Every pin recomputed on this tree, enumerated old → new:

| Pin | Old | New |
| --- | --- | --- |
| `ARTIFACT_IDENTITY`, `crates/tiler-build/src/metal_plan.rs` | `d22c0d11f8486a15b3df7651feee543eb5d0f8d398a7eb9047ae45b15f9ce832` | `e3ac0aee9e9ce35b23edc2ee49ce7fdb4b40cabbb34774b782b7325d4455fa34` |
| `CACHE_SUBJECT`, same test | `6dee9552e5fb3c0cefe12cacab8d15153fd0909923bf7c93f2d5f92c5d679d68` | `14cbccad74c0d2f1c4a05f295a6b04e87aa45aa13be86460e810e76ff478a263` |

Recomputed one at a time in the order the test's own doc records (artifact identity first, then the cache subject, because the second assertion is unreachable until the first passes), never copied from another branch. The superseded-value ledger in that test's doc was appended with the v9 pair.

**Pins that did *not* move, each observed rather than assumed** — the survey was `grep -rEn '\b[0-9a-f]{64}\b' crates/ docs/` and `grep -rEn '\b[0-9a-f]{16}\b' crates/ docs/` before editing, and every candidate is a test that passed on the final tree: `ARTIFACT_DOMAIN` (`tiler.artifact-program.v15`) and the artifact stage key (`tiler.artifact-program.stage.v3`) hold, because a publishing copy is a program-scope declaration and no artifact entry writes that subject itself — both frame the complete stepped program identity with its own separator; the eight `crates/tiler-metal/goldens/*.metal` kernel and scheduled-region digests hold, sitting one fold *below* the program; and `crates/tiler-compiler/src/explain.rs`'s pinned request qualifier holds, because it folds the request subject and no recognized-program encoding changed.

Ledger documents moved in the same commit: `docs/artifact-abi.md` (the current-ledger sentence, the `tiler-ir` ownership sentence, and a new v10 paragraph with the injectivity reasoning for the domains that did not step) and `docs/decisions/0072`'s running domain sentence. `docs/ir.md`'s v9 mentions are historical statements about what the v9 step did and remain true. **Two research records still say `v9`** and were deliberately left as dated observations rather than edited: `docs/research/documentation/production-crate-codebase-audit.md:121` (an audit snapshot) and `docs/research/program-planning/abi-expression-ownership.md:36` (a running ledger sentence in a research record). The second is a stale-reading cost a sweep could take; it is named here rather than absorbed.

### Fact — the flipped gate test, with fresh perturbations

`pipeline::conformance::a_published_and_consumed_intermediate_refuses_by_name` → **`a_published_and_consumed_intermediate_compiles_and_agrees`**. The fixture is respelled with `SemanticProgramBuilder::try_standard` for the reason the measurement recorded: the old one registers `ExternalSemantics` and refuses under `capability` / `semantic-authority-pairing`, which wall 1 was masking. It asserts two cover regions, one materialization edge, three dispatches, the declared interface order, exactly one declared publishing copy, exactly one uncovering stage, that the copy's publisher is that stage and its source stage is not, and bit agreement for **both** outputs against the reference — `scaled` catches a copy that published the wrong buffer or a non-identity expression, `reduced` catches a first dispatch whose write was redirected to the publication and left the edge unwritten (a program that would still publish a correct `scaled`).

New and renamed refusing tests, every one watched failing before its fix:

- `request::an_output_key_pair_naming_one_value_still_refuses_by_name` (renamed from `two_outputs_sharing_one_walk_refuse_rather_than_publish_twice`): two keys naming one value; a publication *inside* one recognized part; and the crossing conjunct driven against a stated member set, with its measurement boundary recorded — for every program the recognizer admits, a part's published value *is* the value crossing its boundary, so that conjunct is defence in depth rather than a live gate.
- `request::both_claimants_of_a_published_and_consumed_part_spell_one_region` — the decided tie-break.
- `composed_family_recognition::a_second_named_output_inside_the_first_s_walk_compiles` (flipped from `…_refuses`): asserted at every contract, tracking its single-output neighbour exactly, including the contraction-permitting decline the prologue's multiply/add adjacency already earned. The neighbouring refusals were moved to the recognizer because composing them into that file's three-input domain reaches `elementwise-reads` first.
- `program::named_output_attribution_can_say_no_in_every_direction`: the `MaterializesAndPublishes` row is retained as an *admission* row, so a reader reconciling the retired refusal sees what replaced it.
- `tiler-ir`: `an_undeclared_uncovering_stage_still_refuses_by_name`, `the_publishing_copy_obligations_can_each_say_no` (four rows across two fixtures), `a_malformed_publishing_copy_declaration_is_rejected_at_insertion`, and `the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps` extended to v10 with a positive control.

**Measurement boundary at `tiler-ir` scope.** No fixture in `crates/tiler-ir/src/program/tests.rs` can state a copy whose obligations *all* hold: every fixture writes its output at a reduced extent (the two-stage temporary is `[2, 3]` against a `[2]` output; both chains of the two-chain fixture are the same shape), and a copy publishes what it read. The complete admitting path is therefore exercised by the compiler conformance test above, and the identity claim is bounded the same way — that the section is folded is evidenced by the domain step and the separator test, while injectivity against an otherwise identical program rests on the section being length-framed and written unconditionally. Building a fixture for it would need a third kernel writing an output at the temporary's extent, which would re-state the compiler's evidence rather than add any. Recorded in the test's own doc.

### Fact — added scopes, and why each is required

`implementation/artifact` — two doc comments in `crates/tiler-artifact/` state the current kernel-program domain and were false at v10. `contracts/artifacts` — `docs/artifact-abi.md` is the identity ledger, and an identity-domain step moves its ledger in the same commit. `contracts/decisions` — ADR 0072 carries a running "the current `tiler.kernel-program.vN`" sentence that the v9 step itself maintained.

### Proposal — the gate-prose wording, for the integrator

`docs/correctness-and-testing.md` is `contracts/numerics` and is **not** in this branch's scopes. The corrected sentence at ~line 112 currently records the published-and-consumed row as refusing. Proposed replacement, to be applied by whoever holds that scope:

> The compiler-facade gate's published-and-consumed row is discharged positively: a program that both publishes a value and reduces it — the conformance suite's own `scaled`/`reduced` fixture — compiles through the ordinary path and both published outputs bit-agree with the reference evaluator, with the publication realized as a second dispatch of the producing region and accounted for at program scope by `tiler_ir::program::PublishingCopy`. `pipeline::conformance::a_published_and_consumed_intermediate_compiles_and_agrees` is the assertion; `request::an_output_key_pair_naming_one_value_still_refuses_by_name` is the neighbour that must keep refusing.

### Boundaries held

The other `output-partition-overlap` shape — two output keys naming one value — keeps refusing, observed. Duplication was not the route: `CoverPolicy::governed` and `DuplicationRefusal::NamedResultProducer` are untouched. The `tiler_ir::program` additions merged **labelled a draft**, and `accept-the-kernel-program-publishing-copy-surface` is parked at `awaiting-decision` carrying the exact surface, the derivation, and the eliminated alternative. Nothing was self-accepted.
