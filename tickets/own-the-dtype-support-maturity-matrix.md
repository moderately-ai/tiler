---
id: own-the-dtype-support-maturity-matrix
title: Own the dtype support maturity matrix
status: todo
priority: p2
dependencies: []
related: [enumerate-the-mature-tensor-dtype-taxonomy, own-operation-family-support-matrix, register-the-accepted-built-in-dtype-catalog, prototype-quantized-value-vertical, implement-first-runtime-semantic-value-precondition-enforcement, scope-first-quantized-lm-profile]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, dtype, breadth]
---
## User-visible outcome

A reader can tell, for every recognized dtype family, exactly which of identity, semantic operations, reference evaluation, numerical policy, physical storage, ABI, optimizer, lowering, runtime validation, target dispatch, backend execution, and conformance are reservations, implemented mechanisms, or tested guarantees. No future enum addition can be mistaken for end-to-end dtype support.

## Why the taxonomy is not this ledger

**Fact.** `docs/research/numerics/mature-dtype-taxonomy.md` intentionally inventories the semantic universe without claiming implementation support. The operation-family matrix links to that taxonomy but tracks maturity by operation family, not by dtype and layer. Its quantization row is also stale after `prototype-quantized-value-vertical`: it still says no governed scheme is registered and no quantization operation exists, while the completed vertical records tested per-tensor strict-affine U4/F32 and U8/F32 semantics/reference behavior plus target-neutral U4 dequantization through artifact construction and typed Metal refusal.

**Inference.** Without a dtype-axis ledger, the accepted catalog, the small registered subset, the narrow physical vocabulary, and the even narrower executable profile can all be described as “supported” by different readers. That ambiguity is a correctness and scheduling defect, not merely missing documentation.

## Required matrix

- Rows cover logical bool; signed and unsigned exact-width integers; IEEE binary floats; BF16; OCP FP8/FP6/FP4/E8M0; decimal; parameterized complex; affine quantized; OCP MX/block-scaled; external/vendor formats; and explicitly unsupported sparse/ragged/non-tensor families where the taxonomy distinguishes them.
- Columns separately state recognized identity, semantic operation signatures, reference evaluation, numerical contract/honourability, physical carrier and encoding, ABI/materialization, optimizer legality, kernel vocabulary, backend lowering, runtime semantic validation, target-family dispatchability, and conformance evidence.
- Every cell names one of: absent/unsupported, type-system reservation, architectural seam, implemented mechanism, or tested guarantee. An implemented generic mechanism is not family support; a tested nominal fixture is not arithmetic support; a target measurement is not a portable guarantee.
- Link each non-absent cell to its construction site or accepted decision and each absent family to a live trigger. Negative source claims include a one-line reproducible check and are read at the construction site before publication.
- Correct the stale quantization row in the operation-family support matrix and reconcile Q-SEM-003 so the operation and dtype axes agree without duplicating authorities.

## Consumer-driven follow-ups

The matrix must make these currently unowned surfaces explicit without prematurely implementing them: logical bool storage and operations distinct from kernel predicate SSA; general integer overflow/division/remainder/conversion families distinct from index arithmetic; complex reference/numerical/storage/lowering contracts independent of planar/interleaved representation; generalized sub-byte bit order, cross-byte layout, tail, alignment, unaligned access, neighbour-safe writes, and repacking beyond the governed whole-component U4 path; profile-driven `StorageScalar`/`KernelType` widening that does not make carrier recognition imply executability; and vendor-format provenance/reference/storage/runtime/dispatch refusals.

A row becomes an implementation ticket only when it has a named producer and consumer. The ticket must then name the exact dtype/scheme, operation set, workload, target, physical layout, numerical contract, runtime predicates, cost evidence, and conformance corpus. The selected quantized backend must depend on an exact physical-widening ticket, dtype dispatchability, internal compound grouping, any selected non-per-tensor map, runtime enforcement when its valid domain requires it, and calibrated costs before making a device-optimal claim.

## Closes when

The dtype matrix and question/navigation indexes have one durable owner; the operation matrix's quantized row matches the delivered vertical; every accepted and implemented family is classified per layer with evidence; every absent family has a concrete reconsideration trigger; no cell promotes recognition into execution support; local links and terminology are read for coherence; `tkt lint` and `git diff --check` pass; and one batch `make full` passes.

## Graph maintenance

- Link `register-the-accepted-built-in-dtype-catalog`, `admit-a-dtype-dispatchability-capability-axis`, runtime semantic enforcement, quantized profile selection, selected map/grouping/backend work, and cost calibration from the exact cells they own.
- Expand or file the first bool, integer, complex, generalized-packing, reduced-precision, decimal, MX, or vendor vertical only when its trigger names a real consumer; do not create one generic “support all dtypes” implementation ticket.
- When a selected backend profile appears, make its dependency on the exact physical vocabulary, dispatchability, runtime, grouping/map, and cost work structural in the ticket graph rather than prose-only.
