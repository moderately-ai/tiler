---
schema: "tiler-doc/v1"
id: "ADR-0014"
kind: "decision"
title: "Separate reassociation from operand permutation"
topics: ["numerics","reductions","optimization"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "numerical-policy-contract"
---

# 0014: Separate reassociation from operand permutation

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Changing `(a + b) + c` to `a + (b + c)` regroups operands without changing
their logical order. Changing it to `(a + c) + b` also permutes them. Both can
change floating-point results, but they are different freedoms. Some reduction
combiners support regrouping but are not commutative, and some numerical
contracts may intentionally permit one transformation without the other.

A single unordered-reduction permission grants both freedoms even when only
one is necessary.

## Decision

Reduction order contracts represent reassociation and operand permutation as
independent dimensions. Reassociation permission never implies permutation
permission, and permutation permission never implies reassociation permission.

Each transformation requires two independent facts:

1. the operation declares the applicable algebraic capability; and
2. the operation's resolved numerical permissions authorize consuming it.

In particular, permutation requires a commutative operation capability as well
as permission to reorder. A physical topology proves its regrouping and
permutation behavior separately against the semantic contract.

## Consequences

- An ordered parallel tree can regroup operands without silently permitting
  arbitrary permutation.
- Associative but noncommutative combiners remain optimizable within their
  actual capabilities.
- Scheduler alternatives and rejection explanations name the precise order
  freedom they require.
- Operation capability does not itself authorize a numerical relaxation; the
  program ceiling and resolved per-operation permissions still govern it.
- Reduction legality has one additional explicit dimension.

## Implementation boundary

Added 2026-08-01 by [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](../../tickets/re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which moved `implementation_status` from `not-started` to `partial` and replaced the 2026-07 exclusion reason "physical reduction topology exists, but no typed semantic order-contract vocabulary". This section states which clauses that value rests on and which it does not, read at `2aa0824`. It is a status record and adds no decision.

**Realized — the two dimensions are independent everywhere they appear.** `NumericalRealization` carries `reassociation` and `permutation` as separate `NumericalPermission` fields (`crates/tiler-ir/src/schedule/numerics.rs:219`, `:221`) with separate accessors at `:272` and `:278`; `FusionObligation::ReductionReassociation` and `::ReductionOperandPermutation` are separate obligations under separate rule keys (`crates/tiler-compiler/src/fusion_legality.rs:388`, `:390`, `:404`, `:405`), pushed independently by `push_reduction_obligations`, whose own comment at `:1216` names this decision; and both permissions are encoded independently into canonical schedule and kernel identity. Neither dimension is derived from the other at any site.

**Realized — the operation declares an algebraic capability, separately from the permission that consumes it.** `OperationAlgebraicCapabilities` (`crates/tiler-ir/src/semantic/operation.rs:922`) is an operation-owned, identity-encoded declaration whose documentation states this decision's rule directly: "a missing declaration is unknown, never evidence that the inverse law holds", and "consuming one still requires the independently resolved numerical permission for the rewrite". The governed `f32` addition and multiplication declare ordered associativity (`crates/tiler-ir/src/semantic/registry.rs:2193`); the contraction family deliberately declares none, and says why (`crates/tiler-ir/src/semantic/contraction.rs:909`).

**Realized — a rewrite requires both facts and reports which one failed.** `OrderedReassociationRule::evaluate` (`crates/tiler-compiler/src/normalize.rs:699`) declines with `semantic.ordered-associativity-undeclared` when the operation declares no capability (`:711`) and with `numerical.reassociation-forbidden` when the resolved contract does not permit the transform (`:749`), producing two separately classified assessments rather than one verdict. That is the decision's "two independent facts", on the compile path.

**Realized — a physical topology proves its regrouping behaviour against the contract, separately from permutation.** The multi-dispatch split is admitted only when reassociation is permitted and the cooperative workgroup tile likewise (`crates/tiler-ir/src/schedule/builder/reduction.rs "family.consumes_reassociation && !*permits_reassociation"`, once in each gate), each checked on reassociation alone; both record the resolved permutation permission and neither consults it to admit the topology, so granting permutation cannot make an otherwise illegal split legal. `crates/tiler-compiler/src/physical.rs:1291` builds the split's topology reading both permissions from the resolved contract rather than hardcoding either.

**Citation repair — 2026-08-19 by [`re-anchor-the-schedule-builder-line-citations`](../../tickets/re-anchor-the-schedule-builder-line-citations.md), and one strategy the paragraph above does not name.** `crates/tiler-ir/src/schedule/builder.rs` became the `crates/tiler-ir/src/schedule/builder/` directory, so the retired `:831` and `:967` pins are replaced above by one quoted anchor that matches both gates; the claim is unchanged and was re-read at the split. A third topology now carries the same rule and postdates this section: `crates/tiler-ir/src/schedule/builder/contraction.rs "|| !*permits_reassociation"` requires the permission outright for the cooperative contraction. This adds no decision — the rule it obeys is the one stated above.

**Unrealized — commutativity has no capability to declare.** `OperationAlgebraicCapabilities` has exactly one law, `ordered_associativity`. The decision's "permutation requires a commutative operation capability as well as permission to reorder" therefore has only its permission half: no operation can declare commutativity, and where a family is in fact commutative the property is recorded as a definition fact string (`SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` in `crates/tiler-ir/src/semantic/softmax.rs` — cited by constant because its line number has drifted) that no rule consults.

**Unrealized — nothing consumes a permutation permission.** No rewrite rule and no admitted physical strategy reorders contributors: `push_reduction_obligations` discharges `ReductionOperandPermutation` unconditionally from the ordered-fold role rather than from a permission (`crates/tiler-compiler/src/fusion_legality.rs:1262`), and the cooperative tile's admitted `ContributorArrival::AscendingParticipant` fixes the combine order in the program. The permission is represented, resolved, recorded, and cross-checked; it authorizes nothing yet.

**Status note, 2026-08-19 — a second realized fact-1 carrier, scoped to the internal fold.** Added at the [ADR 0112](0112-replace-the-strict-contraction-key-with-a-permission-indexed-successor.md) landing, per the accepted algebraic-authority decision (2026-08-18, recorded in [`decide-the-algebraic-capability-authority-for-contraction-splits`](../../tickets/decide-the-algebraic-capability-authority-for-contraction-splits.md)); it is status alignment, not a supersession, and this record's rule is unchanged. `OperationAlgebraicCapabilities` is no longer the sole realized carrier of this decision's first fact: `tiler::tensor-contraction-f32@1`'s reduction descriptor (fact field 15) declares the order-freedom maxima of the operation's *internal F32 reducer* — reassociation `permission-gated`, permutation `unsupported` — as operation-owned, identity-encoded, registration-validated definition content, joined to the independently resolved numerical ceiling by `ContractionF32ReductionDescriptor::resolve`. The two carriers have genuinely different subjects: the operand-level record speaks for the operation's admitted signatures (its operand chain), while the descriptor maxima speak for the fold the operation owns internally, and forcing them into one carrier is the wrong-operands defect the algebraic decision's original result proved. The paragraph above that cites the contraction as "declares none, and says why" stays true of the operand-level record — the successor keeps `OperationAlgebraicCapabilities::none()`, and its registration comment now names the descriptor as the fold's authority. Unlike the softmax fact string this section contrasts with, the descriptor is a closed typed vocabulary with a sole decoder and a resolver every verifier is required to consume; the two `Unrealized` paragraphs remain accurate for the operand-level record — no commutativity law exists there, and nothing consumes a permutation permission.

## Alternatives considered

A single unordered-reduction flag is easier to propagate but over-authorizes
many schedules. Inferring permutation permission from reassociation conflates
associativity with commutativity. Encoding only the chosen physical tree makes
the distinction visible too late for logical rewrite and candidate legality.
