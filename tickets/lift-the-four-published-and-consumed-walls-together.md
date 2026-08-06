---
id: lift-the-four-published-and-consumed-walls-together
title: Lift the four published-and-consumed walls together
status: in-progress
priority: p2
dependencies: []
related: [admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary, admit-elementwise-epilogues-over-a-materialized-intermediate]
scopes: [implementation/compiler, implementation/ir, implementation/build]
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
