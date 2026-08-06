---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.float-to-integer-conversion-precedents"
kind: "research"
title: "Floating-point to integer conversion precedents"
topics: ["numerics","conversion","integers"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
adopted_by: ["ADR-0010","ADR-0041"]
ticket: "numerical-policy-contract"
---

# Floating-point to integer conversion precedents

**Status:** adopted decision research supporting ADRs 0010 and 0041

## Traceability

- **Current disposition:** adopted; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md).
- **Adoption:** [ADR 0010](../../decisions/0010-typed-conversion-contracts.md), [ADR 0041](../../decisions/0041-separate-float-to-integer-conversion-families.md).
- **Work record:** [numerical-policy-contract](../../../tickets/numerical-policy-contract.md).
- **Sources:** every primary source below is named by a preserved-source id in
  [the preserved-source record](sources/README.md); run `sources/verify-sources.sh` to check the
  bytes against their recorded digests. Provenance hardening record:
  [preserve-the-float-to-integer-conversion-precedent-sources](../../../tickets/preserve-the-float-to-integer-conversion-precedent-sources.md),
  which closed seven citations into ids, and
  [preserve-the-pytorch-conversion-platform-variation-source](../../../tickets/preserve-the-pytorch-conversion-platform-variation-source.md),
  which closed the eighth claim that had never been cited at all. Every claim in
  "Existing contracts" below now names a source; there are eight claims and eight resolutions.


## Finding

A floating-point source and integer destination do not determine a conversion.
Rounding, ordered overflow, infinities, NaN, exactness, and subnormal input
handling are independently observable. In particular, saturation does not
mathematically determine a NaN result because NaN is unordered.

## Existing contracts

- LLVM `fptosi`/`fptoui` round toward zero and produce poison when the rounded
  value is not representable. Its separate saturating intrinsics clamp ordered
  values and explicitly map NaN to zero.
- WebAssembly separates trapping truncation from total saturating truncation;
  its total form also maps NaN to zero.
- Rust `as` uses truncation, endpoint saturation, and NaN-to-zero as a fully
  defined language-specific totalization.
- C++ makes an unrepresentable result undefined. StableHLO leaves it TBD, and
  PyTorch documents platform variation. These contracts cannot be imported as
  portable results outside a proven valid domain.
- PTX exposes multiple rounding directions and clamps many out-of-range
  results, but its NaN result varies with source/destination widths. A native
  conversion instruction therefore cannot supply Tiler semantics implicitly.

Primary sources, each named by its id in [the preserved-source record](sources/README.md), which
holds the owner, edition, retrieval route, licence, and verdict for every one:

- LLVM floating-point to integer instructions — `llvm-langref-llvmorg-22.1.8`, the `fptosi`/`fptoui`
  sections of `llvm/docs/LangRef.rst` at `llvmorg-22.1.8`.
- LLVM saturating conversions — `llvm-langref-llvmorg-22.1.8`, the "Saturating floating-point to
  integer conversions" section of the same file, covering `llvm.fptoui.sat` and `llvm.fptosi.sat`.
- WebAssembly numeric execution — `wasm-core-numerics-wg-3.0` for the `trunc` and `trunc_sat`
  operators, and `wasm-core-instructions-wg-3.0` for the sentence that turns a partial operator
  into a trap.
- Rust numeric casts — `rust-reference-operator-expr-rust-1.97.1`, the `Numeric cast` section of
  the Rust reference as shipped with Rust 1.97.1.
- C++ floating-integral conversions — `iso-cpp-working-draft-n5054`, §7.3.11 [conv.fpint] of the
  N5054 working draft. Metadata-only: ISO/IEC holds the copyright and grants no redistribution, so
  the record carries a digest and an acquisition route rather than bytes.
- StableHLO convert — `stablehlo-spec-v1.18.0`, the `convert` section of `docs/spec.md` at
  `v1.18.0`.
- PyTorch dtype conversion — `pytorch-tensor-docs-v2.13.0`, the `Tensor.to` docstring's casting
  note in `torch/_tensor_docs.py` at `v2.13.0`, which renders as the published
  `docs.pytorch.org/docs/2.13/generated/torch.Tensor.to.html` page.
- PTX conversion instructions — `nvidia-ptx-isa-cuda-13.3.0`, the `cvt` section of the PTX ISA as
  published in the CUDA 13.3.0 documentation archive. Metadata-only, for the same reason.

**Re-check, 2026-08-06.** All seven citations this list then held were re-checked against the
preserved or re-acquired bytes when the ids replaced the live URLs they previously carried — the
PyTorch row below arrived later the same day and has its own paragraph — and **every claim in
the "Existing contracts" list above held**; nothing was softened, corrected, or found to have moved.
The per-claim quotations, and two precisions the check surfaced without changing a conclusion, are
recorded beside the sources in [the preserved-source record](sources/README.md) rather than repeated
here. The two precisions are worth knowing before reading the list above too literally: StableHLO
and C++ each *do* specify the rounding — both truncate toward zero — and leave only the
unrepresentable result unspecified or undefined, so "leaves it TBD" and "makes it undefined" are
claims about the overflow case and not about the whole conversion. That is what makes these sources
a contrast set rather than a list. Five of the six systems fix the rounding identically — LLVM,
WebAssembly, Rust, C++, and StableHLO all round toward zero, and PTX alone exposes a choice of four
integer rounding directions — so what actually separates them is the residue: a poison value, a
trap, an endpoint saturation, a NaN-to-zero totalization, a width-dependent NaN, an undefined
result, and a deferred one are seven answers to the question the shared rounding rule leaves open.

**The eighth claim, closed 2026-08-06.** The re-check above covered the seven citations the list
then held, but the "Existing contracts" list makes *eight* claims: the fourth bullet's middle
clause, "PyTorch documents platform variation", had never been cited, in either the pre- or
post-hardening list. It was acquired rather than corrected, which was not the expected outcome —
the ticket that filed the gap predicted the sentence would have to go. **The claim holds, and the
bullet needed no rewording.** PyTorch's `Tensor.to` documentation states that if a truncated float "cannot fit into
the target type (e.g., casting `torch.inf` to `torch.long`), the behavior is undefined and the
result may vary across platforms", and that note renders on the published documentation site, so
this is documented variation and not merely observed variation. Two things follow, neither of which
disturbs the tally above. First, PyTorch does not reach that undefinedness on its own: the same
note attributes it to "C++ type conversion rules", so PyTorch is C++'s residue imported by
reference, which is why it sits in the same bullet rather than becoming an eighth residue —
seven answers, now from seven systems. Second, PyTorch fixes the rounding the same way the others
do, truncating the fractional part, so it joins the majority the paragraph above describes rather
than widening the disagreement. What it adds to the record is a fact about *ecosystem reach*: the
undefined-overflow contract is not confined to systems programming languages, it is what a tensor
framework hands its users through `Tensor.to`, which is precisely the boundary Tiler's strict
portable family exists to replace.

## Boundary details

Validation is defined against the rounded mathematical integer, not a naive
floating comparison with an integer endpoint converted into the source dtype.
An endpoint may not be exactly representable as a float, and values such as an
unsigned input in `(-1, 0)` validly truncate to zero.

Signed zero converts numerically to integer zero. NaN absence must be checked
independently of ordered range comparisons. Exact conversion additionally
requires an integral finite source value. A backend poison-producing cast is
usable only after all required preconditions are proven or enforced.

## Tiler implication

The strict portable family rejects NaN and every unrepresentable rounded
result. Ordered saturation clamps finite overflow and infinities but does not
invent a NaN mapping. NaN-to-zero remains useful as a separately named total
compatibility family for Rust, LLVM saturation, and WebAssembly imports. Other
mappings, validity results, and future seeded rounding families remain additive
versioned contracts.
