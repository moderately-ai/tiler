---
id: correct-the-extrema-familys-identity-ground-and-name-its-padding-identity
title: Correct the extrema family's identity ground and name its padding identity
status: done
priority: p2
dependencies: []
related: [derive-the-multi-round-two-level-reduction-composition]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, numerics, reductions, doc-claim]
---
## User-visible outcome

`ScalarProgram::StrictSerialMaximum`'s doc states a ground its own family refutes, and the next reader of it stops concluding that the extrema fold can never be padded.

## The defect, stated so it can be refuted in one command

**Fact.** `ScalarProgram::StrictSerialMaximum` (`crates/tiler-ir/src/schedule/model.rs`) reads: "**There is deliberately no empty-domain identity, and the omission is the contract rather than an oversight.** … the extrema families have no identity: no binary32 value `i` satisfies `Maximum(i, x) == x` for every `x`, because any candidate is itself a possible contributor."

**Fact.** `BinaryOp::F32Maximum` (`crates/tiler-ir/src/kernel/model.rs`) is "The IEEE 754-2019 `maximum` of two binary32 values … **The NaN-propagating extrema family, with `-0.0` ordered below `+0.0`**", which [ADR 0023](../docs/decisions/0023-floating-point-extrema-semantics.md) fixes.

**Inference — the conclusion stands and the stated ground is false.** `maximum(-inf, x) = x` for every binary32 `x`: for finite and infinite `x` because `-inf` is the order's minimum and `maximum(-inf, -inf) = -inf`; for `±0.0` because the family orders both above `-inf`; and for a NaN because the family propagates, with the fold's per-combine `CanonicalizeF32Nan` making the committed bits the same ones the unpadded fold commits. `0xff80_0000` is therefore a two-sided identity, and the sentence's reason — "any candidate is itself a possible contributor" — is an argument about an *empty-domain result* being indistinguishable from a real fold, which is a different claim from algebraic identity. [Numerical semantics](../docs/numerical-semantics.md) and [ADR 0022](../docs/decisions/0022-reduction-identities-and-initial-values.md) already separate empty result, algebraic identity, and replicable padding; the comment collapses them.

**Why it is load-bearing rather than a wording nit.** [The multi-round two-level reduction composition](../docs/research/scheduling/multi-round-two-level-reduction-composition.md) needs a padding identity for the extrema family, because a two-level composition pads at an imposed subgroup width and the family's non-emptiness argument — "a product of nonzero factors equalling a nonzero total forces every factor nonzero" — is stated for exactly covered splits and does not reach a padded one. A reader who carries the comment forward concludes the composition can never serve the softmax row maximum it exists for.

## Work

- Correct the ground in `ScalarProgram::StrictSerialMaximum`'s doc: keep the empty-domain refusal and its ADR 0023 and ADR 0025 basis, drop or repair the algebraic claim, and state the padding question separately. Describe what the code does now — no field is being added by this ticket.
- Check the sibling comment on `empty_domain_is_satisfied` in `crates/tiler-ir/src/schedule/builder.rs`, whose non-emptiness derivation is correct *for exact coverage* and should say so rather than read as unconditional.
- Add no field and no behaviour. A stated padding identity on a schedule is [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) decision 7's public boundary and is Tom's, not this ticket's; this ticket only stops the doc from denying that such a value exists.
- Size the test to the change: a doc correction needs none, and one asserting a comment's wording would be worse than none.

## Closes when

The comment describes the family the code implements, the empty-domain refusal keeps its authority, and no reader can derive "the extrema family cannot be padded" from either site.

## Outcome

**The derivation was re-verified before either side was written, and it holds against the emitted realization rather than only the specification.** `tiler_maximum_f32` in `crates/tiler-metal/src/emit.rs` is four exhaustive arms: `left < right` returns `right`, `right < left` returns `left`, `left == right` returns the bitwise `and` of the payloads, and the unordered arm returns `0x7fc0_0000`. Against `i = 0xff80_0000`: a finite or `+inf` `x` takes the first arm and comes back with its own bits; `±0.0` takes it too, because `-inf < 0` compares strictly and the equal-arm's sign-clearing `and` is never reached, so `-0.0` survives as `-0.0`; `x = -inf` takes the equal arm, where `0xff80_0000 & 0xff80_0000` is `-inf`; and a NaN `x` takes the unordered arm, after which the fold's `ConvertOp::CanonicalizeF32Nan` commits the same canonical bits the unpadded fold commits for that NaN (`emit_maximum_reduction` canonicalizes the single-contributor seed and every combine; `emit_staged_fold` canonicalizes every staged combine). Every arm is symmetric in its operands, so the identity is two-sided. **No counterexample was found, and the ticket's `-inf` claim is confirmed.**

**Why the empty-domain refusal survives the correction.** The refusal never depended on the algebra. What a maximum over *no* contributors means is a declaration [ADR 0022](../docs/decisions/0022-reduction-identities-and-initial-values.md) puts at the operation rather than at a schedule or a backend, and no registered operation embedding this fold declares one — `SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS` states that softmax is shape-preserving, so the reduction empty-domain rules do not reach it. [ADR 0025](../docs/decisions/0025-reduction-empty-results-and-padding.md) separates empty result from proven padding *in both directions*, so proving `-inf` neutral neither supplies nor weakens the refusal.

**Sites corrected** (doc comments only; no field, no behaviour, no test — a test asserting a comment's wording would be worse than none):

- `crates/tiler-ir/src/schedule/model.rs`, `ScalarProgram::StrictSerialMaximum` — the false algebraic ground is replaced by the semantic one, and a new paragraph names `0xff80_0000` as the proved-neutral padding value, cites ADR 0100 decision 7 for the walk, and records that no schedule here pads (`TailPolicy::Exact`, `ContributorPartition::covers`) and that a field stating a padding identity is an unaccepted public boundary. Its parallel-forms paragraph now says the non-emptiness argument is an *exactly covering* split's.
- `crates/tiler-ir/src/schedule/builder.rs`, `empty_domain_is_satisfied` — the non-emptiness derivation now states exact coverage as a premise and names which premise a padded split would replace.
- `crates/tiler-ir/src/schedule/builder.rs`, `EmptyDomainContract` — "no value that could ever be correct" narrowed to "no *empty-domain* value it could commit", which is the same collapse in the type that defines the term.
- `crates/tiler-ir/src/kernel/lower.rs`, `emit_maximum_reduction` — "`Maximum` has no identity, so there is no value to commit" was the false ground restated at the lowering; it now rests on the absent declaration and notes that `-inf` is neutral and that no emission pads with it.

**Also fixed, found by this ticket's own verification.** `RUSTDOCFLAGS="-W warnings" cargo doc --document-private-items -p tiler-ir` reported an unresolved intra-doc link to `ContributorPartition` at `builder.rs:1399` — the identifier is not imported at module scope, and private-item links are not covered by the gate's rustdoc step. Qualified to `super::model::ContributorPartition`; the run drops from 16 warnings to 15, and that run is also the evidence that the four links added here resolve.

**Sibling sites left alone, with the reason.** `crates/tiler-metal/src/tests.rs:3664`, `crates/tiler-reference/src/softmax.rs:267`, and `crates/tiler-compiler/src/physical.rs:2504` use "the family has no identity" in the empty-domain sense their surrounding assertion is about, and all three are outside `implementation/ir`. `crates/tiler-ir/src/semantic/softmax.rs:313` and the extrema test docs in `builder.rs` use "identity-less" as the corpus's term for "carries no empty-domain identity field", which the corrected `EmptyDomainContract` doc now defines without the absolute claim; none of them restates the refuted algebraic ground. `tickets/admit-a-parallel-topology-for-the-identity-less-extrema-fold.md:23` repeats the false ground verbatim, but it is a `done` ticket — a record of what was believed when it was claimed — and rewriting a closed record's stated Fact is not this ticket's remit; ADR 0100's implementation boundary already carries the correction where a reader would look.

**Checks.** `cargo fmt --check` on the three touched files, `cargo check -p tiler-ir`, `cargo clippy -p tiler-ir --all-targets -- -D warnings`, `cargo nextest run -p tiler-ir` (865 passed), `cargo test -p tiler-ir --doc` (9 + 8 compile-fail passed), and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir` all pass. `git diff --check` clean.
