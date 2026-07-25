---
schema: "tiler-doc/v1"
id: "ADR-0011"
kind: "decision"
title: "Resolve numerical permissions per operation"
topics: ["numerics","optimization","semantics"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.operation-conformance-matrix"]
ticket: "numerical-policy-contract"
---

# 0011: Resolve numerical permissions per operation

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [operation conformance matrix](../research/numerics/operation-conformance-matrix.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

A single graph-wide `exact` or `fast` mode is simple, but numerical freedoms
are not uniformly relevant or desirable. A program may permit contraction for
a multiply-add while forbidding reduction reassociation and approximate
transcendentals. Treating `fast` as one switch can silently relax unrelated
operations.

Fully independent per-operation policy is precise but needs an outer authority
that limits what frontend defaults and optimizer passes may enable.

## Decision

The program numerical policy is a ceiling: the maximum relaxation authorized
anywhere in the program. Each operation carries resolved effective permissions
for its applicable numerical dimensions. Effective permissions combine the
program ceiling, any tighter per-operation restriction, and the operation's
declared capabilities; they can never exceed the ceiling.

Named user-facing modes may initialize the ceiling, and frontends may expose
region or operation overrides. Before semantic optimization, all such controls
resolve to the same canonical per-operation representation. Later passes do
not consult ambient modes or frontend state.

Every semantic rewrite and physical alternative declares which effective
permission it consumes. Backend compiler flags are derived from the resolved
program and cannot grant additional freedoms.

## Consequences

- One program can safely optimize numerically different regions under different
  effective permissions.
- Enabling contraction does not implicitly permit reassociation,
  approximations, or exceptional-value assumptions elsewhere.
- Canonical identity includes both the policy ceiling and resolved
  per-operation permissions.
- Explain output can identify the exact permission that admitted or rejected an
  alternative.
- Frontend APIs may offer global, regional, or local controls without changing
  compiler-core semantics.

## Alternatives considered

A graph-wide exact/fast enum is compact but over-broad. Per-operation policy
without a graph ceiling allows local defaults to exceed the user's overall
authorization. Deferring permission resolution until backend lowering makes
logical rewrite legality depend on the selected target.

## Implementation boundary

Added 2026-07-25 by [`re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening`](../../tickets/re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening.md), which moved `implementation_status` from `not-started` to `partial`. This section states which clauses that value rests on and which it does not, read at `43f685f`. It is a status record and adds no decision.

**Realized — a program-wide resolved contract that no later pass may exceed.** `crates/tiler-compiler/src/session.rs` exposes a `NumericalContract` enum with two named user-facing modes, `StrictF32` and `FlushSubnormalsToZeroF32`, which `resolve` maps to `StrictF32NumericalContract::governed` and `governed_flush_to_zero` in `crates/tiler-compiler/src/request.rs`. `governed_profile` there is the single admission authority three call sites share — the request boundary, the per-target verification, and the physical schedule verifier — so a contract outside the registered set is rejected once rather than at three drifting sites. No later pass consults an ambient mode, and the resolved contract travels as `tiler_ir::schedule::NumericalRealization`.

**Realized — the operation-capability term of the effective permission.** `crates/tiler-compiler/src/fusion_legality.rs` carries `FusionNumericalCapabilities`, a compiler-owned registry mapping an operation family `OpKey` to the `FusionOperationRole` the governed provider declares for it — `ValueSource`, `ElementwiseArithmetic`, or `OrderedReduction`. Resolution is a checked lookup rather than a graph-shape match, and an unregistered family fails closed to `FusionLegality::Unknown`. This is the "operation's declared capabilities" term of the Decision's three-term combination, resolved per member operation.

**Realized — every alternative declares which permission it consumes.** The same module discharges each numerical obligation against the member's role *and* the resolved contract, producing an ordered list of `DerivedObligation`s each carrying a `FusionEvidenceClass`, and it is on the compile path from `crates/tiler-compiler/src/pipeline.rs` rather than behind `#[cfg(test)]`. Contraction, reduction reassociation, and operand permutation are discharged separately, so granting one never discharges another. `docs/compiler/optimizer.md` states the same rule for each logical-exploration rewrite.

**Realized — backend flags are derived and cannot grant freedoms.** `crates/tiler-metal/src/emit.rs`'s `realization_requirements` derives `MetalNumericalRequirement::{NoFloatingPointContraction, SafeMathMode}` from the resolved realization, and `crates/tiler-metal/src/golden_compilation.rs`'s `realization_honours` checks that the driver selection actually delivers each requirement, with a same-crate exhaustive match so a new requirement stops the module compiling until someone names the selection that satisfies it.

**Realized — identity carries the policy.** `crates/tiler-ir/src/schedule/model.rs` and `crates/tiler-ir/src/kernel/model.rs` both encode the profile key, the canonical NaN bits, both subnormal dimensions, and both permissions into canonical identity through exhaustive matches over vocabularies deliberately not marked `#[non_exhaustive]`, so widening one is a build error rather than an identity collision.

**Unrealized — the ceiling is a single value, not a ceiling.** There is no per-operation restriction to intersect with, no region or operation override, and therefore no canonical per-operation permission representation for controls to resolve into before semantic optimization. The Decision's "combine the program ceiling, any tighter per-operation restriction, and the operation's declared capabilities" has two of its three terms; the middle one has no representation at all. A single program-wide contract behaves like a ceiling only because nothing can currently be tighter than it, which is not the same claim.

**Unrealized — the granularity is the region, not the operation.** `NumericalRealization` is carried per scheduled region, and the Decision's "each operation carries resolved effective permissions" is not what the tree does. The per-operation term that exists is the *capability* registry, not a per-operation permission.

**Consequently unrealized — "one program can safely optimize numerically different regions under different effective permissions".** Every region of one program compiles under the one contract stated in its request.
