---
id: preserve-primary-dtype-standards-evidence
title: Preserve primary dtype standards evidence
status: todo
priority: p1
dependencies: []
related: [own-the-dtype-support-maturity-matrix, enumerate-the-mature-tensor-dtype-taxonomy, define-dtype-namespace-admission-policy, register-the-accepted-built-in-dtype-catalog, preserve-ecosystem-dtype-precedent-sources]
scopes: [research/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtype, provenance, documentation]
---

## User-visible outcome

The primary specifications behind Tiler's recognized dtype catalog remain reproducible if an upstream URL moves or disappears. Redistribution-permitted sources are vendored with exact provenance; restricted sources retain immutable bibliographic identity, retrieval metadata, and cryptographic digests without violating their licenses.

## Evidence gap

- **Fact:** `find docs/research -type f -path '*/sources/*' -print | sort` currently returns only Apple Metal source material. The IEEE 754, RISC-V BF16, OCP OFP8 and MX, StableHLO, MLIR, ONNX, DLPack, and PTX sources cited by `docs/research/numerics/mature-dtype-taxonomy.md` remain URL-only.
- **Fact:** the accepted dtype ADRs are durable Tiler authority, but they do not preserve the upstream primary evidence those decisions cite.
- **Inference:** copying every document without a license review risks redistributing restricted material, while retaining only live URLs leaves the research irreproducible. The correct preservation unit is therefore license-aware rather than uniformly “download every PDF.”

## Implementation keys

- Inventory every primary source cited by the dtype taxonomy and accepted dtype ADRs. Record canonical title, owner, edition/version/date, source URL, retrieval date, license or redistribution status, and SHA-256 digest over the exact retrieved bytes.
- Vendor every redistribution-permitted specification under `docs/research/numerics/sources/` with a source README that maps the checked-in filename to the provenance record. Preserve original bytes; do not reflow or transcribe a specification into a new apparent authority.
- For a source that cannot legally be redistributed, retain the immutable digest and enough bibliographic and retrieval metadata to identify the exact document, plus an official acquisition route. Do not check in restricted bytes or rely on a mutable “latest” URL as identity.
- Prefer standards-body and project-owner publications over mirrors. A repository snapshot may supplement, but never silently replace, a normative specification.
- Correct taxonomy and ADR links to the preserved record without changing their accepted semantic conclusions. Distinguish a locally preserved source from a normative guarantee and from Tiler's inference.
- Add a complete expected-source inventory so a missing entry cannot report success through an empty glob. Deliberately remove or alter one expected digest and observe the check fail before restoring it.
- Make that inventory explicit before downloading anything. Each stable source ID records title, owner, edition or exact upstream commit/path, retrieval URL/date, SHA-256, license, redistribution verdict, and either a local file or official acquisition route. Pin living project specifications such as StableHLO, MLIR/LLVM, ONNX, and DLPack to commits/tags and preserve the exact normative files with the repository license/NOTICE rather than a mutable rendered “latest” page.
- Treat ambiguous redistribution permission as metadata-only. IEEE 754 and every other restricted source retain bibliographic identity, digest, and official acquisition route without checked-in bytes. Review OCP, PTX, Posit, RISC-V, and project specifications document by document; one permissive repository license does not automatically classify every externally incorporated document.
- Enumerate the exact accepted dtype ADR set and every primary source actually used by `mature-dtype-taxonomy.md`, including ecosystem precedents beyond the headline eight families. If peripheral framework/compiler precedents would make this ticket unbounded, split a linked preservation ticket and narrow this ticket's closing population explicitly; do not close against an unstated subset.

## Closing population, fixed on 2026-07-31

The accepted dtype ADR set is exactly the `dtypes-quantization` catalog group, ADRs 0026 through 0038. Their `evidence` frontmatter reaches five research documents: the mature dtype taxonomy, the dtype identity admission policy, quantization representation in tensor IRs, the quantized value and transformation contract, and affine quantization numerical semantics. None of the thirteen ADRs cites an external URL directly; every primary source arrives through those five documents.

This ticket closes over the primary sources among them that *define* a format's value set, encoding, or quantization contract: IEEE 754-2019, the ratified RISC-V BF16 operand format, OCP OFP8 revision 1.0, OCP MX version 1.0, the Posit Standard (2022), StableHLO, MLIR built-in and Quant types, the LLVM language reference, ONNX IR/operators/sub-byte packing, DLPack, the NVIDIA PTX ISA, NVIDIA Transformer Engine NVFP4, and the TOSA specification — 25 preserved units in `docs/research/numerics/sources/expected-sources.tsv`.

Ecosystem precedents cited for how an existing system *spells* or *exposes* a format — PyTorch, NumPy, JAX, Arrow, GCC decimal float, SPIR-V, WGSL, the NVIDIA TensorRT accuracy guide, and the StableHLO-compatibility and MLIR-language-reference governance pages — are excluded and owned by [preserve-ecosystem-dtype-precedent-sources](preserve-ecosystem-dtype-precedent-sources.md). Including them would make the population unbounded without strengthening any format definition.

## Closes when

Every explicitly enumerated primary numerical-format source used by the mature dtype taxonomy and named accepted catalog ADRs has a license-reviewed preservation record; every redistribution-permitted source is vendored with required license/NOTICE material; every restricted source has immutable identity and an official acquisition route with no local bytes; the expected population, unique IDs, classifications, and digests are mechanically checked with deletion and digest mutations demonstrated failing; local links are read for coherence; `tkt lint` and `git diff --check` pass; and one batch `make full` passes.

## Graph maintenance

- Link future dtype-catalog or format-semantics tickets to the preserved source record they consume rather than copying format tables into ticket prose.
- When an upstream specification revision changes semantic facts, file an explicit contract/ADR review; refreshing preserved bytes alone does not update Tiler's accepted identity.
- Keep Apple Metal language and compatibility evidence under `docs/research/apple-targets/sources/`; this ticket owns missing numerical-format sources rather than moving already-preserved backend material.
