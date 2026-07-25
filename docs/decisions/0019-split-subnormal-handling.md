---
schema: "tiler-doc/v1"
id: "ADR-0019"
kind: "decision"
title: "Separate subnormal input and result handling"
topics: ["numerics","floating-point","subnormals"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.operation-conformance-matrix"]
ticket: "numerical-policy-contract"
---

# 0019: Separate subnormal input and result handling

**Status:** accepted. One sentence of the Decision was amended on 2026-07-25; the "Amendments" section records what it said, what it says now, and the evidence that moved it. Nothing else in this record changes, and the decision to separate the two subnormal dimensions is untouched.

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [operation conformance matrix](../research/numerics/operation-conformance-matrix.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Treating a subnormal operand as zero before an operation and flushing a newly
produced subnormal result to zero are observably different behaviors. Some
targets couple them in one execution mode, while others expose or require
different combinations. A single `flush_subnormals` boolean cannot state which
behavior occurred.

## Decision

Every applicable floating-point operation resolves subnormal input handling
and subnormal result handling independently. Each dimension initially supports
preservation or an explicit flush-to-zero behavior; a flush-to-zero behavior
states which zero it produces, as part of that behavior.

*Amended 2026-07-25 by [`reconcile-adr-0019-zero-sign-placement-with-the-landed-flush`](../../tickets/reconcile-adr-0019-zero-sign-placement-with-the-landed-flush.md). The final clause previously read "zero-sign behavior is resolved with the signed-zero contract". "Amendments" below records the evidence, what a reader of the earlier text should un-learn, and the obligation this places on whoever admits a signed-zero dimension.*

Portable-bitwise execution preserves both input and result subnormals. Relaxed
operation contracts may permit either or both kinds of flushing. A backend that
cannot realize a requested combination natively must emulate it, consume an
already authorized relaxation, or reject the plan.

Backend switches that couple input and result flushing do not couple Tiler's
semantic permissions.

## Consequences

- Reference evaluation can distinguish input flushing from result flushing.
- Backend feasibility accurately represents partially supported combinations.
- Portable-bitwise results retain gradual underflow.
- Relaxed modes can match useful hardware behavior without silently changing
  both sides of an operation.
- Subnormal and signed-zero policy both participate in artifact identity and
  adversarial tests.

## Alternatives considered

One flush-to-zero flag is compact but loses observable information. Treating
all subnormal behavior as backend-defined makes fusion and fallback disagree.
Requiring preservation in every conformance mode unnecessarily excludes
explicitly requested fast execution.

## Implementation boundary

Added 2026-07-25 by [`re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening`](../../tickets/re-audit-adr-0011-and-0019-status-after-the-vocabulary-widening.md), which moved `implementation_status` from `not-started` to `partial`. This section states which clauses that value rests on, read at `43f685f`, and adds no decision.

**Realized — the two dimensions exist, are independent, and can differ.** `tiler_ir::schedule::NumericalRealization` carries `input_subnormals` and `result_subnormals` as separate `SubnormalMode` fields, and `SubnormalMode` is `Preserve | FlushToZero { zero_sign }`, so the four combinations the conformance matrix requires as adversarial coverage are all expressible and two realizations can differ on one dimension alone. Both dimensions are encoded independently into canonical scheduled-region and kernel identity through exhaustive matches.

**Realized — the reject branch of the backend obligation, per declared dimension.** `crates/tiler-metal/src/emit.rs` matches each subnormal dimension exhaustively against the target's declared behaviour, and `crates/tiler-metal/src/record.rs` carries three typed gap variants — `SubnormalFlushInArithmetic`, `SubnormalPreservationInArithmetic`, and `UndeclaredFlushedZeroSign`. `MetalTranslationUnit::require_declared_realization` fails closed with `MetalEmitError::UnrealizableNumericalObligation`.

**Realized — coupling in the target does not couple the semantic permissions.** The measured Apple row flushes both dimensions in one hardware behaviour, and the contract still carries two fields that the emitter compares separately against the target fact.

**Partially realized — "relaxed operation contracts may permit either or both kinds of flushing".** Both is registrable: `StrictF32NumericalContract::governed_flush_to_zero` flushes input and result to the sign-preserving zero. *Either* — one dimension flushing while the other preserves — is expressible in the type and is not registrable, because `governed_profile` admits exactly the preserve/preserve and flush/flush contracts.

**Unrealized — the emulate branch of the backend obligation.** A backend that cannot realize a requested combination natively must "emulate it, consume an already authorized relaxation, or reject the plan". Only the reject branch exists. `tiler-metal` emits no compensating operations for a dimension the target does not honour, which is deliberate honesty rather than an omission — emission there is pure source lowering — but it means one of the three stated outs is unbuilt.

**Unrealized — the reference-evaluation consequence.** "Reference evaluation can distinguish input flushing from result flushing" is not realized. The exact check is `grep -rn 'SubnormalMode\|NumericalPermission\|NumericalRealization' crates/tiler-reference/src/`, which returns nothing: the reference evaluator names no part of the numerical realization vocabulary, so it cannot distinguish the two dimensions or evaluate either.

**Unrealized — the gap record's granularity.** `record_subnormal_obligation` compares the input and the result dimension against the same target fact and inserts into one set, so a gap on either dimension yields the same variant and the record cannot say which dimension failed. The semantic dimensions stay separate; the *reporting* of a failure over them does not.

## Amendments

An amendment changes a stated clause of this accepted record without reopening the decision it records. Each entry names what the record said before, what it says now, and the evidence that moved it, so a reader who applied the earlier text can see precisely what to revise.

### 2026-07-25 — a flush-to-zero behavior states its own zero (`widen-numerical-vocabulary-and-complete-identity`)

**What the record said.** The Decision's second sentence closed with "zero-sign behavior is resolved with the signed-zero contract". Read either way "with" can be read — resolved *from* that contract, or resolved jointly *alongside* it — the sentence sends a reader to a separate signed-zero contract to find out which zero a flush produces.

**What it says now.** A flush-to-zero behavior states which zero it produces, as part of that behavior. Nothing else in the sentence changes: both dimensions still resolve independently and each still supports preservation or an explicit flush.

**Fact — the question was deliberately reopened before it was answered.** [ADR 0076](0076-declare-target-honourable-numerical-realizations.md) `refines` this record, and its item 1 states: "Whether the sign is carried as a field of the flush behaviour or resolved from the contract's signed-zero dimension is an implementation choice for the IR ticket; leaving it unstated is not." That framing is itself evidence that this record's sentence was not read as settling the placement — a record cannot reopen a question its own refined predecessor had closed. ADR 0076's forcing measurement is why the question exists at all: on the measured Apple row `0x80400000 * 2.0f` returns `0x80000000`, so a flush mode that does not state its zero cannot be checked against that hardware.

**Fact — which way the implementation went, and its reason.** `1f78223` on 2026-07-24 landed `tiler_ir::schedule::SubnormalMode` as `Preserve | FlushToZero { zero_sign }` over `FlushedZeroSign::{PreservesSign, AlwaysPositive}` in `crates/tiler-ir/src/schedule/numerics.rs`. The reason recorded on the type is that the resolution route is unsound rather than merely inconvenient: a permission may leave a zero's sign *unspecified*, and an unspecified flush result is exactly the under-specification ADR 0076 item 1 forbids, so every `SubnormalMode` value must answer "which zero" on its own.

**Fact — the earlier clause names nothing that exists.** `NumericalRealization` in the same module carries `profile_key`, `canonical_arithmetic_nan_bits`, `input_subnormals`, `result_subnormals`, `contraction`, and `reassociation`. There is no signed-zero dimension in the resolved contract. The exact check is `grep -rn 'SignedZero\|signed_zero' crates/`, which returns one Metal driver accessor (`MslOptimization::preserves_signed_zero`, a property of a compiler math mode) and prose in doc comments and tests; nothing in the numerical contract vocabulary. So the amended clause is not one of two available placements — it is the only one the contract can currently express, and the earlier clause deferred to a dimension that has never been admitted.

**Why this is an amendment and not a supersession, and not a "compatible readings" note.** The decision this record makes is that subnormal input and result handling resolve independently, and that is untouched — no clause is reversed, `applies_to` and evidence are unchanged, and ADR 0076 continues to refine this record rather than replace it. It is also not a case where both statements stand: the charitable "resolved jointly alongside the signed-zero contract" reading is unsatisfiable while no such contract exists, so recording the two as compatible would leave a reader of this record alone still looking for a dimension to consult. The sentence is corrected in place, and the earlier text is quoted above rather than deleted.

**Obligation on whoever admits a signed-zero dimension.** When the resolved contract gains one, it constrains a flush's stated `zero_sign` rather than supplying it: the two must agree, and disagreement is a rejection rather than a precedence rule. **Nothing checks this today** — there is no second field to disagree with — so this is a stated obligation on a future change and not an implemented invariant, and the change that admits the dimension owes the check as well as the field.

**What a reader of `docs/numerical-semantics.md` should note.** That contract's sentence "the zero sign follows the resolved signed-zero and subnormal contract rather than an ambient target mode" was never falsified — it names the subnormal contract as a source, and its operative point is the negative one about ambient target modes. It is sharpened in the same change to say which of the two states a flushed zero's sign, and its `SubnormalContract` sketch, which spelled `FlushToZero` with no sign, is corrected for the same reason: a descriptive sketch that omits the sign illustrates precisely the under-specification ADR 0076 item 1 forbids.
