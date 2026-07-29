---
id: implement-first-runtime-semantic-value-precondition-enforcement
title: Implement first runtime semantic value-precondition enforcement
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, carry-semantic-enforcement-plans-through-program-and-artifact]
related: [own-the-dtype-support-maturity-matrix]
scopes: [implementation/reference, implementation/artifact, implementation/runtime, implementation/metal, implementation/build, contracts/foundation, contracts/numerics, contracts/optimizer, contracts/artifacts, contracts/decisions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, runtime, quantization]
---

## User-visible outcome

The first real strict-affine `Quantize` tensor-value semantic preconditions are enforced over their exact authoritative logical views with deterministic typed failures, complete execution-scoped witnesses, and no possibility that invalid input selects another plan, starts result work, publishes output, or falls back.

## Dependency correction

- **Fact:** no typed `SemanticPrecondition` producer exists in `crates/`, and strict-affine `Quantize` has no physical/runtime route.
- **Fact:** the preserved spike proves the ownership state machine and deterministic reduction only; it does not prove compound reconstruction, artifact codecs, checker provenance, or a governed executable consumer.
- **Inference:** a synthetic loader fixture cannot close this outcome. The work is dependency-ordered through `produce-typed-strict-affine-quantize-semantic-preconditions`, `admit-strict-affine-quantize-physical-candidate`, and `carry-semantic-enforcement-plans-through-program-and-artifact`.

## Implementation keys

- Consume the governed strict-affine `Quantize` `NoNaN` expressed-value and `PositiveFiniteScalar` scale obligations. A NaN-only implementation is incomplete under the accepted reference semantics.
- Keep semantic operation preconditions distinct from index-region coordinate safety and physical representation applicability. A failed semantic precondition is invalid input; a disproved lowering access bound is invalid compiler output; an unsupported physical encoding is capability or feasibility.
- Define the typed residual obligation and witness identity over the exact predicate, logical subject and view, complete resolved value type, component roles and coordinate maps, value version or immutability provenance, producer completion, and coherence dependencies. Raw pointer identity is insufficient.
- Select an explicit host `EnforcementPlan` only when the runtime capability can reconstruct and inspect the authoritative logical view. Bind the checker provider and output-affecting revision, resource limits, deterministic error schema, observability requirements, and cost inputs into the selected plan and explain identity.
- Carry the complete derived logical and physical component contract through artifact identity and ABI. Never reintroduce caller-declared ABI facts, infer component roles from slot position, or treat integer codes alone as the quantized logical value.
- Implement exact host reconstruction only for the selected first strict-affine U4/U8 per-tensor representations. Packed Boolean, other packed widths/layouts, complex planar/interleaved, nominal extensions without a versioned evaluator, non-per-tensor maps, codebook/hierarchical/mask/outlier roles, nested encoded values, sparse/ragged values, and device-private or incoherent views remain named unsupported before routing.
- Use canonical logical row-major index, stable error code, and obligation ordinal for deterministic diagnostics, never physical byte offset or worker order.
- Resolve checker capability, observability, cost, and every preparation obligation before `RoutingCommit`, but begin the actual host tensor-value validation only afterward at `EnforcementCommit`, as ADR 0033 requires. Starting the scan while fallback authority still exists would make semantic work coexist with an alternate route and weaken the accepted ownership state machine.
- Represent the one-way runtime state explicitly as preflight, `RoutingCommit`, committed-needs-enforcement, successful enforcement, and executable dispatch. The committed-needs-enforcement state must not expose an executable entry point, and an enforcement witness must not be cached independently of the value version, coherence dependencies, checker revision, and exact obligation identity that make it valid.
- A successful witness may authorize the one committed route. After `EnforcementCommit`, a semantic failure, malformed witness, coherence failure, or enforcement execution failure cannot trigger ordinary fallback.
- Keep the built-in first checker internal. Expose only the smallest reviewed adapter-facing authoritative logical-view and value-provenance boundary; no public registry is justified by one provider.

## Runtime state and authority

- `Preflight` resolves checker support, complete bindings, accessible logical views, producer completion/coherence strategy, hard limits, error framing, and all allocations/preparation without reading tensor contents.
- Consuming `RoutingCommit` returns either a proof-elided executable route or a non-Clone committed-needs-enforcement authority that exposes no entry points, symbols, or executable bindings.
- Consuming host enforcement is `EnforcementCommit`. It binds the exact obligation, subject/view/type/roles/maps, value version or immutability provenance, producer completion, coherence epoch, checker provider/revision, error schema, and committed route identity into a privately constructed witness.
- Success yields executable dispatch for exactly that route and provenance. Semantic failure, checker failure, malformed record, completion failure, or coherence failure consumes the committed state and is terminal.
- The adapter, not device-free `tiler-runtime`, owns foreign buffers, completion observation, coherence establishment, safe byte views, and resource retention through final device use.

## Adversarial evidence matrix

### Exact logical reconstruction

- Complete ordered codes/scale/zero-point roles succeed for the selected U4/U8 per-tensor representation; missing, duplicate, extra, swapped, wrong-type, wrong-shape, wrong-map, or cross-value roles reject.
- Constant and runtime parameter roles produce the same semantic contract; runtime payload changes evaluation but not static artifact identity, while embedded constants change program identity.
- Empty logical views succeed without reading allocation padding. U4 visits exactly logical nibbles and excludes unused tail bits; bad physical tail canonicality remains a separate preflight failure.
- Canonical logical row-major diagnostic indices remain equal across equivalent byte and packed layouts.

### Witness and capability binding

- Independently perturb predicate, obligation/ordinal, subject, view, resolved type, role/order, map, value version, immutability proof, producer completion, coherence epoch, checker key/revision, error schema, hard limit, and committed route; every perturbation prevents reuse or rejects.
- Equal raw addresses never rescue changed provenance. A new runtime value version always requires new enforcement unless exact compiler-proved immutability authorizes reuse.
- Missing evaluator, inaccessible memory, absent coherence, stale revision, or insufficient hard resources rejects before `RoutingCommit`; malformed or dishonest metadata is an invariant error, not unsupported capability.

### Ownership, ordering, and publication

- Preflight cannot read tensor contents. Checker preparation completes before `RoutingCommit`; the first content read occurs only after it.
- Committed-needs-enforcement cannot expose or clone executable authority. Failure performs zero result work and publishes no output, mutation, callback, or dependent effect.
- Every attempted fallback after `EnforcementCommit` is impossible by type or returns a terminal misuse error; proof-elided routes perform no enforcement commit.
- Checker execution failure takes precedence over any populated semantic error record; successful framing is validated rather than inferred from zeroed bytes.

### Deterministic diagnostics and refusals

- Multiple failures select minimum `(logical row-major index, stable error code, obligation ordinal)` under varied traversal, chunking, and worker completion order.
- Same-index code ordering and same-index/code ordinal ordering are explicitly tested.
- Every unsupported packed, complex, extension, mapped, nested, sparse, ragged, private, or incoherent representation names the exact resolved type/scheme/map/encoding and missing capability. No representation is approximated through U4 or f32.

### Required failure-path demonstrations

- Deliberately observe failures when NaN is accepted, non-positive/non-finite scale is accepted, roles are swapped, unused tail is scanned, changed version reuses a witness, content is read before routing, fallback remains after commit, executable state appears before witness, malformed records are interpreted, checker failure is ignored, first-writer diagnostics become nondeterministic, runtime payload enters static identity, provider revision is omitted, or an unsupported representation is approximated.

## Closes when

The reviewed public runtime type-state and adapter view boundary enforces both strict-affine semantic predicates on the real physical candidate; valid and invalid compound fixtures exercise constant and runtime parameter roles; exact selected representations reconstruct correctly and every other family refuses by name; validation starts only after one-way routing commitment and before result work; failures cannot publish or fall back; deterministic diagnostics and complete witness binding survive every perturbation above; all new checks have demonstrated failure paths; targeted per-package tests and Clippy pass; `tkt lint`, `git diff --check`, and one `make full` pass.

## Graph maintenance

- Update numerical semantics, artifact ABI, runtime execution order, correctness testing, and accepted ADR 0033 application status together. Remove stale pre-routing semantic-scan wording while retaining structural/canonical preflight.
- Preserve static artifact/cache domains established by the prerequisite ticket unless this runtime slice changes static fields; dynamic bytes, value versions, and coherence epochs belong only to the execution witness.
- Revisit a public provider registry only when a second independently installable enforcement authority exists. The first runtime integration should expose the smallest reviewed adapter-facing boundary.
- Make `implement-first-quantized-backend-profile` consume this completed vertical before claiming the selected dynamic-input profile executable or optimal.
- Update the dtype maturity matrix only for the exact runtime path and conformance cases proven here.
