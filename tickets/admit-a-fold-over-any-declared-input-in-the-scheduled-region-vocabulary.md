---
id: admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary
title: Admit a fold over any declared input in the scheduled-region vocabulary
status: in-progress
priority: p2
dependencies: []
related: [admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs, admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary, admit-a-reduction-over-a-declared-input-tensor, admit-a-strict-serial-fold-that-writes-a-materialized-intermediate]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, schedule]
claimed_from: todo
assignee: sol-fold-input-ordinal
lease_expires_at: 1786408310
---
## User-visible outcome

A strict serial fold whose contributor tensor is a declared program input other than the *first* is expressible as a scheduled region, so `folded = sum(b)` beside an independent `doubled = a + a` compiles instead of refusing at the request boundary under `sum-contributor-ordinal`, and `sum(b * 2.0 + 1.0)` regains its fused single-region alternative.

## Why this exists

**Fact — the rule is in `tiler-ir` and it names the first input by construction.** `crates/tiler-ir/src/schedule/builder.rs` declares `const FIRST_INPUT: TensorRole = TensorRole::Input { ordinal: InputOrdinal::FIRST }`. The bare `StrictSerialSum` reaches `ContributorTensor::DeclaredDomain`, whose `admits` arm accepts only `TensorRole::Intermediate` or `FIRST_INPUT`; the fused, squared, squared-with-epilogue, and maximum families carry their own exact first-input requirements; and the split and cooperative topologies repeat the relevant exact requirements. A region reading `Input { ordinal: 1 }` under a `ScalarProgram::StrictSerialSum` therefore fails the intrinsic verifier with `ScheduledRegionDiagnostic::NumericalOrAccessRefinement`.

**Fact — current census, corrected 2026-08-09.** Reading every production `FIRST_INPUT` use before the first `#[cfg(test)]` boundary yields the declaration plus fourteen uses. Three bind the strict-affine U4 dequantize's `codes`, `scale`, and `zero_point` components and decide no fold contributor. The remaining **eleven** fold sites are `DeclaredDomain::admits`, four serial verification arms, and six split/cooperative family arms. The prior count of ten predated `SquaredSerialSumThenEpilogue`; its new serial arm is the eleventh site. Use source anchors `enum ContributorTensor`, `fn verify_access_and_semantics`, `fn multi_pass_family`, and `fn cooperative_family` rather than the old line-number census.

**Measurement (2026-08-06, on branch `tkt/admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`).** With the compiler-side refusal removed and the recognizer admitting the program, `sum(b)` beside `a * a` over two declared `[2, 2]` inputs reaches `InvalidCompilerOutput(Frontier(MalformedProposal { provider: tiler::prototype-serial-sum-physical@1, source: Intrinsic { rule: "numerical-or-access-refinement", region: RegionId(1) } }))` — every request-boundary and planning stage succeeds and only the intrinsic region verifier refuses. Reproduce by deleting the `sum-contributor-ordinal` guard in `recognize_reduction` and running `cargo nextest run -p tiler-compiler -E 'test(a_fold_over_a_later_declared_input_refuses_by_name)'`.

**Inference — the state became reachable exactly once, and that is why nothing observed it before.** A prologue-less fold's recognition walk reads exactly one tensor. While `canonical_input_reads` required every elementwise walk to read every declared input, such a program declared exactly one input and the recognized ordinal could only be zero, so `FIRST_INPUT` and "the ordinal the program named" could not differ. [`admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`](admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs.md) lifted that rule, made the divergence reachable, and closed it fail-closed rather than silently: the recognizer already carries the fact as `NormalizedSerialSum::contributor_input`, the request subject already encodes it, and `crate::physical::declared_contributor_tensor` already derives the boundary tensor from it.

## Boundaries

- **This is the read half only.** What a fold *commits* is decided by `CommittedTensor` and is not in question; `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate` already settled that half.
- **Four serial arms are different families with their own reasons.** `SquaredSerialSum` and `SquaredSerialSumThenEpilogue` belong to the RMS/staged-family path, while `StrictSerialMaximum` belongs to `tiler::softmax-f32@1`; none is admitted merely because the bare-sum contributor rule widens, so changing them is separate work. `FusedMultiplyAddSerialSum` *is* part of this ticket — a fused prologue over a later declared input is what `crate::physical::fused_contributor_tensor` currently declines.
- **The split and cooperative topologies must move with the serial one or a widened program loses its parallel alternatives.** Bare-sum serial verification and the multi-pass partial / cooperative bare-sum family arms share `ContributorTensor::DeclaredDomain`; widening `DeclaredDomain::admits` to any in-arity `TensorRole::Input` plus Intermediate is what moves those paths together. Fused serial, multi-pass partial, and cooperative arms name `read.tensor == FIRST_INPUT` / `ContributorTensor::Exactly(FIRST_INPUT)` and move with the fused half of this ticket (squared and maximum keep their own exact first-input rules, as the family bullet above states). On the compiler side the bare-sum path already binds through `contributor_tensor(serial.contributor_input)`, so the bare-sum request change is deleting the `sum-contributor-ordinal` guard; the fused half separately deletes the `InputOrdinal::FIRST` gate in `fused_contributor_tensor` and flips the named refusal / withheld-fusion tests. **Correction — 2026-08-10.** Earlier text claimed both split and cooperative "name `ContributorTensor::Exactly(FIRST_INPUT)` for their partial pass" and that "the whole compiler-side change is deleting the guard." For bare `StrictSerialSum`, multi-pass partial and cooperative use `ContributorTensor::DeclaredDomain`, not `Exactly(FIRST_INPUT)`; `Exactly(FIRST_INPUT)` is what the fused/squared/maximum parallel arms name. Deleting `sum-contributor-ordinal` is necessary for the bare-sum request path but is not the whole compiler-side change.
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
