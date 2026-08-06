---
id: reach-a-verified-kernel-through-the-structural-families
title: Reach a verified kernel through the structural families
status: blocked
priority: p1
dependencies: [admit-the-reindex-and-broadcast-operation-families, admit-the-structural-families-into-the-scheduled-region-vocabulary]
related: [prototype-optimizer-conformance-gate, admit-the-contraction-semantic-profile, own-operation-family-support-matrix, compose-rotary-position-embedding-from-reindex-and-broadcast]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, structural, breadth]
claimed_from: todo
assignee: agent-structural-vertical
lease_expires_at: 1785985712
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

- **A recognized program shape containing a structural occurrence.** The narrowest one that is worth compiling is `Multiply(input, Broadcast(weight))` at rank two, which is 113 of the pinned workload's 197 broadcast occurrences. A `Reindex` composed with a pointwise operation is the second.
- ~~**A third normalized-program variant, with every `serial_sum()` site revisited rather than widened by a wildcard.** A site that cannot yet answer for the new variant must refuse with a typed reason, never panic and never fall through to the serial-sum branch.~~ **Delivered elsewhere and struck 2026-08-05** — `NormalizedOutput` already carries three variants and the per-output partition replaced the whole-program strategy, per the correction above. Nothing here remains to deliver.
- **A `LogicalAccess` relation for each structural family** — the actual precondition, owned by the dependency, not by this ticket.
- ~~**A second program input in the compiled-program profile**, or an explicit typed refusal naming the one-input limit, so a two-input program fails closed with a reason rather than at an index.~~ **Delivered elsewhere and struck 2026-08-02** — the profile is multi-input, per the correction above. Do not re-deliver it; assert the two-input program compiles rather than building a refusal for a limit that is gone.
- **Equivalence against the reference evaluator on the compiled result**, bit-compared, following `assert_fused_matches_reference`'s shape in `crates/tiler-compiler/src/pipeline/tests.rs`.
- **The support-matrix row moved to R6 only if a backend actually emits the fused region**, and left at R5 with the reason if the vertical stops at a verified kernel that no target accepts.

## Non-goals

A `ScalarProgram` copy variant, a standalone materializing reindex kernel, a general program-shape recognizer, and anything about the contraction family — which has its own tickets. ~~which is blocked by the same recognizer and~~ **struck 2026-08-05:** the contraction is admitted, not blocked; `crates/tiler-compiler/tests/contraction_direct_path.rs` compiles one. It remains a non-goal here because it is separate work, not because it shares this ticket's obstruction.

## Closes when

A program containing a `Broadcast` or a `Reindex` reaches a `VerifiedKernel` through `compile()`, its result is bit-compared against the reference evaluator, and the support-matrix row records the rung the evidence actually supports.

## Outcome

**Blocked, not delivered, and the premise this ticket was filed on is gone.** Status moved `in-progress` → `blocked` and [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) added to `dependencies:`. What this ticket named as its blocker — the whole-program recognizer — was rebuilt out from under it by `ca86e1b0` and `08071248` and no longer exists; the wall that actually stands is `LogicalAccess`, one crate down, in a scope this ticket does not hold. The dispatch that produced this entry was offered by the scheduler because the edge was recorded as `related:` rather than `dependencies:`, so the board treated unreachable work as ready. Adding the edge is the durable fix.

**Measurement — the ticket's own named program refuses at the request boundary, under every contract.** Environment: this worktree at base `2af2c6fd`, pinned nightly-2026-07-19, governed target profile, `cargo nextest run -p tiler-compiler`. Procedure: build `Multiply(a[2,2], Broadcast(w[2] → [2,2]))` through `SemanticProgramBuilder` and call `tiler_compiler::session::compile`.

| program | STRICT | FTZ | RELAXED | REASSOC | FTZ+REASSOC |
|---|---|---|---|---|---|
| `a * w` (both declared at `[2,2]`) | Ok | Ok | Ok | Ok | Ok |
| `a * silu(w)` | Ok | Ok | `NoFeasiblePlan` | Ok | Ok |
| `a * broadcast(w)` | `operation-set` | `operation-set` | `operation-set` | `operation-set` | `operation-set` |

Every `operation-set` refusal carries no explain trace, i.e. it precedes any target-qualified planning. The `NoFeasiblePlan` cell is the multiply/add adjacency the sibling `multi_input_elementwise_boundary` file owns, and it arrives *after* recognition admitted the program — which is what makes it evidence that the elementary neighbour is admitted.

**Fact — the reachable remainder was delivered: the broadcast wall is now observed.** Before this change `F32Broadcast` appeared in `crates/tiler-compiler/` at exactly two sites, both inside `fusion_legality.rs`'s unit tests, which derive that a region containing the family is *legal to fuse* and say nothing about whether one can be spelled. The compile boundary pinned `tiler::reindex-f32@1` in two places and `tiler::broadcast-f32@1` in none — so the family carrying 113 of the pinned workload's 197 structural occurrences, and the one this ticket's user-visible outcome names, had its refusal asserted by no test. `a_broadcast_widening_a_declared_weight_refuses_under_the_vocabulary_rule` in `crates/tiler-compiler/tests/composed_family_recognition.rs` closes that gap, with the trio above as its evidence. It is the assertion the dependency will flip.

**Inference — the support-matrix row does not move.** The ticket's `Required delivery` conditions the R6 move on a backend actually emitting the fused region. No region is emitted, so the row stays at R5 and the reason is now pinned by a test rather than asserted in prose.

**Verification.** `cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps`; `cargo nextest run --workspace`; `cargo test --workspace --doc`; `make full`. Each of the four new assertions was perturbed and observed failing: replacing the broadcast with `F32Silu` (refusal assertion returns `Ok(())`), renaming the expected rule to `reduction-prologue`, substituting a refusing program for the control, and flattening the neighbour's contract branch. Identity-pin survey: 21 real 16-hex pins and 8 64-hex pins in `crates/tiler-compiler`; none moved, including `explain.rs`'s `request=8e06e11fdc3a2889`, because the change adds no program shape to the request subject and no registry entry.
