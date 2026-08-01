---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.bf16-computation-accumulator-and-conversion"
kind: "research"
topics: ["numerics", "dtypes", "bf16", "conversion", "accumulator", "contraction"]
title: "BF16 computation, accumulator, and conversion"
catalog_group: "dtypes-quantization"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["executable-model", "exhaustive-finite", "primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.numerical-semantics"]
depends_on: ["tiler.spike.numerics.bf16-second-dtype", "tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.dtype-resolution-precedents", "tiler.research.numerics.mature-dtype-taxonomy"]
ticket: "design-the-bf16-computation-and-accumulator-contract"
---

# BF16 computation, accumulator, and conversion

**Status:** derivation complete; the surviving design is a **Proposal** carried below as a drafted ADR body that this record's scope cannot land. Nothing here registers an operation, moves a dtype-support cell, or authorizes implementation.

## What this record decides and what it deliberately does not

The landed pure-BF16 vertical states computation, accumulator, intermediate-materialization, and result types as four separate fields and sets all four to `tiler::bf16@1`, so that a wider accumulator is later an explicit edit rather than the removal of an assumption nothing wrote down. This record is the edit's justification: it decides **where** a wider accumulator lives, **whether** a mixed-width BF16 program needs a graph-level conversion, **what** a BF16/F32 conversion means in each direction, **whether** a fused BF16 operation is admissible at all, and **what** a rejected combination returns.

It decides none of the following, and each absence is deliberate: no BF16 operation key is proposed for registration here; no reference evaluator, physical carrier, lowering, or target row is designed; no F16, F64, OCP, or MX conclusion is drawn, because every claim below rests on a numeric relationship between BF16's parameters and binary32's that no other pair reproduces.

## Traceability

- **Evidence:** [the BF16 second-dtype spike](../../../spikes/numerics/bf16-second-dtype/README.md), whose six promotion stages and three new perturbations were added by this ticket and run under the spike's own README invocation; [the Apple numerical behaviour record](../apple-targets/numerical-behaviour.md), findings 24, 25, 26, 28, 29, and 30, transcribed with their boundaries; [dtype resolution and mixed-precision precedent](dtype-resolution-precedents.md).
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md).
- **Accepted authorities this record preserves:** [ADR 0009](../../decisions/0009-resolved-numerical-typing.md) (resolved numerical typing; internal mixed precision belongs to an operation's signature), [ADR 0010](../../decisions/0010-typed-conversion-contracts.md) (conversion is a typed contract carrying only its family's fields), [ADR 0015](../../decisions/0015-fma-vs-contraction.md) (required FMA and optional contraction are different contracts), [ADR 0041](../../decisions/0041-separate-float-to-integer-conversion-families.md) (separate conversion families rather than one optional-field bag), [ADR 0026](../../decisions/0026-dtype-representability-vs-operation-support.md) (representability is not operation support), [ADR 0043](../../decisions/0043-use-typed-phased-target-feasibility.md) (`Unknown` fails closed), [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) (honourability is keyed by `(subject, dimension)`), [ADR 0087](../../decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) item 5 (a contraction's numerical signature is stated once, generically).
- **Work record:** [design-the-bf16-computation-and-accumulator-contract](../../../tickets/design-the-bf16-computation-and-accumulator-contract.md).

## The evidence, stated before it is used

### Derived on the host, portable, and exact

**Measurement — the spike run.** `spikes/numerics/bf16-second-dtype`, base commit `59a2fe2`, `rust-toolchain.toml`'s pinned nightly, release profile, 3.8 s, Apple M-series arm64 macOS. Every stage agreed and all ten perturbations were detected. The six new stages returned:

| # | Claim | Population | Result |
| --- | --- | --- | --- |
| 1 | The generic two-format rounder agrees with the trusted BF16 one | 196,608 exact values | 0 disagreements |
| 2 | One multiply or add admits a promoted binary32 route | 524,288 cases: all 65,536 encodings against 4 named partners, both operations | **0 disagreements** |
| 3 | A fused multiply-add does not | 262,144 triples: all 65,536 second operands against 4 named addends | **21,546 disagreements**; witness `0x3fc0 * 0x3fb2 + 0xb300` gives `0x4005` exactly and `0x4006` promoted |
| 4 | The accumulator's width is observable | one derived witness plus its control | `0x3f80` + 4×`0x3b00` gives **`0x3f80`** at a BF16 accumulator and **`0x3f81`** at a binary32 one; at one contributor both give `0x3f80` |
| 5 | Widening is exact, total, and subnormal-class-preserving | all 65,536 encodings | 0 value disagreements; 254 of 254 subnormals widen to binary32 subnormals |
| 6 | Narrowing has three separable decisions and a fourth forced one | 65,536 binary32 patterns covering all of `[1, 2)` | nearest-even ≠ truncation in 32,704; ≠ ties-away in 64; `0x7f7fffff` narrows to `+∞`; `0x7f800001` *truncates* to `0x7f80` |

**Fact — this evidence class.** Every number above is exact rational arithmetic compared against exact rational arithmetic. It is `executable-model` and, where a population is a whole format, `exhaustive-finite`. It is **not** a measurement of any device, and it would be byte-identical on any host and any toolchain. Each population is named and counted, so a stage that silently stopped running would report a changed count rather than a uniform pass; the three perturbations exist because a stage whose only possible answer is "0 disagreements" is not a check.

### Transcribed from the retained Apple record, host-bound

Each of these is a **Measurement** on one row and is quoted with the boundary the record itself states. None is a portable claim about Apple GPUs, Metal, or the `bfloat` type.

- **Finding 29 — no `bfloat` FMA exists, so the question is not expressible at the source level.** "MSL provides no `bfloat` overload of `fma`; the call promotes and `metal` rejects `bfloat v6 = fma(v3, v4, v5)` with 'cannot initialize a variable of type "bfloat" with an rvalue of type "float"'. … Writing `bfloat(fma(...))` would compile and would measure a fusion at `f32` precision narrowed afterwards — a double rounding this format never performs — so the question is not expressible at that width rather than answered negatively at it." **Boundary:** offline `metalfe-32023.883` resolved from Xcode 26.6 build 17F113 on macOS 27.0 build 26A5388g; a compile failure, not a dispatch.
- **Finding 28 — contraction is observable at BF16, and the strictest offline cell differs by dtype.** `contraction_pair_bf16` at operand `0x3eab` with scale `0x3fbe` and bias `1.0` "returns `3fc0` under `off` and `on`, and `3fbf` under `fast` in `relaxed` and `fast`". And: "Under `safe` with `-ffp-contract=fast`, `f16` fuses (`3e01`) and `bf16` does **not** (`3fc0`). … Nothing here explains why, and it is recorded as a measured cell rather than a mechanism." **Boundary:** that host row, that offline toolchain, `-O2`, macOS and iOS-Simulator families only.
- **Finding 30 — an offline contraction measurement does not transfer to the runtime path.** "Runtime-compiled … `contraction_pair_bf16` returns `3fbf` and `3fc0` [under `relaxed`/`fast` and under `safe`] respectively. All three widths agree, on `MacOs` and `IOsSimulator` alike." And: "A profile that declared contraction honourability from an offline `-ffp-contract=off` compilation would be wrong about every runtime-compiled kernel under `relaxed` or `fast`." **Boundary:** runtime compilers `metalfe-32023.921` (macOS) and `metalfe-32023.830.1` (iOS Simulator); `MTLCompileOptions` exposes no contraction property at all (finding 10).
- **Findings 24 and 25 — the subnormal flush follows the exponent field.** `bf16` arithmetic flushes on the qualified row where `f16` preserves, and the surviving explanation is that narrow arithmetic is evaluated at `f32` precision and rounded once, under which a BF16 subnormal is an `f32` subnormal and is destroyed while an `f16` subnormal is an `f32` normal and is not. Finding 25 explicitly refuses to be read as a rule for a fourth format. **Boundary:** `bf16` was dispatched on **macOS only** — finding 26 records the iOS Simulator refusing every `bfloat` pipeline with `XPC_ERROR_CONNECTION_INTERRUPTED`, including for an arithmetic-free `materialize_bf16`, so `bf16` is `Unknown` for both iOS families.

**Fact — the one place these two evidence classes meet, and it is a cross-check rather than a merge.** Finding 28's two measured device values are exactly what the host-side exact model produces for the two contracts: the exact product `0x3eab × 0x3fbe = 0.4957275390625` rounded to BF16 is `0x3efe`, and `0x3efe + 1.0` rounds to the tie at quanta 191.5 which ties-to-even sends up to `0x3fc0`; the unrounded `0.4957275390625 + 1.0` rounds to `0x3fbf`. So the device's `3fc0`/`3fbf` pair is a separate-rounding/single-rounding pair and nothing else. That agreement is evidence that the model states the same semantics the device distinguishes; it is **not** evidence that the device implements the model, and no claim below rests on it.

## The concrete thing being widened

**Fact.** `crates/tiler-ir/src/semantic/bf16.rs` registers `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and `tiler::add-bf16@1`. The two arithmetic keys share one fact record whose sixteen fields include four separate type fields — `BF16_FACT_COMPUTATION_TYPE`, `BF16_FACT_ACCUMULATOR_TYPE`, `BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE`, `BF16_FACT_RESULT_TYPE`, all `tiler::bf16@1` — and five permission fields all `false`: mixed precision, implicit promotion, ADR 0015 contraction, fused multiply-add, reassociation. Mixed precision and implicit promotion are additionally refused *by name at application*, under the codes `bf16.binary.mixed-precision` and `bf16.binary.implicit-promotion`. No algebraic capability is declared, so no rewrite can consume one, and the module's own doc states that a missing capability reads as unknown rather than as the inverse law.

**Fact — the blocking seam below it is unmoved.** `ScalarArithmetic` in `crates/tiler-compiler/src/target.rs` exposes one public constructor, `f32()`, so no BF16 numerical row can be stated on a compiler target profile at all, and the twenty-four `declare_*` honourability methods are unreachable for BF16. Everything this record proposes is downstream of `admit-a-bf16-scalar-arithmetic-subject` closing that seam.

**Inference — the accumulator field on the landed family is structurally present and semantically degenerate, which is why this ticket was separate.** A binary multiply or add performs no fold, so "the type each accumulation step is performed at" has exactly one accumulation step and it is the operation itself. Setting that field to `tiler::f32@1` on `tiler::add-bf16@1` would describe an operation that rounds its single result to binary32 and then reports a BF16 result — which is either a contradiction or a second rounding nothing asked for. **The accumulator question therefore does not become real until a reducing BF16 operation exists, and none does.** That is a boundary on what any answer here can be about, and it is stated first because it eliminates a whole family of otherwise plausible edits.

## Question 1 — is a wider accumulator a property of the operation, the contract, or the schedule?

**Answer (Proposal): a property of the operation, carried in its registered definition facts, and therefore a distinct registered key per `(value type, accumulator type)` pair.** The contraction family is the precedent and ADR 0087 item 5 is the accepted rule.

The elimination, candidate by candidate.

**Eliminated — the schedule, on correctness.** A schedule property is one a planner may choose among on cost. Stage 4 shows two folds of the same contributor sequence, in the same order, differing only in the type each partial is held at, returning `0x3f80` and `0x3f81`. If the accumulator were a schedule property, the planner could return either, so a program's result would depend on a cost estimate. [Numerical semantics](../../numerical-semantics.md) forbids exactly this in two independent sentences — "Cost-based selection ranks implementations of one contract and may never rank contracts against each other, because doing so prices meaning", and "F16 or BF16 inputs do not imply low-precision accumulation; promotion is explicit" — and ADR 0009 adds "backend defaults never silently widen the program's numerical permissions". This is not a preference: two plans with different accumulators compute different functions, so they are not two plans of one program.

**Eliminated — the numerical contract, on a structural impossibility rather than on taste.** The resolved scalar-arithmetic contract "speaks for exactly one `ArithmeticType`", and target honourability is keyed by `(subject, dimension)` where a subject is an arithmetic type paired with the complete resolved value type it computes in. An operation storing BF16 and accumulating in binary32 uses **two** arithmetic types, so no single contract subject can speak for it, and no single honourability query can be asked about it. Putting the accumulator in the contract would therefore require either collapsing the subject key — which finding 24's measured `f32`-flushes/`f16`-preserves/`bf16`-flushes split makes provably wrong — or admitting a contract that names two subjects, which is a much larger redesign than the question warrants. A second, independent reason: every dimension the contract carries is a *permission* or a behaviour a profile can be asked whether it honours. A permission never changes what an operation means, only which realizations are legal. An accumulator width changes the meaning, as stage 4 shows.
  *How to refute this elimination:* exhibit a formulation in which the accumulator is a contract dimension and (a) some target profile can be asked whether it "honours" a binary32 accumulator under BF16 storage in a way that is not simply the binary32 subject's own row, and (b) two programs identical but for the caller's stated contract are not thereby two different functions. Both would have to hold.

**Eliminated — a per-node typed attribute under one key, on what the facts are for.** ADR 0087 put the *index structure* in a per-node attribute, and the shape is superficially available here. It fails on a specific asymmetry the ADR itself states: the structure attribute is admitted precisely because the numerical signature is stated "once, generically, parameterized by the structure" — the signature does not vary with it. The accumulator *is* the signature. Under a per-node accumulator the definition's fact record could no longer state an accumulator type at all; it could only point at an attribute, and a consumer that reads the facts to decide what it can realize would get a pointer instead of a fact. Worse, the reference evaluator is registered per key, so one evaluator would have to branch on the attribute — which is exactly the failure the BF16 module rejects by name when it refuses to widen `tiler::multiply-f32@1`: "widening … would have made one key mean two arithmetics whose roundings differ".
  *The cost this elimination accepts, stated rather than hidden:* key count grows as the useful `(value, accumulator)` pairs do. ADR 0087 rejected fixed keys partly on unbounded growth, and that concern is real for index structures, which are unbounded. Accumulator pairs are drawn from the governed catalog's 27 nominal scalars and, in practice, from a handful — this record proposes exactly one new pair and no more.
  *How to refute:* show a workload needing enough distinct accumulator pairs under one operation that the key set becomes the dominant cost, **and** a formulation in which the per-key facts stay complete.

**Survives — the operation's registered definition facts.** `CONTRACTION_F32_FACT_ACCUMULATOR_TYPE` already carries this exact role on `tiler::strict-tensor-contraction-f32@1`, beside a computation-precision field, a result-type field, and a conversion field that currently reads `none-operands-products-accumulator-and-result-are-binary32`. A BF16-storage, binary32-accumulator contraction is the same record with three of those four fields moved and the conversion field stating what it now is. The facts enter the operation's identity, so the registry snapshot moves when they move — which is the property the landed BF16 module's doc comment names as the reason the four fields exist separately.

## Question 2 — explicit conversion at each boundary, or an accumulator internal to one operation?

**Answer (Proposal): an *instance* of the no-implicit-promotion rule, not an exception — and for BF16/F32 specifically the two spellings are bit-identical, so the internal accumulator buys no expressiveness at a *pointwise* boundary and is needed only inside a reduction.**

**Fact — ADR 0009 already decided the general shape, in its alternatives-considered.** "Requiring graph-level casts for every scalar step inside a reduction would be explicit but unnecessarily encode an operation's internal scalar iteration in the public tensor graph." The precedent review it adopts says the same: "Widening inside a reduction or contraction can be intrinsic to that operation's explicit numerical signature. It does not require a graph-level cast for every scalar iteration." So an internal accumulator is not implicit promotion, because the promotion is stated in the signature, and *stated in the signature* is the whole content of "explicit". The `Cast and convert` row's rule — "a mixed-dtype program cannot be expressed without an explicit conversion operation and no implicit promotion exists after semantic admission" — governs conversions between two *values* produced by two operations. An operation's internal roles are not two values.

**Fact — for BF16 through binary32 the two spellings agree bit for bit, and the reason is two separately checkable inequalities.** Widening BF16 to binary32 is exact and total (stage 5, all 65,536 encodings), so the widened operands denote the same numbers. An exact product of two BF16 values needs at most `2 × 8 = 16` significand bits, which binary32's 24 hold exactly, so the products are the same numbers too. Hence `convert(a) ⋅ convert(b)` in binary32 and "the binary32 products of a BF16-storage operation" are the same values, and the two spellings fold the same contributors in the same order at the same width. Stage 2's 524,288-case zero-disagreement result is the same property observed at the pointwise level.

**Inference — the consequence for what to build first.** A program wanting "BF16 storage, binary32 accumulation, BF16 result" over a pointwise chain can be written today, once the conversion family exists, as `widen → f32 operations → narrow`, and it is *the same function* as an internal-accumulator operation would compute. So the conversion family is the prerequisite and the internal accumulator is not; admitting the accumulator first would add a key whose behaviour the conversion family reproduces.

**Where the equivalence genuinely stops, which is what makes the internal accumulator necessary rather than merely convenient.** Inside a reduction over an index space, the graph-level spelling of "widen each product" requires the products to be *values in the graph* — for a contraction `td,od->to` that is a broadcast, a multiply producing an `[M, N, K]` tensor, a conversion, and a sum over `K`. It is semantically equivalent and it materializes `M·N·K` elements in the public graph, obliging the planner to fuse them away to recover the original program. That is precisely the alternative ADR 0009 considered and rejected. So: **pointwise boundaries take an explicit conversion; a reduction's internal accumulator is an operation fact.** One rule, two boundaries, no exception.

**A third statement of the same rule, for the case that will be written by mistake.** The equivalence above is between the explicit spelling and an internal accumulator. It is *not* an equivalence between either of those and a BF16 accumulator, which stage 4 separates by one ulp. Nothing here licenses a fusion, a schedule, or a target from narrowing an accumulator it was given.

## Question 3 — BF16/F32 conversion semantics, both directions

**Answer (Proposal): two separate conversion families, not one family with a direction field, because the two carry disjoint field sets — the ADR 0041 shape, reached at a float-to-float boundary.**

### Widening, BF16 → binary32: exact and total, and it carries no rounding rule

**Fact — the derivation, which is specific to BF16.** BF16 and binary32 share an exponent field width of 8 and therefore a bias of 127, and BF16's 7 trailing significand bits are a prefix of binary32's 23. Consequently the conversion is a 16-bit left shift of the encoding, and every value class maps to *its own* class: normal to normal, **subnormal to subnormal**, each zero and each infinity to itself, and a NaN payload to a zero-extended NaN payload with the quiet bit in place. Stage 5 checks all 65,536 encodings against an independent field decode with 0 disagreements, and counts 254 of 254 subnormals landing in binary32's subnormal range. Under ADR 0041's own rule that "exact conversion does not carry a rounding rule", the widening contract has **no** rounding field, no overflow field, and no NaN-mapping choice; a widening contract that carried one is malformed rather than redundant.

**Fact — why binary16's exactness has a *different* derivation, which the ticket asks be said explicitly.** Binary16's exponent range is strictly *inside* binary32's rather than equal to it: emin/emax `[-14, 15] ⊂ [-126, 127]`. Its widening is exact because of two independent inclusions — the exponent range and the precision (`24 > 11`) — and it renormalizes: every binary16 subnormal becomes a binary32 **normal**. Stage 5 checks the inequality directly (`-24 > -126`) rather than asserting it. So "widening a narrow float to binary32 is exact" is a conclusion that happens to hold for both formats by two different arguments, and neither argument transfers: F64 and F128 have no wider carrier at all, and BF16 widened to binary16 is not exact in either factor.

**Measurement — the difference has an observed consequence, and it is the reason the derivation matters rather than a curiosity.** Findings 24 and 25 record that on the qualified Apple9 row `bf16` arithmetic flushes subnormals and `f16` arithmetic does not, and attribute the split to exactly this exponent-field difference. **Inference:** a widened BF16 subnormal is an `f32` subnormal, so on a target whose `f32` arithmetic flushes, a BF16 program promoted to binary32 loses precisely the values a native BF16 arithmetic on that same target would also lose — the promotion introduces no *new* subnormal gap on that row. That is a fact about that row and not a portable one, and it must be stated as a target honourability row per subject, never inferred from the conversion's semantics.

### Narrowing, binary32 → BF16: rounding rule, overflow rule, NaN rule, and gradual underflow, each of which changes an answer

**Proposal — the contract's fields, with the witness that makes each one load-bearing.**

1. **Rounding: round-to-nearest, ties-to-even.** It matches ADR 0024's initial arithmetic rounding and the landed BF16 fact `bf16-round-to-nearest-ties-to-even-at-every-observable-materialization`. The alternative a reader will reach for is truncation to the high 16 bits — the "BF16 is binary32's high half" spelling — and stage 6 separates them in **32,704 of 65,536** binary32 patterns covering `[1, 2)`. Concrete witness: `0x3f80c000` (`1.005859375`) narrows to `0x3f81` under nearest-even and to `0x3f80` under truncation. Truncation is a legitimate *named* contract elsewhere in the ecosystem and must be a separate family if ever wanted, never a permitted realization of this one.
2. **Tie direction: to even.** Stage 6 separates ties-to-even from ties-away in **64** of the same population. Concrete witness: `0x3f808000`, exactly halfway between `0x3f80` and `0x3f81`, narrows to `0x3f80` under ties-to-even and `0x3f81` under ties-away.
3. **Overflow: round-to-nearest overflow to a signed infinity, at the inclusive midpoint above the largest finite BF16 value.** This is not a formality: binary32's largest finite magnitude `(2^24 − 1)·2^104` exceeds BF16's overflow threshold `511·2^119`, so **`0x7f7fffff` narrows to `0x7f80`, an infinity**. Stage 6 derives the inequality rather than remembering the encoding. A narrowing contract with no overflow rule has nothing to say about the whole top binade of its source type.
4. **NaN: canonicalize to `tiler::bf16@1`'s canonical quiet-NaN payload `0x7fc0`.** The tempting alternative — preserve the payload prefix — is **not total**: the signalling binary32 NaN `0x7f800001` carries its payload only in the low 16 bits, and truncating it yields `0x7f80`, the *positive infinity* encoding. Totalizing that alternative requires a special case whose behaviour depends on the payload, which is the shape of rule this corpus rejects. Canonicalization is uniform, is the value the landed BF16 arithmetic already installs (`CANONICAL_BF16_ARITHMETIC_NAN_BITS`), and matches the portable-bitwise conformance level's existing rule. A payload-preserving narrowing remains available later as a *separate named family*, exactly as ADR 0041 keeps NaN-to-zero separate from ordered saturation.
5. **Subnormals and signed zero: gradual underflow, signs preserved, and no flush.** A binary32 value in or below BF16's subnormal range rounds into BF16's subnormals under the same nearest-even rule, and underflows to a signed zero carrying the source's sign. Whether a *target* then flushes is a numerical-honourability fact of that target's profile row keyed by arithmetic subject — never a field of this conversion, for the same reason the landed BF16 module gives for keeping the subnormal fact out of consumer-neutral semantics.

**Inference — why these are two families and not one with a direction field.** Fields 1 through 4 are meaningless in the widening direction and absent from it. ADR 0010 requires a contract to define "only the fields relevant to their semantics", and ADR 0041 rejects "one structure containing every cast concern as optional fields" because it "makes invalid combinations representable and weakens diagnostics". A single float-to-float family with optional rounding would make "widening with ties-away" a constructible value, and the whole point of the exactness result is that no such thing exists.

## Question 4 — is a fused or contracted BF16 operation admissible at all?

**Answer (Proposal): a *fused BF16* operation is not admissible, because its only realization does not deliver it. What is admissible, if a workload asks, is a mixed-precision operation whose contract states binary32 evaluation and one narrowing — a different contract, and the name must not imply the other.**

**Measurement — the primitive is absent at the source level.** Finding 29: `metal` rejects `bfloat v6 = fma(v3, v4, v5)` because MSL has no `bfloat` overload; the call promotes to `float`. **Boundary:** offline `metalfe-32023.883`, Xcode 26.6, macOS 27.0 build 26A5388g. This is a compile failure on one toolchain row, not a property of Metal or of `bfloat`; a future MSL revision adding the overload retires the premise, and that is this answer's reconsideration trigger.

**Inference — so the only expressible realization is `bfloat(fma(float(a), float(b), float(c)))`, and the question becomes whether that *is* the BF16 fused result.** It is not, and stage 3 exhibits the operands rather than arguing:

- `a = 0x3fc0` (`1.5`, significand 192), `b = 0x3fb2` (`1.390625`, significand 178), `c = 0xb300` (`−2^-25`).
- `192 × 178 = 267 × 2^7`, so the exact product is `267 × 2^-7 = 2.0859375`, which is **exactly** a BF16 halfway point: quanta `133.5` at exponent 1, between `0x4005` (`2.078125`) and `0x4006` (`2.09375`).
- The exact `a·b + c = 2.0859375 − 2^-25` is strictly below that halfway point, so **one** correctly rounded BF16 fused result is `0x4005`.
- Binary32's ulp near 2 is `2^-22`, so `2^-25` is an eighth of one; the exact sum rounds in binary32 back to `2.0859375`, and the second rounding to BF16 hits the tie, which ties-to-even sends **up** to `0x4006`.
- The halfway point's lower quantum count 133 is **odd**, which is what makes the tie go the other way from the strict inequality. That is why the witness was constructed rather than found.

The sweep found **21,546 disagreements in 262,144 triples**, so this is a population and not a lucky triple, and the perturbation moving `c` from `2^-25` to `2^-20` makes both routes return `0x4005`, so the witness is about the binary32 rounding boundary rather than about the operands.

**Fact — the same statement does *not* hold for one multiply or one add, and the distinction is the whole answer.** Stage 2 finds 0 disagreements in 524,288 cases, and the reason is the classical double-rounding bound `q ≥ 2p + 2` for one `+ − × ÷` or square root, which here is `24 ≥ 18`. The retained Apple record's finding 24 and `crates/tiler-metal/src/target.rs` both already state that inequality at this exact pair, so it is not introduced here; what is new is that the fused multiply-add is **not** one of the operations the bound covers, and stage 3 exhibits the operands that show it does not. This corrects a rationale the spike carried in the opposite direction; [the spike README's finding 5](../../../spikes/numerics/bf16-second-dtype/README.md) records the correction and preserves the original text.

**The contraction half, which is a different permission with the same name.** ADR 0015's contraction is the permission to fuse an *existing* separately rounded multiply/add pair. Finding 28 measures that pair fusing at BF16 under `relaxed` and `fast` offline, and finding 30 measures the runtime compiler fusing it under `relaxed` and `fast` whatever the offline flag says, at all three widths. **Inference — the consequence for the contract:** the landed BF16 family declares `BF16_FACT_ARITHMETIC_CONTRACTION_PERMITTED` false, and finding 30 says that declaration cannot be honoured by choosing an offline flag, because `MTLCompileOptions` has no contraction property and the runtime compiler fuses regardless. So an unfused-BF16 guarantee on the runtime path is currently **unhonourable on the measured rows**, and under ADR 0043's composition that is a *disproved* predicate — an explicit refusal, never a silent one. A source-level `#pragma METAL fp contract(off)` is recorded by finding 10 as an available mechanism deliberately not adopted by the probe; whether it is a defence on the runtime path is unmeasured and is the deferral recorded below.

**What follows for naming.** If a workload ever needs the promoted route, the operation it needs is spelled with binary32 in its own facts: computation precision binary32 over exactly widened BF16 operands, one binary32 rounding at the fused step, one BF16 rounding at the result. Calling that "fused BF16" would state a single-rounding BF16 contract and deliver a double rounding, which is the class of silent wrongness ADR 0015 exists to prevent — its own alternatives-considered rejects "representing every eligible pair as `Fma`" for erasing observable rounding.

## Question 5 — what does an unsupported combination return?

**Answer (Proposal): one of five typed outcomes, discriminated by *what would fix it*, with the two the ticket names — "not defined" and "this target cannot do it" — owned by two different authorities so they cannot collapse.**

The landed corpus already keeps these apart, and the proposal is to reuse each rather than mint a sixth.

| Outcome | What it means | Existing shape and owner | What fixes it |
| --- | --- | --- | --- |
| **Not defined** | No conversion family or operation covers this tuple; Tiler's evidence does not fix a meaning | An unregistered `OpKey`; `UlpFormatError::UnrecognizedClass` → `accuracy.metric.incompatible-dtype` is the precedent for a *refusal in place of a guess* | An accepted decision plus a registration |
| **Malformed request** | The tuple itself is invalid — a widening contract carrying a rounding rule, a narrowing carrying none, a NaN mapping on an exact conversion | Construction-time typed refusal naming the violated rule, as ADR 0087's five structural rules do for a contraction structure | Fix the request |
| **Defined but unimplemented** | Semantics are accepted; no provider registers an evaluator or capability | `EvaluationError::MissingCapability`, what `ReferenceEvaluator::standard()` returns for a registered key its provider set does not reach. That is no longer any *standard* key — `evaluate-bf16-reference-semantics` closed the last three on 2026-08-01 — so the live instance is an operation registered by a second semantic provider that no reference provider implements, which `missing_and_external_reference_capabilities_are_explicit` in `crates/tiler-reference/src/tests.rs` pins from both sides | Implement it |
| **This target cannot do it** | The profile *declares* it does not honour a required behaviour, or declares the dtype `Unsupported` | `UnhonouredDimension` carrying the refusing `NumericalHonourabilityFact` with its full provenance; `DTypeDispatchabilityResolution::Unsupported` resolved at `AvailabilityPhase::CompileProfile`, before ADR 0051's one-way routing commit | A different target, or a contract this one honours — chosen by the caller only |
| **This target has not been asked** | The profile is silent about the `(subject, dimension, behaviour)` or the `(family, dtype)` | `DTypeDispatchabilityResolution::Unknown`; ADR 0043's `Unknown` with no admissible proof path, so it may appear in search and explain and never in an executable frontier | A measurement |

**Fact — the separation is enforced by ownership, not by discipline.** "Not defined" is an answer the semantic registry gives and no target profile can change; "this target cannot do it" is an answer a declaring profile gives and no registry can change. The corpus already relies on this: the spike's routing matrix produces `Dispatchable` / `Unsupported` / `Unknown` for the *same* registered BF16 identity on three profiles, so identity and target-capability are demonstrably independent axes.

**Fact — the "repairs differ" test has a landed precedent worth copying verbatim.** `VariantIneligibility` splits a host-relative refusal into `AssessedProfile`, `UnsupportedRepresentation`, and `PayloadProfile` and its own doc gives the reason: "They stay separate because the repairs differ." That is the discriminator this table uses, and it is the reason the middle row exists at all — a malformed contract and an unimplemented one send a reader to two different places.

**Proposal — the one new obligation this adds.** A rejected conversion tuple must name the **direction** it was rejected in. `bf16 → f32` and `f32 → bf16` are two families; a diagnostic reporting only "no conversion between `tiler::bf16@1` and `tiler::f32@1`" would be true of a request that named the wrong family's fields and equally true of one whose direction is simply unregistered, and those are the malformed-request and not-defined rows.

## Worked end-to-end examples

Three programs. Each states inputs, operations, resolved value types, computation and accumulator types, the numerical contract, and the observable result bits. The first two differ in their **result bits** on the stated inputs, which is what makes the choice between them a decision.

### Example A — the landed contract: pure BF16, BF16 accumulator

```text
inputs   x0 = 0x3f80  (1.0)                          value type tiler::bf16@1
         x1 = 0x3b00  (2^-9)                         value type tiler::bf16@1

program  t1 = tiler::add-bf16@1(x0, x1)
         t2 = tiler::add-bf16@1(t1, x1)
         t3 = tiler::add-bf16@1(t2, x1)
         y  = tiler::add-bf16@1(t3, x1)              value type tiler::bf16@1

facts    computation  tiler::bf16@1
         accumulator  tiler::bf16@1
         intermediate tiler::bf16@1
         result       tiler::bf16@1
         rounding     bf16 round-to-nearest ties-to-even at every observable materialization

contract scalar-arithmetic contract stated for the BF16 subject (unstatable today:
         ScalarArithmetic has no BF16 constructor)

result   y = 0x3f80   (1.0)
```

Every partial rounds back to `1.0`: one BF16 ulp at `1.0` is `2^-7` and half of one is `2^-8`, so a `2^-9` addend is always below the halfway point. **Four contributors are absorbed and the program returns its seed.**

### Example B — the proposed contract: BF16 values, binary32 accumulation, explicit conversions

```text
inputs   x0 = 0x3f80  (1.0)                          value type tiler::bf16@1
         x1 = 0x3b00  (2^-9)                         value type tiler::bf16@1

program  w0 = widen-bf16-to-f32(x0)                  value type tiler::f32@1   exact, total
         w1 = widen-bf16-to-f32(x1)                  value type tiler::f32@1   exact, total
         t1 = tiler::add-f32@1(w0, w1)
         t2 = tiler::add-f32@1(t1, w1)
         t3 = tiler::add-f32@1(t2, w1)
         t4 = tiler::add-f32@1(t3, w1)               value type tiler::f32@1
         y  = narrow-f32-to-bf16(t4)                 value type tiler::bf16@1

facts    widening   no rounding rule, no overflow rule; exact and total
         add-f32    computation/accumulator/result all tiler::f32@1
         narrowing  round-to-nearest ties-to-even; overflow to signed infinity;
                    NaN canonicalized to 0x7fc0; gradual underflow; signs preserved

contract two subjects: the f32 arithmetic subject governs t1..t4; the BF16 subject
         governs nothing here, because no BF16 arithmetic occurs

result   y = 0x3f81   (1.0078125)
```

All four partials are exact in binary32 (`1 + k/512` needs ten significand bits), the exact sum `1 + 2^-7` is exactly one BF16 ulp above `1.0`, and the single narrowing is exact. **The two programs differ by one ulp: `0x3f80` against `0x3f81`.** Stage 4 checks both folds and its control shows one contributor cannot separate them.

**What example B establishes about question 2.** It is written entirely with explicit conversions and F32 operations — no new operation key, no internal accumulator — and it is the program a workload wanting "BF16 storage, F32 accumulation" actually wants. A hypothetical `tiler::strict-tensor-contraction-bf16-f32acc@1` would return the same bits over the same contributors, because widening is exact and BF16 products are exact in binary32. So the internal accumulator earns its key only where the graph-level spelling would materialize an intermediate the reduction does not have — which example B, being pointwise, does not.

### Example C — the fused route, and why it may not be called fused BF16

```text
inputs   a = 0x3fc0  (1.5)          b = 0x3fb2  (1.390625)      c = 0xb300  (-2^-25)

hypothetical A: tiler::fma-bf16@1
  facts   computation tiler::bf16@1, one rounding after a*b+c, result tiler::bf16@1
  result  y = 0x4005   (2.078125)
  realizable?  no.  MSL has no bfloat fma (finding 29); nothing lowers this.

proposed B: a mixed-precision operation stating what it does
  facts   operand type   tiler::bf16@1
          computation    tiler::f32@1, operands exactly widened
          fused step     one binary32 rounding
          result         tiler::bf16@1, one narrowing, ties-to-even
  result  y = 0x4006   (2.09375)
  realizable?  yes, as bfloat(fma(float(a), float(b), float(c))).
```

**The two differ, so the name is not a label on one thing.** Admitting B under A's name would state a single-rounding BF16 contract and deliver a double rounding on 21,546 of 262,144 swept triples. Neither is proposed for registration here; what is proposed is that if either is ever admitted it is admitted under its own facts.

## Portable claims and host-bound claims, separated

**Portable — properties of BF16's and binary32's parameters, true anywhere.** Widening is exact, total, and subnormal-class-preserving. A BF16 product is exact in binary32. A promoted route is bit-exact for one multiply or add and not for a fused multiply-add, with the witness above. The accumulator width is observable. Narrowing's rounding, tie, overflow, and NaN rules each change an answer. Binary16's widening exactness has a different derivation and neither transfers to F64 or F128.

**Host-bound — one Apple row, quoted with its boundary above.** That no `bfloat` FMA compiles; that `bf16` fuses under `relaxed` and `fast` and not under `safe` with `-ffp-contract=fast` offline; that the runtime compiler fuses at all widths regardless; that `bf16` arithmetic flushes subnormals where `f16` preserves them; that the iOS Simulator refuses every `bfloat` pipeline and a physical iOS device is unmeasured. None of these may be inherited by another GPU family, toolchain build, OS build, or dtype.

**Neither — and this is the row most likely to be misread.** Nothing here establishes what any Apple GPU *computes* for a promoted BF16 fused operation. Stage 3 is conditional: *if* the realization is a promotion, it is not the correctly rounded BF16 result. Establishing the antecedent on a device is a measurement nobody has taken.

## Public-boundary consequences, identified for Tom and not designed into acceptance

Every item below is a public boundary under ADR 0075 and is listed rather than proposed for self-acceptance. None is implemented by this record.

1. **Two new operation keys for the conversion family**, one per direction — a `bf16 → f32` widening with no rounding field and an `f32 → bf16` narrowing carrying rounding, overflow, NaN, and subnormal fields. Their names, versioning, and whether the narrowing's rounding rule is a key suffix or a typed attribute are boundary decisions. Whether the family generalizes over `(source, destination)` or stays BF16/F32-specific is a further one, and this record deliberately proposes only the specific pair.
2. **A new typed conversion-contract vocabulary** in the semantic layer. `docs/numerical-semantics.md` names "floating-point widening and narrowing" as an initial family and nothing implements it; the `Cast and convert` row of the support matrix sits at R2 with `ConvertOp::CanonicalizeF32Nan` as its only realized construct. The shape — separate discriminated families per ADR 0010 and 0041 — is derived; the exact Rust boundary is not.
3. **A `(value type, accumulator type)` operation key for a reducing BF16 family**, if and when a workload asks. It does not exist and is not proposed for registration; what is proposed is that when it arrives it is a new key with complete facts rather than an attribute on an existing one.
4. **Fact-field vocabulary growth.** A mixed-precision operation's fact record needs a computation-type field whose value differs from its result type, and the landed BF16 numbering reserves fields 1 through 16. Whether a mixed record reuses that numbering or starts its own is an identity decision, because field IDs are record-local and equal integers are never normalized.
5. **Whether `ScalarArithmetic` gains a second subject at all**, which `admit-a-bf16-scalar-arithmetic-subject` owns and which everything above is downstream of.
6. **The refusal vocabulary's fifth row**, the malformed-conversion-request class, and the obligation that a conversion refusal name its direction.

## Deferrals, each with the evidence that would close it

- **Whether an unfused-BF16 contract is honourable on any Apple runtime path.** Finding 30 measures the runtime compiler contracting at all three widths under `relaxed` and `fast` with no `MTLCompileOptions` counterpart to `-ffp-contract`; finding 10 records `#pragma METAL fp contract(off)` as an accepted source-level mechanism deliberately not used by the probe. **Closes with:** a probe that compiles the `contraction_pair_bf16` source *with* the pragma through `newLibraryWithSource:` and reports whether the runtime result moves from `3fbf` to `3fc0` under `relaxed`. **Trigger:** before any BF16 contract declaring contraction forbidden is offered to a Metal profile.
- **Whether a chain can separate native `bfloat` arithmetic from binary32-precision evaluation on a device.** Finding 24 names this as the experiment that would separate its two surviving hypotheses and records that none has been taken. **This record supplies a candidate:** example A against example B, which differ by one ulp and whose difference is exactly "was the intermediate held at BF16 or at binary32". **The obstacle, stated so the experiment is not attempted naively:** in MSL an intermediate assigned to a `bfloat` variable is rounded by the language's own typing, so the kernel must be written so the compiler is free to keep the value wider, and a null result would then be about the source rather than about the hardware. **Trigger:** only if some contract comes to depend on the answer; today none does, because a contract is written against what a target delivers.
- **Whether a truncating `f32 → bf16` narrowing is ever wanted.** It is a real ecosystem contract and stage 6 separates it from nearest-even in 32,704 of 65,536 patterns. **Closes with:** a named producer or consumer requiring it. **Trigger:** a frontend importing a model whose weights were produced by truncation, where round-tripping under nearest-even would move bits.
- **Whether the conversion family generalizes beyond BF16/F32.** Every derivation here rests on BF16-and-binary32-specific inequalities. **Closes with:** the second float pair a workload selects, at which point the shared structure is visible rather than guessed. **Trigger:** the F16 vertical, whose widening is exact by a *different* argument and whose narrowing from binary32 has its own overflow behaviour.

## Drafted ADR body

**Proposal.** The following is a complete ADR draft, written to be landed verbatim as `docs/decisions/00NN-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md` with `decision_status: proposed`. **This record's scope cannot create it:** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md` to `contracts/navigation`, and this ticket holds `research/numerics`, `contracts/numerics`, and shared `project/tickets` only. `land-the-bf16-conversion-and-accumulator-adr` carries it.

> ## Context
>
> The landed pure-BF16 family states computation, accumulator, intermediate-materialization, and result types as four separate fields, all `tiler::bf16@1`, so that any widening is an explicit edit. The first workload wanting BF16 storage with binary32 accumulation, or any BF16/F32 conversion, has no accepted answer, and the difference between a decided accumulator and an inherited one is invisible in the code and decisive in the results. Separately, no `bfloat` FMA exists in MSL on the measured toolchain row, so a fused BF16 operation has no primitive to lower to.
>
> ## Decision
>
> 1. **A wider accumulator is a property of the operation**, carried in its registered definition facts and therefore in its identity, as `tiler::strict-tensor-contraction-f32@1` already carries `CONTRACTION_F32_FACT_ACCUMULATOR_TYPE`. It is never a schedule choice and never a dimension of the resolved numerical contract. A different accumulator type is a different registered operation key, not an attribute value under one key.
> 2. **BF16/F32 conversion is two separate typed families, one per direction.** The widening family carries no rounding, overflow, or NaN-mapping field, because BF16-to-binary32 widening is exact and total. The narrowing family carries round-to-nearest-ties-to-even, overflow to a signed infinity at the inclusive midpoint above the largest finite BF16 value, canonicalization of every NaN to `0x7fc0`, gradual underflow, and preserved signed zero. A contract carrying a field its direction does not have is refused at construction.
> 3. **A pointwise mixed-width BF16 program uses explicit conversion operations**, and an internal accumulator is admitted only inside a reduction or contraction, where a graph-level per-contributor conversion would encode the operation's internal scalar iteration in the public graph. This is an instance of the no-implicit-promotion rule and not an exception to it.
> 4. **No fused BF16 operation is admitted.** If a workload requires the promoted route, it is admitted as a mixed-precision operation whose facts state binary32 computation over exactly widened operands, one binary32 rounding at the fused step, and one narrowing to BF16 — under a name that does not imply single-rounding BF16 semantics.
> 5. **Every rejected combination returns one of five typed outcomes** — not defined, malformed request, defined but unimplemented, unhonourable on this target, or unknown on this target — with the first owned by the semantic registry and the last two by the declaring target profile, so a registry answer and a target answer can never be confused. A rejected conversion names its direction.
>
> ## Consequences
>
> - Two new operation keys and a typed conversion-contract vocabulary enter the public boundary; the `Cast and convert` support-matrix row moves off R2 only when they are registered.
> - A BF16 program can state binary32 accumulation without a new accumulating key, because widening is exact and BF16 products are exact in binary32, so the explicit spelling is bit-identical to an internal accumulator at a pointwise boundary.
> - A BF16 contract declaring ADR 0015 contraction forbidden is currently unhonourable on the measured Apple runtime path, and that is an explicit typed refusal rather than a silent one.
> - Truncating narrowing, payload-preserving NaN narrowing, and directed roundings remain separately admittable named families and are not reachable by relaxing this one.
>
> ## Alternatives considered
>
> **The accumulator as a schedule property.** Rejected on correctness: two folds differing only in accumulator width return `0x3f80` and `0x3f81` on the same contributors in the same order, so a planner choosing between them would price meaning.
>
> **The accumulator as a numerical-contract dimension.** Rejected structurally: the resolved contract speaks for exactly one arithmetic type and honourability is keyed by that subject, so an operation using two arithmetic types cannot be spoken for by one contract; and every contract dimension is a permission, which by construction does not change what an operation means.
>
> **One float-to-float conversion family with a direction field.** Rejected under ADR 0010 and 0041: it makes "widening with ties-away" constructible, and the exactness result is precisely that no such thing exists.
>
> **Admitting `tiler::fma-bf16@1` and letting the backend promote.** Rejected because the promotion is observably not the contract: it differs by one ulp on a derived witness and on 21,546 of 262,144 swept triples.

## Measurement boundary

- **The spike stages measure nothing.** They are exact-rational derivations, host-independent, with named and counted populations. Each population sweeps one operand exhaustively against a named set of partners; none sweeps all `2^32` pairs or all `2^48` triples, so every count is a count over its own population.
- **Every Apple fact is transcribed**, not re-measured, from `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` — Apple M4 Max, macOS 27.0 build 26A5388g, arm64, Xcode 26.6 build 17F113, offline `metalfe-32023.883`, runtime `metalfe-32023.921` (macOS) and `metalfe-32023.830.1` (iOS Simulator). Re-running them is that spike's procedure.
- **`bf16` was dispatched on macOS only.** Finding 26 records the iOS Simulator refusing every `bfloat` pipeline including an arithmetic-free one; `IOsDevice` was never asked. Both are `Unknown` and neither may be inferred from the macOS row.
- **No semantic, reference, physical, ABI, artifact, kernel, lowering, or runtime layer was crossed by this record.** No key was registered, no evaluator installed, no profile constructed beyond the spike's own three modelled ones.
- **No performance claim.** Nothing was timed except the spike's own 3.8 s wall clock, which is a convenience note and not a measurement.
