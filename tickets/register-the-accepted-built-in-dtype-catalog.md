---
id: register-the-accepted-built-in-dtype-catalog
title: Register the accepted built-in dtype catalog
status: in-progress
priority: p1
dependencies: [define-dtype-namespace-admission-policy, prototype-resolved-value-type-registry, preserve-primary-dtype-standards-evidence]
related: [enumerate-the-mature-tensor-dtype-taxonomy, prototype-quantized-value-vertical, admit-a-dtype-dispatchability-capability-axis, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, implementation/compiler, contracts/numerics, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, identity, validation]
claimed_from: todo
assignee: worker-dtype-catalog
lease_expires_at: 1785536988
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

## Outcome

**Fact — the negative half reproduced first.** The ticket's check, run at base `b623670`, returned five lines, all in `registry.rs` and all naming the test-only `tiler::complex@1` fixture; no accepted catalog row was constructed anywhere. Reading `StandardSemantics::register` and `register_standard_quantization` in full confirmed four registered identities: `tiler::f32@1`, `tiler::u4@1`, `tiler::u8@1`, and `tiler::strict-affine@1`.

**Fact — one table, thirty-one added identities.** `crates/tiler-ir/src/semantic/catalog.rs` is the single source. It registers twenty-seven nominal scalars (`bool`; `i2/i4/i8/i16/i32/i64`; `u2/u4/u8/u16/u32/u64`; IEEE `f16/f32/f64/f128`; `bf16`; OCP `f8e4m3fn/f8e5m2/f6e2m3fn/f6e3m2fn/f4e2m1fn/f8e8m0fnu`; IEEE `decimal32/64/128`), the parameterized `tiler::complex@1` constructor admitting exactly the f16/f32/f64 components ADR 0037 lists, and the six OCP MX v1.0 scheme identities. `f32`, `u4`, and `u8` moved into the same table, so no identity has two construction sites; `registry.rs` and `quantization.rs` now bind only Rust markers and operations. Each row carries a canonical key at semantic version 1, a class, complete static parameters, the uniform ADR 0027/0034 alias-and-equivalence policy, and a normative reference naming its authority, edition, exact format, preserved-source id, and its own key.

**Fact — recognition creates nothing else.** No operation signature, reference evaluator, storage carrier, kernel type, dispatch fact, or lowering was added. The reachable boundaries within `tiler-ir` are named and tested: `FrozenSemanticRegistry::infer_operation` refuses every recognized-but-unsupported identity across every registered operation at arities 0–3; `SemanticProgramBuilder::apply` refuses one with a typed `RejectedOperationApplication` while `input_resolved`/`output_resolved` accept it, which is ADR 0026's representable-without-support case built end to end; and family instance validation refuses every unadmitted complex component, every malformed complex argument list, and every MX static contract.

**Fact — the MX schemes admit no value, deliberately.** The only parameter-index map that exists is per-tensor, which is the wrong association for a 32-element block, so every MX instance is refused with `microscaling.unsupported-contract` while an unregistered scheme spelling still fails as `UnregisteredTypeAuthority`. Registering the scheme identity buys exactly ADR 0026's distinction between unknown and unsupported. The conversion, rounding, saturation, and block-wide special-value rules stay with `ocp-mx-v1.0`, whose bytes are metadata-only, and are not restated in the descriptor.

**Inference — the exponent bias is a stated evidence boundary.** The descriptor records a bias for the six OCP rows, where the vendored `mlir-builtin-types-llvmorg-22.1.8` states each value exactly and the mature dtype taxonomy tabulates the same numbers, and omits it for the IEEE and BF16 rows, whose pinned references (`ieee-754-2019`, metadata-only) are not re-derived here. The field's documentation states that condition, so absence reads as "Tiler's evidence does not fix this", never "this format is unbiased".

**Measurement — identity movement, recomputed on this branch.** The registry snapshot encodes every registered definition, so `tiler.semantic-registry.v7` content moved. No domain *version* advanced: the resolved-value-type encoding is unchanged at v3, the definition projection at v5, the registry at v7, and the standard provider stays at revision 7 — bumping it would have invalidated the admission provenance of `f32`, `u4`, `u8`, and strict-affine, none of whose meanings changed, and the whole-registry subject is already versioned by the snapshot. Exactly one pinned value moved: `crates/tiler-compiler/src/explain.rs`, `09d719dd4c2c2f37` → `928bbdb50eb505ed`. It moved for two reasons in this change — the registered identity set, and the strict-affine scheme's normative reference gaining its own key — and must be recomputed on the merged tree if any concurrent branch also moved it.

**Measurement — perturbation evidence.** Ten deliberate perturbations were applied one at a time and reverted, each run against its own focused test. Eight failed on the first attempt; two survived and exposed real gaps, which were then closed and re-perturbed. The fingerprint check could not detect facts-level duplication because the key and reference still differed, so the decimal parameters are now pinned explicitly and the fingerprint check is perturbed by collapsing `canonical_descriptor` to its domain separator; and the public enumerations' documented canonical-key order was unasserted, so both are now compared against their own sort and against the registry's own order. All ten now fail as required.

**Fact — pre-existing fixtures that squatted on governed keys.** Three `tiler-ir` tests and one `tiler-compiler` fixture registered `tiler::complex@1`, `tiler::affine@1`, or `tiler::bool@1` under the governed namespace. The `tiler-ir` fixture moved to `test::pair@1`/`test::affine@1`; the compiler fixture now uses the real governed `bool` and `complex` identities and keeps only its encoded family test-owned. Separately, the strict-affine scheme's normative reference did not name its own key and now does.

**Fact — scopes.** The brief placed the dtype maturity ledger under `contracts/numerics`; `ticketsplease.toml` maps `docs/dtype-support.md` and `docs/roadmap.md` to `contracts/navigation`. Registering the catalog also forced edits in `crates/tiler-compiler`. Both scopes were added to this ticket before those files were touched.
