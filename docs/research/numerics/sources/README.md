# Preserved sources for the dtype catalog

These records keep the documents behind Tiler's recognized dtype catalog reproducible when an upstream URL moves, changes, or disappears. Preservation is licence-aware rather than uniform: a document whose own terms permit dissemination is vendored here byte-for-byte, and a document whose terms do not — or whose terms could not be read — keeps bibliographic identity, a retrieval fingerprint where one exists, and an official acquisition route, with no bytes checked in.

**Retrieval date for every record below: 2026-07-31.** Both preservation waves — the format-defining specifications and the ecosystem precedents added after them — were retrieved on that date.

Digests are deliberately not repeated in this file. `expected-sources.tsv` is the single authority for the expected population, each record's classification, and each SHA-256; `verify-sources.sh` enforces it. A digest written twice drifts silently, and the copy nothing checks is the one a reader would trust.

## What these records claim, and what they do not

- **Preserved source.** The bytes under this directory are an unmodified copy of what the named upstream source served for the named document at the named version. Nothing here is transcribed, reflowed, summarized, or corrected; a specification excerpt in Tiler prose is never the authority, the preserved file is. "Unmodified" is a claim about this repository, not about the delivery path: one record's byte stream carries content the publisher's CDN injects per request, and that record states the consequence for reproducing its digest.
- **Normative guarantee.** What a preserved specification requires of a conforming implementation. Preserving a document does not extend its guarantees to Tiler, nor claim Tiler implements it.
- **Tiler inference.** Every classification, alias rejection, and admission conclusion drawn from these documents lives in the research and ADR text that cites them, not here. This directory records provenance only.

Vendor documentation preserved under an open-source licence (NVIDIA Transformer Engine below) is a primary description by the format's owner, not a standards-body specification; the distinction survives the fact that both are preserved the same way.

## Population boundary

This record closes over every document the dtype research cites as a primary source, in two classes that stay distinguishable because a reader acts differently on each.

**Format-defining sources** are the **numerical-format and quantization-semantics** specifications cited by [the mature dtype taxonomy](../mature-dtype-taxonomy.md) and by the accepted dtype ADRs 0026–0038 (the `dtypes-quantization` catalog group) through their evidence documents — documents that *define* a format's value set, encoding, or quantization contract. Losing one of these would leave an admitted Tiler identity without its normative definition.

**Ecosystem precedents** are cited by the same documents for how an existing system *spells*, *exposes*, or *governs* a format rather than for what a format means: PyTorch, NumPy, JAX, Arrow, the GCC manual's decimal float extension, SPIR-V, WGSL, and the NVIDIA TensorRT accuracy guide, joined by two namespace-governance precedents — the StableHLO compatibility page and the MLIR language reference. None of them defines a value set, so losing one cannot invalidate an admitted identity; they carry the alias-resolution and naming evidence instead — that `complex64` means different things in different ecosystems, that a PyTorch shell dtype proves representation can precede arithmetic — which is why they are worth preserving rather than citing from a live URL.

Keeping both classes in one manifest is deliberate: the check's value comes from a single declared population, and two manifests would each be able to agree with themselves. A further wave of citations extends this record and restates the boundary here rather than starting a third one.

Apple Metal language and capability evidence stays under [its own record](../../apple-targets/sources/README.md).

## Filename map

| File under this directory | Source id |
| --- | --- |
| `riscv-isa/riscv-unprivileged-20260120.pdf` | `riscv-unprivileged-isa-20260120` |
| `riscv-isa/LICENSE` | `riscv-isa-manual-license` |
| `stablehlo-v1.18.0/spec.md` | `stablehlo-spec-v1.18.0` |
| `stablehlo-v1.18.0/compatibility.md` | `stablehlo-compatibility-v1.18.0` |
| `stablehlo-v1.18.0/LICENSE` | `stablehlo-license-v1.18.0` |
| `llvm-project-llvmorg-22.1.8/BuiltinTypes.td` | `mlir-builtin-types-llvmorg-22.1.8` |
| `llvm-project-llvmorg-22.1.8/QuantBase.td` | `mlir-quant-base-llvmorg-22.1.8` |
| `llvm-project-llvmorg-22.1.8/LangRef.rst` | `llvm-langref-llvmorg-22.1.8` |
| `llvm-project-llvmorg-22.1.8/mlir-LangRef.md` | `mlir-langref-llvmorg-22.1.8` |
| `llvm-project-llvmorg-22.1.8/LICENSE.TXT` | `llvm-project-license-llvmorg-22.1.8` |
| `onnx-v1.22.0/IR.md` | `onnx-ir-v1.22.0` |
| `onnx-v1.22.0/Operators.md` | `onnx-operators-v1.22.0` |
| `onnx-v1.22.0/int4.md` | `onnx-int4-v1.22.0` |
| `onnx-v1.22.0/int2.md` | `onnx-int2-v1.22.0` |
| `onnx-v1.22.0/LICENSE` | `onnx-license-v1.22.0` |
| `onnx-v1.22.0/NOTICE` | `onnx-notice-v1.22.0` |
| `dlpack-v1.3/dlpack.h` | `dlpack-header-v1.3` |
| `dlpack-v1.3/c_api.rst` | `dlpack-c-api-v1.3` |
| `dlpack-v1.3/LICENSE` | `dlpack-license-v1.3` |
| `transformer-engine-v2.17/nvfp4.rst` | `nvidia-te-nvfp4-v2.17` |
| `transformer-engine-v2.17/LICENSE` | `nvidia-te-license-v2.17` |
| `tosa-1.0.1/tosa_spec_1_0_1.html` | `tosa-spec-1.0.1` |
| `pytorch-v2.13.0/tensor_attributes.md` | `pytorch-tensor-attributes-v2.13.0` |
| `pytorch-v2.13.0/complex_numbers.md` | `pytorch-complex-numbers-v2.13.0` |
| `pytorch-v2.13.0/ScalarType.h` | `pytorch-scalar-type-v2.13.0` |
| `pytorch-v2.13.0/LICENSE` | `pytorch-license-v2.13.0` |
| `pytorch-v2.13.0/NOTICE` | `pytorch-notice-v2.13.0` |
| `numpy-v2.5.1/routines.dtypes.rst` | `numpy-dtype-classes-v2.5.1` |
| `numpy-v2.5.1/LICENSE.txt` | `numpy-license-v2.5.1` |
| `jax-v0.11.0/9263-typed-keys.md` | `jax-typed-keys-v0.11.0` |
| `jax-v0.11.0/LICENSE` | `jax-license-v0.11.0` |
| `arrow-25.0.0/Columnar.rst` | `arrow-columnar-25.0.0` |
| `arrow-25.0.0/CanonicalExtensions.rst` | `arrow-canonical-extensions-25.0.0` |
| `arrow-25.0.0/Schema.fbs` | `arrow-schema-fbs-25.0.0` |
| `arrow-25.0.0/LICENSE.txt` | `arrow-license-25.0.0` |
| `arrow-25.0.0/NOTICE.txt` | `arrow-notice-25.0.0` |
| `gcc-16.1.0/gcc.pdf` | `gcc-manual-16.1.0` |
| `spirv-1.6-rev7/SPIRV.html` | `spirv-unified-1.6-rev7` |
| `wgsl-20260716/CRD-WGSL-20260716.html` | `wgsl-crd-20260716` |
| `wgsl-20260716/w3c-software-and-document-license-2023.html` | `w3c-document-license-2023` |

Six source ids retain no bytes here: `ieee-754-2019`, `ocp-ofp8-v1.0`, `ocp-mx-v1.0`, `posit-standard-2022`, `nvidia-ptx-isa-cuda-13.3.0`, and `nvidia-tensorrt-accuracy-11.2.1`.

## Vendored records

### `riscv-unprivileged-isa-20260120`, `riscv-isa-manual-license`

- **Document:** The RISC-V Instruction Set Manual, Volume I: Unprivileged Architecture — "Version 20260120: Official Release", 696 pages.
- **Owner:** RISC-V International.
- **Retrieved from:** `https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf` (4 580 174 bytes, `application/pdf`).
- **Identity:** that URL carries no version segment, so identity rests on the document's own version string, printed on its title page, together with the recorded digest — not on the URL.
- **Licence:** Creative Commons Attribution 4.0 International, stated in the document itself: "This document is released under a Creative Commons Attribution 4.0 International License." The full licence text is preserved as `riscv-isa/LICENSE`, taken from `riscv/riscv-isa-manual` at commit `310a111489a0bad6e60ef4cbfba574417c6f825f`.
- **Verdict:** vendored; CC BY 4.0 permits redistribution with attribution.
- **Cited for:** `tiler::bf16@1` in [ADR 0036](../../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md) and the [dtype identity admission policy](../dtype-identity-admission-policy.md). Chapter 25, "BF16 Extensions for BFloat16-precision Floating-Point, Version 1.0", carries the operand-format contract at §25.3.1; the preface ratified-extension table lists BF16 1.0 as `Ratified`.

### `stablehlo-spec-v1.18.0`, `stablehlo-compatibility-v1.18.0`, `stablehlo-license-v1.18.0`

- **Owner:** OpenXLA / the StableHLO project.
- **Pinned to:** `openxla/stablehlo` tag `v1.18.0`, commit `e6f81ebd06b3509f2c7fa6175430aadbd4d724ca`.
- **Documents and upstream paths:**
  - `spec.md` ← `docs/spec.md`, the StableHLO Specification.
  - `compatibility.md` ← `docs/compatibility.md`, the source of the `openxla.org/stablehlo/compatibility` page. It states the versioning scheme and the compatibility window — five years of backward compatibility and two years of forward compatibility for portable artifacts, with forward deserialization limited to artifacts that do not use features introduced since the older version.
- **Retrieved from:** `https://raw.githubusercontent.com/openxla/stablehlo/v1.18.0/docs/spec.md` and `.../docs/compatibility.md`.
- **Licence:** Apache-2.0, preserved as `stablehlo-v1.18.0/LICENSE`. The repository has no `NOTICE` file at that tag (`https://raw.githubusercontent.com/openxla/stablehlo/v1.18.0/NOTICE` returned HTTP 404 on the retrieval date).
- **Verdict:** vendored.
- **Cited for:** element types, quantized tensor types, and the token type, in the taxonomy and in [ADR 0028](../../../decisions/0028-recognize-sub-byte-integers.md), [ADR 0037](../../../decisions/0037-parameterize-complex-dtype-identity.md), and the quantization research. The taxonomy's `openxla.org/stablehlo/spec#…` links render this same file from the project's live branch, which is a mutable view of it rather than a second authority. `compatibility.md` is cited by the [dtype identity admission policy](../dtype-identity-admission-policy.md) as a namespace-governance precedent — versioned opset evolution with an explicit compatibility window — not as a format definition.
- **Boundary:** the compatibility page is pinned to the same commit as the specification deliberately, so one project does not appear here at two versions.

### `mlir-builtin-types-llvmorg-22.1.8`, `mlir-quant-base-llvmorg-22.1.8`, `llvm-langref-llvmorg-22.1.8`, `mlir-langref-llvmorg-22.1.8`, `llvm-project-license-llvmorg-22.1.8`

- **Owner:** the LLVM Project.
- **Pinned to:** `llvm/llvm-project` tag `llvmorg-22.1.8`, commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`.
- **Documents and upstream paths:**
  - `BuiltinTypes.td` ← `mlir/include/mlir/IR/BuiltinTypes.td`, the definition file the rendered MLIR Builtin dialect types page is generated from — `Float4E2M1FNType`, `Float8E8M0FNUType` and siblings, `IntegerType`, `ComplexType`.
  - `QuantBase.td` ← `mlir/include/mlir/Dialect/Quant/IR/QuantBase.td`, the definition file behind the rendered Quant dialect page, including its per-layer, per-axis, and blockwise granularity text.
  - `LangRef.rst` ← `llvm/docs/LangRef.rst`, which distinguishes `x86_fp80` and `ppc_fp128` from IEEE `fp128`.
  - `mlir-LangRef.md` ← `mlir/docs/LangRef.md`, the MLIR language reference, source of the `mlir.llvm.org/docs/LangRef/` page. It states that each dialect carries a unique namespace prefixed to every attribute, operation, and type it defines, and gives the dialect-type and dialect-attribute grammars that make that namespace part of the printed identity.
- **Licence:** Apache-2.0 with LLVM Exceptions, preserved as `LICENSE.TXT`.
- **Verdict:** vendored. Preserving the `.td`/`.rst`/`.md` definition files rather than the rendered `mlir.llvm.org` and `llvm.org` pages pins the generated documentation to its source at an exact commit; the rendered pages track the project's main branch and are not a stable identity.
- **Cited for:** the nominal FP4/FP6/FP8 identities, arbitrary-width integers, structural complex, quantization granularity, and target ABI float formats throughout the taxonomy and the quantization research. `mlir-LangRef.md` is cited by the [dtype identity admission policy](../dtype-identity-admission-policy.md) for the namespace-ownership precedent — a custom type lives in its owning dialect's namespace — not for any format definition.
- **Boundary:** the two `LangRef` files are different documents from different projects that happen to share a name, which is why the MLIR one carries an `mlir-` prefix here; `LangRef.rst` is LLVM IR's and `mlir-LangRef.md` is MLIR's.

### `onnx-ir-v1.22.0`, `onnx-operators-v1.22.0`, `onnx-int4-v1.22.0`, `onnx-int2-v1.22.0`, `onnx-license-v1.22.0`, `onnx-notice-v1.22.0`

- **Owner:** the ONNX project (LF AI & Data).
- **Pinned to:** `onnx/onnx` tag `v1.22.0`, commit `2bb50465112feca9003e1ed654d77f01ff1415ca`.
- **Documents and upstream paths:**
  - `IR.md` ← `docs/IR.md`, rendered as `onnx.ai/onnx/repo-docs/IR.html`; carries the element-type table including `int2`/`uint2`, `int4`/`uint4`, the FP8 variants, `float4e2m1`, complex, and string tensors.
  - `Operators.md` ← `docs/Operators.md`, the normative operator specification containing `QuantizeLinear` and `DequantizeLinear`.
  - `int4.md`, `int2.md` ← `docs/docsgen/source/technical/int4.md` and `int2.md`, rendered as `onnx.ai/onnx/technical/int4.html` and `int2.html`; these state the LSB-first sub-byte packing.
- **Licence:** Apache-2.0, preserved as `LICENSE`, with the repository `NOTICE` preserved alongside it.
- **Verdict:** vendored.

### `dlpack-header-v1.3`, `dlpack-c-api-v1.3`, `dlpack-license-v1.3`

- **Owner:** the DLPack project (DMLC).
- **Pinned to:** `dmlc/dlpack` tag `v1.3`, commit `84d107bf416c6bab9ae68ad285876600d230490d`.
- **Documents and upstream paths:**
  - `dlpack.h` ← `include/dlpack/dlpack.h`. This is the substantive record: `DLDataTypeCode` (including `kDLComplex`), the `(code, bits, lanes)` triple, and `DLPACK_FLAG_BITMASK_IS_SUBBYTE_TYPE_PADDED` are declared and documented here.
  - `c_api.rst` ← `docs/source/c_api.rst`, the page skeleton behind `dmlc.github.io/dlpack/latest/c_api.html` that the taxonomy links. It is a list of Doxygen directives and contains no prose of its own; it is preserved so the cited page's structure is reproducible, not because it carries content.
- **Licence:** Apache-2.0, preserved as `LICENSE`. No `NOTICE` file exists at that tag (HTTP 404 on the retrieval date).
- **Verdict:** vendored.

### `nvidia-te-nvfp4-v2.17`, `nvidia-te-license-v2.17`

- **Document:** "NVFP4", `docs/features/low_precision_training/nvfp4/nvfp4.rst` — the source of the NVFP4 user-guide page the taxonomy cites.
- **Owner:** NVIDIA (Transformer Engine).
- **Pinned to:** `NVIDIA/TransformerEngine` tag `v2.17`, commit `2e559f062497bef768dfbe9d7e45548fadeca80a`.
- **Licence:** Apache-2.0, preserved as `LICENSE`.
- **Verdict:** vendored.
- **Boundary:** this is NVIDIA's own documentation of a vendor block-scaled recipe, released under an open-source licence — a primary description by the format's owner, not a ratified specification. NVFP4's E2M1 elements, 16-value blocks, FP8 E4M3 local scale, and FP32 global scale are that owner's claims about its own format; the taxonomy's separate conclusion that NVFP4 must not alias OCP MXFP4 is Tiler's inference from comparing this document with the OCP MX specification.

### `tosa-spec-1.0.1`

- **Document:** TOSA 1.0.1 specification (complete rendered specification, single HTML file, 899 100 bytes).
- **Owner:** Arm Limited, published through ML Platform.
- **Retrieved from:** `https://www.mlplatform.org/tosa/tosa_spec_1_0_1.html` — the path is version-qualified, so it is not a mutable "latest" alias. The TOSA specification repository itself moved from `git.mlplatform.org` to `gitlab.arm.com/tosa/tosa-specification` (that redirect was observed on the retrieval date), which is why the version-qualified published document is preserved rather than a repository file.
- **Licence:** the "TOSA Specification License" is embedded in the document. It states that the specification is non-confidential, that "nothing in this License shall restrict you from further disseminating this Specification", and that "You shall provide a copy of this License upon disseminating the Arm Specification to a third party". The preserved file is the complete unmodified document, so the licence text travels with the specification exactly as that clause requires.
- **Verdict:** vendored.
- **Cited for:** quantization operator semantics in [quantization representation in tensor IRs](../quantization-ir-precedents.md) and [affine quantization numerical semantics](../affine-quantization-semantics.md), the evidence behind ADRs 0029 and 0031–0033.

### `pytorch-tensor-attributes-v2.13.0`, `pytorch-complex-numbers-v2.13.0`, `pytorch-scalar-type-v2.13.0`, `pytorch-license-v2.13.0`, `pytorch-notice-v2.13.0`

- **Owner:** the PyTorch project. Its `LICENSE` aggregates the Torch, Caffe2, and related copyrights it lists — Facebook, Idiap Research Institute, Deepmind Technologies, NEC Laboratories America, NYU, and others — under one grant.
- **Pinned to:** `pytorch/pytorch` tag `v2.13.0`, commit `cf30153c4c131c8164ee7798e5022d810682e2cb`.
- **Documents and upstream paths:**
  - `tensor_attributes.md` ← `docs/source/tensor_attributes.md`, the source of `docs.pytorch.org/docs/stable/tensor_attributes.html`. It carries the `torch.dtype` table, `torch.complex32`/`torch.chalf` described as 32-bit complex with two `float16` components, the FP8 variants including `torch.float8_e8m0fnu`, and the footnote that defines the shell-dtype concept verbatim: "a shell dtype is a specialized dtype with limited op and backend support".
  - `complex_numbers.md` ← `docs/source/complex_numbers.md`, the source of the complex-numbers page, which names `torch.cfloat` and `torch.cdouble` and their `complex64`/`complex128` spellings.
  - `ScalarType.h` ← `c10/core/ScalarType.h`. This is the substantive record for the internal width claim: it declares `UInt1` through `UInt7` alongside the barebones `Bits1x8`, `Bits2x4`, `Bits4x2`, `Bits8`, and `Bits16` scalar types, and `ComplexHalf`.
- **Licence:** BSD 3-clause, preserved as `LICENSE`, with the repository `NOTICE` preserved alongside it. That `NOTICE` covers software under `third_party`, none of which is preserved here; it is kept because the repository ships it beside the licence, not because it attributes any of the three files above.
- **Verdict:** vendored.
- **Identity note:** the repository source files are preserved rather than the rendered pages, and in PyTorch's case the rendered page is not an alternative: a plain retrieval of `https://docs.pytorch.org/docs/stable/tensor_attributes.html` on the retrieval date returned a 1 340-byte client-side shell carrying no document text and no terms of its own. The repository file is both the source of the cited page and the only copy whose licence can be read.
- **Cited for:** the internal/barebones integer widths in the taxonomy's integer section, the `complex32`/`chalf` spelling in its complex section, and the cross-system claim that a shell dtype proves representation and view support can precede arithmetic coverage. These are spelling and exposure precedents; PyTorch defines no value set that Tiler admits.

### `numpy-dtype-classes-v2.5.1`, `numpy-license-v2.5.1`

- **Document:** "Data type classes (`numpy.dtypes`)", `doc/source/reference/routines.dtypes.rst` — the source of `numpy.org/doc/stable/reference/routines.dtypes.html`.
- **Owner:** the NumPy developers.
- **Pinned to:** `numpy/numpy` tag `v2.5.1`, commit `5e1d03ffac5f2c0a9c39bfcaa9fc853b2b83151e`.
- **Licence:** BSD 3-clause, preserved as `LICENSE.txt`. The rendered page carries only "© Copyright 2008-2026, NumPy Developers" and states no terms differing from the repository licence.
- **Verdict:** vendored.
- **Cited for:** the taxonomy's cross-system claim that host array domains include strings, objects, temporal, structured, and ABI-dependent types. The preserved file supports each part of that claim by enumeration: `StrDType`/`BytesDType`/`StringDType`, `ObjectDType`, `DateTime64DType`/`TimeDelta64DType`, `VoidDType`, and the C-named `LongDType`/`ULongDType`/`LongDoubleDType`/`CLongDoubleDType` group whose widths are ABI-dependent.

### `jax-typed-keys-v0.11.0`, `jax-license-v0.11.0`

- **Document:** JEP 9263, "Typed keys & pluggable RNGs", `docs/jep/9263-typed-keys.md` — the source of `docs.jax.dev/en/latest/jep/9263-typed-keys.html`.
- **Owner:** the JAX authors (Google).
- **Pinned to:** `jax-ml/jax` tag `jax-v0.11.0`, commit `a1521744c6dc074443fe549f19f48d7197abf759`.
- **Licence:** Apache-2.0, preserved as `LICENSE`. No `NOTICE` file exists at that tag (`https://raw.githubusercontent.com/jax-ml/jax/jax-v0.11.0/NOTICE` returned HTTP 404 on the retrieval date).
- **Verdict:** vendored.
- **Cited for:** the taxonomy's claim that typed PRNG keys deliberately reject ordinary arithmetic, and the cross-system observation that extended key dtypes show a library dtype need not be a numeric tensor scalar. The document's "Key arithmetic" and "Extended dtypes" sections carry both.
- **Identity note:** the cited URL is an `en/latest` path, which tracks the project's main branch. The preserved file is that page's source at a released tag, so the pin is the tag rather than the URL.

### `arrow-columnar-25.0.0`, `arrow-canonical-extensions-25.0.0`, `arrow-schema-fbs-25.0.0`, `arrow-license-25.0.0`, `arrow-notice-25.0.0`

- **Owner:** the Apache Software Foundation (Apache Arrow).
- **Pinned to:** `apache/arrow` tag `apache-arrow-25.0.0`, commit `59bea6ec485e7fe351d1aa6753f964f6a6bc353a`. That tag is doubly annotated upstream — it dereferences through `apache-arrow-25.0.0-rc1` before reaching the commit.
- **Documents and upstream paths:**
  - `Columnar.rst` ← `docs/source/format/Columnar.rst`, the source of `arrow.apache.org/docs/format/Columnar.html`. It carries the data-type table in which every logical type names a physical memory layout, and the `Decimal` row parameterized by bit width, scale, and precision.
  - `CanonicalExtensions.rst` ← `docs/source/format/CanonicalExtensions.rst`, the source of the canonical-extensions page. Its standardization rules require a canonical extension name to start with `arrow.`, and require described parameters, described serialization, stated semantics with ambiguities addressed, and at least one submitted implementation.
  - `Schema.fbs` ← `format/Schema.fbs`. `Columnar.rst` names this file as "the authoritative source for the description of the standard Arrow data types" and provides its own table only "for convenience", so the exact decimal widths live here rather than in the cited page: `table Decimal` records that "The accepted widths are 32, 64, 128 and 256."
- **Licence:** Apache-2.0, preserved as `LICENSE.txt`, with the repository `NOTICE.txt` preserved alongside it. The rendered pages carry an ASF copyright and trademark notice and no terms differing from the repository licence.
- **Verdict:** vendored.
- **Boundary:** `Schema.fbs` is not itself cited by the research. It is preserved because the cited document delegates authority to it, and the taxonomy's claim that decimal32/64/128/256 fixed-point is mature in Arrow is reproducible only from it. Preserving the cited page alone would have left that claim resting on a live URL, which is the failure this record exists to prevent.
- **Cited for:** the fixed-point and decimal fixed-point maturity claim and the cross-system layout claim in the taxonomy, and the namespace-reservation precedent in the [dtype identity admission policy](../dtype-identity-admission-policy.md).

### `gcc-manual-16.1.0`

- **Document:** "Using the GNU Compiler Collection (GCC)", for gcc version 16.1.0 — the complete manual as a single PDF, 3 321 026 bytes.
- **Owner:** Free Software Foundation, Inc.; "Copyright © 1988-2026".
- **Retrieved from:** `https://gcc.gnu.org/onlinedocs/gcc-16.1.0/gcc.pdf`, the version-qualified path rather than the `onlinedocs/gcc/` alias the taxonomy links, which always serves the current release.
- **Licence:** GNU Free Documentation License version 1.3 or later, "with the Invariant Sections being 'Funding Free Software', the Front-Cover Texts being (a)... and with the Back-Cover Texts being (b)".
- **Verdict:** vendored — but as the whole manual, not as the cited page. GFDL section 2 permits verbatim copying of the Document provided the licence notice, the invariant section, and the cover texts travel with it; extracting the single `Decimal-Float.html` page would be a partial copy, and a partial copy is a modified version subject to obligations a one-page HTML file cannot carry. The complete PDF satisfies the verbatim-copying condition directly: it contains the licence notice, the Invariant Section "Funding Free Software", both cover texts, and the full GFDL appendix, all in one unmodified file.
- **Cited for:** GCC's `_Decimal32`, `_Decimal64`, and `_Decimal128` extension types, alongside IEEE 754-2019, in the taxonomy's decimal floating-point section. The manual's "Decimal Float" node carries them. It is a compiler's exposure of the decimal formats, not a definition of them — IEEE 754-2019 remains the normative reference for the value sets, and it has no local copy.

### `spirv-unified-1.6-rev7`

- **Document:** SPIR-V Specification, version 1.6, Revision 7, Unified — the complete rendered specification, single HTML file, 2 238 984 bytes.
- **Owner:** The Khronos Group Inc.; "Copyright 2014-2026".
- **Retrieved from:** `https://registry.khronos.org/SPIR-V/specs/unified1/SPIRV.html`.
- **Identity:** the `unified1` path carries no revision segment, so identity rests on the document's own version string — "version 1.6, Revision 7, Unified" — rather than on the URL.
- **Licence:** the specification's own terms, embedded in the document: it "is protected by copyright laws and contains material proprietary to Khronos", and "Khronos grants a conditional copyright license to use and reproduce the unmodified Specification for any purpose, without fee or royalty, EXCEPT no licenses to any patent, trademark or other intellectual property rights are granted under these terms."
- **Verdict:** vendored. The grant is explicit and covers reproduction of the unmodified specification, and the preserved file is the complete unmodified byte stream the registry served, so the terms travel inside the document exactly as the grant requires. No patent or trademark licence is claimed here, and none is needed to preserve evidence.
- **Reproducibility boundary:** **Measurement, 2026-07-31** — three retrievals of this URL produced three distinct SHA-256 digests at an identical 2 238 984-byte length. The differing bytes are twelve characters inside a Cloudflare-injected `window.__CF$cv$params={r:...,t:...}` script near the end of the file: a per-request nonce and timestamp added by the edge, not by Khronos. The recorded digest therefore verifies the integrity of the copy in this repository, which is what `verify-sources.sh` checks, and a fresh retrieval will *not* reproduce it. Do not treat a digest mismatch against a newly downloaded copy as evidence the specification changed; compare the document's own version and revision string instead. Of the five documents in this wave identified by URL rather than by a git tag, this is the only non-deterministic one — the WGSL draft, the W3C licence page, the GCC manual, and the TensorRT page each produced identical digests across two retrievals, and every tag-pinned file came from `raw.githubusercontent.com` at a fixed commit.
- **Cited for:** the taxonomy's cross-system claim that storage capabilities, arithmetic capabilities, and packed-dot capabilities are explicitly separate. It is a shader-language exposure precedent; Tiler admits no SPIR-V-defined identity.

### `wgsl-crd-20260716`, `w3c-document-license-2023`

- **Document:** WebGPU Shading Language, W3C Candidate Recommendation Draft, 16 July 2026, 2 987 074 bytes.
- **Owner:** World Wide Web Consortium; published by the GPU for the Web Working Group.
- **Retrieved from:** `https://www.w3.org/TR/2026/CRD-WGSL-20260716/` — the dated snapshot the document itself names as "This version", deliberately in place of the `w3.org/TR/WGSL/` alias the taxonomy links, which is the mutable latest-published-version pointer.
- **Licence:** the document states "Copyright © 2026 World Wide Web Consortium. W3C liability, trademark and permissive document license rules apply" and links the W3C Software and Document License. That licence grants "Permission to copy, modify, and distribute this work, with or without modification, for any purpose and without fee or royalty... provided that you include the following on ALL copies of the work or portions thereof", the first condition being "The full text of this NOTICE in a location viewable to users of the redistributed or derivative work."
- **Verdict:** vendored, with the licence preserved beside the document to satisfy that condition. `w3c-software-and-document-license-2023.html` is the licence page as served from `https://www.w3.org/copyright/software-license/`, which redirects to `https://www.w3.org/copyright/software-license-2023/`; that is the version in effect since 1 January 2023 and the one the preserved draft links. Nothing here is modified, so the licence's change-notice condition does not arise.
- **Cited for:** the taxonomy's cross-system claim that a narrow portable shader arithmetic set — `i32`/`u32`/`f32` with optional f16 — demonstrates backend independence. The preserved draft carries the scalar-type sections and the `f16` enable-extension table entry gating the type behind the WebGPU `shader-f16` feature.

## Metadata-only records

### `ieee-754-2019`

- **Document:** IEEE Std 754-2019, IEEE Standard for Floating-Point Arithmetic (revision of IEEE Std 754-2008), approved 2019-06-13 and published 2019-07-22.
- **Owner:** IEEE / IEEE Computer Society, Microprocessor Standards Committee.
- **Reference URL:** `https://standards.ieee.org/ieee/754/6210/`.
- **Licence:** IEEE copyright; the standard is sold under IEEE terms and carries no redistribution grant.
- **Verdict:** metadata-only, and **no byte stream was ever retrieved** — the document is behind a purchase or subscription wall. The digest field for this id is therefore `-` rather than a value; there is no digest to record and none is invented.
- **Official acquisition route:** purchase from the IEEE Standards Store, or read through an institutional IEEE Xplore subscription, at the reference URL above.
- **Cited for:** binary16/32/64/128 in [ADR 0036](../../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md) and decimal32/64/128 in [ADR 0035](../../../decisions/0035-recognize-ieee-decimal-floating-formats.md). Because no local copy exists, a future audit of those value sets must acquire the standard through the route above; nothing in this repository substitutes for it.

### `posit-standard-2022`

- **Document:** "Standard for Posit™ Arithmetic (2022)", dated 2022-03-02, produced by the Posit Working Group (chair John Gustafson) and sponsored by the National Supercomputing Centre (NSCC) Singapore. PDF creation date 2022-04-04, 138 415 bytes.
- **Retrieved from:** `https://posithub.org/docs/posit_standard-2.pdf`.
- **Licence:** none stated. The document carries no copyright notice and no licence or redistribution grant. Check performed: `pdftotext -layout posit_standard-2.pdf - | grep -iE "copyright|licen|redistribut|permission|public domain|creative commons"` returned no licence statement — only unrelated prose matches on "reproducible" and "non-compliant".
- **Verdict:** metadata-only. Absent permission is not permission; ambiguity resolves against redistribution. The retrieved bytes were digested and then discarded, and no copy is checked in.
- **Official acquisition route:** download from the Posit Standard page at `posithub.org`, at the URL above.
- **Cited for:** the reserved `positN`/`quireN` extension family in the taxonomy — a catalog reservation, not an admitted Tiler identity.

### `nvidia-ptx-isa-cuda-13.3.0`

- **Document:** Parallel Thread Execution ISA, as published in the CUDA Toolkit 13.3.0 documentation archive. The page states "PTX ISA Version 9.3" and "Last updated on May 21, 2026"; 3 577 575 bytes of `text/html`.
- **Owner:** NVIDIA.
- **Retrieved from:** `https://docs.nvidia.com/cuda/archive/13.3.0/parallel-thread-execution/index.html` — the version-qualified archive path, deliberately in place of the `docs.nvidia.com/cuda/parallel-thread-execution/` alias the taxonomy links, which always serves the current toolkit.
- **Licence:** "Copyright © 2007-2026, NVIDIA Corporation & affiliates. All rights reserved." No redistribution grant.
- **Verdict:** metadata-only. The retrieved bytes were digested and then discarded; no copy is checked in.
- **Identity caveat:** a rendered documentation page can be regenerated without the specification changing. The durable identity here is the archived CUDA toolkit version plus the document's own "PTX ISA Version 9.3"; the recorded digest is a fingerprint of that page **as served on the retrieval date**, and a future mismatch is evidence to investigate rather than proof the specification changed.
- **Cited for:** the `.ue4m3` and `.ue8m0` alternate floating-point data formats, which [ADR 0036](../../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md) explicitly refuses to alias to the OCP E8M0 identity without equivalence evidence. Both spellings were confirmed present in the retrieved page under "Alternate Floating-Point Data Formats" before the bytes were discarded.

### `nvidia-tensorrt-accuracy-11.2.1`

- **Document:** "Accuracy Considerations", TensorRT 11.2.1 documentation; 47 408 bytes of `text/html`, last updated 28 July 2026 per the page itself.
- **Owner:** NVIDIA.
- **Retrieved from:** `https://docs.nvidia.com/deeplearning/tensorrt/11.2.1/inference-library/accuracy-considerations.html` — the version-qualified path, deliberately in place of the `.../tensorrt/latest/...` alias the taxonomy links, which always serves the current release.
- **Licence:** "Copyright © 2021-2026, NVIDIA Corporation." The page states no redistribution, reproduction, or permission grant; checks for `reproduc`, `redistribut`, and `permission` over the rendered text each returned zero matches.
- **Verdict:** metadata-only. Absent permission is not permission. The retrieved bytes were digested and then discarded, and no copy is checked in. **Measurement, 2026-07-31:** two retrievals produced the identical digest, so this fingerprint is reproducible from the recorded URL.
- **Official acquisition route:** read at the version-qualified URL above; NVIDIA publishes it without registration.
- **Cited for:** the taxonomy's statement that NVIDIA documents TF32 as an execution precision for Tensor Core paths. The retrieved page describes TF32 as enabled by default with an 8-bit exponent and 10-bit mantissa, "combining the dynamic range of FP32 with the computational efficiency of FP16", among the reduced-precision formats TensorRT supports; that was confirmed before the bytes were discarded. It is a vendor exposure precedent, not a format definition.

## Pending-acquisition records

### `ocp-ofp8-v1.0`

- **Document:** OCP 8-bit Floating Point Specification (OFP8), Revision 1.0, dated 2023-12-01.
- **Owner:** Open Compute Project Foundation.
- **Reference URL:** `https://www.opencompute.org/documents/ocp-8-bit-floating-point-specification-ofp8-revision-1-0-2023-12-01-pdf-1`.
- **Retrieval attempt, 2026-07-31:** HTTP 403 for both a plain `curl` request and one carrying a browser `User-Agent`; the response body was a 5 979-byte Cloudflare interstitial (`<title>Just a moment...</title>`), not the specification. **No bytes of the document were obtained, so no digest exists and none is recorded.**
- **Licence:** unreviewed. The document's own terms could not be read, and an OCP repository-level licence is not evidence about a specific document's terms.
- **Official acquisition route:** download from the OCP document page above through an interactive browser session; OCP gates specification downloads behind an account and acceptance of its specification licence terms.
- **Cited for:** `tiler::f8e4m3fn@1` and `tiler::f8e5m2@1` in [ADR 0036](../../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md).
- **Closes when:** the document is acquired through that route, its own terms are reviewed, and this record becomes either vendored or metadata-only with a digest.

### `ocp-mx-v1.0`

- **Document:** OCP Microscaling Formats (MX) Specification, Version 1.0.
- **Owner:** Open Compute Project Foundation.
- **Reference URL:** `https://www.opencompute.org/documents/ocp-microscaling-formats-mx-v1-0-spec-final-pdf`.
- **Retrieval attempt, 2026-07-31:** HTTP 403, 5 858-byte Cloudflare interstitial, same as OFP8. **No bytes obtained; no digest recorded.**
- **Licence:** unreviewed, for the same reason.
- **Official acquisition route:** as for OFP8, through the OCP document page in an interactive browser session.
- **Cited for:** `tiler::f6e2m3fn@1`, `tiler::f6e3m2fn@1`, `tiler::f4e2m1fn@1`, and `tiler::f8e8m0fnu@1` in [ADR 0036](../../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md), and the MX scheme identities in [ADR 0038](../../../decisions/0038-recognize-ocp-mx-schemes.md).
- **Closes when:** as for OFP8.

Two of the three format families ADR 0036 pins to an external normative definition therefore have no local copy. That is a real gap in this record, stated here rather than hidden: the ADRs remain accepted Tiler authority, but re-deriving their pinned value sets today requires acquiring the OCP documents by hand.

## Verifying this record

```sh
docs/research/numerics/sources/verify-sources.sh
```

The check reads `expected-sources.tsv` and enforces a declared population — 46 records, of which 40 vendored, 4 metadata-only, 2 pending-acquisition — before it inspects anything, so a manifest that lost rows fails rather than agreeing with itself. It then verifies that ids are unique, that every vendored file exists and matches its recorded digest, that no metadata-only or pending record retains local bytes, and that every file present on disk is claimed by exactly one record. A deleted file, a mutated digest, mutated preserved bytes, an emptied manifest, and an unrecorded stray file were each observed failing before the first wave was committed, and a deleted file and a mutated digest were observed failing again over the extended population before the ecosystem precedents were committed.

Two ids may legitimately share a digest: `jax-license-v0.11.0` and `onnx-license-v1.22.0` are both the stock Apache-2.0 text. The check requires unique ids, not unique digests, which is why that agreement is a consistency signal rather than a failure.

Adding or refreshing a source means updating the manifest row, the counts declared at the top of `verify-sources.sh`, and the record above in the same change.

`.gitattributes` here marks the preserved files `-text -whitespace`. Both settings protect the bytes: end-of-line conversion on a checkout would silently break every recorded digest, and `git diff --check` would otherwise report trailing whitespace that belongs to the upstream document and must not be removed. The record's own files stay under the normal checks.

An upstream revision that changes a semantic fact is not handled by refreshing bytes. Refreshing a preserved document updates evidence only; changing what Tiler recognizes requires an explicit contract or ADR review.
