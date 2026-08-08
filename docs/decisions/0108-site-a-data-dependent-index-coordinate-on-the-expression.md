---
schema: "tiler-doc/v1"
id: "ADR-0108"
kind: "decision"
title: "Site a data-dependent index coordinate on the expression, and do not admit one yet"
topics: ["indexing", "semantics", "ir", "gather", "verification"]
catalog_group: "physical-planning-lowering"
decision_status: "proposed"
implementation_status: "none"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.transformer-operation-and-shape-surface"]
depends_on: ["ADR-0046", "ADR-0075", "ADR-0107"]
ticket: "admit-the-indirect-access-class-into-the-index-layer"
---

# 0108: Site a data-dependent index coordinate on the expression, and do not admit one yet

**Status:** proposed

## Context

[ADR 0107](0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md) admitted `tiler::gather-f32@1` as a semantic operation family **and as nothing below it**, and named the index-layer question as a separate decision it deliberately did not answer. This record answers that question. It extends ADR 0107 rather than revising it: everything ADR 0107 decided about the family — its key, its operands, its derived result shape, its bounds precondition, its refusal to clamp or wrap, its admission of duplicate indices, its refusal of a signed index identity — stands unchanged and unrestated here.

[ADR 0046](0046-separate-logical-access-from-storage-addressing.md) attaches one condition to the whole class: "Indirect operations remain addable without weakening the verifier for the initial direct-access language." ADR 0107 satisfied that condition the only way available to a record that admits no expression form — by admitting none. Any record that admits one owes the condition an argument. Supplying that argument, and finding where it fails, is this record's work.

### What the verifier actually guarantees, read rather than assumed

The guarantee is **not** "every access's bounds are proved". A region whose bounds nothing proved still builds. `verify_accesses` in `crates/tiler-ir/src/index/builder/proof.rs` leaves an undischarged read obligation as `PendingIndexDomainDisposition::Unknown`, and the region carries it out through `unknown_index_domain_predicates`, documented as a record "a consumer must discharge every record before program work".

The guarantee is the **disjunction**: every bounds obligation over every reachable access is either discharged with retained sound evidence naming the argument and the premises it read, or retained as an explicit unknown carrying a typed reason. Three mechanisms discharge it and a fourth refutes:

- the per-axis cheap predicates in `cheap_index_domain_predicates`, which close a bound by `VacuousEmptyDomain`, by `Interval`, or by the structural `ProvedExtentEquality`;
- `interval_verdict`, whose `interval_proved` half feeds `bounds_proved_without_enumeration`;
- the budgeted finite walk gated by `coordinates_are_evaluable` and executed by `verify_access_exhaustively`; and
- `interval_verdict`'s `definitely_outside` half, which **refutes** an access and fails the build with `CoordinateOutOfBounds`.

Write ownership is a separate obligation with its own three mechanisms — `write_is_permutation`, the rectangle placement in `decide_partition_by_interval`, and `verify_partition_exhaustively` — and, unlike bounds, it has no unknown disposition: a write whose ownership nothing proved is refused.

### Two routes, and the reason the obvious one is wrong

ADR 0107, this contract's `docs/ir.md` paragraph, and the owning ticket each say, in wording that wraps differently in each, that the obstacle is the access record's shape rather than a missing expression form. That is a correct diagnosis of why the family is inexpressible today: `AccessData` carries one `tensor: u32` and has nowhere to name a second. It does **not** follow that the remedy belongs on the access record, and reading the three mechanisms that would have to carry it shows the remedy belongs on the expression.

## Decision

**1. A data-dependent coordinate, if admitted, is an index-expression form.** It enters as an `IndexNode` variant naming a tensor and its own coordinate expressions, and as a fourth `IndexExprClass` member. It is **not** a second tensor ordinal, an optional indirection record, or a per-axis source discriminator on `AccessData`. Three independent reasons, each read from the implementation:

*The identity encoding is tag-dispatched, so an expression form is additive and an access field is not.* `encode_index_node` and `structural_index_key` both write an explicit leading tag — `1` for a constant through `5` for a modulo — so a sixth form tagged `6` changes the canonical bytes of no region that does not contain one. `encode_region`'s access block writes `mode | tensor | domain | coordinates` with no optional slot, and `encoded_region_len` charges a fixed five bytes of header per access to match, so any presence discriminator moves the bytes of **every region ever encoded**, forcing `tiler.index-region.v11` to `v12` and a recomputation of every dependent pin — for a capability nothing yet consumes. The expression's `class` is not encoded at all, so widening `IndexExprClass` moves no identity either.

*The obligation vocabulary is expression-keyed.* `IndexDomainPredicate` names a `VerifiedIndexExprId` in both of its variants, and `validate_index_domain_predicate` refuses a predicate whose expression is not one of the subject access's coordinates. A data-dependent coordinate that **is** an expression states its bound as `LessThanExtent { expression, extent: IndexExtentRef::TensorAxis { .. } }` with no new predicate kind and no widening of the retained-evidence surface. A data-dependent *axis* that is not an expression has no handle to name, so the access-record route requires a second predicate subject — a strictly larger change to the exact public surface consumers already read.

*The per-axis correspondence is a load-bearing invariant spelled as `zip`.* `access.coordinates.iter().zip(shape.extents())` and its siblings appear in seven functions of `proof.rs` — `cheap_index_domain_predicates`, `interval_verdict`, `coordinates_are_bounded_dimensions`, `verify_access_exhaustively`, `write_is_permutation`, `write_partition_box`, and `verify_partition_exhaustively`. Every one encodes "each axis has exactly one coordinate expression". The access-record route falsifies that invariant at all seven sites simultaneously, and `zip` truncates rather than failing, so the falsification is silent. The expression route preserves it: an indirect axis still has exactly one coordinate expression, and that expression declines in every mechanism.

**2. ADR 0046's condition is satisfied for soundness and violated for one guarantee, and the violated one is nameable.**

*Soundness is preserved, per mechanism.* An expression that reads tensor data has no propagated interval, so both branches of `interval_verdict` — which each require `Some((min, max))` — decline: it can neither prove `interval_proved` nor set `definitely_outside`, so it can neither discharge a bound nor refute one. `coordinates_are_bounded_dimensions` and `write_is_permutation` each require the coordinate to be `IndexNode::Dimension`, so both decline. `coordinates_are_evaluable` declines, which withholds the finite walk before any budget is charged — the same gate that already withholds it for an undetermined divisor or coefficient. `coordinate_offset_dimension` declines, so no partition rectangle is placed. Every one of these is decided per coordinate, so **no direct-access coordinate's answer changes**. The verifier does not become unsound; it becomes unable to discharge, which is the state it is already designed for.

*One guarantee is weakened, and inability to discharge is what weakens it.* The residual obligation lands in `IndexDomainUnknownReason::InsufficientFacts`, which is the only reason `cheap_index_domain_predicates` can produce. All three existing reasons — `InsufficientFacts`, `UnsupportedFragment`, and `ResourceLimit` — mean *in principle dischargeable by supplying more*: more facts, a stronger engine, a larger budget. A consumer reading an unknown record today may correctly conclude that binding the region's symbols and re-verifying could close it. For a data-dependent coordinate that conclusion is false in every environment, because the deciding value is tensor data the region does not own and no shape environment can supply. Admitting the class without a distinguishing reason makes `UnknownIndexDomainPredicate` mean two incompatible things under one type, and makes the contract sentence requiring a consumer to discharge every record unsatisfiable by static means for one of them.

**The narrowest repair, and it is required rather than optional.** A fourth `IndexDomainUnknownReason` naming undecidability in principle — an obligation no admitted fact, engine, or budget can close, because its subject is tensor data. It is additive to a `pub` enum, it is a build error at every exhaustive consumer, and with it the direct-access population's meaning is strictly unchanged and strictly identified. **Admission of the expression form and admission of this reason are one change, not two**: the form without the reason is exactly the weakening ADR 0046 forbids.

**3. The class is not admitted now.** The shape above is decided; taking it is not. Nothing consumes an index region containing an indirect coordinate: no realization law is registered for the family, no lowering capability resolves it, `FusionNumericalCapabilities::classify` returns `None` for it, and the physical access vocabulary `LogicalAccess` has no relation that realizes it. Admitting the form today would replace an early, typed, named refusal at the request boundary with a **buildable region carrying an obligation nothing can discharge** — which is strictly worse, and worse in the direction this corpus refuses. ADR 0107's counterpoint was that a registered-unplannable family traps a *reader*; a verifiable-undischargeable region traps a *consumer*, because a region is the artifact that carries proof.

**Reconsideration trigger.** A physical route exists that could consume such a region: a target profile declares an access construct realizing a data-dependent read, and a named `LogicalAccess` relation is proposed to denote it. `emit-the-indirect-gather-on-metal` is the ticket that would fire it. Until then the trigger is not fired and the refusal stands.

**Public boundary.** Under [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) the widenings this record shapes — an `IndexNode` variant, an `IndexExprView` variant, an `IndexExprClass` member, and an `IndexDomainUnknownReason` member — are named here as a **decided shape and an undrafted surface**. None is written, so none is yet a labelled draft; accepting this record accepts the shape and the deferral, not a surface.

## Consequences

- The index-expression vocabulary stays at five `IndexNode` forms and three `IndexExprClass` members, and that is now enforced rather than asserted: `crates/tiler-ir/src/index/builder/tests.rs` pins both counts from the types, so widening either is a build error that names this record.
- ADR 0107 stays `accepted` and unchanged. Its statement that an occurrence "reaches no index region" remains true, and now carries a reason with a stated condition rather than an open question.
- ADR 0046 stays `accepted` with its rejection of tensor-data-derived indices intact. Its non-weakening condition is answered rather than assumed: satisfiable, at the cost of one additional unknown reason, and unsatisfiable without it.
- Q-SHAPE-007's second open half — whether the index layer admits a data-dependent access class — is answered in shape and deferred in timing. The scatter half stays reserved and unfired.
- `docs/ir.md`'s admitted-vocabulary paragraph names four classes where three are implemented. The fourth is now identified as `IndexExprClass::DataDependent` with a stated admission condition, so the gap between the contract paragraph and the enum is documented rather than latent.
- The access-record route is closed, not merely unchosen. A later worker reaching for it must supersede this record with the identity, predicate-subject, and per-axis-invariant costs answered.

## Alternatives considered

**Admit the expression form now, with the fourth unknown reason.** It is the smallest sound admission and it was rejected on delivered value rather than on correctness: it produces regions that build and that nothing can plan, price, fuse, or emit, and it widens four public types ahead of the consumer that would shape them. The corpus's own precedent cuts the other way here — ADR 0107 accepted a registered-unplannable *family* because a family is a statement of meaning, whereas a region is a carrier of proof, and a proof carrier that cannot discharge its own obligation is not the same kind of legitimate delivered state.

**Site the indirection on `AccessData` as a second tensor ordinal.** Rejected on the three reasons in decision 1. It is the reading the ticket's own framing invites, and each of its three costs runs opposite to the intuition that framing creates: it breaks region identity where the expression route is additive, it needs a new predicate subject where the expression route needs none, and it silently falsifies a `zip`-spelled invariant at six sites where the expression route preserves it.

**Reuse `IndexDomainUnknownReason::UnsupportedFragment` for the data-dependent case.** It reads plausibly — the proof engine does not decide the fragment — and it is wrong in the direction that matters: its documented meaning is that *the current* engine does not decide an *admitted* fragment, which is a statement about this build. A data-dependent bound is undecidable by any engine over any facts, and collapsing the two would tell a consumer that a stronger verifier might close it.

**Discharge the runtime validation through `IndexDomainEvidence::Empirical`.** That variant is reserved and unfired, documented as having "no empirical proof lane", and a pre-dispatch data validation is the closest thing in the vocabulary to a non-proof discharge. It is not decided here, deliberately: whether a check performed against one dispatch's data may enter a region's *retained* evidence — and therefore its canonical identity, which is a claim about the program rather than about a run — is a question about what a region asserts, and it belongs with whoever admits the form.

**Leave the question open.** Rejected because "open" was doing work an answer does better. The refusal at the request boundary is unchanged either way; what changes is that a worker reaching for the access-record route now finds the costs already counted.
