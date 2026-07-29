---
id: admit-strict-affine-quantize-physical-candidate
title: Admit strict-affine Quantize as a committed physical candidate
status: todo
priority: p2
dependencies: [produce-typed-strict-affine-quantize-semantic-preconditions, group-internal-compound-materializations-by-logical-value, scope-first-quantized-lm-profile, admit-a-dtype-dispatchability-capability-axis]
related: [implement-first-quantized-backend-profile, implement-first-runtime-semantic-value-precondition-enforcement, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/reference, implementation/metal, implementation/build, implementation/runtime, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, physical-candidate, metal]
---

## User-visible outcome

The profile selected by the workload ticket has one real strict-affine `Quantize` physical candidate whose complete compound result can reach a later stage and whose execution remains unavailable until every residual semantic precondition has an enforcement dependency. This supplies genuine result work for the runtime-enforcement vertical without pretending a synthetic artifact is an executable consumer.

## Why this ticket exists

- **Fact:** no physical `QuantizeStrictAffine` program exists; the only strict-affine scalar program is U4 dequantization.
- **Fact:** internal compound grouping and the workload-backed scheme/backend selection remain open prerequisites, while the structural U4 dequantization artifact is not safely executable on the measured Metal target under its current subnormal-preservation contract.
- **Inference:** carrying enforcement records through fabricated program fixtures would test serialization but would not satisfy the promised governed end-to-end route. The candidate must be real, but it need not claim device optimality while costs remain `Unknown`.

## Implementation keys

- Refine this ticket from `scope-first-quantized-lm-profile` before implementation with the exact scheme, code type, parameter map, operation signature, target family, storage layouts, accumulator/output types, and conformance corpus.
- Lower the exact strict-affine `Quantize` occurrence and its complete producer-derived logical compound result. Consume `LogicalValueId` grouping and bind stage accesses by logical value plus component role; never infer association from slot order or same-shaped storage.
- Preserve expressed input, scale, zero-point, complete result type, ordered component roles, maps, storage encoding, producer dependencies, and residual semantic obligations through candidate feasibility and program construction.
- The candidate may be selected only when target numerical honourability, dtype dispatchability, storage access, resource feasibility, and an enforcement mechanism for every residual predicate are all available. Before the enforcement ticket lands, retain it as a committed physical candidate that cannot expose executable dispatch authority.
- Implement only the physical vocabulary required by the selected profile. Packed U4 and byte U8, native low-bit arithmetic, helper-code arithmetic, and unpack/dequantize paths are distinct capabilities.
- Match exact codes and exact reference bits where the contract is exact. Record every unsupported scheme, map, layout, target realization, and operation signature with a typed refusal.
- Keep semantic invalidity separate from physical tail canonicality, malformed grouping, target infeasibility, and numerical unhonourability.
- Do not claim device optimality until analytical and calibrated cost evidence can compare all legal alternatives. `Unknown` cost remains truthful.

## Adversarial evidence

- Missing, duplicate, extra, swapped, wrong-type, wrong-shape, wrong-map, or cross-logical-value components reject before stage verification.
- Constant and runtime scale/zero-point roles preserve the same logical contract while runtime payload bytes remain outside static artifact identity.
- Packed logical iteration excludes unused tail bits; noncanonical tail storage is rejected by the physical owner rather than reported as semantic NaN.
- Every residual predicate protects the exact first consuming stage; removing or retargeting that edge prevents executable admission.
- Each unselected code type, parameter map, encoding, accumulator, output type, target family, and numerical policy has a named typed refusal.
- Perturb grouping, role, map, storage, target honourability, or residual dependency and observe the check fail before restoring it.

## Closes when

The selected strict-affine `Quantize` candidate performs real result work against the normative reference for the exact bounded profile; its complete internal compound result reaches a consumer; it is structurally impossible to expose executable authority while an unplanned residual predicate remains; all neighbouring profiles fail closed by name; all identity pins are recomputed on the merged tree; every new check has a demonstrated failure path; targeted package tests and Clippy pass; `tkt lint`, `git diff --check`, and the batch gate pass.

## Graph maintenance

- Add the exact dtype-dispatchability, physical-vocabulary, numerical-policy, and cost tickets selected by `scope-first-quantized-lm-profile` before claiming this ticket.
- Release `carry-semantic-enforcement-plans-through-program-and-artifact` when the real candidate exists.
- Make `implement-first-quantized-backend-profile` depend on this candidate and the completed runtime-enforcement vertical. That later profile owns broader workload coverage, conformance, calibrated performance, and any claim of backend optimality.
- Update the dtype maturity matrix only for the exact candidate cells proven here.
