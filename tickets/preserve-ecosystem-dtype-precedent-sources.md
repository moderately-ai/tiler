---
id: preserve-ecosystem-dtype-precedent-sources
title: Preserve ecosystem dtype precedent sources
status: in-progress
priority: p3
dependencies: []
related: [preserve-primary-dtype-standards-evidence, enumerate-the-mature-tensor-dtype-taxonomy, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtype, provenance, documentation]
claimed_from: todo
assignee: loop-preserve-eco
lease_expires_at: 1785521298
---

## User-visible outcome

The framework and shader-language documents the dtype research cites as *spelling* and *exposure* precedents stay reproducible under the same licence-aware preservation record as the format-defining specifications, so a future naming or alias audit does not depend on a live vendor URL.

## Why this is needed

- **Fact:** `preserve-primary-dtype-standards-evidence` closed over the primary numerical-format and quantization-semantics specifications and deliberately excluded ecosystem precedents, recording that boundary in `docs/research/numerics/sources/README.md`.
- **Fact:** the excluded documents are still cited as primary sources by `docs/research/numerics/mature-dtype-taxonomy.md` and `docs/research/numerics/dtype-identity-admission-policy.md`: PyTorch tensor attributes, PyTorch complex numbers, PyTorch `c10/core/ScalarType.h`, NumPy dtype classes, JAX typed keys, Arrow columnar format, Arrow canonical extensions, GCC decimal floating types, the SPIR-V unified specification, WGSL, and the NVIDIA TensorRT accuracy-considerations guide. Two governance precedents join them: the StableHLO compatibility page and the MLIR language reference, cited for namespace and versioning policy rather than for any format definition.
- **Inference:** these documents do not define a value set, so losing one cannot invalidate an admitted Tiler identity. They carry the evidence for alias-resolution and naming conclusions — that `complex64` means different things in different ecosystems, that a PyTorch shell dtype proves representation can precede arithmetic — which is why they are worth preserving separately rather than not at all.

## Implementation keys

- Extend `docs/research/numerics/sources/expected-sources.tsv` and its README rather than starting a second record; update the declared counts in `verify-sources.sh` in the same change and re-run it.
- Pin every open-source-licensed document to an exact commit or tag and preserve the repository `LICENSE`/`NOTICE` beside it, as the existing records do. PyTorch and NumPy are BSD-licensed, JAX and Arrow are Apache-2.0; review each document's own terms rather than assuming the repository licence covers a rendered documentation page. The StableHLO compatibility page and the MLIR language reference come from repositories already pinned by the parent record (`openxla/stablehlo` and `llvm/llvm-project`); prefer the same commits so one project does not appear at two versions unless a later revision is the point.
- The GCC decimal-float manual page is GFDL-licensed documentation, the SPIR-V specification is published under Khronos terms, WGSL under W3C document terms, and the TensorRT guide is NVIDIA-proprietary. Review each individually; ambiguous permission is metadata-only with a digest and an official acquisition route, and a rendered "latest" page is not an identity.
- Correct the taxonomy and admission-policy links to the preserved records without changing any accepted naming or classification conclusion.
- Demonstrate the extended check failing on a deleted file and on a mutated digest before recording success, exactly as the parent ticket did.

## Closes when

Every listed ecosystem precedent has a licence-reviewed preservation record in the shared manifest; permitted sources are vendored with their licence material; restricted or ambiguous ones carry identity, digest where bytes were retrieved, and an acquisition route with no local bytes; `verify-sources.sh` passes with updated counts and has been observed failing; `tkt lint` and `git diff --check` pass; and one batch `make full` passes.

## Graph maintenance

Keep one preservation record for numerics rather than splitting the manifest by source class — the check's value comes from a single declared population. If a further wave of citations appears, extend this same record and note the new boundary in its README rather than filing a third preservation ticket.
