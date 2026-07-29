---
id: preserve-primary-dtype-standards-evidence
title: Preserve primary dtype standards evidence
status: todo
priority: p2
dependencies: []
related: [own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
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

## Closes when

Every primary numerical-format source used by the mature dtype taxonomy and accepted catalog decisions has a license-reviewed preservation record; every redistributable source is vendored; every restricted source has immutable identity and an official acquisition route; the expected population and digests are mechanically checked with a demonstrated failure path; local links are read for coherence; `tkt lint` and `git diff --check` pass; and one batch `make full` passes.

## Graph maintenance

- Link future dtype-catalog or format-semantics tickets to the preserved source record they consume rather than copying format tables into ticket prose.
- When an upstream specification revision changes semantic facts, file an explicit contract/ADR review; refreshing preserved bytes alone does not update Tiler's accepted identity.
- Keep Apple Metal language and compatibility evidence under `docs/research/apple-targets/sources/`; this ticket owns missing numerical-format sources rather than moving already-preserved backend material.
