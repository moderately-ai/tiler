---
id: reach-a-verified-kernel-through-the-structural-families
title: Reach a verified kernel through the structural families
status: done
priority: p1
dependencies: [admit-the-reindex-and-broadcast-operation-families, admit-the-structural-families-into-the-scheduled-region-vocabulary]
related: [prototype-optimizer-conformance-gate, admit-the-contraction-semantic-profile, own-operation-family-support-matrix, compose-rotary-position-embedding-from-reindex-and-broadcast, emit-the-structural-region-on-metal]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, structural, breadth]
---
## User-visible outcome

A program that multiplies a `[T, 1024]` activation by a broadcast `[1024]` weight compiles to a verified kernel, so the two structural families the workload cannot be written without stop being statable-but-uncompilable.

## Why this is a separate ticket

**Fact — the families are complete through R5 and the blocker is upstream of them.** [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) registered `tiler::reindex-f32@1` and `tiler::broadcast-f32@1`, their reference evaluators, their `CoordinateRelation` fusion role, and one index-access lowering capability each. `governed::tests` executes both emitted regions on the index-region oracle, and `fusion_legality::tests::a_region_containing_both_structural_families_derives_legality` derives legality for a region containing both. Nothing about either family is missing.

~~**Fact — what blocks the last rung is the whole-program recognizer, and it blocks the contraction identically.** `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` recognizes exactly two whole-program shapes and produces a `NormalizedProgram` with exactly two variants. `NormalizedProgram::serial_sum` *panics* on any other variant, and the `if let Some(pointwise) = request.pointwise() { … } else { request.serial_sum() }` shape recurs across `physical.rs`, `frontier.rs`, `selection.rs`, and `program.rs`, so a third variant is a panic at every one of those sites until each is revisited. The support matrix already names this limit and assigns it to [`prototype-optimizer-conformance-gate`](prototype-optimizer-conformance-gate.md).~~

**Corrected 2026-08-05 — every clause above is falsified, and this paragraph must not be read forward.** The recognizer was rebuilt twice since this ticket was filed, and the blocker it names no longer exists.

- **Not two shapes.** `select_supported_strategy` checks two program-wide properties and then delegates to `recognize_program_outputs`, which walks *each declared output* through `recognize_output` and classifies it by the occurrence that produces it. `ca86e1b0` ("Recognize a program by its occurrences, not by three taught shapes") is the commit.
- **Not two variants, and `NormalizedProgram` is no longer an enum.** It is a struct holding `outputs: Vec<NormalizedOutput>` — one implementable region partition per ordered named output — and `NormalizedOutput` has three variants (`SerialSum`, `Pointwise`, `Contraction`). `08071248` is the commit.
- **The `serial_sum()` panic is not the wall.** It moved to `VerifiedTargetRequest::sole_output()` and is `#[cfg(test)]` on the accessors that reach it; compile-path authorities resolve a region's owning output through `output_for_region`. The contraction is *already admitted* — `crates/tiler-compiler/tests/contraction_direct_path.rs` — so the claim that the recognizer "blocks the contraction identically" is false in both directions.
- **`prototype-optimizer-conformance-gate` is `done`.**

Reproduce: `rg -n 'struct NormalizedProgram' -A4 crates/tiler-compiler/src/request.rs`.

**Fact — the real blocker is the scheduled-region access vocabulary, and it is out of this ticket's scope.** `crates/tiler-ir/src/schedule/model.rs`'s `LogicalAccess` carries `LinearIdentity`, `ScalarBroadcast`, `PackedU4LsbZeroTail`, `ReductionContributor`, and `ContractionOperand`. There is no reindex map, and `ScalarBroadcast` is a rank-zero operand read once — it does not express a `[1024]`-against-`[T, 1024]` widening. Both families therefore refuse at the request boundary under `operation-set`, which `select_supported_strategy`'s own documentation states and this ticket's measurement below confirms. `LogicalAccess` lives in `crates/tiler-ir/**` = scope `implementation/ir`, which this ticket does not hold and must not, because [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) holds it and owns that widening. That ticket is now a declared dependency.

~~**Fact — the useful programs are also multi-input, and the compiled-program profile is not.** A broadcast is only worth compiling beside the operand it widens, so the workload's occurrence is `Multiply(activation, Broadcast(weight))` — two program inputs. `build_kernel_program` in `crates/tiler-compiler/src/program.rs` declares one external allocation and one input value, and `declare_host_abi` takes one input element count. One input is a property of the bounded profile, not of the recognizer alone.~~

**Corrected 2026-08-02 — the one-input limit was lifted, and this paragraph must not be read forward.** The compiled-program profile is multi-input. `declare_host_abi` takes `input_elements: &[u64]` — a slice — at `crates/tiler-compiler/src/program.rs:781`, and loops it at `:787`; the program core declares one external allocation and one input value **per declared input**, at `:411` and `:533` (`for (key, bytes) in subject.input_keys.iter().zip(&abi.input_bytes)`). `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs:3` records that the region `sym n; in a, b, c; out (a * b) + c` "now compiles", reaching a complete verified plan rather than merely passing strategy selection, and `admit-multi-input-elementwise-programs-at-the-compiler-boundary` is `done`. Reproduce:

```sh
rg -n 'fn declare_host_abi' -A4 crates/tiler-compiler/src/program.rs
```

The broadcast motivation above is unaffected — `Multiply(activation, Broadcast(weight))` is still the occurrence worth compiling. What changed is that its two program inputs are no longer an obstacle.

~~**The paragraph below is a separate blocker and was NOT re-checked here — treat it as live.** The whole-program recognizer limit (`select_supported_strategy` recognizing exactly two shapes, `NormalizedProgram::serial_sum` panicking on a third) is what this ticket still turns on.~~ **Re-checked and struck 2026-08-05** — the recognizer limit it forwards is the one falsified above. What this ticket turns on is the `LogicalAccess` gap.

**Inference — the deliverable is a region shape, not a kernel construct.** Neither family emits a structured-kernel operation and neither should: `ScalarProgram` has no copy variant, and adding one would realize a standalone reindex as a materializing copy kernel — the outcome the admission ticket's non-goals rule out. What reaches a kernel is a *fused* region in which a structural occurrence contributes an access map and an arithmetic neighbour contributes the scalar program.

## Required delivery

- ~~**A recognized program shape containing a structural occurrence.** The narrowest one that is worth compiling is `Multiply(input, Broadcast(weight))` at rank two, which is 113 of the pinned workload's 197 broadcast occurrences. A `Reindex` composed with a pointwise operation is the second.~~ **Delivered 2026-08-06** — both, and each bit-compared; see the Outcome.
- ~~**A third normalized-program variant, with every `serial_sum()` site revisited rather than widened by a wildcard.** A site that cannot yet answer for the new variant must refuse with a typed reason, never panic and never fall through to the serial-sum branch.~~ **Delivered elsewhere and struck 2026-08-05** — `NormalizedOutput` already carries three variants and the per-output partition replaced the whole-program strategy, per the correction above. Nothing here remains to deliver.
- **A `LogicalAccess` relation for each structural family** — the actual precondition, owned by the dependency, not by this ticket.
- ~~**A second program input in the compiled-program profile**, or an explicit typed refusal naming the one-input limit, so a two-input program fails closed with a reason rather than at an index.~~ **Delivered elsewhere and struck 2026-08-02** — the profile is multi-input, per the correction above. Do not re-deliver it; assert the two-input program compiles rather than building a refusal for a limit that is gone.
- ~~**Equivalence against the reference evaluator on the compiled result**, bit-compared, following `assert_fused_matches_reference`'s shape in `crates/tiler-compiler/src/pipeline/tests.rs`.~~ **Delivered 2026-08-06** for three programs; the harness extension it needed is recorded in the Outcome.
- **The support-matrix row moved to R6 only if a backend actually emits the fused region**, and left at R5 with the reason if the vertical stops at a verified kernel that no target accepts. **Resolved 2026-08-06: left at R5**, and the reason is narrower than "no target accepts" — no target has been *asked*, and one construct is definitively refused. See the Outcome's rung section.

## Non-goals

A `ScalarProgram` copy variant, a standalone materializing reindex kernel, a general program-shape recognizer, and anything about the contraction family — which has its own tickets. ~~which is blocked by the same recognizer and~~ **struck 2026-08-05:** the contraction is admitted, not blocked; `crates/tiler-compiler/tests/contraction_direct_path.rs` compiles one. It remains a non-goal here because it is separate work, not because it shares this ticket's obstruction.

## Closes when

A program containing a `Broadcast` or a `Reindex` reaches a `VerifiedKernel` through `compile()`, its result is bit-compared against the reference evaluator, and the support-matrix row records the rung the evidence actually supports.

## Outcome — 2026-08-05 (superseded by the 2026-08-06 section below)

**Blocked, not delivered, and the premise this ticket was filed on is gone.** Status moved `in-progress` → `blocked` and [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) added to `dependencies:`. What this ticket named as its blocker — the whole-program recognizer — was rebuilt out from under it by `ca86e1b0` and `08071248` and no longer exists; the wall that actually stands is `LogicalAccess`, one crate down, in a scope this ticket does not hold. The dispatch that produced this entry was offered by the scheduler because the edge was recorded as `related:` rather than `dependencies:`, so the board treated unreachable work as ready. Adding the edge is the durable fix.

**Measurement — the ticket's own named program refuses at the request boundary, under every contract.** Environment: this worktree at base `2af2c6fd`, pinned nightly-2026-07-19, governed target profile, `cargo nextest run -p tiler-compiler`. Procedure: build `Multiply(a[2,2], Broadcast(w[2] → [2,2]))` through `SemanticProgramBuilder` and call `tiler_compiler::session::compile`.

| program | STRICT | FTZ | RELAXED | REASSOC | FTZ+REASSOC |
|---|---|---|---|---|---|
| `a * w` (both declared at `[2,2]`) | Ok | Ok | Ok | Ok | Ok |
| `a * silu(w)` | Ok | Ok | `NoFeasiblePlan` | Ok | Ok |
| `a * broadcast(w)` | `operation-set` | `operation-set` | `operation-set` | `operation-set` | `operation-set` |

Every `operation-set` refusal carries no explain trace, i.e. it precedes any target-qualified planning. The `NoFeasiblePlan` cell is the multiply/add adjacency the sibling `multi_input_elementwise_boundary` file owns, and it arrives *after* recognition admitted the program — which is what makes it evidence that the elementary neighbour is admitted.

**Fact — the reachable remainder was delivered: the broadcast wall is now observed.** Before this change `F32Broadcast` appeared in `crates/tiler-compiler/` at exactly two sites, both inside `fusion_legality.rs`'s unit tests, which derive that a region containing the family is *legal to fuse* and say nothing about whether one can be spelled. The compile boundary pinned `tiler::reindex-f32@1` in two places and `tiler::broadcast-f32@1` in none — so the family carrying 113 of the pinned workload's 197 structural occurrences, and the one this ticket's user-visible outcome names, had its refusal asserted by no test. `a_broadcast_widening_a_declared_weight_refuses_under_the_vocabulary_rule` in `crates/tiler-compiler/tests/composed_family_recognition.rs` closes that gap, with the trio above as its evidence. It is the assertion the dependency will flip. **Citation repointed 2026-08-06** — the dependency flipped it and this ticket renamed it to match its body: it is `a_broadcast_widening_a_declared_weight_compiles_as_a_replication_relation`.

**Inference — the support-matrix row does not move.** The ticket's `Required delivery` conditions the R6 move on a backend actually emitting the fused region. No region is emitted, so the row stays at R5 and the reason is now pinned by a test rather than asserted in prose.

**Verification.** `cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `cargo nextest run --workspace`; `cargo test --workspace --doc`; `make full`. Each of the four new assertions was perturbed and observed failing: replacing the broadcast with `F32Silu` (refusal assertion returns `Ok(())`), renaming the expected rule to `reduction-prologue`, substituting a refusing program for the control, and flattening the neighbour's contract branch. Identity-pin survey: 21 real 16-hex pins and 8 64-hex pins in `crates/tiler-compiler`; none moved, including `explain.rs`'s `request=8e06e11fdc3a2889`, because the change adds no program shape to the request subject and no registry entry.

## Unparked — 2026-08-06

The blocking dependency (`admit-the-structural-families-into-the-scheduled-region-vocabulary`) is done: `LogicalAccess` carries `ReindexBijection` and `BroadcastReplication`, the pinned wall tests were flipped by that landing, and a standalone reindex already compiles bit-compared end-to-end. What this ticket still owes, per that landing's own boundary statement: the broadcast's reference-oracle bit comparison (the workload occurrence is two-input and the KIR test interpreter binds one buffer) and the R6 rung's remaining evidence.

## Outcome — 2026-08-06

**The user-visible outcome is delivered and the rung does not move, and those two facts do not conflict: what was missing was never one thing.** A program multiplying an activation by a broadcast weight compiles to a verified kernel whose result is the reference evaluator's bit for bit, so the two structural families have stopped being statable-but-uncompilable. R6 additionally requires a *backend* to emit the region, and none has been asked to — so the support-matrix row stays at R5 and now says exactly which crate the residual is in, rather than attributing it to a recognizer or a vocabulary that no longer refuse anything.

### What the two-input oracle needed, and why it is the harness rather than the compiler

**Fact — the compiler needed no change; the test machine did.** The KIR interpreter in `crates/tiler-compiler/src/pipeline/tests.rs` took its two buffer parameters positionally — `buffers.next()` twice, one read and one write — and held a single `KirElements` payload, so `OperationView::Load` ignored the buffer it named and read the one payload it had. That models every one-input region exactly and cannot model a structural one at all, because a widening broadcast's whole content is that its two reads address *different ranges*: `a` at four elements and `w` at two, over one four-element domain.

**Fact — the extension is one list and one lookup.** `KirMachine` now walks `declared_buffers()` once, collecting each `BufferAccess::Read` into a `VerifiedBufferId -> position` map in signature order and requiring exactly one `Write`; `Load` resolves its position from the buffer it names. Two properties are deliberate rather than incidental. The read population is *counted* against the payloads offered (`reads.len() == inputs.len()`), so a fixture binding a payload the kernel has no buffer for fails instead of leaving it silently unread — the "count your population" rule, applied to a two-element population. And each payload's length is checked against its *parameter's* `element_count`, not against the region's domain, because for a widening read those are different numbers and comparing against the domain would admit a payload the kernel can address past the end of. No new framework, no new file, and `interpret_fused` survives unchanged as the one-payload wrapper its twelve existing callers use.

### Measurement — three programs, bit-compared

Environment: this worktree at base `c2271a1f`, pinned `nightly-2026-07-19`, governed target profile, `cargo nextest run --workspace`. Procedure: build each program through `SemanticProgramBuilder`, compile through `tiler_compiler::pipeline::compile` with `CompilationRequest::governed`, interpret the retained `Fused` alternative's kernel on the KIR machine, and compare against `ReferenceEvaluator::standard()` evaluated on the same inputs.

| program | region reads | elements compared | differing |
|---|---|---|---|
| `out = reverse(a)`, `a` at `[2, 2]` | 1 (mirrored reindex) | 4 | 0 |
| `out = a * broadcast(w)`, `a` at `[2, 2]`, `w` at `[2]` | 2 (replication + dense) | 4 | 0 |
| `out = permute(a) * b`, both at `[2, 2]` | 2 (bijection + dense) | 4 | 0 |

The first row is the dependency's, re-run here rather than re-claimed. The second is this ticket's user-visible outcome — the `[1024]`-against-`[T, 1024]` normalization-weight shape at the extents the governed four-thread grid axis launches, which is 113 of the pinned workload's 197 broadcast occurrences. The third is Milestone 2's "reindex plus pointwise fusion" bullet written as the einops `rearrange('i j -> j i')`: one region carrying a rearrangement and a multiply over a second declared input, with no materialized intermediate between them, which is what makes it *fusion* rather than two passes.

**Every comparison is stated twice.** Once against the oracle, and once against a hand-derived literal, so a reference evaluator that agreed with a wrong compiler would still be caught. Each fixture is chosen to be discriminating rather than convenient: the weight's two entries differ (a uniform weight makes replication along axis 0 indistinguishable from axis 1, and would have passed a wrong compiler), `a` is not symmetric under transposition (`3, 10, 28, 88` is what a dropped permutation produces against the asserted `3, 20, 14, 88`), and every value is a power of two so each product is exact and any disagreement is a wrong *element* rather than a rounding.

**Bounded by:** four elements per program, one domain rank, `f32` only, the governed profile only, and the KIR machine rather than a device. A wider domain declines for a launch reason and would stop being evidence about the access relation.

### The rung, and the bound on it

**Fact — R5, and the residual is one crate wide.** R6 requires that "a backend emits it". No test in this repository puts a structural region through one, and the reason is structural rather than an oversight: `tiler-metal` depends on `tiler-ir` and `tiler-artifact` and not on `tiler-compiler`, so its fixtures build regions by hand, and this ticket holds `implementation/compiler`, not `implementation/metal`.

**Fact — one construct is definitively unemittable.** `emit_binary` in `crates/tiler-metal/src/emit.rs` maps `IndexAdd`, `IndexMultiply`, `IndexDivide`, `IndexModulo`, `I32Subtract`, the `f32`/`bf16` arithmetic, and `F32Maximum`, and falls through every other tag to `MetalEmitError::UnsupportedOperation { family: Binary }`. `BinaryOp::IndexSubtract` — appended at `0x0c` by the dependency, emitted by exactly one producer (`emit_offset` at `crates/tiler-ir/src/kernel/lower.rs:2093`) for a reindex mirror's `extent − 1 − c` — has no arm. So the `reverse-axis` form, which is row one of the table above, verifies as a kernel and refuses at the backend. Reproduce: `rg -n 'IndexSubtract' crates/` returns four hits, none under `crates/tiler-metal/`.

**Fact — every other ingredient is already emitted, which is what bounds the residual.** `crates/tiler-metal/goldens/reduction_multi_axis.metal` emits `/`, `%`, `*`, and `+` over `uint64_t` structured index arithmetic (lines 59–65) — exactly the construct set a broadcast replication decode and every non-mirrored reindex decode produce — and `crates/tiler-metal/goldens/contraction_strict_tensor.metal` declares two read buffers at different element counts (8 and 12) beside one write, which is the signature shape a widening broadcast needs.

**Inference — and the reason the rung did not move on it.** It is tempting to read those two goldens as evidence that a backend emits the structural region. They are not: a vocabulary that covers the tags a region emits and a backend that emits that region are different maturity claims, and the mirror is the counterexample *inside this same family* that shows they come apart. [`emit-the-structural-region-on-metal`](emit-the-structural-region-on-metal.md) is filed at `todo` with `implementation/metal`, carrying the missing arm, a golden, and the question the arm raises — MSL `uint64_t` subtraction wraps where the KIR contract asserts non-negativity, so the emitter has to decide whether it asserts, widens, or rests on the producer's proof.

### The stale claims found and corrected along the way

**Fact — three tests asserted the opposite of their own names, and the roadmap cited one as evidence for a refusal that no longer happens.** The dependency flipped four assertions and did not rename them, so `a_broadcast_widening_a_declared_weight_refuses_under_the_vocabulary_rule`, `perturbing_one_occurrence_out_of_the_vocabulary_refuses_by_name`, and `a_family_outside_the_expression_vocabulary_refuses_with_a_typed_reason` each stood over an `assert_eq!(…, Ok(()))`. They are now `a_broadcast_widening_a_declared_weight_compiles_as_a_replication_relation`, `a_structural_occurrence_beside_an_elementary_one_compiles_as_a_mapped_read`, and `a_family_with_no_node_of_its_own_compiles_by_projection_or_by_addressing`, with both module headers and the fixture doc comments rewritten to describe what the code does now. Every citation was repointed in the same change: `docs/roadmap.md`, and the three tickets that named them — this one, [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md), and [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md). The two done tickets' own sentences are left standing and annotated rather than rewritten; only the symbols moved.

**The pairs are kept, and their purpose inverts.** Each flipped assertion still travels with the elementary neighbour it differs from in one occurrence. They no longer attribute a refusal; they show that a family contributing *addressing* and a family contributing *arithmetic* reach a region by routes that are not interchangeable — which is what would catch a later widening that admitted the structural half by projecting it, since a projection has to materialize the operand and add the observable rounding boundary the family's admission exists to avoid.

**Two prose claims in the navigation corpus were false and are corrected in tense rather than deleted**, because both were the row's stated reason and the derivations are what the widening was accepted on: the roadmap's structural row ("no program containing either reaches a `VerifiedKernel`", plus the `LogicalAccess` inventory behind it, plus the five-contract refusal measurement), and the roadmap's closing recognizer paragraph ("The first limit is what holds the structural row at R5 — … the capability is never resolved on the compile path"). `docs/status.md`'s recognizer paragraph had the same clause and is corrected the same way.

### Watched-failing evidence

Every new admitting path was perturbed and observed failing before the result was acted on.

1. **The multi-buffer binding is load-bearing** — made `Load` resolve position `0` regardless of the buffer it names, i.e. the machine's behaviour before this change. Both new tests went red (`a_broadcast_…`: `[1.0, 4.0, 4.0, 16.0]` against the oracle's `[3.0, 10.0, 12.0, 40.0]`), and the one-input reindex test stayed green — which is the correct split and is itself the evidence that the extension changed only what it should.
2. **The derived access map is load-bearing** — perturbed `pointwise_region`'s `relation_for` in `crates/tiler-compiler/src/physical.rs` to look up a nonexistent ordinal, so every read falls back to `LinearIdentity`. All three tests went red, and the *shapes* of the three failures are the finding: the reindex returned the unreversed tensor, the permute chain returned `3, 10, 28, 88`, and the broadcast **failed closed** with `InvalidCompilerOutput(… StageElementCount { position: 1, expected: 4, actual: 2 })` rather than compiling — a widened read bound against a domain-sized proof is caught by construction, not by the oracle.
3. **The harness's declared-length check can say no** — bound the four-element activation payload to the two-element weight buffer. `the payload bound to read buffer 1 is not its declared length: left 4, right 2`, from the assertion added in this change.

Each perturbation was reverted and `git status` confirmed clean before the commit; `crates/tiler-compiler/src/physical.rs` and `crates/tiler-ir/**` carry no change in the final diff.

### Verification

`cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `cargo nextest run --workspace` (2724 passed, 7 skipped); `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

**Identity-pin survey — nothing moved, and nothing should have.** 22 real 16-hex pins in `crates/tiler-compiler` (all in `explain.rs`; the `0123456789abcdef` and `0000000000000000` literals in `region.rs`, `fusion.rs`, `fusion_legality.rs`, and `pipeline/planning.rs` are constructed placeholders, not pins) and 5 64-hex pins. `explain.rs`'s `request=f3244b2242ebcb5c` is unchanged, which is the expected result rather than a lucky one: this change adds no program shape to the request subject, no registry entry, and no schedule or kernel construct — the two new programs are spelled from operations the subject already encodes. The seven `crates/tiler-metal/goldens/*.metal` digests are likewise untouched, and none of them appears in the diff. A full-workspace green run is what would have caught any of these moving.

### Scope

The branch stayed inside `implementation/compiler` (`crates/tiler-compiler/**`), `contracts/navigation` (`docs/roadmap.md`, `docs/status.md`), and the shared `project/tickets`. No scope was added. `implementation/metal` was *not* taken: the emission work is a second crate's product surface with its own golden and toolchain obligations, and it is filed rather than absorbed.
