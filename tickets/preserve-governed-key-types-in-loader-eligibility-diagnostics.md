---
id: preserve-governed-key-types-in-loader-eligibility-diagnostics
title: Preserve governed key types in loader eligibility diagnostics
status: in-progress
priority: p1
dependencies: [select-executable-variants-across-registered-backend-families]
related: [accept-the-loader-variant-eligibility-vocabulary]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, public-boundary, correctness]
claimed_from: todo
assignee: worker-loader-eligibility-keys
lease_expires_at: 1786649004
---
## User-visible outcome

Loader eligibility refusals retain governed backend, representation, and target-profile key types rather than erasing them to strings, so callers can compare and route diagnostics without reparsing governed text or mixing key domains.

## Facts to re-verify

**Fact — routing decisions are already typed.** `ExecutionEnvironment` carries `BackendKey`, `RepresentationKey`, and `TargetProfileRef`; `variant_eligibility` compares those typed values before constructing a diagnostic.

**Fact — the diagnostic boundary erases them afterwards.** `UnsupportedRepresentation` clones four `.as_str()` values into `String`; `UndispatchableDType::host_profile` does the same; `TargetCompatibility` stores profile-key mismatches as `String`. The conversions cannot admit a route, but they make a public value documented as a governed key accept arbitrary text and lose compile-time distinction between key domains.

**Fact — no dependency or asymptotic-cost reason requires erasure.** The runtime already imports the owned artifact key newtypes. Cloning a typed key clones the same owned string allocation the current `.to_owned()` path makes.

## Required outcome

- Use `BackendKey` and `RepresentationKey` in `UnsupportedRepresentation`.
- Use `TargetProfileKey` throughout the directly coupled `TargetCompatibility` and `UndispatchableDType::host_profile` payloads.
- Update every construction, public pattern match, display implementation, test, conformance probe, and prototype consumer as one source-breaking pre-production sweep.
- Preserve exact displayed key text and every routing/refusal decision.
- Keep `FilteredVariant`'s public leaf-data fields as accepted; do not turn this repair into an accessor redesign.

## Stop conditions

Stop if a typed replacement introduces a dependency cycle, requires changing artifact identity/bytes, or reveals a genuine external compatibility commitment. None is known at filing.

## Required evidence

Perturb one backend, representation, and target-profile typed subject independently with assertions unchanged; show each diagnostic still reaches and names the intended field. Run runtime/package consumers, doctests, Clippy and rustdoc with warnings denied, citations, lint, exact-base guard, and the exact-tip publication gate required by the touched crate paths.
