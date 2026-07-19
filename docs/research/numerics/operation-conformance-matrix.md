# Initial operation conformance matrix

**Status:** contract audit; implementation support remains unmeasured  
**Ticket:** `numerical-policy-contract`

This matrix records which numerical questions the semantic operation contract
must answer. It is not a claim that every listed operation is implemented in
the first executable slice.

## Legend

- **fixed:** an accepted Tiler-wide invariant;
- **typed family:** selected explicitly by the operation's semantic type;
- **resolved permission:** strict meaning is fixed and a separately authorized
  permission may admit another implementation;
- **operation contract:** mandatory, but specialized per operation;
- **open:** a concrete contract or supported subset remains to be chosen;
- **N/A:** the dimension is not meaningful for the operation.

## Cross-operation invariants

| Dimension | Initial contract | Status |
|---|---|---|
| Value and computation dtype | Resolved before semantic optimization; no ambient promotion or autocast | fixed |
| Conversion boundary | Explicit typed conversion, preserved across fusion/materialization changes | fixed |
| Floating exception observation | Value results only; no observable flags or traps | fixed |
| NaN result bits | Portable-bitwise arithmetic produces a versioned canonical quiet NaN | fixed |
| Subnormals | Input handling and result handling resolved independently | fixed |
| Numerical freedoms | Program ceiling plus resolved per-operation permissions | fixed |
| Value assumptions | Compiler-proven or runtime-validated before use for legality | fixed |
| Backend result | Exact native, exact emulation, declared-relaxation-only, or unsupported | fixed |
| Determinism | Explicit scope; initial practical guarantee is plan determinism | fixed |
| Explain and identity | Resolved contract, consumed permissions, target realization, and topology are recorded | fixed |

## Operation families

| Operation family | Strict semantic boundary | Separately selectable or permitted | Remaining work |
|---|---|---|---|
| Constants, views, bit-preserving copies | Preserve declared/selected bits, including NaN payload and signed zero | N/A | Supported dtype/encoding subset |
| `Add`, `Subtract`, `Multiply` | Resolved homogeneous computation/result type, round-to-nearest-ties-even, and distinct operation rounding | Reassociation, operand permutation where capable, signed-zero/NaN assumptions, subnormal handling | Per-dtype backend vectors; built-in capability table |
| Integer add, subtract, multiply | Explicit wrapping, saturating, checked, or widening family; no ambient overflow behavior | Required-no-overflow only with a discharged proof or runtime-validation obligation | Initial supported family/width matrix; MIN/MAX and sub-byte vectors |
| Floating `Divide` | IEEE resolved value results under round-to-nearest-ties-even, including zero/NaN/infinity cases | Reciprocal transform and approximation are independent permissions | Exact per-dtype backend behavior and reciprocal error contracts |
| Integer division/remainder | Explicit signed truncating, floor, Euclidean, ceiling, or unsigned family; nonzero divisor and representable quotient are proven/validated preconditions; standalone `MIN rem -1` is valid zero | Exact division adds a divisibility precondition; future total/masked families remain distinct | Initial supported family/width matrix and validation realization |
| `Fma` | Required correctly rounded fused result under round-to-nearest-ties-even | No decomposition unless exact emulation or an authorized relaxation proves compatibility | Per-dtype native/emulated support |
| `Multiply` then `Add` | Two semantic rounding boundaries | Contraction permission may admit FMA; reassociation does not imply contraction | Backend flag and generated-code verification |
| Numeric conversions | Typed conversion family defines rounding and exceptional-value behavior | Only freedoms named by that conversion contract | Concrete initial family matrix, especially float-to-int and quantization |
| Affine `Quantize` / `Dequantize` | Positive finite scales, in-range codes/zero points, widened difference, explicit evaluation dtype/order, strict subnormal preservation, clamp then nearest-even conversion, endpoint/infinity saturation; strict `Quantize` rejects NaN | Alternative NaN mappings, computation dtypes, rounding, or subnormal behavior only through a separately resolved conversion family | Initial dtype/profile subset and validation realization |
| Affine `Requantize` | Source decode followed by destination encode in an explicit intermediate dtype, preserving both conversion boundaries | Direct integer realization only with an equivalence proof | Initial product-profile subset |
| Integer `Rescale` | Named multiplier/shift widths, integer rounding algorithm, zero-point interpretation, clip range, and intermediate widths | Approximate result only under a separately declared bounded-error family | Initial exact algorithms and imported-dialect profiles |
| `Minimum` / `Maximum` | NaN-propagating; deterministic `-0.0 < +0.0` | NaN-absence and signed-zero relaxations remain independent | Exact Metal fixup and cost measurement |
| `MinimumNumber` / `MaximumNumber` | Prefer the number when exactly one operand is NaN; deterministic zero ordering | Signed-zero and NaN assumptions remain independent | Exact Metal fixup and reduction conformance |
| Transcendentals | Per-operation accuracy, domain, special-value, and subnormal contract | Approximate implementation only when its bound satisfies the contract or consumes permission | Initial metric vocabulary and supported operation subset are open |
| `Select` and bit-selecting operations | Preserve the selected operand's bits; predicate semantics are explicit | Speculation of arms requires proof that doing so is semantically harmless | Initial predicate/dtype subset |
| Sum/product/logical reductions | Resolved accumulator/result types, empty result, seed, padding capability, and order contract | Reassociation and permutation are independent; topology is physical | Built-in capability table and target topology conformance |
| Extrema reductions | Named extrema scalar family plus explicit empty result-or-error, seed, padding capability, and order contract | Same independent order and exceptional-value permissions | All-NaN, signed-zero, empty, and tree-shape validation |

## Minimal adversarial corpus

Each supported floating dtype uses exact bit-pattern inputs where representable.

| Concern | Required examples |
|---|---|
| Rounding boundaries | Halfway values; overflow/underflow edges; fused versus separately rounded multiply-add; cancellation |
| NaN | qNaN and sNaN in every operand position; one and several payloads; arithmetic production versus bit-preserving selection |
| Infinity | Both signs; finite overflow; `0 * inf`; `inf - inf`; division by zero and infinity |
| Signed zero | Both operand orders of `-0`/`+0`; add/subtract/multiply/divide; extrema; clamp/ReLU patterns |
| Subnormals | Smallest/largest subnormal as input; smallest normal boundary; newly produced subnormal; all four input/result preserve/flush combinations |
| Conversion | Exact, halfway, just-out-of-range, NaN, infinities, signed zero, subnormal, and integer extrema |
| Quantization | Every code endpoint; widened subtraction; zero-point neighborhood; parameter-grid sentinels; invalid scale/code/zero point; saturation thresholds; QDQ and requantize double-rounding witnesses |
| Reduction order | Three-element reassociation witness; operand permutations; serial/SIMD/threadgroup/multi-pass trees; repeated executions |
| Empty and seeded reduction | Empty result; identity-less empty rejection; dynamic non-empty guard; non-identity seed included exactly once; singleton `[-0]` without injected `+0` |
| Extrema reduction | All-NaN; one numeric plus NaNs; opposite-signed zeros in every order and tree; infinities |
| Assumption guards | Passing and failing data scan; no dependent work before validation; alternate-plan/fallback selection |

Positive and negative tests accompany every rewrite: one witness where the rule
is legal, and one adversarial input demonstrating why each unmet permission or
precondition rejects it.

## Backend compiler verification

Compiler flags are a realization mechanism, not semantic evidence by
themselves. Each supported backend/toolchain tuple uses this protocol:

1. Canonical operation contracts select a versioned, explicit flag bundle; no
   compiler numerical default is inherited.
2. A compile probe verifies every required flag is accepted by the selected
   compiler and records compiler, linker, SDK, language, and deployment
   fingerprints.
3. Generated source is audited so operation-specific helpers and intrinsics do
   not exceed resolved permissions. A global flag is rejected if it relaxes any
   affected operation beyond its contract.
4. The completed artifact records the exact flags, generated source digest,
   toolchain fingerprint, operation contracts, and declared feasibility class.
5. Device conformance executes the adversarial vectors for every claimed
   operation/dtype/target profile. Source inspection or successful compilation
   alone cannot establish numerical behavior.
6. Optimized versus deliberately unfused/materialized references are compared
   under the selected conformance level and reduction topology scope.
7. An untested toolchain or failed probe is `Unsupported` or `Unknown`; it is
   never promoted to exact support from similar version numbers.

Measurements populate a versioned backend conformance table. Until a cell has
toolchain- and device-specific evidence, the scheduler cannot claim exact
native support for it.

## Remaining bounded decisions

The unresolved work is now concrete rather than architectural:

1. choose the first vertically supported operation/dtype combinations;
2. define initial transcendental error metric types and operations;
3. finish float-to-integer conversion families;
4. enumerate algebraic capabilities for each built-in operation;
5. run and publish Metal flag/intrinsic/device conformance measurements; and
6. choose ergonomic frontend policy presets, which expand into the already
   resolved canonical representation.
