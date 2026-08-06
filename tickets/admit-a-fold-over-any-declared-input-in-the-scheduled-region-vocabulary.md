---
id: admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary
title: Admit a fold over any declared input in the scheduled-region vocabulary
status: todo
priority: p2
dependencies: []
related: [admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs, admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary, admit-a-reduction-over-a-declared-input-tensor]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, schedule]
---
## User-visible outcome

A strict serial fold whose contributor tensor is a declared program input other than the *first* is expressible as a scheduled region, so `folded = sum(b)` beside an independent `doubled = a + a` compiles instead of refusing at the request boundary under `sum-contributor-ordinal`, and `sum(b * 2.0 + 1.0)` regains its fused single-region alternative.

## Why this exists

**Fact — the rule is in `tiler-ir` and it names the first input by construction.** `crates/tiler-ir/src/schedule/builder.rs` declares `const FIRST_INPUT: TensorRole = TensorRole::Input { ordinal: InputOrdinal::FIRST }` and spends it in five places that decide a fold's contributor read: `ContributorTensor::DeclaredDomain::admits` returns `tensor == TensorRole::Intermediate || tensor == FIRST_INPUT`; the `FusedMultiplyAddSerialSum`, `SquaredSerialSum`, and `StrictSerialMaximum` arms of `verify_access_and_semantics` each require `read.tensor == FIRST_INPUT` directly; and the split and cooperative topologies carry `read_tensor: ContributorTensor::Exactly(FIRST_INPUT)`. A region reading `Input { ordinal: 1 }` under a `ScalarProgram::StrictSerialSum` therefore fails the intrinsic verifier with `ScheduledRegionDiagnostic::NumericalOrAccessRefinement`.

**Fact — the count above is five *categories*, not five sites; the sites are ten (verified 2026-08-06 by reading every non-test `FIRST_INPUT` use in `crates/tiler-ir/src/schedule/builder.rs`).** `DeclaredDomain::admits` is one, the three `verify_access_and_semantics` arms are three, and the "split and cooperative topologies" clause is six — one per fold family in each of `multi_pass_family` and `cooperative_family`. Three further non-test uses bind the strict-affine `u4` dequantize's `codes`, `scale`, and `zero_point` component reads and decide no fold's contributor, which is why they are outside this ticket. Reproduce with `grep -n FIRST_INPUT crates/tiler-ir/src/schedule/builder.rs` and discard the matches below the `#[cfg(test)]` module.

**Measurement (2026-08-06, on branch `tkt/admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`).** With the compiler-side refusal removed and the recognizer admitting the program, `sum(b)` beside `a * a` over two declared `[2, 2]` inputs reaches `InvalidCompilerOutput(Frontier(MalformedProposal { provider: tiler::prototype-serial-sum-physical@1, source: Intrinsic { rule: "numerical-or-access-refinement", region: RegionId(1) } }))` — every request-boundary and planning stage succeeds and only the intrinsic region verifier refuses. Reproduce by deleting the `sum-contributor-ordinal` guard in `recognize_reduction` and running `cargo nextest run -p tiler-compiler -E 'test(a_fold_over_a_later_declared_input_refuses_by_name)'`.

**Inference — the state became reachable exactly once, and that is why nothing observed it before.** A prologue-less fold's recognition walk reads exactly one tensor. While `canonical_input_reads` required every elementwise walk to read every declared input, such a program declared exactly one input and the recognized ordinal could only be zero, so `FIRST_INPUT` and "the ordinal the program named" could not differ. [`admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`](admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs.md) lifted that rule, made the divergence reachable, and closed it fail-closed rather than silently: the recognizer already carries the fact as `NormalizedSerialSum::contributor_input`, the request subject already encodes it, and `crate::physical::declared_contributor_tensor` already derives the boundary tensor from it.

## Boundaries

- **This is the read half only.** What a fold *commits* is decided by `CommittedTensor` and is not in question; `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate` already settled that half.
- **Three of the five sites are different families with their own reasons.** `SquaredSerialSum` belongs to `tiler::rms-norm-f32@1` and `StrictSerialMaximum` to `tiler::softmax-f32@1`, neither of which the request boundary recognizes, so widening them is unreachable today and should be argued separately rather than swept in. `FusedMultiplyAddSerialSum` *is* reachable — a fused prologue over a later declared input is what `crate::physical::fused_contributor_tensor` currently declines — and is part of this work.
- **The split and cooperative topologies must move with the serial one or a widened program loses its parallel alternatives.** Both name `ContributorTensor::Exactly(FIRST_INPUT)` for their partial pass; the compiler side already routes all four spellings through one `contributor_tensor`, so the whole compiler-side change is deleting the guard.
- **The positional-binding obligation must be restated, not dropped.** `ContributorTensor::DeclaredDomain` exists so a consumer binding buffers positionally cannot bind the wrong one. Whatever replaces it must still answer that question for a fold — a declared input ordinal inside the program's declared arity, or an intermediate — rather than admitting any `TensorRole`.

## Identity expectation, to determine at the owning site

The precedent is [`admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`](admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md): a widened *admission rule* that edits no encoder moves no already-encodable region's bytes, so it is appends-only with no domain step and no ledger entry. `push_tensor_role` already writes `0x01` plus a four-byte big-endian ordinal for every `Input`, so an ordinal other than zero is already encodable and already distinguished. Verify that rather than assume it: the check that ticket used was comparing the distinct maximal `[0-9a-f]` runs of length 16 and 64 over `crates/` between the base and the branch.

The compiler side moves no pin either — `NormalizedSerialSum::contributor_input` and its subject field are already carried and already encoded, and `crates/tiler-compiler/src/request.rs`'s `encode_elementwise_reads` already writes an unread-ordinal marker for every declared input a fold does not read, so the widened subjects exist today and are already distinct.

## Closes when

1. A `ScalarProgram::StrictSerialSum` region under `ReductionTopology::Serial` verifies with its contributor read at any declared input ordinal, and a region naming an ordinal outside the program's declared arity is still refused — by the assembler, exactly as an elementwise region's read is.
2. The split's partial pass and the cooperative tile carry the same ordinal, with a region whose passes disagree observed refusing.
3. `FusedMultiplyAddSerialSum` admits its recognized ordinal, and `crate::physical::fused_contributor_tensor`'s `InputOrdinal::FIRST` gate is deleted rather than left as an unreachable narrowing.
4. `crates/tiler-compiler/src/request.rs`'s `sum-contributor-ordinal` refusal is deleted, `request::tests::a_fold_over_a_later_declared_input_refuses_by_name` is flipped into an admission asserting the region's bound ordinal, and `pipeline::conformance::outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read`'s withheld-fusion assertion is inverted in the same change.
5. The identity determination above is executed at its owning site: either shown by recomputation not to move, or moved as a complete step with every pin enumerated.
