# Transcendental accuracy precedents

**Status:** research basis for ADR 0042

## Question

What minimum semantic vocabulary lets Tiler state useful transcendental
accuracy requirements without inventing guarantees that a backend does not
make?

## Primary-source findings

### Khronos OpenCL

The [OpenCL C specification](https://registry.khronos.org/OpenCL/specs/unified/html/OpenCL_C.html#relative-error-as-ulps)
defines ULP using the spacing around the infinitely precise reference result,
including an explicit rule at representable values and zero. Its ordinary
function table gives normative per-function bounds, while unsafe and native
families may use different domains, absolute-error clauses, input-dependent
ULP formulas, or implementation-defined behavior. Edge-case behavior is
specified separately from ordinary error bounds.

This is direct evidence that one scalar `max_ulp` or one `fast` bit cannot
describe a mature GPU math interface.

Jean-Michel Muller's [On the definition of
ulp(x)](https://inria.hal.science/inria-00070503/document) shows that several
plausible definitions differ at radix/binade boundaries and overflow. Error is
scaled at the exact reference, not the approximation; otherwise a farther
approximation can appear better. In the adopted reference-gap definition, an
exact power of two retains the smaller predecessor gap and the scale increases
immediately above it. Fractional bounds remain meaningful because the numerator
is a real distance rather than an integer count of representable steps.

Correct rounding is not equivalent to a
half-ULP bound: at an exact halfway case both neighbors satisfy the distance
bound, but only the tie-rule-selected neighbor is correctly rounded.

### Metal

Apple's [Metal math-mode API](https://developer.apple.com/documentation/metal/mtlmathmode)
distinguishes safe, relaxed, and fast modes, while the Metal Shading Language
specification separately describes fast and precise elementary functions and
their operation-specific accuracy. Current tables include ULP bounds,
absolute-error regions, input-dependent formulas, and undefined regions.
Compiler math mode, FP32 function selection, and contraction controls are
orthogonal enough that one compiler switch does not express Tiler's full
contract.

These names are backend implementation profiles. They become usable for a
Tiler contract only through the applicable normative versioned specification
and explicit exceptional-value/subnormal behavior.

### CUDA

The [CUDA C Programming Guide mathematical functions appendix](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/mathematical-functions.html)
uses per-function and sometimes piecewise error descriptions. NVIDIA explicitly
describes current ULP tables as maximum errors observed through extensive but
non-exhaustive tests and says they are not guaranteed bounds. Such data is
valuable empirical qualification, not proof of a portable worst case.

### Compiler IRs

[StableHLO](https://openxla.org/stablehlo/spec#accuracy) states that it does not
guarantee numerical accuracy for individual operations. MLIR math operations
and LLVM approximate-function permissions likewise do not supply a complete
portable error bound. Importers must therefore attach or resolve a Tiler
contract; the source operation name alone is insufficient evidence.

### Math libraries

[SLEEF](https://sleef.org/purec.xhtml) exposes families named by bounds such as
1.0 ULP and 3.5 ULP, with separate domains and special-case behavior. The
[CRlibm paper](https://www.numdam.org/article/ITA_2007__41_1_85_0.pdf) and
[RLIBM-32](https://arxiv.org/abs/2104.04043) demonstrate the stronger claim of
correctly rounded results for enumerated functions, formats, rounding modes,
and complete input sets. These claims are quantified by function and format,
not by a library-wide `precise` label.

Faithful rounding is also distinct: an exact representable result is preserved;
otherwise either adjacent floating-point value bracketing the exact result is
allowed. It should not be silently translated into a generic `<= 1 ULP`
contract whose ULP definition may differ.

### Combined tolerances

[PyTorch `allclose`](https://pytorch.org/docs/stable/generated/torch.allclose.html)
uses the additive predicate `|y-r| <= atol + rtol*|r|`. Intel's
[floating-point function accuracy controls](https://www.intel.com/content/www/us/en/docs/dpcpp-cpp-compiler/developer-guide-reference/2025-1/fimf-absolute-error-qimf-absolute-error.html)
instead permit satisfaction of the relative requirement or the absolute
requirement. The latter is a disjunction, not the additive formula. A generic
`abs_rel` label would conflate two different result sets, so ADR 0042 names the
additive form and supplies explicit `AnyOf`/`AllOf` composition.

Current GPU specifications also contain input-dependent bounds such as an ULP
tolerance that grows with input magnitude, and textual constants that are not
exact rationals. ADR 0042 deliberately keeps the first generic algebra to
constant rational bounds. Such a formula uses an immutable named behavior
profile until a separately justified exact bound-expression language exists.
Similarly, a guarantee stated only on a limited interval is usable only when a
proof restricts the semantic input domain to that interval; “larger outside”
does not become an unbounded implicit fallback.

## Facts, inferences, and adopted design

Facts:

- production interfaces use correctly rounded, ULP, absolute, relative, and
  piecewise guarantees;
- ordinary-domain accuracy and exceptional values are commonly separate;
- backend/compiler labels do not consistently carry portable bounds; and
- empirical maximum error is not a proof of a worst-case guarantee.

Inference:

- Tiler needs a typed result-set contract over an immutable exact reference,
  not an implementation-quality adjective;
- ULP and named behavior require versioned definitions; and
- conformance evidence must be distinct from semantic and implementation
  identity.

ADR 0042 therefore adopts correctly rounded, faithful, typed piecewise bounded,
and immutable named-behavior contracts. It admits absolute, relative,
explicitly additive absolute-plus-relative, Boolean combinations, and
versioned Muller/OpenCL-style ULP predicates with exact tolerances. Backend
flags and intrinsic names remain physical choices.

## Verification implications

- Use exact arithmetic or certified enclosures with precision escalation for
  bounded cases and independent known-answer vectors for special cases. If an
  enclosure still straddles the threshold, the result is inconclusive rather
  than passing.
- Exhaust formats with tractable finite domains such as f16, bf16, and FP8;
  use adversarial, stratified, and boundary-focused tests for larger formats.
- Label each result as proof, exhaustive, normative guarantee, empirical, or
  unknown. Never promote sampling to a universal bound.
- Test clause boundaries, binade transitions, zero and the smallest positive
  format value, the normal/subnormal boundary where present, overflow
  thresholds, large argument-reduction cases, and every separately specified
  NaN/infinity/signed-zero behavior.
