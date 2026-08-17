---
id: decide-the-backend-provider-conformance-harness-public-surface
title: Decide the backend-provider conformance harness public surface
status: in-progress
priority: p1
dependencies: [exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio]
related: [publish-the-backend-provider-conformance-suite, audit-backend-authoring-against-all-thirteen-responsibilities, specify-the-consumer-neutral-backend-provider-composition-contract, make-explain-dispositions-assertable-by-a-conformance-suite]
scopes: [implementation/conformance, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, backend-providers, conformance]
claimed_from: todo
assignee: worker-conformance-facade
lease_expires_at: 1786979471
---
## User-visible outcome

Third-party backend authors have one accepted reusable conformance-harness facade, or one explicit typed deferral with a reconsideration trigger, before Tiler publishes a suite that claims bounded provider correctness.

## Exact-current discovery — 2026-08-17 at `d002cd55406522922e5eb750c8c4d9033dde4469`

1. **Fact — verified.** ADR 0106 admits `tiler-conformance` as the cross-layer evidence member and its complete crate header says `There is none` under `# Public surface`: every module is test-only and every item remains crate-private. The crate is the mechanically correct owner, but its admission accepted no reusable facade.
2. **Fact — verified.** `publish-the-backend-provider-conformance-suite` expressly requires reusable public types and calls for third-party authors. Implementing it under the current crate boundary would either export an unaccepted namespace or create a second owner elsewhere. ADR 0075 reserves that choice to Tom.
3. **Fact — verified.** ADR 0090 deliberately has no runtime-adapter registry. A consumer calls `route_with_adapter` with the adapter it selected, so “missing runtime adapters” is not a constructible discovery failure. Wrong, refusing, or incompatible explicitly supplied adapters are constructible conformance subjects.
4. **Fact — verified.** `ProviderOffer` says an empty offer is legitimate. The packet must preserve the difference between silence, a typed decline, malformed provider output, and absence of any feasible global plan.
5. **Fact — repaired current population.** The thirteen-row backend-composition matrix is historical audit structure, not thirteen current public seams. Scalar lowering is retired under ADR 0105; opaque-call declaration remains compiler-owned; the remaining externally participated rows and these exclusions must each be named and counted at the packet base. A green suite may not imply it exercised rows it deliberately cannot expose.
6. **Fact — verified dependency boundary.** Complete artifact-level selected-physical provenance and compile-profile selection provenance are not yet carried; their exact implementation tickets remain blocked on earlier public decisions. The suite may decide its facade now, but the implementation cannot claim those rows until the carriers land.
7. **Fact — verified explain boundary.** Public explain products expose human rendering, not a structured disposition iterator, and their contract forbids parsing that rendering. The exact facade must choose structured explain assertion or an explicit documented exclusion; `make-explain-dispositions-assertable-by-a-conformance-suite` cannot depend on the finished suite without creating the old backward edge.

## Required decision packet

- Re-audit the complete current `tiler-conformance` module/test population, ADR 0106, the provider-composition responsibility matrix and corrections, every public provider/compiler/build/artifact/runtime seam the suite would exercise, and every out-of-crate fixture or spike proposed as evidence.
- Enumerate the nondominated ownership/facade candidates, including retaining the private gate-only crate, publishing a minimal harness from `tiler-conformance`, splitting reusable structural checks from device-reaching executions, further bounded research, and typed deferral where each is genuinely applicable. Do not manufacture a new crate or put provider conformance in a production layer merely to avoid the existing no-public-surface decision.
- Fix exact modules, types, constructors, result/refusal vocabulary, caller-owned fixtures, host-unavailable reporting, async-lifetime boundary, deterministic population counts, and which checks are structural versus provider-supplied semantic evidence. Keep `tiler-reference` as the sole oracle and exclude benchmarking, certification of arbitrary mathematics, performance, dynamic plugins, and silent hardware skips.
- Decide explain coverage atomically: a structured non-rendered assertion surface, or a documented suite exclusion that prevents any full-disposition claim. State the downstream effect on `make-explain-dispositions-assertable-by-a-conformance-suite`.
- Separate the facade decision from implementation prerequisites. The physical-provenance and compile-selection carriers may remain blocked while this packet fixes the public shape, but the suite implementation must depend on them and may not default, infer, or omit their rows.
- State correctness, fail-closed strictness, public compatibility, host runtime/memory, identity/schema, unsupported-population, strongest-counterargument, reversal evidence, and independent subject perturbations for every survivor. Pass independent strongest-reasoning review before queueing one Tom question.

## Stop boundary

This ticket authorizes research and ticket/document corrections only. It authorizes no public export, new crate, module move, production implementation, provider default, adapter discovery, or conformance claim. Do not add it to the Tom decision queue until the exact current packet is Pareto-complete and independently reviewed.

## Closes when

Tom accepts one exact current-source facade or an explicit typed deferral with a trigger; the suite, explain, provenance, and selection dependencies then reflect that answer without a cycle or an implicit coverage gap.
