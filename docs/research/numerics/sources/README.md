# Preserved primary sources for the dtype catalog

These records keep the primary numerical-format specifications behind Tiler's recognized dtype catalog reproducible when an upstream URL moves, changes, or disappears. Preservation is licence-aware rather than uniform: a document whose own terms permit dissemination is vendored here byte-for-byte, and a document whose terms do not — or whose terms could not be read — keeps bibliographic identity, a retrieval fingerprint where one exists, and an official acquisition route, with no bytes checked in.

**Retrieval date for every record below: 2026-07-31.**

Digests are deliberately not repeated in this file. `expected-sources.tsv` is the single authority for the expected population, each record's classification, and each SHA-256; `verify-sources.sh` enforces it. A digest written twice drifts silently, and the copy nothing checks is the one a reader would trust.

## What these records claim, and what they do not

- **Preserved source.** The bytes under this directory are an unmodified copy of the named upstream document at the named version. Nothing here is transcribed, reflowed, summarized, or corrected; a specification excerpt in Tiler prose is never the authority, the preserved file is.
- **Normative guarantee.** What a preserved specification requires of a conforming implementation. Preserving a document does not extend its guarantees to Tiler, nor claim Tiler implements it.
- **Tiler inference.** Every classification, alias rejection, and admission conclusion drawn from these documents lives in the research and ADR text that cites them, not here. This directory records provenance only.

Vendor documentation preserved under an open-source licence (NVIDIA Transformer Engine below) is a primary description by the format's owner, not a standards-body specification; the distinction survives the fact that both are preserved the same way.

## Population boundary

This record closes over the primary **numerical-format and quantization-semantics** specifications cited by [the mature dtype taxonomy](../mature-dtype-taxonomy.md) and by the accepted dtype ADRs 0026–0038 (the `dtypes-quantization` catalog group) through their evidence documents — that is, documents that *define* a format's value set, encoding, or quantization contract.

Ecosystem and framework precedents cited by the same documents for how an existing system *spells* or *exposes* a format — PyTorch, NumPy, JAX, Arrow, GCC decimal float, SPIR-V, WGSL, and the NVIDIA TensorRT accuracy guide — are deliberately outside this population; including them makes the set unbounded without strengthening any format definition. They are preserved separately under [preserve-ecosystem-dtype-precedent-sources](../../../../tickets/preserve-ecosystem-dtype-precedent-sources.md). Apple Metal language and capability evidence stays under [its own record](../../apple-targets/sources/README.md).

## Filename map

| File under this directory | Source id |
| --- | --- |
| `riscv-isa/riscv-unprivileged-20260120.pdf` | `riscv-unprivileged-isa-20260120` |
| `riscv-isa/LICENSE` | `riscv-isa-manual-license` |
| `stablehlo-v1.18.0/spec.md` | `stablehlo-spec-v1.18.0` |
| `stablehlo-v1.18.0/LICENSE` | `stablehlo-license-v1.18.0` |
| `llvm-project-llvmorg-22.1.8/BuiltinTypes.td` | `mlir-builtin-types-llvmorg-22.1.8` |
| `llvm-project-llvmorg-22.1.8/QuantBase.td` | `mlir-quant-base-llvmorg-22.1.8` |
| `llvm-project-llvmorg-22.1.8/LangRef.rst` | `llvm-langref-llvmorg-22.1.8` |
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

Five source ids retain no bytes here: `ieee-754-2019`, `ocp-ofp8-v1.0`, `ocp-mx-v1.0`, `posit-standard-2022`, and `nvidia-ptx-isa-cuda-13.3.0`.

## Vendored records

### `riscv-unprivileged-isa-20260120`, `riscv-isa-manual-license`

- **Document:** The RISC-V Instruction Set Manual, Volume I: Unprivileged Architecture — "Version 20260120: Official Release", 696 pages.
- **Owner:** RISC-V International.
- **Retrieved from:** `https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf` (4 580 174 bytes, `application/pdf`).
- **Identity:** that URL carries no version segment, so identity rests on the document's own version string, printed on its title page, together with the recorded digest — not on the URL.
- **Licence:** Creative Commons Attribution 4.0 International, stated in the document itself: "This document is released under a Creative Commons Attribution 4.0 International License." The full licence text is preserved as `riscv-isa/LICENSE`, taken from `riscv/riscv-isa-manual` at commit `310a111489a0bad6e60ef4cbfba574417c6f825f`.
- **Verdict:** vendored; CC BY 4.0 permits redistribution with attribution.
- **Cited for:** `tiler::bf16@1` in [ADR 0036](../../../decisions/0036-recognize-standard-binary-and-microscaling-formats.md) and the [dtype identity admission policy](../dtype-identity-admission-policy.md). Chapter 25, "BF16 Extensions for BFloat16-precision Floating-Point, Version 1.0", carries the operand-format contract at §25.3.1; the preface ratified-extension table lists BF16 1.0 as `Ratified`.

### `stablehlo-spec-v1.18.0`, `stablehlo-license-v1.18.0`

- **Document:** StableHLO Specification, `docs/spec.md`.
- **Owner:** OpenXLA / the StableHLO project.
- **Pinned to:** `openxla/stablehlo` tag `v1.18.0`, commit `e6f81ebd06b3509f2c7fa6175430aadbd4d724ca`.
- **Retrieved from:** `https://raw.githubusercontent.com/openxla/stablehlo/v1.18.0/docs/spec.md`.
- **Licence:** Apache-2.0, preserved as `stablehlo-v1.18.0/LICENSE`. The repository has no `NOTICE` file at that tag (`https://raw.githubusercontent.com/openxla/stablehlo/v1.18.0/NOTICE` returned HTTP 404 on the retrieval date).
- **Verdict:** vendored.
- **Cited for:** element types, quantized tensor types, and the token type, in the taxonomy and in [ADR 0028](../../../decisions/0028-recognize-sub-byte-integers.md), [ADR 0037](../../../decisions/0037-parameterize-complex-dtype-identity.md), and the quantization research. The taxonomy's `openxla.org/stablehlo/spec#…` links render this same file from the project's live branch, which is a mutable view of it rather than a second authority.

### `mlir-builtin-types-llvmorg-22.1.8`, `mlir-quant-base-llvmorg-22.1.8`, `llvm-langref-llvmorg-22.1.8`, `llvm-project-license-llvmorg-22.1.8`

- **Owner:** the LLVM Project.
- **Pinned to:** `llvm/llvm-project` tag `llvmorg-22.1.8`, commit `ca7933e47d3a3451d81e72ac174dcb5aa28b59d1`.
- **Documents and upstream paths:**
  - `BuiltinTypes.td` ← `mlir/include/mlir/IR/BuiltinTypes.td`, the definition file the rendered MLIR Builtin dialect types page is generated from — `Float4E2M1FNType`, `Float8E8M0FNUType` and siblings, `IntegerType`, `ComplexType`.
  - `QuantBase.td` ← `mlir/include/mlir/Dialect/Quant/IR/QuantBase.td`, the definition file behind the rendered Quant dialect page, including its per-layer, per-axis, and blockwise granularity text.
  - `LangRef.rst` ← `llvm/docs/LangRef.rst`, which distinguishes `x86_fp80` and `ppc_fp128` from IEEE `fp128`.
- **Licence:** Apache-2.0 with LLVM Exceptions, preserved as `LICENSE.TXT`.
- **Verdict:** vendored. Preserving the `.td`/`.rst` definition files rather than the rendered `mlir.llvm.org` and `llvm.org` pages pins the generated documentation to its source at an exact commit; the rendered pages track the project's main branch and are not a stable identity.
- **Cited for:** the nominal FP4/FP6/FP8 identities, arbitrary-width integers, structural complex, quantization granularity, and target ABI float formats throughout the taxonomy and the quantization research.

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

The check reads `expected-sources.tsv` and enforces a declared population — 25 records, of which 20 vendored, 3 metadata-only, 2 pending-acquisition — before it inspects anything, so a manifest that lost rows fails rather than agreeing with itself. It then verifies that ids are unique, that every vendored file exists and matches its recorded digest, that no metadata-only or pending record retains local bytes, and that every file present on disk is claimed by exactly one record. A deleted file, a mutated digest, mutated preserved bytes, an emptied manifest, and an unrecorded stray file were each observed failing before this record was committed.

Adding or refreshing a source means updating the manifest row, the counts declared at the top of `verify-sources.sh`, and the record above in the same change.

`.gitattributes` here marks the preserved files `-text -whitespace`. Both settings protect the bytes: end-of-line conversion on a checkout would silently break every recorded digest, and `git diff --check` would otherwise report trailing whitespace that belongs to the upstream document and must not be removed. The record's own files stay under the normal checks.

An upstream revision that changes a semantic fact is not handled by refreshing bytes. Refreshing a preserved document updates evidence only; changing what Tiler recognizes requires an explicit contract or ADR review.
