---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.metal-elementary-function-accuracy"
kind: "research"
title: "Metal elementary-function accuracy guarantee"
topics: ["numerics", "transcendentals", "accuracy", "metal", "apple-targets", "ulp", "subnormals", "language-model"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["normative-guarantee", "primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.metal-backend"]
depends_on: ["tiler.research.numerics.transcendental-accuracy-precedents", "tiler.research.numerics.transformer-nonlinear-normalization-and-reductions", "tiler.research.apple-targets.numerical-behaviour"]
ticket: "record-the-metal-elementary-function-accuracy-guarantee"
---

# Metal elementary-function accuracy guarantee

**Status:** a reading of a primary source already retained in this repository. It quotes what Apple normatively guarantees for single-precision elementary functions, states four reasons the quoted numbers cannot be written into a Tiler contract as they stand, and stops there. It declares nothing on the Metal target profile, registers no implication, admits no operation, and opens no device.

## Traceability

- **Work record:** [`record-the-metal-elementary-function-accuracy-guarantee`](../../../tickets/record-the-metal-elementary-function-accuracy-guarantee.md).
- **What it unblocks:** the backend half of **D-4** in the [L3′ derivation](transformer-nonlinear-normalization-and-reductions.md#unresolved-decisions), and therefore the three verticals [`admit-the-silu-activation-family`](../../../tickets/admit-the-silu-activation-family.md), [`admit-the-rms-normalization-family`](../../../tickets/admit-the-rms-normalization-family.md), and [`admit-the-softmax-family`](../../../tickets/admit-the-softmax-family.md).
- **Vocabulary this record is written against:** [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) and [ADR 0016](../../decisions/0016-transcendental-accuracy-contracts.md), with the carrier implemented at `crates/tiler-ir/src/semantic/accuracy/`. Every spelling this record cites from the carrier is quoted from that module, so a vertical can consume both halves together.
- **Neighbouring evidence:** the [Metal transcendental emission probe](../../../spikes/numerics/metal_transcendental_emission/README.md) for which intrinsic the governed flags select, and [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) for what one measured host row delivers. Neither is a normative guarantee and this record does not treat them as one.
- **Cross-vendor context:** [Transcendental accuracy precedents](transcendental-accuracy-precedents.md), which established the *vocabulary* this record's numbers would have to be expressed in.

Claims are labelled **Fact** when traced to the retained specification at a verified digest or to inspected source, **Inference** when derived from stated facts, **Proposal** when not yet accepted, and **Measurement** when tied to an exact environment and procedure.

## Why this is a new record rather than an extension of the precedents record

**Inference — the two answer different questions and have different lifetimes.** [Transcendental accuracy precedents](transcendental-accuracy-precedents.md) asks "what minimum semantic vocabulary lets Tiler state useful transcendental accuracy requirements without inventing guarantees that a backend does not make?", surveys OpenCL, Metal, CUDA, compiler IRs, math libraries, and combined-tolerance conventions to answer it, and was `adopted` by ADRs 0016 and 0042 as the evidence for a vocabulary. This record asks a target-specific quantitative question — what does one named revision of one vendor's specification promise for the functions one named workload evaluates — and its answer expires when Apple publishes a revision whose Table 8.1 differs. Folding it into the precedents record would put pending, revision-pinned target evidence inside a record two accepted ADRs cite as their vocabulary basis, and would make those ADRs appear to have adopted material they never saw. The precedents record's Metal paragraph is instead pointed here.

## The documents, and how to reproduce every quotation

**Fact — two retained revisions, both used.**

| Short name in this record | Document identity | Retained path | SHA-256 |
| --- | --- | --- | --- |
| **MSL 4.1** | Metal Shading Language Specification, Version 4.1, dated 2026-06-04 | `docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4.1-2026-06-04.pdf` | `41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5` |
| **MSL 4** | Metal Shading Language Specification, Version 4, dated 2025-10-23 | `docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4-2025-10-23.pdf` | `eed87a82d4d2d475423b91b3c529c5313a85433f83e22b7fe3ec50e90254f44a` |

The [emission probe](../../../spikes/numerics/metal_transcendental_emission/README.md) records `-std=metal4.0` as the pinned offline toolchain's language selection, so MSL 4 is the revision documenting the language that toolchain compiles and MSL 4.1 is the newest retained revision; every quotation below is taken from both, so nothing here depends on which one governs. Page numbers are the printed page numbers, which equal the PDF page indices in both files — checked by rendering page 371 of MSL 4.1 and page 331 of MSL 4 and reading their footers.

**Fact — chapter 8 "Numerical Compliance" opens by bounding itself.** MSL 4.1 page 368, MSL 4 page 331: "This chapter covers how Metal represents floating-point numbers regarding accuracy in mathematical operations. **Metal is compliant to a subset of the IEEE 754 standard.**"

**Fact — the whole of §8.4 is identical between the two revisions.** Exact check, reproducible in one line from the repository root, over each retained PDF in turn:

```sh
pdftotext -layout docs/research/apple-targets/sources/<pdf> - \
  | sed -n '/^8.4 ULPs and Relative Error/,/^8.5 Edge Case Behavior/p' \
  | grep -v 'Apple Inc. | All Rights Reserved' | grep -v '^ *Page [0-9]* of [0-9]*$' \
  | sed 's/[[:space:]]\{1,\}/ /g;s/^ //;s/ $//' | grep -v '^$' | shasum -a 256
```

Both revisions produce `99202c88fa96864c20f9ddf14ba681c26630e59cc84f467743123b9f8d264de8` over 216 normalized lines: same tables, same entries, same ULP definition. The pipeline strips only page footers and horizontal whitespace, so the equality is over the section's content rather than over its typesetting.

**Fact — the extraction flattens superscripts, and every superscripted entry below was re-read from a rendered page.** `pdftotext` renders `2⁻¹²⁶` as `2-126` and `2⁻¹³` as `2-13`, which a reader copying the text would silently turn into subtraction. Pages 369, 371, and 375 of MSL 4.1 were rendered with `pdftoppm -png -r 110 -f <page> -l <page>` and read as images to confirm the exponents, the `exp` row of Table 8.1, and the ULP definition. The superscripts are restored in the quotations below.

## Table 8.1 — the applicable single-precision table

**Fact.** MSL 4.1 §8.4, "Table 8.1. Accuracy of single-precision floating-point operations and functions", pages 368–370; MSL 4 pages 331–333. Its preamble reads: "Table 8.1 describes the minimum accuracy of single-precision floating-point basic arithmetic operations and math functions given as ULP values. **The reference value used to compute the ULP value of an arithmetic operation is the infinitely precise result.**"

The complete table, transcribed. The final column names which of the three L3′ verticals or which workload element reaches the entry, per the [L3′ derivation's](transformer-nonlinear-normalization-and-reductions.md) pinned formulas; a blank cell is an entry quoted for contrast only.

| Math function | Minimum accuracy (ULP values) | Reached by |
| --- | --- | --- |
| `x + y` | Correctly rounded | SiLU `1 + Exp(-x)`; softmax denominator; RMS sum of squares and `+ eps` |
| `x - y` | Correctly rounded | softmax `s_i - m` |
| `x * y` | Correctly rounded | RMS squares, normalize, and weight; softmax `e_i * (1/d)`; attention scale |
| `1.0 / x` | Correctly rounded | softmax reciprocal of the denominator |
| `x / y` | Correctly rounded | SiLU `x / (1 + Exp(-x))`; RMS division by the extent |
| `acos` | `<= 4 ulp` | |
| `acosh` | `<= 4 ulp` | |
| `asin` | `<= 4 ulp` | |
| `asinh` | `<= 4 ulp` | |
| `atan` | `<= 5 ulp` | |
| `atan2` | `<= 6 ulp` | |
| `atanh` | `<= 5 ulp` | |
| `ceil` | Correctly rounded | |
| `copysign` | `0 ulp` | |
| `cos` | `<= 4 ulp` | |
| `cosh` | `<= 4 ulp` | |
| `cospi` | `<= 4 ulp` | |
| **`exp`** | **`<= 4 ulp`** | **SiLU and softmax — the one inexact element of both** |
| `exp2` | `<= 4 ulp` | |
| `exp10` | `<= 4 ulp` | |
| `fabs` | `0 ulp` | |
| `fdim` | Correctly rounded | |
| `floor` | Correctly rounded | |
| `fma` | Correctly rounded | |
| `fmax` | `0 ulp` | softmax row maximum, subject to the extrema-family question D-2 |
| `fmin` | `0 ulp` | |
| `fmod` | `0 ulp` | |
| `fract` | Correctly rounded | |
| `frexp` | `0 ulp` | |
| `ilogb` | `0 ulp` | |
| `ldexp` | Correctly rounded | |
| `log` | `<= 4 ulp` | |
| `log2` | `<= 4 ulp` | |
| `log10` | `<= 4 ulp` | |
| `modf` | `0 ulp` | |
| `nextafter` | `0 ulp` | |
| `pow` | `<= 16 ulp` | |
| `powr` | `<= 16 ulp` | |
| `rint` | Correctly rounded | |
| `round` | Correctly rounded | |
| **`rsqrt`** | **Correctly rounded** | **RMS normalization** |
| `sin` | `<= 4 ulp` | |
| `sincos` | `<= 4 ulp` | |
| `sinh` | `<= 4 ulp` | |
| `sinpi` | `<= 4 ulp` | |
| `sqrt` | Correctly rounded | |
| `tan` | `<= 6 ulp` | |
| `tanpi` | `<= 6 ulp` | |
| `tanh` | `<= 5 ulp` | |
| `trunc` | Correctly rounded | |

**Fact — the table has no row for unary negation, and `divide` is not the same spelling as `x / y`.** SiLU's pinned formula evaluates `Exp(-x)`, and the negation is not an entry. Neither is `metal::divide(x, y)`, which MSL 4.1 Table 6.4 (§6.6, page 206) defines as "Compute `x / y`" while Table 8.1 states its accuracy against the operator spelling only. Whether the two are the same operation is a reading, not a quotation, and any vertical that lowers a division through `divide()` rather than `/` is relying on that reading.

**Inference — the notable entry is `rsqrt`, and it changes the shape of D-4.** A correctly rounded reciprocal square root is a strong guarantee, and it is stated for the ordinary — non-fast-math — table only; Table 8.2 gives the same function `<= 2 ulp`. D-4 names "the accuracy contract for `Exp` and `Rsqrt`" as one decision; under Table 8.1 the two are stated in different contract *forms*, and the ULP-metric problem below binds only one of them. That asymmetry is the most consequential thing in this record.

**Inference — under Table 8.1 the only inexact element of the pinned SiLU formula is the exponential.** `y = x / (1 + Exp(-x))` uses one negation (not stated by the table, exact by IEEE sign manipulation), one addition, and one division, both of the latter correctly rounded by Table 8.1. This is what the L3′ record predicted from ADR 0024's side, now with the backend half stated rather than assumed.

## Table 8.2 — the fast-math table, which is a different contract

**Fact.** MSL 4.1 §8.4, "Table 8.2. Accuracy of single-precision operations and functions with fast math enabled", pages 370–372; MSL 4 pages 333–335. Its preamble is the applicability clause: "Table 8.2 describes the minimum accuracy of single-precision floating-point arithmetic operations given as ULP values with fast math enabled (which is the default unless you specify `-fno-fast-math` as a compiler option)."

The complete table, transcribed, with superscripts restored.

| Math function | Minimum accuracy (ULP values) |
| --- | --- |
| `x + y` | Correctly rounded |
| `x - y` | Correctly rounded |
| `x * y` | Correctly rounded |
| `1.0 / x` | `<= 1 ulp` for `x` in the domain of 2⁻¹²⁶ to 2¹²⁶ |
| `x / y` | `<= 2.5 ulp` for `y` in the domain of 2⁻¹²⁶ to 2¹²⁶ |
| `acos(x)` | `<= 5 ulp` for `x` in the domain [-1, 1] |
| `acosh(x)` | Implemented as `log(x + sqrt(x * x – 1.0))` |
| `asin(x)` | `<= 5 ulp` for `x` in the domain [-1, 1] and `\|x\| >= 2⁻¹²⁵` |
| `asinh(x)` | Implemented as `log(x + sqrt(x * x + 1.0))` |
| `atan(x)` | `<= 5 ulp` |
| `atanh(x)` | Implemented as `0.5 * (log( (1.0 + x) / (1.0 – x) )` |
| `atan2(y, x)` | Implemented as: if `x > 0`, `atan(y / x)`; if `x < 0` and `y > 0`, `atan(y / x) + M_PI_F`; if `x < 0` and `y < 0`, `atan(y / x) – M_PI_F`; and if `x = 0` or `y = 0`, the result is undefined. |
| `ceil` | Correctly rounded |
| `copysign` | `0 ulp` |
| `cos(x)` | For `x` in the domain [-pi, pi], the maximum absolute error is `<= 2⁻¹³` and larger otherwise. |
| `cosh(x)` | Implemented as `0.5 * (exp(x) + exp(-x))` |
| `cospi(x)` | For `x` in the domain [-1, 1], the maximum absolute error is `<= 2⁻¹³` and larger otherwise. |
| `exp(x)` | `<= 3 + floor(fabs(2 * x)) ulp` |
| `exp2(x)` | `<= 3 + floor(fabs(2 * x)) ulp` |
| `exp10(x)` | Implemented as `exp2(x * log2(10))` |
| `fabs` | `0 ulp` |
| `fdim` | Correctly rounded |
| `floor` | Correctly rounded |
| `fma` | Correctly rounded |
| `fmax` | `0 ulp` |
| `fmin` | `0 ulp` |
| `fmod` | Undefined |
| `fract` | Correctly rounded |
| `frexp` | `0 ulp` |
| `ilogb` | `0 ulp` |
| `ldexp` | Correctly rounded |
| `log(x)` | For `x` in the domain [0.5, 2], the maximum absolute error is `<= 2⁻²¹`; otherwise if `x > 0` the maximum error is `<= 3 ulp`; otherwise the results are undefined. |
| `log2(x)` | For `x` in the domain [0.5, 2], the maximum absolute error is `<= 2⁻²²`; otherwise if `x > 0` the maximum error is `<= 2 ulp`; otherwise the results are undefined. |
| `log10(x)` | Implemented as `log2(x) * log10(2)` |
| `modf` | `0 ulp` |
| `pow(x, y)` | Implemented as `exp2(y * log2(x))`. Undefined for `x = 0` and `y = 0`. |
| `powr(x, y)` | Implemented as `exp2(y * log2(x))`. Undefined for `x = 0` and `y = 0`. |
| `rint` | Correctly rounded |
| `round(x)` | Correctly rounded |
| `rsqrt` | `<= 2 ulp` |
| `sin(x)` | For `x` in the domain [-pi, pi], the maximum absolute error is `<= 2⁻¹³` and larger otherwise. |
| `sinh(x)` | Implemented as `0.5 * (exp(x) – exp(-x))` |
| `sincos(x)` | ULP values as defined for `sin(x)` and `cos(x)` |
| `sinpi(x)` | For `x` in the domain [-1, 1], the maximum absolute error is `<= 2⁻¹³` and larger otherwise. |
| `sqrt(x)` | Implemented as `x * rsqrt(x)` with special cases handled correctly. |
| `tan(x)` | Implemented as `sin(x) * (1.0 / cos(x))` |
| `tanh(x)` | Implemented as `(t – 1.0)/(t + 1.0)`, where `t = exp(2.0 * x)` |
| `tanpi(x)` | Implemented as `tan(x * pi)` |
| `trunc` | Correctly rounded |

**Inference — Table 8.2 is three kinds of statement wearing one heading, and ADR 0042 routes each differently.** A constant ULP bound (`atan`, `rsqrt`) is a bounded-piecewise candidate. An input-dependent formula (`exp`, `exp2`) or an absolute-error region with an unstated remainder (`sin`, `cos`, `log`) is what ADR 0042 sends to "a governed, nominal `NamedElementaryProfileKey` plus its immutable canonical descriptor digest; they are not approximated into constants" — and an entry whose bound is stated on an interval and merely "larger otherwise" is usable only where a proof restricts the semantic input domain to that interval, which the precedents record already fixes. An *implementation* (`sqrt`, `tanh`, `pow`, `atan2`) is not a bound at all: it defines the result by composition, so its error is whatever the composition delivers under the same table. Table 8.1 and Table 8.2 are therefore two contracts over the same function names, not one operation with a mode, and nothing in this record permits reading a bound out of one and a domain out of the other.

**Fact — a correction to this ticket's own premise.** The ticket filing this work recorded Table 8.2 as giving "`x / y` ≤ 0.6 ulp for y in the domain of 2⁻¹²⁶ to 2¹²⁶". That entry belongs to **Table 8.5** — "Accuracy of brain floating-point operations and functions with fast math enabled", MSL 4.1 page 375, MSL 4 page 338 — where both `1.0 / x` and `x / y` are `<= 0.6 ulp` over that domain. Table 8.2's single-precision divisions are `<= 1 ulp` and `<= 2.5 ulp`. The single-precision and `bfloat` tables are adjacent and identically shaped, which is exactly how the two get conflated; the check that caught it is the transcription of both tables in full.

**Fact — the two revisions also agree on the `bfloat` and `half` tables' existence and placement**, Table 8.3 (half, MSL 4.1 page 373) and Tables 8.4 and 8.5 (`bfloat` without and with fast math, both on MSL 4.1 page 375). This record quotes none of their entries beyond the correction above, because the workload is F32-widened and no vertical reaches them. A future narrower profile must read them rather than infer them from Table 8.1: Table 8.4's `1.0 / x` and `x / y` are correctly rounded where Table 8.5's are not, so the dtype and the math mode move independently.

## What §8.4 says about ULP itself

**Fact — the definition, quoted in full.** MSL 4.1 §8.4, page 375; MSL 4 page 338; identical in both. It appears at the *end* of §8.4, after all five tables:

> The ULP is defined as follows:
>
> If x is a real number that lies between two finite consecutive floating-point numbers a and b, without being equal to one of them, then `ulp(x) = |b − a|`, otherwise `ulp(x)` is the distance between the two nonequal finite floating-point numbers nearest x. Moreover, `ulp(NaN)` is NaN.

**Fact — the paragraph immediately above it bounds every table at once.** Same page: "Even though the precision of individual math operations and functions are specified in Table 8.1, Table 8.2, Table 8.3, Table 8.4, and Table 8.5, the Metal compiler, in fast math mode (see section 1.6.5), may do various optimization like reassociate floating-point operations that may dramatically change results in floating-point. Reassociation may change or ignore the sign of zero, allow optimizations to assume the arguments and result are not NaN or +/-INF, inhibit or create underflow or overflow and thus cannot be in code that relies on rounding behavior such as `(x + 2⁵²) - 2⁵²`, or ordered floating-point comparisons." The clause is scoped to fast math mode, so it does not qualify Table 8.1 under the governed flags; it is quoted because a reader who took a per-operation bound as a per-program bound would be wrong in exactly the direction this repository's reduction-order work already guards.

## Four gaps that stop the numbers being adopted as they stand

Each is a reason a vertical must not simply write `Ulp(tiler::ulp-reference-gap@1, 4)` for `exp`, or `CorrectlyRounded` for `rsqrt`, and move on. The ticket named the first three; the fourth was found while reading §8.2 and is recorded because it binds the entries the first three leave alone.

### Gap 1 — the metric definition differs, and ADR 0042 forbids silent translation

**Fact — what Tiler's metric resolves that Apple's text does not.** ADR 0042 fixes `tiler::ulp-reference-gap@1`: "If `r` is representable, one selected value is `r` and the other is its nearest numerically unequal finite neighbor; **where predecessor and successor gaps differ, the smaller gap is used.** Thus binary `ulp(2^e)` uses the predecessor gap, while the scale increases immediately above that value." The carrier states the same rule at `crates/tiler-ir/src/semantic/accuracy/metric.rs`, and adds the two boundary rules: at zero "the smallest positive finite representable value", and the metric "is **defined only** when `r` and `z` are finite and `r` lies within the result format's finite numerical range", deliberately not inheriting "OpenCL's additional hypothetical-successor overflow allowance".

**Inference — Apple's second clause admits two readings, and they differ.** "the distance between the two nonequal finite floating-point numbers nearest x" can be read as *the nearest adjacent pair of distinct representable values* (reading A) or as *the distance between the two nearest values that are unequal to x*, that is, from predecessor to successor (reading B). Reading B is the more literal parse of the words; reading A is the one consistent with the first clause, since approaching a representable value from either side inside a binade gives the containing gap while reading B gives twice it. Nothing in the retained text chooses. Against `tiler::ulp-reference-gap@1`, which takes the smaller adjacent gap, the readings diverge as follows, writing `g` for the successor gap:

| Reference `r` | `ulp` under `tiler::ulp-reference-gap@1` | Reading A | Reading B |
| --- | --- | --- | --- |
| strictly inside a binade, non-representable | the containing gap `g` | `g` | `g` |
| representable, interior of a binade | `g` | `g` | `2g` |
| representable power of two `2^e` | `g/2` (predecessor gap) | `g/2` or `g` — the clause does not choose | `1.5g` |
| zero | minimum positive subnormal | minimum positive subnormal | twice it |

**Inference — so a factor exists, and it is not one.** A candidate satisfying `|z − r| ≤ 4 · ulp_apple(r)` satisfies `|z − r| ≤ 4 · k · ulp_tiler(r)` where `k` is the largest ratio `ulp_apple/ulp_tiler` over the domain in use. Over the whole finite domain `k = 2` under reading A and `k = 3` under reading B. Restricted to a domain containing no power of two and no zero, `k = 1` under reading A and `k = 2` under reading B, because reading B doubles the scale at every representable point rather than only at a binade boundary. A conservative conversion covering both readings therefore turns Apple's `exp <= 4 ulp` into **12 ULPs** under Tiler's metric, and a derivation that claims 4, or 8, is claiming a reading and a domain it must name. The carrier's shape for this is `RegisteredImplication::ScaledMetric { from, to, factor }`, whose own documentation states the obligation: "registering one asserts that the two definitions of `ulp` agree up to `factor` **over the domain in use**, which is a derivation about two specifications rather than an observation that both are spelled 'ULP'".

**Fact — the carrier already fails closed on exactly this, and its `standard()` registry deliberately supplies no row.** `RegisteredImplicationRegistry::standard` registers three rows — correctly-rounded-satisfies-ULP at floor ½, faithful-satisfies-ULP at floor 1, and correctly-rounded-satisfies-faithful — and its documentation says "**No cross-metric row**: adopting a vendor's ULP bound under Tiler's metric needs that vendor's own definition read and reconciled, which is evidence work that belongs to the record that quotes the specification, not a default this vocabulary can supply." An `apple::msl-ulp@1` bound presented against a `tiler::ulp-reference-gap@1` requirement therefore returns `RefinementOutcome::Unknown { reason: RefinementUnknown::UnregisteredMetricImplication { .. } }`, diagnostic code `accuracy.refinement.unregistered-metric-implication`, which `RefinementOutcome::is_physically_feasible` reports as infeasible. **This record is that evidence work and it deliberately registers nothing**: registering the row is the vertical's act, with the table above as its citation and its chosen reading as its stated derivation.

**Fact — `ulp(NaN)` is defined by Apple and outside Tiler's metric by construction, and that difference is inert rather than reconcilable.** Apple's "Moreover, `ulp(NaN)` is NaN" makes the predicate `|z − r| ≤ t · ulp(r)` a NaN comparison at a NaN reference, so it constrains nothing there; Tiler's metric refuses a non-finite `r` instead. The two arrive at "no bound at NaN" by opposite routes and neither supplies an exceptional-value rule, which is Gap 3.

**Inference — one divergence does have teeth, and SiLU reaches it.** Apple's clause is stated for any real `x`, so under either reading it yields a value where the reference overflows the format — the nearest finite pair below the largest finite value. `tiler::ulp-reference-gap@1` refuses that case by design. SiLU evaluates `Exp(-x)`, whose exact reference exceeds the largest finite F32 for `x` at or below about `-88.73`, so the SiLU vertical reaches inputs at which Apple's `exp <= 4 ulp` states something and Tiler's ULP predicate is undefined. Those inputs must be covered by the operation's `FiniteOverflowRule` rather than by an accuracy clause; a bounded-piecewise contract whose clauses tried to cover them would be rejected by the carrier's own definedness rule rather than silently accepted.

**Fact — ADR 0042 says the initial metric matches "the definition used by OpenCL", and the corpus's OpenCL evidence is not retained here.** The precedents record states that OpenCL's definition includes "an explicit rule at representable values and zero"; the OpenCL specification itself is cited by URL and is not a retained document in this repository. A derivation that argued "Apple's wording is OpenCL's, and OpenCL resolves the representable case, therefore Apple does" would rest on a document nobody can diff at a digest. Retaining the relevant OpenCL section is one of the two things that would close this gap; the other is Apple stating which gap applies.

### Gap 2 — the applicability clause names a flag spelling Tiler does not use

**Fact — the tables are selected by fast math being on or off, and Table 8.2's caption is the only place either table states its own applicability.** "with fast math enabled (which is the default unless you specify `-fno-fast-math` as a compiler option)", MSL 4.1 page 370. Table 8.1 carries no such clause; it is the non-fast-math table by contrast with Table 8.2's caption rather than by its own statement.

**Fact — the specification itself defines the legacy spelling in terms of the modern flags.** MSL 4.1 §1.6.3 "Math Intrinsics Compiler Options", page 16; MSL 4 page 16; identical wording. Under "Metal supports the following legacy options": "`-ffast-math` — Equivalent to `-fmetal-math-fp32-functions=fast` and `-fmetal-math-mode=fast`." and "`-fno-fast-math` — **Equivalent to `-fmetal-math-fp32-functions=precise` and `-fmetal-math-mode=safe`.**"

**Inference — that equivalence, not the emission measurement, is what makes Table 8.1 the applicable table.** The governed baseline the [workload profile](../program-planning/first-metal-lm-workload.md) records is `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`. Its first two flags are exactly the expansion the specification gives for `-fno-fast-math`, so a program compiled under the governed baseline is compiled under the condition Table 8.2's caption names as its own negation. The inference is that "not fast math" in §8.4 means the same thing as the §1.6.3 expansion; the two sections do not cross-reference each other, and Apple never writes "Table 8.1 applies when `-fmetal-math-mode=safe`". This is the whole of the residual gap, and it is a much smaller one than a flag-spelling mismatch would be.

**Inference — the third governed flag is a strengthening, not a divergence.** §1.6.3 says `safe` "disables unsafe floating-point optimizations by preventing the compiler from making any transformations that might affect the results. **This sets the FP contract to on.**" The governed baseline adds `-ffp-contract=off`, which is strictly more restrictive than the `-fno-fast-math` equivalence, and both settings are covered by Table 8.1 in any case: a contracted `a*b+c` is the table's correctly rounded `fma`, and an uncontracted one is its correctly rounded `x * y` followed by its correctly rounded `x + y`. So the extra flag cannot move the program outside Table 8.1's scope.

**Measurement — the independent corroboration, and what it is worth.** The [emission probe](../../../spikes/numerics/metal_transcendental_emission/README.md) measured that under the governed baseline `exp(x)` and `precise::exp(x)` both select `air.exp.f32` while `fast::exp(x)` selects `air.fast_exp.f32`, that `rsqrt` selects `air.rsqrt.f32`, that `a / b` lowers to an LLVM `fdiv`, and that no call site carries fast-math flags — and that with no flags at all, or under either fast selection, the unqualified spellings select the `fast_` family instead. **Inference.** Matching intrinsic families is consistent with the governed baseline selecting the precise family Table 8.1 describes; it is compile-side evidence about *which function is called*, not about which table governs it, and it cannot by itself establish an applicability claim. It is recorded as corroboration of the §1.6.3 reading, not as its basis. The probe's own boundary stands: no device was opened and no value was compared.

**Inference — the failure mode this gap protects against is one compiler flag wide.** The default is fast math. A build that omitted the governed flags would silently be governed by Table 8.2, where `exp` has an input-dependent bound instead of a constant one and `rsqrt` is `<= 2 ulp` instead of correctly rounded, and — by §8.1 — where INF and NaN handling becomes undefined. Selecting `air.fast_exp.f32` to satisfy a contract stated against Table 8.1 is exactly the substitution [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) forbids, and the L3′ record already lists the refusal it owes.

### Gap 3 — the table states accuracy and no exceptional-value contract

**Fact — there is no edge-case table for math functions anywhere in chapter 8.** Table 8.1 states a ULP column and nothing else; §8.5 is the only edge-case section and it is about flush-to-zero mode rather than about NaN, infinity, or domain errors per function. The one per-function exceptional statement in the language reference is Table 6.4's for `fma` — "Edge case behavior is per the IEEE 754-2008 standard" (MSL 4.1 page 206) — and for `fmax`/`fmin`, whose NaN and denormal behaviour Table 6.4 states in prose. Nothing of the kind exists for `exp` or `rsqrt`.

**Fact — §8.1 "INF, NaN, and Denormalized Numbers", MSL 4.1 page 368, MSL 4 page 331, quoted in full.** "INF must be supported for single-precision, half-precision, and brain floating-point numbers. NaNs must be supported for single-precision, half-precision, and brain floating-point numbers (with fast math disabled). If fast math is enabled the behavior of handling NaN or INF (as inputs or outputs) is undefined. Signaling NaNs are not supported. Denormalized single-precision, half-precision, or brain floating-point numbers passed as input to or produced as the output of single-precision, half-precision, or brain floating-point arithmetic operations **may be flushed to zero**."

**Inference — "may" is permissive and therefore licenses neither behaviour.** §8.1 does not say subnormals *are* flushed and does not say they are preserved, so it cannot be the basis of either a flushing or a preserving declaration on the target profile. What is measured is a different class of claim: [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) findings 2, 3, and 11 measure input flushing and result flushing both occurring for F32 arithmetic, sign-preserving, and byte-identically declared across the three artifact families on one host row — an empirical qualification, which under ADR 0042 "detects regressions and characterizes implementations but does not prove an unmeasured worst-case bound". The specification permits; the measurement observes; neither promises.

**Fact — §8.3 "Floating-Point Exceptions", MSL 4.1 page 368, in its entirety.** "Floating-point exceptions are disabled in Metal." Nothing states what an overflowing, invalid, or domain-error evaluation returns.

**Fact — §8.5 "Edge Case Behavior in Flush to Zero Mode", MSL 4.1 pages 375–376, MSL 4 pages 338–339, quoted in full.** "If denormalized values are flushed to zero, then a function may return one of four results: 1. Any conforming result when not in flush to zero mode. 2. If the result given by step 1 is a subnormal before rounding, it may be flushed to zero. 3. Any nonflushed conforming result for the function if one or more of its subnormal operands are flushed to zero. 4. If the result of step 3 is a subnormal before rounding, the result may be flushed to zero. In each of the above cases, **if an operand or result is flushed to zero, the sign of the zero is undefined.**"

**Inference — so `exp`'s exceptional-value, signed-zero, and subnormal policies stay `Unknown` from the specification.** ADR 0042 composes the observable result set in five steps and requires the input-subnormal contract, the NaN/infinity/domain/finite-overflow contract, and the result-subnormal and signed-zero mappings to be stated independently of the error metric — the carrier makes that structural, since `ExceptionalValueContract::new` takes a `NanReferenceRule`, an `InfiniteReferenceRule`, a `DomainErrorRule`, and a `FiniteOverflowRule` as four separate required arguments, and `refines` refuses outright when two contracts state different ones. Table 8.1 supplies a value for none of the four. A vertical that wrote `FiniteOverflowRule::SignedInfinity` for `exp` would be stating a rule the specification does not contain; what §8.1 does supply is that INF is supported with fast math disabled, which makes `+inf` a *representable* outcome rather than a guaranteed one.

### Gap 4 — "correctly rounded" does not say which rounding, and §8.2 permits two

**Fact — §8.2 "Rounding Mode", MSL 4.1 page 368, MSL 4 page 331, in its entirety.** "Either round ties to even or round toward zero rounding mode may be supported for single-precision, half-precision, and brain floating-point operations."

**Fact — the carrier can spell only one of the two.** `ReferenceRoundingRule` in `crates/tiler-ir/src/semantic/accuracy/contract.rs` has exactly one variant, `NearestTiesToEven`, and `AccuracyContractForm::CorrectlyRounded` carries it as a required field. ADR 0042 defines the correctly rounded form as rounding the infinitely precise reference "once to the result dtype **using the named rounding rule**", and ADR 0024 fixes round-to-nearest ties-to-even for Tiler's initial arithmetic.

**Inference — so Table 8.1's correctly rounded entries cannot be adopted as `CorrectlyRounded { NearestTiesToEven }` on the strength of the specification.** Correct rounding under round-toward-zero and correct rounding under round-to-nearest-ties-to-even are different result sets, differing at every input whose exact result is not representable, and §8.2 leaves the choice to the implementation. This gap binds precisely the entries Gap 1 leaves alone — `rsqrt`, `x / y`, `1.0 / x`, `x + y`, `x - y`, `x * y`, `fma`, `sqrt` — so *no* Table 8.1 entry the three verticals need is adoptable without an explicit derivation: the ULP entries need Gap 1's factor, and the correctly rounded entries need Gap 4's rounding mode.

**Fact — Apple pins the mode elsewhere and not here, which is evidence the omission is deliberate.** §8.6 (MSL 4.1 page 376) states that conversions from `float` to `half` or `bfloat` "round the mantissa using the round ties to even rounding mode", that float-to-integer conversion "uses round toward zero rounding mode", and — new in Metal 4.1, §1.6.3 page 17 — that `-fmetal-rtz-fp-conversion` changes the default rounding for float-to-float conversions "from RTNE (round to nearest, ties to even) to RTZ (round toward zero)". The specification names the mode where it means to fix one; §8.2 names both.

**Measurement — what the corpus has instead, and its class.** [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) finding 5 measures `(-0.0) * 1.0 + (+0.0)` returning `00000000` under `safe` and observes that "IEEE-754 round-to-nearest requires the former". That is consistent with round-to-nearest on that host row and is an empirical qualification bounded to it; under ADR 0042 it cannot discharge a hard accuracy feasibility requirement, which `ConformanceEvidenceClass::discharges_hard_requirement` returns `false` for by construction. The registered `add-f32` and `multiply-f32` rows at R6 rest on that measurement, correctly classified; what this record establishes is that the *normative* route to the same statement is unavailable and that nobody had recorded §8.2's permissiveness. The exact check behind that last clause: `grep -rn "round toward zero\|ties to even" docs/research/apple-targets/ docs/backends/metal.md docs/research/target-profiles/` returns nothing at `ccba2d5`.

## Which corpus expectations §8.5 reaches, and which it does not

Getting this wrong in either direction is the failure mode: an over-broad reading makes an exact result look like a permitted flush, and an under-broad one lets a golden pin a bit pattern the specification leaves open.

**Reached — softmax's underflow band, where the sign of an output zero becomes undefined.** The L3′ record measures that `Exp` returns a subnormal for arguments below about `−87.34` (`0xc2aeac50`) and exactly `+0.0` below about `−103.97` (`0xc2cff1b5`). In the band between those two the exact reference *is* subnormal before rounding, so §8.5 clause 2 applies and the target may return zero — and its final sentence makes the sign of that zero undefined. The consequence propagates: the contributor `e_i` enters the denominator, where `+0.0 + (−0.0)` is `+0.0` under round-to-nearest so the sum is unaffected, and it enters the result as `e_i * (1/d)`, which is `±0.0`. So an output element in that band has an undefined zero sign under the specification, and a conformance golden that pinned `0x00000000` there would be asserting something Apple does not promise. **Note what is *not* the flush case.** The masked contributor of the L3′ worked example evaluates `Exp(0xff7fffff)`, whose exact reference is far below the smallest subnormal and rounds to `+0.0` with no flush involved; that `+0.0` is delivered by ordinary rounding, and it is the band, not the mask, that reaches §8.5.

**Reached — RMS normalization's subnormal row, by a route that is weaker still.** The L3′ record measures that a row of `1e-40` normalizes to `0x02081cb9` on the reference and to zeros on a flushing target, because the subnormal *inputs* flush before the squaring. The permission for that is §8.1's "passed as input to ... arithmetic operations may be flushed to zero", and §8.1 says nothing at all about the sign of the resulting zero: §8.5's "the sign of the zero is undefined" sentence is scoped to its own four cases, which are about what "a function may return". Both routes leave the sign unavailable to a contract, and they leave it unavailable by different mechanisms — the arithmetic route by silence, the function route by an explicit "undefined". A record that cited §8.5 for the RMS row would be citing the wrong clause for the right conclusion.

**Not reached — SiLU's `-88.73` band, and the reason is arithmetic rather than a flush.** For `x` at or below about `-88.73` the L3′ record measures `silu(x)` to be exactly `-0.0`. Under the pinned division form the route is: `Exp(-x)` has an exact reference above the largest finite F32, so it is a finite-overflow case and not a subnormal one — §8.5 clause 2 needs "a subnormal before rounding" and clause 3 needs a subnormal operand, and neither is present; §8.1 guarantees INF is supported with fast math disabled, so `+inf` is available as the result; `1 + inf` is `inf`; and `x / inf` for finite negative `x` is exactly `-0.0` by IEEE sign rules, an exact value requiring no rounding, delivered under Table 8.1's correctly rounded `x / y`. No subnormal is produced anywhere in the computation and no flush is involved. **Inference — and this is the one place the SiLU vertical's correctness depends on Gap 2 directly.** The whole route rests on §8.1's INF guarantee, which §8.1 itself conditions on fast math being disabled; under fast math the handling of INF "as inputs or outputs is undefined", and the `-0.0` would have no basis at all. The L3′ record's parenthetical that "the qualified Metal row flushes subnormals to zero anyway" describes a route that agrees with this one by coincidence rather than the route that produces the result.

## What the three verticals may now write, and what they may not

**Inference — the honest state of D-4, per function.**

| Function | What Table 8.1 states | What still blocks a Tiler contract | Residual class |
| --- | --- | --- | --- |
| `exp` at F32 | `<= 4 ulp` under Apple's ULP definition | Gap 1: the metric is a different key and the conversion factor depends on a reading of an ambiguous sentence. Gap 3: no exceptional-value, signed-zero, or subnormal rule. | ordinary-domain bound available *after* a stated derivation; exceptional values `Unknown` |
| `rsqrt` at F32 | Correctly rounded | Gap 4: §8.2 permits round-toward-zero, and the carrier can only spell ties-to-even. Gap 3 as above. | `Unknown` normatively; the ordinary-domain form is otherwise metric-free |
| `x / y`, `1.0 / x`, `x + y`, `x - y`, `x * y`, `fma` at F32 | Correctly rounded | Gap 4 and Gap 3 | as above |
| every entry | — | Gap 2: applicability rests on the §1.6.3 equivalence, an inference rather than a quotation | qualified, not blocked |

**Proposal — the shape a vertical's adoption would take, stated so it can be refuted rather than reinvented, and deliberately not performed here.** Mint `apple::msl-ulp@1` as a second `AccuracyMetricKey` carrying Apple's definition verbatim; register a `RegisteredImplication::ScaledMetric { from: apple::msl-ulp@1, to: tiler::ulp-reference-gap@1, factor }` whose `NormativeDefinitionRef` basis names the reading it adopts and the domain it is proved over, with `factor` no smaller than the table in Gap 1 permits for that reading; state the ordinary-domain clause over a domain excluding the finite-overflow region, since `tiler::ulp-reference-gap@1` is undefined there and SiLU reaches it; and record the whole thing as a `ConformanceEvidence` of class `NormativeGuarantee`, whose nine required fields — scope, target, implementation identity, toolchain, optional device, oracle, corpus, and digest — this record supplies for all but the target and toolchain the vertical selects. `NormativeGuarantee` is one of the three classes `discharges_hard_requirement` admits, which is what makes this route worth taking rather than measuring.

**Non-goal, restated because it is load-bearing.** This record performs none of the above. Promoting a table entry to a Tiler contract inside it would be promoting a number across a metric reconciliation and a rounding-mode question that the record itself is the evidence for.

## Consequences for Q-SEM-004 and D-4

**Inference — Q-SEM-004 asks for "an operation/dtype/accuracy allowlist with reference *and backend* conformance evidence", and this record supplies part of the backend half.** For `exp`, `rsqrt`, and division at F32 on a Metal target compiled under the governed baseline, the backend evidence is now a quoted normative guarantee with a named revision, digest, and page rather than an `Unknown` — qualified by four gaps, two of which (1 and 4) must be closed by a derivation before any tuple is selected. The reference half is untouched: nothing here evaluates a reference oracle or exercises `crates/tiler-reference/src/accuracy.rs`, and Q-SEM-004 does not close.

**Inference — D-4's closure condition is partially met.** D-4 closes on "an applicable normative guarantee, an exhaustive evaluation over a tractable domain, or a proof". An applicable normative guarantee exists and is recorded; the applicability rests on the §1.6.3 equivalence (Gap 2) and the numbers require a metric derivation (Gap 1) or a rounding-mode statement (Gap 4) before they may be written down. The support matrix's transcendentals row and the L3′ record's D-4 entry are updated to say that rather than to say the question closed.

## What this record does not decide

- **Whether Tiler declares any fact on the Metal target profile.** ADR 0076's honourability declaration and `MetalTargetFacts` are `implementation/metal` and `contracts/decisions` work.
- **Which reading of Apple's ULP sentence to adopt, and therefore which factor.** Stating both readings and their factors is the deliverable; choosing one is the vertical's derivation, and it may equally choose to ask Apple.
- **Anything about `half` or `bfloat`.** Tables 8.3, 8.4, and 8.5 exist, are cited by page, and are not transcribed. A narrower profile must read them; Table 8.4's correctly rounded divisions against Table 8.5's `<= 0.6 ulp` show that dtype and math mode move independently.
- **Anything about delivered values.** No device was opened. A normative guarantee is a promise about an implementation, not an observation of one, and the two remain different evidence classes.
- **The reciprocal `evidence` edge on the contracts this record informs.** `informs` and `evidence` are independent predicates under the [metadata contract](../../document-metadata.md#typed-relationships), and the corpus does not maintain the reciprocal: of the 117 research-to-contract `informs` edges present with this record added, 57 — this record's two included — have no matching `evidence` entry on the target contract. No edit is owed and none is made here; whoever next edits `docs/numerical-semantics.md` under `contracts/numerics` may add one.

## The checks, and that they can say no

Each check below was run against a case that must fail, and failed.

1. **The two-revision equality is content-sensitive.** The normalized §8.4 pipeline gives `99202c88…` for both retained PDFs. Rerunning it over MSL 4 with `| sed 's/exp *<= 4 ulp/exp <= 5 ulp/'` interposed — one entry rewritten, nothing else — gives `d0d58216…`. A check that reported equality regardless of content would not have moved.
2. **The table extraction reads rows rather than reporting a constant.** `pdftotext -layout <MSL 4.1> - | sed -n '/^Table 8.1. Accuracy of single-precision/,/^Table 8.2 describes/p'` and then `grep -cE "^ *<name> +(<=|Correctly|0 ulp)"` returns 1 for `exp` and 1 for `rsqrt`, and **0** for `divide`, `abs`, `max`, `min`, `fmax3`, and `fmedian3` — every one of which is a math function MSL 4.1 Table 6.4 defines in the standard library (page 205–208) and none of which Table 8.1 gives a row. It also returns 0 for `erf` and `cbrt`, which the document does not contain at all. The population the table covers is therefore a proper subset of the functions the language exposes, and the check distinguishes the two.
3. **The superscripts survive extraction only as a hazard.** `pdftotext` renders `2⁻¹²⁶` as `2-126`; pages 369, 371, and 375 of MSL 4.1 were rendered to PNG and read directly to confirm every superscripted entry and the ULP definition. The rendered page 371 also shows `<= 2.5 ulp` for Table 8.2's `x / y`, which is what caught the ticket's `0.6 ulp` misattribution to Table 8.5.
4. **The absence claim in Gap 4 names its command.** `grep -rn "round toward zero\|round-toward-zero\|ties to even\|ties-to-even" docs/research/apple-targets/ docs/backends/metal.md docs/research/target-profiles/` returns nothing at `ccba2d5`, which is the exact check behind "nobody had recorded §8.2's permissiveness" and is one line for a reader to refute.
