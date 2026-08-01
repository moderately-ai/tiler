---
id: correct-the-softmax-worked-example-and-its-recorded-divergence
title: Correct the softmax worked example and attribute its recorded divergence
status: in-progress
priority: p1
dependencies: []
related: [admit-the-softmax-family, scope-transformer-nonlinear-normalization-and-reductions, design-model-level-qualification-and-optimization, retain-the-c1-attention-block-conformance-evidence, correct-the-softmax-divergence-attribution-in-code]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, softmax, measurement, correction, transformer]
claimed_from: todo
assignee: worker-softmax-record
lease_expires_at: 1785602431
---
## User-visible outcome

A reader of the L3′ derivation's softmax worked example is told which implementation produced its bit patterns, so the example stops reading as a demonstration that the pinned formula reproduces the reference — which, at that row, it does not.

## Why this is filed

**Fact — the record's own numbers do not follow from its own formula.** [Transformer non-linear, normalization, and reduction contracts](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) states the softmax formula as `r_i = e_i * (1 / d)` and then gives a worked example over `[1.0, 2.0, 3.0, mask]` whose recorded intermediates are `e = 0x3e0a9555 0x3ebc5ab2 0x3f800000 0x00000000` and `d = 0x3fc06957`, and whose recorded outputs are `0x3db861f2 0x3e7a9a18 0x3f2a4d3a 0x00000000` summing to `0x3f7ffffe`. Applying the record's own formula to the record's own `e` and `d` gives `0x3db861f3 0x3e7a9a1a 0x3f2a4d3b 0x00000000`, summing to exactly `0x3f800000`. The recorded outputs require a reciprocal of `0x3f2a4d3a`, while the correctly rounded `1.0 / d` is `0x3f2a4d3b`.

**Measurement — 2026-08-01, in the retained probe's own pinned environment** (`torch` 2.6.0, `transformers` 4.51.0, CPU, F32, run from `spikes/numerics/transformer_reference_semantics/` with `uv run --offline`):

- `torch.nn.functional.softmax` on that row returns the record's bits, so the record's *observation* is correct.
- Computed from the reference's own `e` and `d`, **both** `e * (1/d)` and `e / d` return `0x3db861f3 0x3e7a9a1a 0x3f2a4d3b`. So the reference's fused kernel matches neither of the two spellings the record names, at the row the record uses to illustrate them.
- A single constant explains every finite output: dividing each recorded output by its exponential yields `0x3f2a4d3a` at all three positions. **The divergence is in the reciprocal**, not in the exponential and not in the sum, both of which the reference reproduces bit for bit.
- Over 20,000 random rows per width, `F.softmax` equals `e * f32(1/d)` at **every** element at width two (40,000 elements) and width three (60,000 elements), and diverges from *both* spellings by up to four ULP from width four upward.

**Inference — this is the same class of finding as the `rsqrt` one, and it lands in the same place.** The reference model performs an approximation its own formula distinguishes, and only a discriminating argument detects it. It is a finding *about the reference model*, feeding a model-level bound rather than a per-operation tolerance, exactly as the correction under *RMS normalization* records for `torch.rsqrt`.

**Inference — the probe's attribution of `matches_neither` is at least incomplete.** [The reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md) explains its `matches_neither` column as "the denominator's own accumulation order disagreeing with the naive sum". At the worked example the denominator agrees exactly with the strict left fold, so accumulation order cannot be the cause there. The approximate reciprocal is a second, unrecorded source, and the probe's prose currently reads as if there is one.

## Required delivery

- **Correct the worked example's presentation** in the L3′ record: keep the measured bits, label them as `torch.nn.functional.softmax`'s output at that row, and state beside them what the pinned formula gives and why the two differ. The current table reads as a derivation of the formula's own result.
- **Correct the row-sum claim's evidence.** "The outputs sum to `0x3f7ffffe`, not to `0x3f800000`" is true of the reference at that row and *false* of the pinned formula there. The claim it supports — softmax does not produce a row summing to exactly one — is still true, and `admit-the-softmax-family` pins two rows that carry it under the pinned formula: `[0.0, 2.0]` sums to `0x3f7fffff` and `[0.0, 1.0, 0.0]` to `0x3f800001`, both at widths where the reference and the pinned formula agree at every element.
- **Correct the probe's `matches_neither` explanation** to name both sources, or measure which of them dominates at each width.
- **Decide whether the probe gains rows.** Two are missing and each was needed by the admission and had to be measured outside the retained record: a worked-example row, and a softmax row with a NaN score. The exact check for the second: `grep -i nan spikes/numerics/transformer_reference_semantics/results/*/record.tsv` returns only `silu_inputs` and the SiLU result rows. Adding them makes the D-2 evidence and the divergence reproducible from the retained record instead of from a re-run.
- **State the boundary.** The measurements above are one host class, CPU, F32, and those two package versions. Whether the divergence is the CPU vectorized path, a NEON reciprocal estimate, or something else is *not* established, and this ticket must not assert a mechanism it did not measure.

## Non-goals

Changing `tiler::softmax-f32@1`'s pinned formula. The width-two and width-three agreement is what selects the reciprocal form, and it is stronger evidence than the record originally carried: agreement at every element rather than at discriminating elements only. Reproducing `torch`'s approximate reciprocal is likewise a non-goal — the registered contract states what the operation means, and the reference model falling outside it is recorded rather than adopted.

## Reconsideration trigger

Active now: the record is cited by an implemented family whose conformance corpus disagrees with it at a row the record presents as agreeing.

## Outcome

**Done.** The L3′ record's softmax worked example now distinguishes the pinned formula's result from the reference model's, the derivation of the correct value is recorded beside it with its citing tests, and the retained probe carries the evidence the admission had to measure outside it. **One finding overturns this ticket's own central inference**, and it is the most important result here: the divergence is *not* an approximate reciprocal.

### The attribution this ticket asserted is wrong, and the corrected one is stronger

**Measurement — the check is one line.** The reference's implied normalization constant at the worked example is `0x3f2a4d3a`, one ULP below the correctly rounded `1.0 / 0x3fc06957 = 0x3f2a4d3b`. It is **exactly the correctly rounded reciprocal of `0x3fc06958`**, which this row's own four exponentials reach under the contributor order `(e₀, e₂, e₁, e₃)`: `((e₀ + e₂) + e₁) + e₃` is `0x3fc06958` where the strict left fold `((e₀ + e₁) + e₂) + e₃` is `0x3fc06957`. So the reference is performing *the pinned formula* over a permuted contributor sequence, not an approximation the formula excludes.

**Where this ticket's reasoning failed, stated so the error is reusable.** *Why this is filed* argued: "At the worked example the denominator agrees exactly with the strict left fold, so accumulation order cannot be the cause there." The premise is true of the record's own recomputation and says nothing about the reference's *internal* denominator, which was never observed — the inference assumes the reference sums in the strict left order, which is the very thing in question. The retained probe's original prose ("the denominator's own accumulation order") was right, and this ticket's challenge to it was not.

**Measurement — at scale, with the boundary.** Over 20,000 rows per width, the reference's output row is *exactly* one scalar multiple of these exponentials at every element of every row, at all five widths — 100,000 rows, no exception. The constant equals the naive sum's correctly rounded reciprocal in 20,000/20,000 rows at widths two and three, 14,680 at width four, and is never more than three ULP away there. At width four, where the summation orders are enumerable, **19,895 of 20,000 constants are the correctly rounded reciprocal of a denominator these exponentials reach under some strict left fold or the balanced tree.** The enumeration is not every legal grouping, so the count is a lower bound on reachability: it eliminates the approximate-reciprocal hypothesis where it is high and does not establish it where it falls short. Widths eight and eighteen are not enumerable and are left open rather than generalized.

**This strengthens the record's reciprocal-*form* evidence and widens it past the narrow rows.** A division by a denominator is not a single-constant multiply, so the single-constant result selects the reciprocal form at *every* width, where the record's discriminating-element counts could only do it at widths two and three. Perturbation, run to prove the check can say no: substituting `e_i / d` for the reference drops the single-constant count from 20,000 to 16,963, 13,875, 11,091, 5,505, and 2,356 at widths two, three, four, eight, and eighteen.

### A second, independent error found in the same landing's evidence

`torch.max` over `[+0.0, -0.0]` is **`-0.0`** (`0x80000000`), not `+0.0` as the admission recorded. It is an order *dependence*, not an ordering rule: `torch.max` returns the second operand and `torch.amax` the first, each reversing when the operands do, so neither implements the `-0.0 < +0.0` total ordering ADR 0023's Tiler families share — the same defect Metal's `fmax` has. **Nothing in decision D-2 rests on it**; its three stated grounds are about NaN, and the signed-zero ordering is Tiler's own choice. Recorded as an evidence correction, not a decision reopening.

### Each sentence changed in the L3′ record

The worked-example table's single result row, labelled `e · (1/d)` and holding `0x3db861f2 0x3e7a9a18 0x3f2a4d3a 0x00000000`, became two rows — "result under the pinned formula" with `0x3db861f3 0x3e7a9a1a 0x3f2a4d3b 0x00000000` and "result from `torch.nn.functional.softmax`" with the original bits — and the denominator row now says "under the strict left fold". **No recorded bit changed value; the measurement stands and its attribution moved**, exactly the `rsqrt` idiom. A new `#### Correction` subsection carries the arithmetic, the reordered denominator, the scale measurement and its boundary, the citing tests, the order-contract consequence, the non-goal, and the propagation question.

The row-sum sentence "the outputs sum to `0x3f7ffffe`, not to `0x3f800000`" became "the reference model's outputs sum to `0x3f7ffffe`; the pinned formula's sum to exactly `0x3f800000`", and now says the claim it supports is *not* carried by this row — it is carried by `[0.0, 2.0]` at `0x3f7fffff` and `[0.0, 1.0, 0.0]` at `0x3f800001`, both at widths where the two agree at every element, citing `a_rows_outputs_do_not_sum_to_exactly_one` and `the_row_sum_fact_forbids_a_unit_sum_check`.

Swept and corrected: the header's "two of the three … softmax remains at R2 and its ticket remains open" → all three landed at R5; Traceability's two-landing list → three; the reciprocal-form measurement gained the every-width strengthening and the measured `matches_neither` attribution; the pinned formula block's `MaximumNumber` first line, which D-2 settled the other way; the dtype-signature claim that the maximum's accumulator is "F32 with no widening question" (it declares none, because it selects bit patterns); the exceptional-values table's D-1, D-2 and "not measured" NaN rows; the order contract, which landed as *two* facts with `ExtremumShiftedOrderedReduction`; "a maximum reduction still resolves to no fusion legality" (now false, with the standalone-family absence and the serial-topology-only narrowing preserved); "confirmed by both landings" → three, with the softmax's absence check; "what remains missing on Tiler's side is the *maximum* reduction" (arrived, but as an exact fixup rather than `air.fmax.f32`); the typed-refusal status; D-1 and D-2 closed with their eliminations; D-4 closed for all three families with Gap 3 unmoved; D-5 consumed for both sums with the general contract still elsewhere; the capability table's row 3; the ladder table's softmax, `Select`, reductions and transcendentals rows; and the ladder's closing inference, which now distinguishes the two implementation-found corrections rather than collapsing them — the `rsqrt` one is outside a contract, this one is inside it at a withheld freedom.

### The symbolic-extent claim

The typed refusal "a softmax whose reduced axis is a symbol with no proved upper bound refuses" is refuted and now says so: `Extent` is a `u64` newtype, so the precondition can never be false and the check could never fire. The rule is **kept** with `the_reduced_extent_is_always_literal_so_no_symbolic_refusal_can_fire` cited and the three tickets that would make it implementable named — deleting it would lose the requirement along with the check.

### Propagation to the model-level surface

**Stated, not resolved, and no fixture recomputed.** The C1 attention block's retained softmax-dependent values come from this same implementation (`torch.nn.functional.softmax`, `spikes/program-planning/attention-block-reference/probe.py`) over a `[8, 2, 10, 10]` score tensor whose causally masked rows carry one to ten live contributors — so rows at four or more live contributors sit in the band where the constant is not always the naive sum's reciprocal. **The bound owner already covers it**: `design-model-level-qualification-and-optimization` owns the model-level bound *and* the reference's own F32 sensitivity envelope obtained by evaluating the pinned reference under two independently legal orderings, which is this divergence's mechanism at model scale. Pointers added at the worked example and in *What this record does not decide*, so a reader reaches the owner rather than composing a bound from a per-operation deviation — which L1 records is the defect rather than the method.

### The probe gained rows, and a second retained record

Decided **yes**. `probe.py` gains the worked example with its intermediates, both spellings, the reference's implied constant and the reordered denominator that produces it; a softmax row with a NaN score plus `torch.max` on that row; signed zeros in both orders and two spellings; and the `softmax_constant_width_*` attribution rows plus the width-four exhaustive order enumeration. The new rows draw no random numbers, so the pre-existing rows are unchanged: `diff` against the 2026-07-31 record reports **added lines only, zero changed or deleted**, and two consecutive runs were byte-identical. `results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv` is retained; the 2026-07-31 record is **kept rather than replaced**, because the L3′ derivation and the SiLU and RMS-normalization landings were written against those exact bytes. The probe README's `matches_neither` note now reports a measurement instead of an attribution, and gains the third perturbation.

### Measurement boundary

One host class, CPU, F32, `torch` 2.6.0 and `transformers` 4.51.0, via `uv run --offline` from the probe's own directory. **The mechanism is not established and is not asserted**: which summation order the reference's kernel uses, and whether the 105 unexplained width-four rows are reachable under a grouping the enumeration omitted or by something else, are both open. Widths eight and eighteen carry no reachability evidence at all.

### Out of scope, filed rather than absorbed

[`correct-the-softmax-divergence-attribution-in-code`](correct-the-softmax-divergence-attribution-in-code.md) — the same misattribution in `crates/tiler-reference/src/softmax.rs`, its test's doc comment, `crates/tiler-ir/src/semantic/softmax.rs` (which additionally carries the false signed-zero claim and a now-stale `grep -i nan` gap statement), and the `docs/roadmap.md` matrix row. **No test fails and no bit is wrong** — every assertion in the softmax corpus is correct and stays correct; the defect is prose. A dated correction note was added to `admit-the-softmax-family`'s Outcome, which is in this ticket's shared `project/tickets` scope.

### This ticket's own title and premise, corrected

The title read "record its **reciprocal** divergence" and the id still says `-and-its-recorded-divergence`. The id is deliberately unchanged so every inbound link keeps resolving; the title now says *attribute* rather than name a mechanism the measurement rejected. The *Why this is filed* and *Non-goals* sections above are preserved unedited, because the reasoning that produced a wrong attribution is more useful to a later reader than a silently corrected file — the error was assuming the reference's internal denominator equalled the strict left fold the record recomputed.
