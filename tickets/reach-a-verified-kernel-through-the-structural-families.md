---
id: reach-a-verified-kernel-through-the-structural-families
title: Reach a verified kernel through the structural families
status: in-progress
priority: p1
dependencies: [admit-the-reindex-and-broadcast-operation-families]
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

**Fact — what blocks the last rung is the whole-program recognizer, and it blocks the contraction identically.** `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` recognizes exactly two whole-program shapes and produces a `NormalizedProgram` with exactly two variants. `NormalizedProgram::serial_sum` *panics* on any other variant, and the `if let Some(pointwise) = request.pointwise() { … } else { request.serial_sum() }` shape recurs across `physical.rs`, `frontier.rs`, `selection.rs`, and `program.rs`, so a third variant is a panic at every one of those sites until each is revisited. The support matrix already names this limit and assigns it to [`prototype-optimizer-conformance-gate`](prototype-optimizer-conformance-gate.md).

~~**Fact — the useful programs are also multi-input, and the compiled-program profile is not.** A broadcast is only worth compiling beside the operand it widens, so the workload's occurrence is `Multiply(activation, Broadcast(weight))` — two program inputs. `build_kernel_program` in `crates/tiler-compiler/src/program.rs` declares one external allocation and one input value, and `declare_host_abi` takes one input element count. One input is a property of the bounded profile, not of the recognizer alone.~~

**Corrected 2026-08-02 — the one-input limit was lifted, and this paragraph must not be read forward.** The compiled-program profile is multi-input. `declare_host_abi` takes `input_elements: &[u64]` — a slice — at `crates/tiler-compiler/src/program.rs:781`, and loops it at `:787`; the program core declares one external allocation and one input value **per declared input**, at `:411` and `:533` (`for (key, bytes) in subject.input_keys.iter().zip(&abi.input_bytes)`). `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs:3` records that the region `sym n; in a, b, c; out (a * b) + c` "now compiles", reaching a complete verified plan rather than merely passing strategy selection, and `admit-multi-input-elementwise-programs-at-the-compiler-boundary` is `done`. Reproduce:

```sh
rg -n 'fn declare_host_abi' -A4 crates/tiler-compiler/src/program.rs
```

The broadcast motivation above is unaffected — `Multiply(activation, Broadcast(weight))` is still the occurrence worth compiling. What changed is that its two program inputs are no longer an obstacle.

**The paragraph below is a separate blocker and was NOT re-checked here — treat it as live.** The whole-program recognizer limit (`select_supported_strategy` recognizing exactly two shapes, `NormalizedProgram::serial_sum` panicking on a third) is what this ticket still turns on.

**Inference — the deliverable is a region shape, not a kernel construct.** Neither family emits a structured-kernel operation and neither should: `ScalarProgram` has no copy variant, and adding one would realize a standalone reindex as a materializing copy kernel — the outcome the admission ticket's non-goals rule out. What reaches a kernel is a *fused* region in which a structural occurrence contributes an access map and an arithmetic neighbour contributes the scalar program.

## Required delivery

- **A recognized program shape containing a structural occurrence.** The narrowest one that is worth compiling is `Multiply(input, Broadcast(weight))` at rank two, which is 113 of the pinned workload's 197 broadcast occurrences. A `Reindex` composed with a pointwise operation is the second.
- **A third normalized-program variant, with every `serial_sum()` site revisited rather than widened by a wildcard.** A site that cannot yet answer for the new variant must refuse with a typed reason, never panic and never fall through to the serial-sum branch.
- ~~**A second program input in the compiled-program profile**, or an explicit typed refusal naming the one-input limit, so a two-input program fails closed with a reason rather than at an index.~~ **Delivered elsewhere and struck 2026-08-02** — the profile is multi-input, per the correction above. Do not re-deliver it; assert the two-input program compiles rather than building a refusal for a limit that is gone.
- **Equivalence against the reference evaluator on the compiled result**, bit-compared, following `assert_fused_matches_reference`'s shape in `crates/tiler-compiler/src/pipeline/tests.rs`.
- **The support-matrix row moved to R6 only if a backend actually emits the fused region**, and left at R5 with the reason if the vertical stops at a verified kernel that no target accepts.

## Non-goals

A `ScalarProgram` copy variant, a standalone materializing reindex kernel, a general program-shape recognizer, and anything about the contraction family — which is blocked by the same recognizer and has its own tickets.

## Closes when

A program containing a `Broadcast` or a `Reindex` reaches a `VerifiedKernel` through `compile()`, its result is bit-compared against the reference evaluator, and the support-matrix row records the rung the evidence actually supports.
