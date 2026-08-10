---
id: implement-first-runtime-semantic-value-precondition-enforcement
title: Enforce the first direct encoded-value conformance contract at runtime
status: todo
priority: p2
dependencies: [carry-semantic-enforcement-plans-through-program-and-artifact, enforce-resolved-encoded-value-binding-conformance]
related: [implement-first-quantized-backend-profile, own-the-dtype-support-maturity-matrix, admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode, implement-workload-selected-quantized-parameter-maps]
scopes: [implementation/reference, implementation/artifact, implementation/runtime, implementation/metal, implementation/build, contracts/foundation, contracts/numerics, contracts/optimizer, contracts/artifacts, contracts/decisions, research/runtime, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, runtime, quantization]
---

## User-visible outcome

The selected direct strict-affine U8 weight input is checked against its complete resolved-value conformance contract after `RoutingCommit` and before the selected alternative's exact first consumer or any result work. The witness authorizes only the fused contraction access or the `Dequantize`/materialization stage selected for that route. Invalid scale, component structure, parameter extent, provenance, or coherence produces one deterministic terminal failure: no alternate route runs, no output is allocated or published, and no dependent effect begins.

## Corrected subject — 2026-08-08

- **Fact:** [the selected profile](../docs/research/numerics/first-quantized-lm-profile.md) is `input(compound weight) → DequantizeStrictAffine → Contraction`. Its weights are role-addressed compound interface inputs; the executed graph contains no `Quantize` or `Assemble`.
- **Fact:** [`enforce-resolved-encoded-value-binding-conformance`](enforce-resolved-encoded-value-binding-conformance.md) already derives the complete contract **for currently admitted per-tensor strict-affine U4/U8** from the `ResolvedValueType`, scans the exact logical view, orders failures deterministically, and binds evidence to origin, stability, route, view, type, shape, and validator revision. Derive admits only `strict_affine_scheme` and refuses any parameter map other than `ParameterIndexMap::per_tensor()` via `UnsupportedValueRepresentation::ParameterMap`. The selected profile is per-axis over axis 0; that map form is not constructible or checkable until [`implement-workload-selected-quantized-parameter-maps`](implement-workload-selected-quantized-parameter-maps.md) admits it into `ParameterIndexMap`.
- **Fact:** the runtime already has a one-way, consuming `Preflight::commit` that yields the only authority from which result allocation and dispatch may proceed. It has no enforcement-commit state or runtime integration for the conformance scan.
- **False, repaired:** this first runtime vertical is not about `Quantize` operation predicates and does not require an internally produced compound value. Operation-precondition enforcement for a future `Quantize` or `Assemble` producer remains outside this selected workload and is not preserved here as a hidden second subject.

**Correction — 2026-08-10.** (a) Completeness of the enforce-resolved checker is over currently admitted schemes/maps only: `ParameterIndexMapKind` is sole-variant `PerTensor`, and derive refuses non-per-tensor maps at `if map != &ParameterIndexMap::per_tensor()`. The selected per-axis subject remains blocked on map admission owned by [`implement-workload-selected-quantized-parameter-maps`](implement-workload-selected-quantized-parameter-maps.md); keep the hard dependency chain through carry/fuse rather than duplicating a second hard edge here. (b) **Fact:** `check_bound_value` still visits every logical element of every component (including full-domain U8 codes and zero points); see numerical-semantics "the implemented scan currently visits every logical element of every component". U8 proof-elision is an **Inference** this ticket may implement (contingent on physical carrier/element-width honesty and logical-view scalar-kind honesty), not present behaviour delivered by enforce-resolved.

## Vocabulary — ADR names vs runtime types

- ADR / contract lifecycle name `RoutingCommit` maps to the live consuming `Preflight::commit(self) -> RoutedDispatch` path in `tiler-runtime`. There is no Rust type named `RoutingCommit`.
- ADR `EnforcementCommit` is the missing post-routing type-state this ticket adds (committed-needs-conformance / enforcement witness). Production runtime has no `EnforcementCommit` type today: `RoutedDispatch` is immediately the allocation authority after `Preflight::commit`.

## Implementation keys

- Reconstruct the exact authoritative logical view for the selected artifact input binding and construct the existing direct-binding `ValueConformanceSubject`; do not restate the encoded value's domains in the adapter or convert them into `Dequantize` predicates.
- Consume the selected alternative's prerequisite static plan, validator key/revision, and exact protected first consumer. The fused route names its fused contraction access; the materializing route names its `Dequantize`/materialization stage. A route without an exact plan for every required direct-input conformance check is ineligible before `RoutingCommit` (`Preflight::commit`), and one alternative's plan or witness cannot authorize another.
- Resolve checker capability, host observability, producer completion/coherence strategy, hard limits, error framing, and every allocation needed by the checker before `RoutingCommit` (`Preflight::commit`), without reading tensor contents.
- Extend the one-way type state so consuming `RoutingCommit` (`Preflight::commit`) yields either a proof-elided executable route or a non-`Clone` committed-needs-conformance authority (the runtime shape of ADR `EnforcementCommit`). The latter exposes no result-allocation, encoding, submission, symbol, or executable-binding entry point.
- Consuming host enforcement is the first content read. It runs after `RoutingCommit`, before result work, and yields a privately constructed conformance witness or a terminal failure. Success alone produces the authority accepted by result allocation and dispatch.
- Bind the witness to the exact input origin/key, logical view, complete resolved type/shape/roles/maps, value version or immutability provenance, producer completion, coherence epoch, validator key/revision, committed physical alternative, exact protected first-consumer stage, and error schema. Raw pointer identity and slot position are insufficient.
- Use the existing canonical row-major diagnostic index and `(logical index, stable error code, component ordinal)` order. Traversal, chunking, worker completion, and equivalent physical views must not change the reported failure.
- Admit only the selected unpacked U8 per-axis strict-affine representation. **Proposal (this ticket introduces as implemented support):** after physical carrier/element-width establishment and logical-view scalar-kind honesty, proof-elide U8 code and zero-point content reads — `LogicalScalar::UnsignedCode(u8)` has no out-of-range U8 inhabitant once a trusted scalar is produced. **Fact (current):** `check_bound_value` still scans all U8 content (every logical element of every component) until that disposition lands. Scan positive-normal F32 scale content regardless, and validate complete ordered roles, component carrier/type/kind coherence, and parameter extents derived from the codes' axis-0 extent. There is no packed-tail check on this route.
- Preserve independent feasibility and cost for the fused and materializing alternatives. Refuse an alternative with no complete enforcement route before cost comparison; never borrow the other alternative's enforcement cost, omit the cost so an infeasible route wins, or treat `Unknown` as zero.
- Refuse every packed or other-width layout, per-tensor/per-block/other map, codebook or hierarchical scheme, complex or extension representation, nested encoded value, sparse/ragged value, device-private view, inaccessible memory, and incoherent binding by exact missing capability. Do not approximate any of them through U8 or dense F32.
- Keep the built-in checker internal. Any adapter-facing view/provenance surface remains the smallest reviewed boundary needed by this selected route; one provider does not justify a public registry.

## Runtime state and authority

- `Preflight` decides checker support, exact bindings, logical-view reconstruction, observability/coherence, hard limits, error framing, and preparation without reading contents.
- Consuming `RoutingCommit` (`Preflight::commit` → `RoutedDispatch` today; this ticket inserts committed-needs-conformance / `EnforcementCommit` before executable authority) destroys fallback authority and yields either proof-elided executable dispatch or committed-needs-conformance.
- Consuming committed-needs-conformance begins the scan and cannot fall back. Failure consumes the state, performs zero result work, and returns a typed terminal error.
- A successful `ValueConformanceEvidence` plus the exact alternative-specific static plan yields executable dispatch for only that route, input provenance, physical alternative, and protected first consumer.
- The adapter owns foreign buffers, completion observation, coherence establishment, safe byte views, and retention through final device use; device-free `tiler-runtime` owns the one-way orchestration contract.

## Adversarial evidence

- Complete ordered U8 codes/scale/zero-point roles with matching per-axis extents succeed; missing, duplicate, extra, swapped, wrong-type, wrong-shape, wrong-map, cross-input, and stale-provenance components reject.
- Once this ticket's proof-elision disposition lands: minimum/maximum U8 codes and zero points pass through the proof-elided content path. Until then, they remain content-scanned by `check_bound_value` like every other logical element. Fault-inject a carrier/type/element-width mismatch, a component presented as the wrong resolved type, an unrepresentable scalar, and `F32Bits` where the U8 scalar kind is required; each reachable dishonesty fails before evidence is minted. Normal positive scales pass; zero, negative, subnormal, infinity, qNaN, and sNaN fail through the logical content scan.
- Preflight reads zero tensor contents. The first content read occurs after `RoutingCommit` (`Preflight::commit`) and before any result allocation, encoding, submission, publication, callback, or dependent effect.
- Committed-needs-conformance cannot clone or expose executable authority. Every semantic, checker, coherence, or malformed-witness failure is terminal and cannot recover fallback.
- Perturb every subject, stability, route, view, type, role/map, version, coherence, validator, selected-alternative, protected-first-consumer, and error-schema field; evidence reuse or route authorization fails each time. Substituting the materializing consumer for the fused one, or the fused consumer for the materializing one, must fail.
- Runtime bytes and versions never enter static artifact identity. Unsupported representations name the exact refused type, scheme, map, encoding, or observability capability.
- Every new property is fault-proved by perturbing its subject and recording the resulting failure before restoration.

## Closes when

The selected direct U8 per-axis input is reconstructed and checked through the selected alternative's prerequisite plan after one-way routing commitment (`Preflight::commit` / ADR `RoutingCommit`) and before that alternative's first consumer or result work; valid evidence authorizes exactly the fused contraction access or the `Dequantize`/materialization stage selected for the route; **this ticket has introduced** U8 content proof-elision from checked carrier/type/kind (Inference→implemented support, contingent on physical carrier/element-width and scalar-kind honesty) while positive-normal F32 scales remain content-scanned; invalid input is deterministic and terminal with zero output/dependent work; every unsupported representation refuses by name; the type state (including committed-needs-conformance as ADR `EnforcementCommit`) makes pre-witness execution and post-commit fallback unrepresentable; all new checks have demonstrated failure paths; targeted package tests, doctests, Clippy, and rustdoc pass; and `tkt lint`, `make citations`, `git diff --check`, the exact-base guard, and one `make full` pass.

## Graph maintenance

- [`implement-first-quantized-backend-profile`](implement-first-quantized-backend-profile.md) consumes this completed direct-input vertical before claiming the selected dynamic-input profile executable.
- Consume the prerequisite four-document correction from [`reconcile-direct-input-conformance-order-with-adr-0033`](reconcile-direct-input-conformance-order-with-adr-0033.md). If implementation exposes a new contract discrepancy, return it to that owner rather than silently forking the lifecycle here.
- [`implement-workload-selected-quantized-parameter-maps`](implement-workload-selected-quantized-parameter-maps.md) owns admission of the selected per-axis `ParameterIndexMap`; derive currently refuses non-per-tensor maps. Related only — do not duplicate a hard dependency here; map admission sequences through fuse → carry → this ticket.
- Preserve static identity from the prerequisite plan unless this runtime slice changes a static field; dynamic bytes, versions, coherence epochs, and witnesses remain execution-scoped.
- Update the dtype maturity matrix only for the exact selected runtime path and conformance cases proven here.
