---
id: register-the-accepted-built-in-dtype-catalog
title: Register the accepted built-in dtype catalog
status: todo
priority: p2
dependencies: [define-dtype-namespace-admission-policy, prototype-resolved-value-type-registry, preserve-primary-dtype-standards-evidence]
related: [enumerate-the-mature-tensor-dtype-taxonomy, prototype-quantized-value-vertical, admit-a-dtype-dispatchability-capability-axis, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, identity, validation]
---
## User-visible outcome

Every built-in dtype identity accepted by the numerical contracts has one governed, versioned source definition with its normative reference and canonical descriptor fingerprint, while operation, reference, storage, lowering, dispatch, and backend support remain separate claims that still fail closed when absent.

## Why this is a correctness gap

**Fact.** The accepted catalog recognizes logical bool; signed and unsigned integers at 2, 4, 8, 16, 32, and 64 bits; IEEE binary16/32/64/128; BF16; the accepted OCP FP8/FP6/FP4/E8M0 formats; decimal32/64/128; parameterized complex over f16/f32/f64; and the accepted OCP MX compound schemes. The standard semantic registry currently constructs only nominal f32, u4, u8, and two per-tensor strict-affine schemes. Reproduce the negative half with `rg -n 'tiler.*(bool|i2|i4|i8|i16|i32|i64|u2|u16|u32|u64|f16|f64|f128|bf16|decimal|complex|mxfp|mxint)' crates/tiler-ir/src/semantic -g '*.rs'` and then read the registration function rather than treating comments or fixtures as registrations.

**Inference.** The generic nominal, parameterized, and encoded-numeric seams are suitable, but the source does not yet make the accepted recognition claim true for most of the catalog. Adding arithmetic or physical enums while fixing recognition would falsely promote identity into support and is outside this ticket.

## Implementation keys

- Register exactly the built-in identities accepted by the governing ADRs and dtype admission policy. Each definition carries its canonical key/version, kind, complete static parameters, normative reference, immutable descriptor bytes or fingerprint, and alias/equivalence policy.
- Keep TF32 and accumulator or execution precision out of logical value identity. Keep boolean distinct from `i1`, plain integers distinct from quantized codes, physical packing distinct from logical width, decimal DPD/BID and complex planar/interleaved as physical choices, and MX or quantized values as ordered compound schemes rather than scalar aliases.
- Represent parameterized complex and compound schemes through the existing parameterized/encoded mechanisms. Do not add a universal affine struct, a universal scalar enum, or caller-declared ABI fields.
- External and vendor identities remain owner-namespaced descriptors admitted through the extension policy, not built-ins guessed from similar bit layouts.
- Recognition alone must not create an operation signature, reference evaluator, storage carrier, kernel type, target dispatch fact, or backend lowering. Add negative tests proving recognized-but-unsupported identities still reject at each currently reachable boundary.
- Exercise canonical key, alias, descriptor, fingerprint, registry ordering, duplicate, malformed-parameter, and exact-equivalence behavior table-wise for every admitted identity. Perturb the table/check once and observe it fail.

## Closes when

The accepted built-in identity catalog is constructed by the standard registry from one canonical table or equally single-source mechanism; every accepted identity has normative provenance and stable identity fixtures; aliases cannot create a second semantic identity; unsupported operations and physical paths still reject by typed reason; the implementation does not imply support beyond recognition; targeted `tiler-ir` tests and Clippy pass; every new check has demonstrated its failure path; and one batch `make full` passes.

## Graph maintenance

- Update the dtype maturity ledger owned by `own-the-dtype-support-maturity-matrix` from reservation to recognized identity only for rows actually registered here.
- Do not add physical or operation tickets merely because an identity now exists. The first named workload needing bool, integer, complex, decimal, reduced-precision, MX, or vendor execution must file a profile-specific vertical with reference, numerical, storage, dispatch, lowering, runtime, and conformance owners.
- If registration changes a versioned semantic identity domain, advance it once on the merged tree and recompute every affected pin there.
