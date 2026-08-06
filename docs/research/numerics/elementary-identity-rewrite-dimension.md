---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.elementary-identity-rewrite-dimension"
kind: "research"
title: "The elementary-identity rewrite dimension"
topics: ["numerics", "accuracy", "transcendentals", "optimizer"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "adopted"
adopted_by: ["ADR-0101"]
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.numerical-semantics"]
depends_on: ["tiler.research.numerics.certified-bounds-as-rewrite-permissions", "tiler.research.numerics.transcendental-accuracy-precedents"]
ticket: "name-the-elementary-identity-rewrite-dimension"
---

# The elementary-identity rewrite dimension

**Status:** derivation complete and adopted; the dimension is defined, its grain is derived, its refusal wording is specified, and the one product choice in the neighbourhood — whether to admit a permission — is deferred with a stated trigger rather than made. The vocabulary proposal in Part 8 was carried to [ADR 0101](../../decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md), which Tom accepted on 2026-08-06; the dimension's definition is now normative in [Numerical semantics](../../numerical-semantics.md#elementary-function-identity-is-a-fourth-dimension).

## Traceability

- **Current disposition:** adopted, by [ADR 0101](../../decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md), accepted by Tom on 2026-08-06 at the live decision review in the coordination session, witnessed first-hand by the coordinator; the `adopted_by` edge is set in the same change, per the sweep this line previously promised.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md#elementary-function-identity-is-a-fourth-dimension) now owns the dimension's definition, in the same place it owns [distributivity](../../numerical-semantics.md#distributivity-is-outside-the-order-contract), edited in the acceptance's own sweep as this record stated it would be.
- **Evidence:** the executable witnesses at [`spikes/numerics/elementary_identity_folding/`](../../../spikes/numerics/elementary_identity_folding/README.md), and the crate sources read in full and cited by exact path below.
- **Builds on:** [Certified rounding-error bounds as rewrite permissions](certified-bounds-as-rewrite-permissions.md), whose Part 2 found the freedom and named it unnamed, and whose Part 3 fixed the admission-rule shape this dimension would sit inside; and [Transcendental accuracy precedents](transcendental-accuracy-precedents.md), which is the evidence behind the accuracy contract this dimension is repeatedly mistaken for.
- **Work record:** [`name-the-elementary-identity-rewrite-dimension`](../../../tickets/name-the-elementary-identity-rewrite-dimension.md).

## Outcome

**The freedom is one numerical dimension, not a family, and it is a fourth one: rewriting an expression through an elementary function's own identity.** Four results carry that, and the fourth is the one that decides the outcome.

**Inference — no dimension Tiler declares reaches it, checked by exhausting the list rather than by intuition.** The eleven governed dimensions of `ScalarNumericPolicy` and the named-but-unpermissioned distributivity dimension are all statements about *ring operations over a contributor sequence* — which grouping, which order, which products, which roundings. This freedom rewrites *through* the function, exchanging one composition of elementary evaluations for another that is equal over the reals by a functional equation. Part 1 walks all twelve.

**Measurement — the rewrite is observable under the strongest exponential any target could declare, so no accuracy contract can absorb it.** Over the non-positive integer grid `[-40, 0]` in both arguments — the region the governed softmax's exponential admits — 502 of 1681 argument pairs disagree between `fl(exp(a)) * fl(exp(b))` and `fl(exp(a + b))` with `exp` *correctly rounded* to binary32. The smallest is `a = b = -1.0`: `exp(-1.0)` is `0x3ebc5ab2`, its square rounds to `0x3e0a9556`, and `exp(-2.0)` is `0x3e0a9555`. One ulp apart, in the ordinary regime, with every individual evaluation exact to the last bit.

**Inference — one dimension, with the per-function content carried at two other layers.** [ADR 0014](../../decisions/0014-reassociation-vs-permutation.md)'s standard for splitting a dimension is evidence of a *capability asymmetry*, and the asymmetries that exist between `exp`'s, `log`'s, and `sqrt`'s identities are not that. They are an error-magnitude asymmetry, which the rule's own derived bound already prices, and a real-domain side-condition asymmetry, which [ADR 0021](../../decisions/0021-validated-value-assumptions.md)'s value-assumption machinery already governs. Splitting the permission would duplicate in the contract vocabulary what the operation capability and the rule's bound carry injectively. Part 2 runs the elimination.

**Inference — the correct outcome today is named-and-unpermissioned, in the shape [ADR 0080](../../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) used, because this dimension has no caller that is not already blocked elsewhere.** Its one identified caller is the online-softmax rescaling fold, which independently consumes distributivity, which [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) declined. Admitting this permission alone would enable no rewrite — the same "the vocabulary carries no caller-less permissions" ground ADR 0095 rests on, applied to a dimension ADR 0095 does not mention. **This does not presume the distributivity reassessment's outcome**; Part 6 states what each of its four outcomes implies, and the deferral's trigger is written so that the two decisions can be taken together.

## Part 1 — What the freedom is, and why no declared dimension reaches it

### The definition

**Proposal.** An **elementary-function identity rewrite** replaces one composition of evaluations of a registered elementary function with another composition that is equal to it over the reals by a *functional equation of that function*, where the two compositions differ in the number of evaluations, in the arguments the evaluations receive, or in both.

The three instances the certified-bounds record names, with their real-domain side conditions:

```text
exp(a) * exp(b)  =  exp(a + b)          for all real a, b
log(a) + log(b)  =  log(a * b)          for a > 0 and b > 0
sqrt(a) * sqrt(b) = sqrt(a * b)         for a >= 0 and b >= 0
```

Two things are deliberately *not* in the definition, and each exclusion is doing work.

**Not "transcendental".** `sqrt` is an elementary function and not a transcendental one, and its identity has exactly the shape the other two do. Tiler's own vocabulary already says *elementary* where the property is about this class — `assess_program_elementary_accuracy`, `ElementaryRealization`, `ApproximationEnvelope::BackendElementary`, the *backend elementary* conformance level — while [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) and the [Transcendental accuracy](../../numerical-semantics.md#transcendental-accuracy) section say *transcendental* for the accuracy contract. The wider word is the correct one here, and the narrower one would exclude a case the definition has to cover.

**Not "any real-valued identity of the expression".** Distributivity, associativity, and commutativity are identities too, and they are ring identities: they hold for `+` and `*` whatever values the leaves happen to be, and Tiler's existing dimensions are about exactly them. A functional equation is a property of *one named function*, so a permission over it is meaningless without knowing which function is involved — which is the structural fact Part 2's grain argument turns on.

### The worked counterexample

**Measurement**, from [`identity_counterexample.py`](../../../spikes/numerics/elementary_identity_folding/README.md) at `spikes/numerics/elementary_identity_folding/`, exact in `Decimal` at 120 digits with one rounding to binary32 per operation, retained as `counterexample.tsv`:

| quantity | binary32 bits |
| --- | --- |
| `exp(-1.0)`, correctly rounded | `0x3ebc5ab2` |
| `fl(exp(-1.0) * exp(-1.0))` | `0x3e0a9556` |
| `exp(-2.0)`, correctly rounded | `0x3e0a9555` |

502 of 1681 non-positive integer argument pairs in `[-40, 0]` disagree — 29.9%.

**Inference, and it is the load-bearing one for Part 4.** The exponential here is *correctly rounded*, which is the strongest form [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) defines and the strongest any realization could refine. Both sides of the identity therefore satisfy every accuracy contract that could be written for `exp`, simultaneously, and they still return different bits. **So the divergence is not an accuracy failure of any evaluation. It is a property of the rewrite**, and no tightening of any operation's accuracy contract reduces it. A dimension is the only vocabulary Tiler has for a freedom of that kind.

**Measurement boundary.** The grid is the region the governed softmax's exponential admits and no wider: `SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` is `+0.0` because the maximum subtraction confines the argument to the non-positive reals, so a counterexample above zero would not reach the rewrite in question. The retained run also records what it cannot separate: substituting a host `math.exp` rounded from float64 for the correctly rounded one changes no verdict on this grid, so these numbers separate the identity's error from a *large* implementation error and not from a one-ulp one.

### Walking the twelve, so the gap is checked rather than asserted

**Fact — the declared dimension set is eleven, read at `crates/tiler-ir/src/numerics.rs:149`.** `CANONICAL_DIMENSIONS` lists `InputSubnormals`, `ResultSubnormals`, `Contraction`, `Reassociation`, `Permutation`, `SignedZero`, `ReciprocalTransform`, `ApproximateIntrinsics`, `NanAssumptions`, `InfinityAssumptions`, `MaterializationRounding`. `NumericalContract` at `crates/tiler-compiler/src/session.rs:1361` resolves the same eleven. Distributivity is a twelfth that is *named* by [Numerical semantics](../../numerical-semantics.md#distributivity-is-outside-the-order-contract) and declared nowhere.

**Inference — nine of the twelve are eliminated by their subject.** The two subnormal dimensions and the two exceptional-value assumptions are about which *values* the arithmetic may see or produce; `SignedZero` is about eliminating a distinction between two values; `MaterializationRounding` is about an observable boundary; `NanAssumptions` and `InfinityAssumptions` are value-domain assumptions with provenance classes. None is a statement about replacing one expression with another, so none can be evaluated for this rewrite at all — which is the same "there is nothing for it to be a statement about" argument ADR 0080 made for distributivity, applied to a different set.

**Inference — reassociation and permutation are eliminated by the certified-bounds record's own derivation, and the elimination is sharper here than there.** Both are defined over a fixed contributor sequence: reassociation varies the grouping and permutation varies the order, and both hold the leaf values fixed. In `exp(a) * exp(b) → exp(a + b)`, the leaves are `exp(a)` and `exp(b)` before and `exp(a + b)` after — one leaf where there were two, and not a value either of the originals had. There is no sequence of which both are groupings and no permutation carrying one to the other.

**Inference — contraction is eliminated, and the reason is [ADR 0015](../../decisions/0015-fma-vs-contraction.md)'s.** Contraction permits one existing `a * b + c` pattern to round once instead of twice. The identity rewrite forms no fused multiply-add: it removes a multiply and creates an add *inside the function's argument*. ADR 0015's own rule that a permission over an existing pattern does not authorize manufacturing a new one runs in the same direction here.

**Inference — distributivity is eliminated, and stating this precisely matters because the two dimensions appear together.** Distributivity exchanges a product of a sum for a sum of products; it changes which products the ring operations form. The identity rewrite changes what the *function* is applied to. The online-softmax fold consumes both, which is exactly why they cannot be one dimension: a rewrite consuming two freedoms needs two names, or its refusal cannot say which one is missing. This is the same accounting ADR 0080 item 2 performed when it concluded a chain regroup consumes distributivity *and* reassociation *and* permutation.

**Inference — `ReciprocalTransform` is eliminated, and it is the closest existing precedent rather than a match.** It governs replacing a stated `x / y` by `x * (1 / y)`, which is an identity of the field over ring operations and which the governed softmax pins in the *other* direction (`SOFTMAX_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED` is `false`, withholding permission to turn the pinned reciprocal multiplication back into a division). Its existence is the useful precedent: **Tiler already gives one named algebraic identity its own dimension rather than folding it into a general "algebraic rewriting" permission.** The proposal in Part 8 is that shape, one identity class further out.

**Inference — `ApproximateIntrinsics` is eliminated, and this is the elimination a reader is likeliest to skip.** It is the dimension that *sounds* like it covers elementary functions, and it is the one whose misreading would be most damaging. `ApproximationEnvelope` has exactly two resolutions: `Forbidden`, under which every elementary function follows its own resolved accuracy contract, and `BackendElementary`, which bounds the approximation by the backend's own stated accuracy. Both are statements about **which realization evaluates one declared elementary evaluation**. Neither says anything about how many evaluations there are or what arguments they receive. Three consequences follow and each is a separate reason the envelope cannot stand in:

1. The counterexample above holds with `Forbidden` resolved and a correctly rounded `exp` on both sides. An envelope that authorizes *no* approximation still leaves the rewrite's divergence untouched, so the divergence is outside what the envelope governs by construction.
2. After the rewrite, each surviving evaluation is still governed by whatever envelope the contract resolved. The rewrite's error is therefore **on top of** the envelope rather than inside it, and the two compose rather than one bounding the other.
3. Reading the envelope as covering the rewrite would make the envelope's bound a claim about a composition it was never derived over — an unbounded compounding wearing a stated tolerance, which is the shape [the certified-bounds record's](certified-bounds-as-rewrite-permissions.md) trust-boundary section lists among the things that must never be admitted.

**Conclusion of Part 1.** Twelve dimensions, twelve eliminations, and the gap is a gap in the dimension set rather than a missing permission inside one. That is the ticket's premise, now derived from the dimension list rather than asserted from the certified-bounds record's Part 2.

## Part 2 — One dimension or a family, on ADR 0014's standard

[ADR 0014](../../decisions/0014-reassociation-vs-permutation.md) split reassociation from operand permutation because **some combiners have one capability and not the other** — an associative but noncommutative combiner is a real object, so a single permission over-authorizes. [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) refused to split distributivity by direction on the same standard: "No analogous asymmetry has been established … so admitting a permission today would force a one-or-two-permissions choice with nothing behind it." That standard is what this part applies.

### Three candidate grains

- **(a) One dimension.** `elementary_identity: Forbidden | Permitted`, one field, covering every registered elementary family's identities in both directions.
- **(b) Per function family.** `exp_identity`, `log_identity`, `sqrt_identity`, …
- **(c) Per identity and direction.** `exp_product_to_sum`, `exp_sum_to_product`, `log_product_rule`, …

### The two asymmetries that exist, and what each one is evidence for

**Fact — the identities differ in real-domain side condition.** `exp(a + b) = exp(a) exp(b)` holds on all of `R`. `log(a) + log(b) = log(a * b)` holds only for positive `a` and `b`; the identity is *false* over the reals otherwise, because the left side is undefined where the right side is defined. `sqrt(a) sqrt(b) = sqrt(a b)` needs both non-negative.

**Inference — that is not a capability asymmetry, and Tiler already has the machinery it is an asymmetry in.** A side condition on the leaves is a value-domain fact, and [ADR 0021](../../decisions/0021-validated-value-assumptions.md) requires every value-domain fact used for correctness to carry provenance — compiler-proven, runtime-validated, or caller-declared-and-therefore-ineligible. [The certified-bounds record's](certified-bounds-as-rewrite-permissions.md) fifth admission obligation already states this for a bound that consults a precondition. So the logarithm's positivity requirement is discharged the way `PositiveNormalScalar` is discharged for a strict-affine scale: as a predicate the rule proves or the compilation refuses. Encoding it as a *second permission* would put a value-domain obligation in a vocabulary that has no way to check it, and would leave a caller who granted `log_identity` believing the positivity requirement had been granted too.

**Fact — the identities differ in floating-point error behaviour.** The exponential's telescoping error is governed by the argument magnitudes and by `eps_exp`, and the certified-bounds record derives it in closed form. The logarithm's product rule carries a cancellation hazard the exponential's does not: `log(a) + log(b)` sums two values of opposite sign whenever `a < 1 < b`, and catastrophic cancellation there has no counterpart in a product of two positive exponentials.

**Inference — that is an argument for two *bounds*, not two permissions, and the architecture already puts bounds somewhere else.** [The certified-bounds record's](certified-bounds-as-rewrite-permissions.md) Part 3 eliminated per-instance analysis and concluded that **a rewrite's rounding cost is a derived parametric bound carried by the rule**, checked against a caller-stated tolerance by exact rational comparison. Under that conclusion, "the logarithm's rewrite is more dangerous than the exponential's" is a statement about two rules' two bounds, and it is expressed by the two bounds differing. A caller who wants the exponential's rewrite and not the logarithm's states a tolerance the first meets and the second does not — which is a *quantitative* discrimination the permission vocabulary is deliberately not the place for, because a categorical permission cannot express "at this width and not that one".

### The elimination

**(c) is eliminated first and hardest.** It multiplies the contract by one field per identity per direction, and every one of those fields would have to be answered by every target profile forever. It also forces exactly the unevidenced directional cut ADR 0095 refused: nothing establishes that folding `exp(a) exp(b)` into `exp(a+b)` and splitting it back are separately capable operations, and over the reals the equation is symmetric. Under ADR 0095's own words, a directional cut without evidence is "an arbitrary cut presented as caution".

**(b) is eliminated on the standard, and on where the per-function content actually lives.** ADR 0014's two-fact structure already separates *which law holds* from *whether it may be consumed*: an operation declares an algebraic capability (`OperationAlgebraicCapabilities`, `crates/tiler-ir/src/semantic/operation.rs:922`, whose own documentation states that "a missing declaration is unknown, never evidence that the inverse law holds"), and the contract independently resolves the permission. **The per-function variation therefore belongs in the capability, where it is already carried per operation and already enters operation identity.** A per-function permission would be a second copy of the same information in a vocabulary keyed to the program rather than the operation, and the two copies could disagree — the failure mode ADR 0014's split exists to prevent, inverted.

**(a) survives, and the survival is what makes the three-layer answer statable.**

### The three layers, stated because the answer is not "one field"

**Proposal.** Consuming an elementary-identity rewrite requires three facts, of which the dimension is one:

1. **The operation declares the functional equation it satisfies**, with the equation's real-domain side condition, as an operation-owned identity-encoded capability. This is *not* a boolean: `exp` satisfies one equation and `log` another, so the declaration names an equation rather than asserting a law. This is the layer that carries per-function content.
2. **The contract resolves one permission**, `elementary_identity`, `Forbidden` by default under the omission-never-widens rule, in the canonical dimension vector.
3. **The rule carries its own derived bound and discharges the equation's side condition**, per the certified-bounds record's admission rule and ADR 0021's provenance classes. This is the layer that carries per-identity quantitative content.

**Fact — layer 1 does not exist today and its shape is a real cost.** `OperationAlgebraicCapabilities` (`crates/tiler-ir/src/semantic/operation.rs:922`) has exactly one law, `ordered_associativity`. [ADR 0014's implementation-boundary section](../../decisions/0014-reassociation-vs-permutation.md) already records that even commutativity has no capability to declare, and that where a family is in fact commutative "the property is recorded as a definition fact string … that no rule consults". **That claim is re-read here rather than quoted, and its line citation has drifted:** ADR 0014 cites `crates/tiler-ir/src/semantic/softmax.rs:543`, which is now a field-name line inside the accuracy-contract fact; the string it means is at `:554`, the value of `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY`. The claim itself holds. A functional-equation capability would be the vocabulary's first *parameterized* law, where the two existing candidates are flags. Part 5 counts what that costs.

## Part 3 — Per-operation or global

**Fact — every one of the eleven dimensions is resolved once for the program, not per operation.** [ADR 0011's implementation-boundary section](../../decisions/0011-per-operation-numerical-permissions.md) is explicit that the middle term of its own three-term resolution has no representation: "There is no per-operation restriction to intersect with, no region or operation override, and therefore no canonical per-operation permission representation." The granularity actually delivered is the scheduled region, and every region of one program compiles under the one contract stated in its request.

**Inference — the dimension is program-level, and making it the first per-operation one would be a defect rather than a feature.** The ticket asks whether the dimension should be per-operation "since accuracy contracts are already per-operation and per-target under ADR 0042", and the answer is that the two are per-operation in different senses that must not be merged. An accuracy contract is per-operation because it is an *obligation the operation carries* — a property of what `tiler::softmax-f32@1` means, which is why it lives in the definition facts and enters operation identity. A permission is per-program because it is *what the caller authorizes*, and ADR 0011 makes the program ceiling the outer authority precisely so that a local default cannot exceed it. A per-operation elementary-identity permission would be the only dimension resolved that way, would have no ceiling to be bounded by, and would make this dimension the one place where a rewrite's authorization was not traceable to a single caller-stated contract.

**Inference — the per-operation content the ticket is reaching for is layer 1, and it is per-operation already.** Which functional equation a family satisfies is an operation property, it is declared by the operation, and it enters the operation's identity. So the vocabulary the ticket wants at operation granularity exists at operation granularity; what is program-level is the authorization, exactly as it is for the other eleven.

## Part 4 — How it composes with the accuracy obligation

This is the part the ticket names as "the concrete question", and it has four answers.

### The accuracy contract is pointwise; the rewrite is compositional

**Fact — an ADR 0042 contract bounds one evaluation against one reference result.** Its predicates are `|z - r| <= t` and its relatives, where `r` is the infinitely precise reference result of *the operation* and `z` is the mathematical value of the finite candidate. The whole vocabulary — `Absolute`, `Relative`, `AbsoluteRelative`, `Ulp`, `AllOf`, `AnyOf` — quantifies over one `(input, result)` pair.

**Inference — and the counterexample is the proof rather than the illustration.** At `a = b = -1.0` every evaluation on both sides is correctly rounded, so the tightest contract ADR 0042 can express is satisfied on both sides, and the two sides differ by one ulp. **A pointwise contract cannot bound a compositional difference, and this one demonstrably does not.** The accuracy machinery is not merely insufficient for the identity rewrite; it is structurally unable to see it, and a design that reached for it would be reaching for the wrong authority rather than an incomplete one.

### The obligation set is invariant under the rewrite, so `readmit_candidate` does not close the gap

**Fact, read in full at `crates/tiler-compiler/src/target/accuracy.rs:739` and `:795`.** `required_elementary_accuracy` is a lookup keyed on `OpKey` — three arms, for `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, and `tiler::softmax-f32@1`. `assess_program_elementary_accuracy` walks a program's operation keys, **deduplicates the requirement set by operation**, and assesses each distinct contract once; its own documentation states the rule plainly: "a program containing one `tiler::silu-f32@1` occurrence and a program containing a hundred owe the same contract".

**Inference — an identity rewrite therefore cannot change the required set.** The online-softmax rewrite sits *inside* one atomic key, so it changes no key at all. A rewrite that crossed keys — folding two `Exp` occurrences into one, in a graph that registered a general `Exp`, which this one does not — would remove occurrences and not keys. Either way the deduplicated set is unchanged or shrinks.

**Fact — `readmit_candidate` rechecks, and its own comment says why that recheck is not this.** At `crates/tiler-compiler/src/request.rs:2139` it calls `require_elementary_accuracy` per candidate, over a comment reading "The obligation is a property of the candidate's operation multiset, and a rewrite that introduced a family this target cannot realize would otherwise inherit an admission granted to a program that did not contain it. Today's algebraic rules preserve the multiset, so this cannot fire."

**Inference — the recheck is correct, necessary, and answers a different question.** It guards *family introduction*. The identity rewrite introduces no family, so the recheck asks the same question and receives the same answer, and a design that treated "`readmit_candidate` asks again" as the machinery covering this rewrite would be relying on a check that provably cannot fire for it. The permission is a **separate feasibility answer at the same stage, in the same shape** — asked per candidate, fail-closed, never a cost — which is precisely the placement the certified-bounds record's Part 3 derived for a rewrite bound. Two authorities, one position.

### The rewrite can invalidate the accuracy contract's *domain* while every declared field stays byte-identical

**Fact — the softmax exponential's admitted domain is derived from the pinned evaluation order.** `SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS` is `+0.0`, and `crates/tiler-ir/src/semantic/softmax.rs:179` gives the derivation: "Every argument the exponential receives is `s_i - m` where `m` is the maximum of the same row, so the exact difference is never positive". The subordinate reference semantics carries the same clause in prose — "the maximum subtraction confines `t` to the non-positive reals" — and the whole `FiniteOverflowRule` below it is vacuous *because of* that confinement.

**Inference — a rewrite changes which arguments reach the function, and nothing rechecks the derivation the domain rests on.** The contract's declared fields do not move: the same clause, the same ceiling, the same tolerance. What moves is whether the sentence justifying the ceiling is still true. For the online form it happens to remain true, and the derivation is short enough to state: the running maximum is non-decreasing and includes `x_j`, so `x_j - m_j <= 0`, and `m_{j-1} - m_j <= 0` for the same reason — both argument families stay non-positive. **That is a discharge, not an observation**, and a rewrite that did not have it — a global rescale onto an earlier maximum, say — would place obligations on arguments the contract declares unreachable, with no field in which the difference could be seen.

**Proposal — a sixth admission obligation, beside the five [the certified-bounds record](certified-bounds-as-rewrite-permissions.md) states.** *The rewritten program's elementary arguments are proved to lie inside every accuracy clause's declared domain.* It fails closed like the other five, it is a property of the complete scheduled candidate like obligation 3, and it is the one obligation whose absence is invisible in the contract's own fields.

### The numeric accuracy is consumed as a bound *parameter*, in a role the refinement verdict cannot fill

**Fact.** The certified-bounds record's derived price for the online-softmax fold is `(1 + eps_exp)^(V-1) * (1 + gamma_{2(V-1)}) / (1 + gamma_{V-1}) - 1`. The `(1 + eps_exp)^(V-1)` factor is there because the rewrite changes how many exponentials each contributor passes through: one in the two-pass fold, `V` in the online fold — its own, plus one rescale factor per later step.

**Inference — so the elementary accuracy enters this dimension twice, in two different shapes, and only one of them exists today.** As a yes-or-no refinement obligation it is `assess_elementary_accuracy`'s answer and it is already asked. As the numeric `eps_exp` that instantiates the price, it is not retrievable from the authority that owns it — which is [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](../../../tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md), already filed, and which this record makes a prerequisite of any *quantitative* admission rather than of the dimension's definition.

## Part 5 — What declaring and consuming it costs

### The identity domain steps, counted rather than estimated

**Fact — the contract key is the canonical injective encoding of the dimension vector, so a twelfth dimension moves it.** `canonical_contract_key` (`crates/tiler-compiler/src/request.rs:143`) renders the arithmetic tag, the canonical arithmetic-NaN bits, and then each dimension's tag and behaviour in `CANONICAL_DIMENSIONS` order. `F32_NUMERICAL_CONTRACT_KEY_DOMAIN` is `tiler.contract.f32.v2` and `BF16_NUMERICAL_CONTRACT_KEY_DOMAIN` is `tiler.contract.bf16.v1` (`crates/tiler-ir/src/schedule/numerics.rs:546` and `:563`). Both would step, because the version counts the domain's own rendering revisions and this is a rendering change.

**Fact — the pinned key literals are two, and they are in `tiler-ir`.** `crates/tiler-ir/src/schedule/numerics.rs:1247` and `:1276` pin the rendered strict-`f32` and strict-`bf16` keys as literals, deliberately, so that a change to the shared dimension writer fails there rather than silently restating an artifact identity. Adding a dimension moves both, and per AGENTS.md an identity-domain step is executed completely or not at all: the domains, the ledger documents, and every pinned identity move in one commit with each moved pin enumerated.

**Fact — the exhaustive injectivity check widens with the space.** Injectivity over the whole statable space is checked exhaustively rather than sampled, in `crates/tiler-compiler/src/request.rs`. A twelfth binary dimension doubles that space.

### The cheap half: omission never widens

**Fact.** A composition starts at the strict resolution of every dimension, so "a dimension added to the vocabulary later arrives forbidden in every contract written before it existed". **Inference — no existing program's meaning changes**, and no registered contract silently becomes able to perform the rewrite. This is the property that makes an addition additive, and it is the same property ADR 0095 cited when it observed that admitting a permission later "is purely additive".

### The expensive half: every target profile owes a declaration in the same change

**Fact — silence about a dimension is `Unknown`, and `Unknown` never reaches an executable frontier.** [Numerical semantics](../../numerical-semantics.md#per-dimension-honourability-and-how-it-composes-with-feasibility) states that "a dimension the profile does not speak to at all, in the arithmetic type asked about, contributes `Unknown` in ADR 0043's exact sense — no admissible proof path — so it may appear in search and explain state and never in an executable frontier."

**Inference — this is the real cost of the addition and it is not optional.** A twelfth dimension added without a declaration on every profile makes *every* compilation on those profiles unexecutable, for a dimension nothing consumes. So an admission must land the declaration with the definition, and a declaration needs evidence. **This is what the compiler measurement in this record was run to supply.**

**Measurement, from [`probe.sh`](../../../spikes/numerics/elementary_identity_folding/README.md), retained as `record.tsv`.** Sixteen kernels across six flag sets on offline `metalfe-32023.921` (Xcode 27.0 build 27A5228h, MSL 4.0, target `air64-apple-macos26.0`): **no elementary identity was folded, under any flag set.** `exp(a) * exp(b)` kept two `exp` calls and a multiply where `exp(a + b)` had one call and an add; `log(a) + log(b)`, `sqrt(a) * sqrt(b)`, `exp(a) / exp(b)`, `1 / exp(a)`, `pow(x, 2.0f)`, `pow(x, 0.5f)`, and the softmax-shaped `exp(x - m1) * exp(m1 - m2)` all likewise kept the operations their source named. The mechanism is read in the IR rather than inferred: `exp(0.0f)` is not even constant-folded, because `air.exp.f32` is an opaque AIR intrinsic that LLVM's constant folder and identity combiners never match.

**The same run's positive controls fire**, so the negative is a reading rather than a silence: `x + x` becomes a multiply by two under every relaxing mode and stays an add under the governed set, and `x * y + x` becomes `llvm.fmuladd.f32` when `-ffp-contract=off` is dropped. And the perturbation that matters — respelling `exp_product`'s body as the folded form by hand — produces a row byte-identical to `exp_of_sum`'s, which is exactly what a compiler-performed fold would have looked like in the record.

**Inference — so a declaration is available for the offline path, on measured evidence, and for nothing else.** A profile could declare that it honours `elementary_identity: Forbidden` for `f32` on this offline compiler. It could not declare anything about the runtime path, and it must not: finding 30 of [the Apple GPU numerical behaviour record](../apple-targets/numerical-behaviour.md) measured the *runtime* compiler contracting a multiply/add pair whatever the offline selection said, and drew the general conclusion that "an offline contraction measurement is not transferable to the runtime path". Whether the runtime AIR-to-ISA stage folds an elementary identity is `Unknown`, and it is filed rather than assumed.

**Measurement boundary, stated because the toolchains differ.** This host's `xcrun metal` resolves Xcode 27.0 beta and a downloaded MetalToolchain reporting `metalfe-32023.921`, where the qualified Apple row in the numerical-behaviour record is Xcode 26.6 with offline `metalfe-32023.883`. **These rows are not comparable to that record's** and a profile declaration would have to be re-measured on the toolchain it claims.

### The coherence enumeration, re-run

**Inference — no combination involving the new dimension is self-contradictory, checked against the five eliminations [Numerical semantics](../../numerical-semantics.md#coherence-enumerated-rather-than-discovered) already enumerates.** In particular, *permitted elementary identity with forbidden reassociation* is coherent and is the combination the softmax caller would want: telescoping `exp(a) exp(b)` into `exp(a+b)` regroups no same-operation operand sequence, so it consumes no reassociation and a contract may grant one without the other. Under ADR 0011's rule that one permission never implies another, refusing the pair would re-couple two dimensions this record separates.

## Part 6 — Composition with the distributivity reassessment, without presuming it

[`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](../../../tickets/reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md) is Tom's and is open. This record neither presumes nor recommends its outcome. What it can do is state the composition exactly, so that whichever way that decision goes, this one is already derived.

### The two gates are independent and conjunctive

**Fact, from [the certified-bounds record's](certified-bounds-as-rewrite-permissions.md) Part 2.** The online-softmax rescaling fold consumes distributivity — the Horner nesting expands to a sum of products — **and** the exponential's functional equation, in the telescoping step `exp(x_j - m_j) * exp(m_j - m_V) = exp(x_j - m_V)`. Both are consumed by the same rewrite and neither implies the other.

**Inference — the four outcomes, and only one of them makes the rewrite reachable.**

| distributivity | elementary identity | the online-softmax fold |
| --- | --- | --- |
| declined (today) | unpermissioned (this record's outcome) | refused, naming **both** missing dimensions |
| declined | admitted | refused, naming the missing distributivity dimension |
| admitted | unpermissioned | refused, naming the missing elementary-identity dimension |
| admitted | admitted | reachable, subject to the bound and the six obligations |

### Why the outcome today is named-and-unpermissioned, and what would change it

**Inference — this dimension has no caller that is not already blocked, and the search for one was made rather than assumed.** ADR 0095's ground is that "the vocabulary carries no caller-less permissions". Applying it here requires asking whether any *identified* rewrite consumes this freedom **without** also consuming distributivity, since one that consumes both cannot be spent while ADR 0095 stands. The candidates, checked:

- **A pure elementwise fold, `exp(a) * exp(b) → exp(a + b)` over two `Exp` occurrences.** Not statable: `crates/tiler-ir/src/semantic/softmax.rs:6` records that the graph "admits none of a `Maximum` reduction, a general `Exp`, or a general `Divide` as a semantic key", so there is no occurrence to fold. The same disposes of `exp(a)/exp(b)` and `1/exp(a)`.
- **The log-sum-exp shift, `log(sum_j exp(x_j - m)) + m`.** Consumes distributivity too — the `exp(m)` factor multiplies through a sum — so it is blocked identically to the softmax caller, and no `Log` key is registered either.
- **`sqrt(a) * sqrt(b) → sqrt(a * b)`.** Elementwise and free of distributivity, so it would be a clean caller — but the pinned workload's only square-root use is the `rsqrt` inside `tiler::rms-norm-f32@1`, whose formula is pinned, and no chain of two square-root products exists anywhere in it.
- **The flash-class attention kernels.** [`derive-the-capability-set-for-search-discovered-flash-class-attention-kernels`](../../../tickets/derive-the-capability-set-for-search-discovered-flash-class-attention-kernels.md) is the standing owner, and every rescaling form it would reach is the online-softmax fold or a tree variant of it — the same conjunction.

**Inference — so the honest position is symmetric with ADR 0095's, and stating it that way is what keeps it from reading as an oversight.** A permission admitted here today would widen every contract, oblige every profile to answer for it, and step both key domains, in order to authorize a rewrite that a *different* accepted decision independently refuses. That is the caller-less permission ADR 0095 declined to admit, arrived at from the other side.

**Inference — and the converse is what the deferral's trigger has to be written on.** The moment the distributivity reassessment admits a permission, this dimension acquires a caller in the same instant, and the two decisions are better taken together than sequentially — because an admission of distributivity alone leaves the flash-class rewrite refused for a reason the reassessment's own question would have to disclose, which its ticket already requires it to say. So the trigger is *the distributivity reassessment resolving in the admitting direction*, **or** a workload whose natural spelling consumes an elementary identity without consuming distributivity, of which the square-root product is the shape to watch for.

## Part 7 — What a refusal says

[ADR 0080](../../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) item 5 requires a rewrite consuming distributivity to reject naming the missing distributivity dimension rather than reporting a forbidden reassociation, on the ground that a rejection naming reassociation "implies that a contract permitting reassociation would admit the rewrite. That inference is false". [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) observed that a decline makes the wording *more* load-bearing, "because it is now the only thing that tells a caller the freedom is withheld by decision rather than absent by oversight". Both apply here unchanged, and this record adds one requirement they do not state.

**Proposal — the refusal names the elementary-identity dimension, the function, and the identity.** Naming the dimension alone is insufficient for this dimension in a way it is not for distributivity, because the dimension is one and the identities are many: a caller told only that "an elementary-function identity is not permitted" cannot tell which of its operations is implicated, where distributivity has one identity and therefore one implication.

**Proposal — a rewrite consuming two missing dimensions names both, and this is the new requirement.** ADR 0080's rule is written for one missing dimension. The online-softmax fold consumes two, and naming only one is *worse than naming neither*, because it implies that granting the named one would suffice — the exact false inference ADR 0080 item 5 exists to prevent, reproduced by a rule that was written before a two-dimension rewrite existed. The refusal enumerates every missing dimension the rewrite consumes.

**Proposal — the wording for the worked case**, so the requirement is checkable rather than described:

```text
rewrite `online-softmax-rescaling-fold` is not admitted: it consumes two
numerical dimensions no permission grants.
  - distributivity: exchanging a product of a sum for a sum of products
    (ADR 0080 defines the dimension; ADR 0095 declines a permission for it)
  - elementary identity: rewriting through the functional equation
    exp(a) * exp(b) = exp(a + b) of tiler::softmax-f32@1's subordinate
    exponential (the dimension is defined and no permission grants it)
neither is a forbidden reassociation, and no contract permitting
reassociation, permutation, or contraction admits this rewrite.
```

**Inference — this wording is specified whether or not any permission is ever admitted**, which is what the ticket requires as its minimum outcome. Under a decline it is the only thing distinguishing a decision from an omission; under an admission it is what a refused *quantitative* check reports beside the stated tolerance and the derived price.

## Part 8 — The drafted vocabulary proposal, not applied

**Nothing in this part is proposed for self-acceptance.** [Numerical semantics](../../numerical-semantics.md) is the normative owner of the dimension's definition, `docs/decisions/` owns the record, and both were outside the producing ticket's scopes. Each item is a public boundary under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md).

### The additive shape

**Proposal.** A twelfth field in the canonical dimension vector, `elementary_identity: NumericalPermission`, taking the two general resolutions every declared dimension takes; a dimension key `numerics.elementary-identity` in `CANONICAL_DIMENSIONS`; and a capability layer in which an operation declares the functional equation its subordinate elementary evaluation satisfies together with that equation's real-domain side condition. A contract that resolves it `Forbidden` — which is every contract written before it exists — behaves exactly as today.

**What an admission would additionally require, and none of it is drafted here:** the key domain step at both widths with the two pinned literals recomputed; a declaration on every target profile for every arithmetic subject it speaks about, on evidence of that profile's own toolchain; the widened exhaustive injectivity check; and the sixth admission obligation of Part 4.

### Drafted ADR body, landed as ADR 0101 and retained here as provenance

**This span is no longer a draft, and since 2026-08-06 it is an accepted decision.** It landed on 2026-08-05 as [ADR 0101](../../decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md), `decision_status: proposed`, accepted by Tom on 2026-08-06, and the decision record — not this section — is the authority a reader should cite. It was written verbatim-landable so the transfer could be byte-identical, following the convention [the subgroup execution tier](../scheduling/subgroup-execution-tier.md) and [the two-dimensional cooperative staging relation](../scheduling/two-dimensional-cooperative-staging-relation.md) both record: a transfer that edits is a fork, and byte-identity is what makes "unreworded at acceptance" checkable rather than asserted. At landing, `diff` between this span with `#### ` mapped to `## ` and the ADR's `**Status:**`-through-alternatives range reported **no differences** and `cmp` reported the two byte-identical, while the raw `diff` reported **exactly eight changed lines, four heading pairs**, because this record nests its headings two levels under its own. That was the one deliberate deviation and it is a nesting level, not content. The check was proved able to fail before being believed: substituting "it is a fifth one" for "it is a fourth one" in decision 1 made the normalized `diff` report the differing line.

**The `**Status:**` paragraph transfers as body and the two directive lines above it do not**, which is the one structural difference from the sibling records and is stated so the byte range is reproducible. `**Title:**` supplies the ADR's H1 and its frontmatter `title`, and `**Frontmatter:**` supplies the remaining frontmatter fields; neither line appears in any landed ADR, checked with `grep -ln '^\*\*Title:\*\*' docs/decisions/*.md`, which returns nothing. The transferred range is therefore `**Status:**` through the last alternatives-considered paragraph.

**The `**Frontmatter:**` line below is true as landed, and it is the line that will go stale first.** It reads `decision_status: proposed` and ADR 0101 reads `proposed`, so unlike the ADR 0094 landing there is nothing to flag today. **If Tom accepts the record, that line becomes false and must be flagged here rather than edited**, because an edit inside the span destroys the identity the transfer claim rests on — the drafted body is provenance for what was transferred, not a second statement of current fact.

**Flagged, 2026-08-06 — that acceptance happened and the span's frontmatter line is now stale exactly as predicted.** ADR 0101 reads `decision_status: accepted`; the `proposed` in the span below is the value as drafted and transferred, kept byte-identical as provenance. Read the ADR for current status, never this span.

**The span is the content between the two horizontal rules below**, excluding the blank line on each side, beginning at `**Title:**` and ending at the last alternatives-considered paragraph. A reader **re-derives the line numbers rather than trusting any stated here**, because every edit above the span moves them — including the paragraphs just added: `grep -n '^---$'` gives the rule positions in this file, and the span is the first plus two through the second minus two, counting only the rules inside this section.

**The span carries no traceability section and therefore no relative links at all**, which avoids the tension AGENTS.md records for drafted bodies: a traceability section written with `docs/decisions/`-relative paths resolves at the ADR's destination and not from here, so this record would otherwise have to state that beside the span, and repointing would break the identity. Checked rather than assumed, and the check was proved able to answer a nonzero before its zero was believed: at the time of writing the span was at `289,337` and `sed -n '289,337p' docs/research/numerics/elementary-identity-rewrite-dimension.md | grep -c ']('` returned `0` while `sed -n '1,100p' … | grep -c ']('` returned `12` and the whole-file count returned `40`. **The carrier re-derived all three at landing and every one reproduced**: the paragraphs added above moved the span to `293,341`, where `sed -n '293,341p' … | grep -c ']('` returns `0` against an unchanged whole-file `40` — a line count, not an occurrence count, which is why adding a link to an existing paragraph did not move it. A reader re-derives the range before re-running it, and treats a nonzero span count as a defect to repair rather than a number to restate. Cross-references the span needs are made by ADR number and by contract name in prose, which resolve from either location. The traceability, normative-owner, work-record, and open-questions sections were written fresh at the destination, as new text authored there rather than as an edit to the transferred span.

**The number was taken by reading the directory rather than by remembering one, and this time it did not move.** `0100` was the highest ADR present when this record was written, so `0101` was drafted below; `ls docs/decisions/01*.md` at the carrier's base `de377fb1` returned `0100` alone, so `0101` was still free and is the number taken. Nothing in the span depended on it either way, because the span's H1 is supplied by the `**Title:**` line rather than written into the body — which is the property that made the warning cheap to honour rather than a number to reconcile.

**The scope split that makes a carrier ticket necessary is read from the config rather than asserted.** `ticketsplease.toml` routes `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md` to `contracts/navigation`, and the normative definition would additionally touch `docs/numerical-semantics.md` under `contracts/numerics`. This record's ticket holds `research/numerics` and `contracts/navigation` with shared `project/tickets`, and neither of the other two. [`carry-the-elementary-identity-dimension-adr`](../../../tickets/carry-the-elementary-identity-dimension-adr.md) takes them and carries the `docs/decisions/README.md` catalog row.

---

**Title:** Treat elementary-function identities as a fourth numerical dimension

**Frontmatter:** `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "numerical-operations"`, `topics: ["numerics", "transcendentals", "accuracy", "optimizer"]`, `applies_to: ["tiler.contract.numerical-semantics"]`, `evidence: ["tiler.research.numerics.elementary-identity-rewrite-dimension", "tiler.research.numerics.certified-bounds-as-rewrite-permissions"]`, `depends_on: ["ADR-0011", "ADR-0014", "ADR-0015", "ADR-0042", "ADR-0080"]`, `ticket: "carry-the-elementary-identity-dimension-adr"`.

**Status:** proposed. Every clause below is derived from the numerical contract's own definitions, from ADRs 0011, 0014, 0015, 0042, and 0080, and from a measurement recorded in the evidence. The one product choice in the neighbourhood — whether to admit a permission for the dimension — is explicitly reserved by item 5 and owned elsewhere. This record supersedes nothing: no accepted decision claims the dimension set is exhaustive, so adding a fourth contradicts none of them.

#### Context

**Fact — a rewrite Tiler wants consumes a freedom the vocabulary does not name.** The certified rounding-error bounds research record derived the online-softmax rescaling fold's worst-case price and found the fold equal to the two-pass fold only by telescoping `exp(x_j - m_j) * exp(m_j - m_V) = exp(x_j - m_V)`. That step is a functional equation of the exponential rather than an algebraic identity of the ring, and in floating point it is false.

**Fact — the divergence survives the strongest accuracy contract statable.** With `exp` correctly rounded to binary32, `exp(-1.0)` is `0x3ebc5ab2`, its square rounds to `0x3e0a9556`, and `exp(-2.0)` is `0x3e0a9555`. Over the non-positive integer grid `[-40, 0]` in both arguments — the region the governed softmax's exponential admits — 502 of 1681 pairs disagree. Every individual evaluation is exact to the last bit on both sides.

**Fact — no declared dimension reaches it, checked against the list.** `CANONICAL_DIMENSIONS` declares eleven, and Numerical semantics names distributivity as a twelfth that no permission grants. The subnormal, signed-zero, exceptional-value, and materialization dimensions are about which values arithmetic sees or produces. Reassociation and permutation are defined over a fixed contributor sequence, and this rewrite leaves one leaf where there were two. Contraction governs an existing multiply-add pattern. Distributivity governs which products the ring operations form. `ReciprocalTransform` governs one field identity over ring operations. `ApproximateIntrinsics` governs which realization evaluates one declared elementary evaluation and says nothing about how many there are.

#### Decision

1. **Elementary-function identity is a numerical dimension, and it is a fourth one.** Tiler's numerical vocabulary recognizes it alongside reassociation, operand permutation, and distributivity. It authorizes replacing one composition of evaluations of a registered elementary function with another that is equal to it over the reals by a functional equation of that function, where the two differ in the number of evaluations, in the arguments the evaluations receive, or in both. **Why it cannot be folded into any of the other three:** all three are statements about ring operations over a contributor sequence, and this one rewrites *through* the function, so a permission over `+` and `*` has nothing to say about it — the same "there is nothing for it to be a statement about" argument ADR 0080 made for distributivity, applied to a different set.

2. **It is additional to the other three, not a substitute for any.** The online-softmax rescaling fold consumes **both** distributivity and elementary-function identity. Naming only one would be as incomplete as naming only reassociation is for a contraction-chain regroup.

3. **It is one dimension, with per-function content carried at two other layers.** Consuming an elementary-identity rewrite requires three independent facts: the operation declares the functional equation it satisfies, with that equation's real-domain side condition, as an operation-owned identity-encoded capability; the contract resolves one permission for the dimension; and the rule carries its own derived bound and discharges the equation's side condition under ADR 0021's provenance classes. **Why one rather than one per function or one per identity:** ADR 0014 requires evidence of a capability asymmetry before a dimension is split, and the two asymmetries that exist between the exponential's, the logarithm's, and the square root's identities are not that. The real-domain side condition — the logarithm's product rule needs positive operands where the exponential's needs nothing — is a value-domain fact ADR 0021 already governs, and encoding it as a second permission would put an obligation in a vocabulary with no way to check it. The floating-point error asymmetry — the logarithm's product rule carries a cancellation hazard the exponential's does not — is a difference between two rules' derived bounds, and the certified-bounds record already places a rewrite's rounding cost on the rule. Splitting the permission would copy into the contract what the capability and the bound already carry, where the copies could disagree. **Why the directions are not cut apart:** nothing establishes that folding and splitting a functional equation are separately capable operations, and over the reals the equation is symmetric; ADR 0095 refused exactly this cut for distributivity on the ground that a directional admission without evidence is "an arbitrary cut presented as caution".

4. **It is resolved for the program, like every other dimension.** ADR 0011 makes the program ceiling the outer authority so that no local default can exceed what the caller authorized, and no dimension is resolved per operation today. The per-operation content is the *capability* — which equation a family satisfies — which is an operation property, is declared by the operation, and enters operation identity.

5. **No elementary-identity permission is admitted, and admitting one is reserved.** The dimension is defined; no permission grants it. Whether to admit one is a product choice that does not follow from any definition above. Its trigger is the distributivity reassessment resolving in the admitting direction, or a workload whose natural spelling consumes an elementary identity without also consuming distributivity. **Why the reservation rather than an admission:** the only identified caller is the online-softmax rescaling fold, which independently consumes distributivity, for which ADR 0095 declines a permission. A permission admitted here today would widen every contract, oblige every target profile to answer for a dimension, and step the contract-key domain at both widths, in order to authorize a rewrite that a separate accepted decision independently refuses. That is precisely the caller-less permission ADR 0095 declined to admit.

6. **A rewrite consuming this dimension rejects, and the rejection names every missing dimension.** Such a rewrite is rejected under every contract Tiler can express, and the rejection names the elementary-identity dimension, the function, and the identity — the dimension alone is insufficient because the dimension is one and the identities are many. **A rewrite consuming more than one missing dimension names all of them.** ADR 0080 item 5's rule was written for a rewrite consuming one; naming only one of two is worse than naming neither, because it implies that granting the named one would suffice, which is the same false inference that rule exists to prevent.

7. **The accuracy contract is a separate authority and cannot stand in for this dimension.** ADR 0042's contract bounds one evaluation against one reference result, and the divergence above holds with every evaluation correctly rounded on both sides, so a pointwise contract cannot bound it. A rewrite additionally has to prove that the rewritten program's elementary arguments still lie inside every accuracy clause's declared domain, because a clause's domain is derived from the pinned evaluation order while its declared fields do not move when the order does.

#### Consequences

- The dimension is defined while its permission is withheld, which is a vocabulary reservation rather than implemented support. `implementation_status` is `not-started` and that is the honest value: no field, variant, or capability names elementary-function identity anywhere in `crates/`, and the only thing the tree contains is the *absence* the rejection is required to name.
- Admitting a permission later is additive. A composition starts at the strict resolution of every dimension, so a dimension added later arrives `Forbidden` in every contract written before it existed and no registered contract silently becomes able to perform the rewrite.
- Admitting a permission is an identity-domain step and is not free. The contract key is the canonical injective encoding of the dimension vector, so `tiler.contract.f32.v2` and `tiler.contract.bf16.v1` both step, the two pinned rendered key literals in `crates/tiler-ir/src/schedule/numerics.rs` both move, and the exhaustive injectivity check doubles its space.
- Admitting a permission obliges every target profile to declare for it in the same change. Silence about a dimension is `Unknown`, which never reaches an executable frontier, so a dimension added without declarations would make every compilation on those profiles unexecutable for a freedom nothing consumes.
- The flash-class one-pass softmax remains illegal rather than unimplemented, and now for two named reasons rather than one named and one unnamed.

#### Alternatives considered

**Read `ApproximateIntrinsics` as covering the rewrite.** This is the reading a reader arrives at from the words "elementary" and "approximate" alone, and it is the most damaging one available. Rejected on the envelope's own definition: `ApproximationEnvelope` has two resolutions, both of which govern which realization evaluates one declared elementary evaluation. Neither says how many evaluations there are. The divergence holds with the envelope resolved `Forbidden`, and after a rewrite each surviving evaluation is still governed by whatever envelope the contract resolved — so the rewrite's error composes *with* the envelope rather than being bounded by it. Accepting this reading would turn a stated tolerance into a claim about a composition it was never derived over.

**Extend the distributivity dimension to cover it.** Attractive because the one identified caller consumes both. Rejected because a rewrite consuming two freedoms needs two names, or its refusal cannot say which is missing — and because a caller might one day want the elementary identity without the distributive exchange, which a merged dimension makes unstatable.

**Split the dimension per function or per identity.** Rejected under ADR 0014's standard in item 3.

**Define the dimension and admit a permission for it in the same record.** Rejected because admitting is a product choice with no derivation behind it while the only caller is jointly blocked, and because bundling would put a preference inside a record whose whole authority is that it contains none. This is ADR 0080's own reasoning about its item 4, and the parallel is deliberate.

**Leave the freedom unnamed and let the softmax rewrite be refused for its distributivity alone.** The status quo, and it is the worst option rather than the neutral one: the refusal would be *correct* today and would become silently wrong the moment a distributivity permission was admitted, at which point a rewrite consuming an unnamed freedom would pass a check that named every freedom it knew about.

---

## Part 9 — A defect found on the way, stated where a reader will meet it

**Fact, read in full at `crates/tiler-ir/src/semantic/softmax.rs`.** The module header states that "The online single-pass form is a reassociation, which is a legality question and not a cost one. Rescaling a running sum whenever the maximum changes regroups the contributor sequence of the *sum*, so it is legal exactly where reassociation is granted." `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM`'s doc comment says the same, and the registered fact value is the string `a-reassociation-of-the-sum-and-not-a-free-implementation-choice`.

**Inference — the claim is refuted by [the certified-bounds record](certified-bounds-as-rewrite-permissions.md), which landed after it.** The online fold is a Horner nesting, not a re-parenthesized sum: its contributors are `exp(x_j - m_j) * prod_{k>j} exp(m_{k-1} - m_k)`, which share no floating-point value with the two-pass fold's `exp(x_j - m_V)`. No reassociation permission reaches it. The fold consumes distributivity, for which ADR 0095 declines a permission, and the elementary identity this record names.

**Inference — the direction of the error is the dangerous one.** The doc comment's own stated purpose is that the fact is stated "so that a scheduler reaching for it has to consume the permission". A scheduler that reads it consumes *reassociation* and believes itself legal, under a registered contract that permits reassociation — which is exactly the false inference [ADR 0080](../../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) item 5 exists to prevent, present in the tree, in identity-carrying data.

**Fact — correcting it is an identity-domain step.** `encode_operation_definition` (`crates/tiler-ir/src/semantic/registry.rs:2811`) writes `definition.canonical_facts().value()` into the definition projection, which is `tiler.semantic-definition-projection.v5`, which feeds the registry snapshot identity that the compiler's explain request qualifier pins. Changing the fact string moves all of it, so the correction is executed completely or not at all — which is why it is filed as its own ticket in `implementation/ir` rather than described here.

## Open axes, each with a filed destination

- **Whether to admit a permission for the dimension.** → [`decide-whether-to-admit-an-elementary-identity-permission`](../../../tickets/decide-whether-to-admit-an-elementary-identity-permission.md), filed `deferred` with its trigger and a trigger check log, because its trigger is another decision's outcome and the board must not offer non-work.
- **The drafted ADR body has been carried and accepted.** → [`carry-the-elementary-identity-dimension-adr`](../../../tickets/carry-the-elementary-identity-dimension-adr.md) transferred it byte-identically on 2026-08-05 as [ADR 0101](../../decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md); the coordinator filed [`accept-adr-0101-elementary-identity-dimension`](../../../tickets/accept-adr-0101-elementary-identity-dimension.md) and Tom accepted on 2026-08-06. This axis is closed.
- **A registered fact says the online single-pass softmax form is a reassociation, and the certified-bounds derivation refutes it.** → [`correct-the-online-single-pass-softmax-fold-legality-fact`](../../../tickets/correct-the-online-single-pass-softmax-fold-legality-fact.md), which Part 9 states.
- **Whether the Metal runtime compiler folds an elementary identity, which this record's offline measurement cannot reach.** → [`measure-whether-the-metal-runtime-compiler-folds-an-elementary-identity`](../../../tickets/measure-whether-the-metal-runtime-compiler-folds-an-elementary-identity.md), filed `deferred` with its trigger.
- **The numeric `eps_exp` a parametric bound needs is not retrievable from the authority that owns it.** → [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](../../../tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md), already filed by the certified-bounds record. This record makes it a prerequisite of a *quantitative* admission and not of the dimension's definition.
- **Whether the distributivity decline survives its new caller.** → [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](../../../tickets/reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md), already filed. Tom's, and nothing here presumes it.

**One thing deliberately not filed, so the omission is reasoned rather than missed.** [Q-SEM-002](../../open-questions.md) asks for a "complete operation/dtype/signature reassociation and commutativity matrix with verifier tests", and this record proposes a third, parameterized capability law. No sentence of Q-SEM-002 is added, because its closure condition is stated over the two *accepted* laws and a proposal no decision admits must not enter an index of commitments. If the drafted ADR is accepted, widening Q-SEM-002 is part of that acceptance's own sweep.

## What this record does not establish

- **No contract changed and no decision was made.** No dimension was added, no permission was admitted, no ADR was accepted, and no crate changed. `implementation_status` is `not-started` and that is the honest value.
- **The counterexample survey is a bounded observation of a 1681-pair integer grid** with a correctly rounded `exp` no real target provides. It establishes that the identity is observably violated in binary32 on that grid; it is not a bound on the violation, and it does not separate the identity's error from a one-ulp implementation error.
- **The compiler measurement is one offline compiler, six flag sets, sixteen kernels, and no device.** It says nothing about the runtime AIR-to-ISA stage, nothing about any other compiler build, and nothing about what any emitted intrinsic returns. It is on a *different* Xcode and offline toolchain from the qualified Apple row, so it is not a row of that record's table.
- **The three-layer proposal is drafted and untested.** A capability that declares a parameterized functional equation does not exist, no spelling of it has been compiled, and the drafted ADR is `proposed` with no carrier having run.
- **Nothing here establishes that any elementary-identity rewrite is profitable.** It establishes what one costs numerically and what naming it costs the vocabulary. Whether a one-pass fold is faster on any target is a cost-model and scheduling question this record does not touch.
