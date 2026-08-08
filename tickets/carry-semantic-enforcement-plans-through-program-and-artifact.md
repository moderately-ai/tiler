---
id: carry-semantic-enforcement-plans-through-program-and-artifact
title: Carry direct bound-value conformance enforcement plans through program and artifact identity
status: todo
priority: p2
dependencies: [enforce-resolved-encoded-value-binding-conformance, fuse-quantized-weight-decode-into-the-strict-contraction]
related: [implement-first-runtime-semantic-value-precondition-enforcement, implement-first-quantized-backend-profile]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/foundation, contracts/numerics, contracts/optimizer, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, artifact, identity, quantization]
---

## User-visible outcome

The selected direct encoded weight input has one static enforcement plan derived from its resolved-value conformance contract. That plan survives selected-plan, verified-program, artifact, codec, cache, and explain boundaries and protects the first fused contraction stage that consumes the value, without caller restatement or identity loss.

## Corrected subject and dependencies — 2026-08-08

- **Fact:** [`enforce-resolved-encoded-value-binding-conformance`](enforce-resolved-encoded-value-binding-conformance.md) already delivers `ResolvedValueConformanceContract`, exact `ValueConformanceSubject`/`ValueConformanceEvidence`, the deterministic logical scan, proof reuse, and typed unsupported-representation refusals for direct bindings.
- **Fact:** [the selected profile](../docs/research/numerics/first-quantized-lm-profile.md) supplies role-addressed strict-affine U8 weights as compound interface inputs and reaches them through `DequantizeStrictAffine`; it contains no `Quantize` or `Assemble` producer.
- **Fact:** [`fuse-quantized-weight-decode-into-the-strict-contraction`](fuse-quantized-weight-decode-into-the-strict-contraction.md) is the first real physical consumer. It owns the stage at which decode becomes a weight operand access and retains the materializing alternative.
- **False, repaired:** a future physical `Quantize` candidate is not needed to make this record real. [`admit-strict-affine-quantize-physical-candidate`](admit-strict-affine-quantize-physical-candidate.md) is obsolete and no longer appears in this ticket's dependency graph.

## Implementation keys

- Derive each record from the selected direct input's complete conformance contract and exact protected fused-consumer stage. Do not translate value conformance into a `Dequantize` operation predicate or attach a route-global untyped list.
- Represent proof-elided conformance and required host-scan conformance as distinct physical dispositions over the same resolved-value meaning. Device pre-scan and transactional execution remain named unsupported until a provider can populate every required field and observation boundary.
- Bind the exact interface input key, logical view, complete resolved type and shape, ordered component roles/types/shapes/maps, validator key/revision, checker mechanism, protected stage, observability/coherence requirements, hard resource limits, deterministic error schema, and cost inputs into selected-plan and kernel-program identity.
- Keep dynamic bytes, addresses, value versions, and coherence epochs out of static artifact/cache identity. Encode the static dependency schema the execution witness must later satisfy.
- Add builder-owned insertion and verified read-only views on `VerifiedKernelProgram`; artifact construction derives the record from that verified authority. Do not add caller-declared semantic facts to `BindingSpec`.
- Project the record through artifact construction, verification, canonical codec, decoded views, required-feature/schema validation, and explain output. Reject dangling, duplicate, out-of-order, unknown, or oversized records before routing.
- Select hard feasibility before cost. Missing logical-view reconstruction, host observability, coherence, checker revision, or resource capacity is a typed pre-routing refusal; an `Unknown` cost is not silently treated as zero.
- Prepare the checker and prove that the committed route can enforce it before `RoutingCommit`; the plan must state that the actual tensor-value scan begins only after the one-way commit and before result work.
- Support only the selected unpacked U8, per-axis strict-affine input after its parameter-map dependency has landed through the fused-consumer ticket. Every packed, other-map, other-scheme, nested, sparse/ragged, complex, extension, private, or incoherent representation remains explicitly unsupported.
- Advance only identity or schema grammars that the implemented record actually changes, and derive each step or hold from the final producer-filled fields. Do not require version steps in advance or duplicate fields already committed by an authoritative child identity.

## Adversarial evidence

- Encode/decode/re-encode is byte-identical; an older reader fails closed on the new required feature/schema if one is needed.
- Proof-elided and host-enforced plans preserve the same semantic contract but differ in physical/program/artifact identity.
- Independently perturb input key, view, resolved type/shape, component role/order/type/map, validator key/revision, protected stage, mechanism, observability, error schema, and hard limit; each changes identity or rejects as appropriate.
- Runtime payload/version changes do not alter static identity, while an embedded constant or static plan-field change does.
- Missing evaluator or host observability rejects before cost; unknown predicate/provider/schema versions, dangling references, duplicate ordinals, noncanonical ordering, and oversized counts reject deterministically.
- Removing the protected fused-stage dependency or a component map from canonical encoding makes a fault-injection test fail before restoration.

## Closes when

The selected direct input's conformance contract reaches an exact static enforcement plan protecting the fused contraction consumer and round-trips through every required static boundary; proof-elided and host-scan dispositions are explained; unsupported representations fail closed by name; any changed identities are derived and recomputed on the merged tree; every new check is proven able to fail; targeted package tests and Clippy pass; and `tkt lint`, `make citations`, `git diff --check`, the exact-base guard, and the batch gate pass.

## Graph maintenance

- Release [`implement-first-runtime-semantic-value-precondition-enforcement`](implement-first-runtime-semantic-value-precondition-enforcement.md) when this static record is complete.
- Correct numerical, artifact ABI, runtime-order, correctness-testing, and ADR 0033 application text together, including stale claims that direct bound-value scans happen before `RoutingCommit` or that this first plan carries `Quantize`/`Assemble` operation predicates.
- Do not introduce a public enforcement-provider registry until a second independently installable provider exists.
- Recompute expansion-cache and artifact pins only for grammars changed on the merged tree.
