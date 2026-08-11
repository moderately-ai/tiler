---
schema: "tiler-doc/v1"
id: "ADR-0108"
kind: "decision"
title: "Choose how a data-dependent index coordinate enters the index layer"
topics: ["indexing", "semantics", "ir", "gather", "verification"]
catalog_group: "physical-planning-lowering"
decision_status: "proposed"
implementation_status: "none"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.transformer-operation-and-shape-surface"]
depends_on: ["ADR-0046", "ADR-0075", "ADR-0107", "ADR-0109"]
ticket: "revise-adr-0108-with-a-complete-data-dependent-index-vertical"
---

# 0108: Choose how a data-dependent index coordinate enters the index layer

**Status:** proposed — returned for revision on 2026-08-08.

**Direction accepted for revision — 2026-08-11.** Tom authorized the revision to design a narrowly invocation-bound validation lane while keeping this ADR proposed. Static proof remains first. The only initial dynamic subject is a host-visible `tiler::u32@1` input validated before routing commit into a sealed receipt over an immutable snapshot; the receipt is scoped to the exact occurrence, extent, type, binding, and invocation and supplies no timeless program proof. Missing, stale, mismatched, or failed validation refuses without clamp, wrap, plan substitution, reference execution, or backend fallback. Mutable zero-copy and device-resident inputs remain unsupported and have named deferred owners. This direction permits the revision to propose the exact narrow supersession of ADR 0109 decision 2 that such a receipt requires; it accepts neither representation candidate, no public API, and no identity-domain change. Direct provenance is Tom's T3 Code instruction that the first pass remain strict and narrow and that every future lane stay represented in the work tree.

**Decision provenance.** In the 2026-08-08 interactive orchestration session,
Tom delegated the coordinator to make the correct decision after independent
source audit. The resulting decision was **not to accept this record as written**
and to return it for the revisions below. This is not acceptance provenance.

## Context

[ADR 0107](0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md)
admits `tiler::gather-f32@1` as a registered, reference-evaluated semantic family
and as nothing below the semantic layer. The current index language therefore
admits no data-dependent coordinate: `IndexNode` still has five forms,
`IndexExprClass` still has three members, and `IndexDomainUnknownReason` still has
three reasons. A gather occurrence reaches no index region, no scheduled
`LogicalAccess`, and no executable plan. That typed, fail-closed boundary remains
the governing state while this record is revised.

[ADR 0046](0046-separate-logical-access-from-storage-addressing.md) requires any
future indirect access to preserve the verifier guarantees of the initial
direct-access language. The returned draft correctly treated that requirement as
load-bearing, but it chose a representation and an admission condition from
premises the implementation does not support.

[ADR 0109](0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md)
additionally governs when retained index-domain obligations may cross into
executable work. Its accepted decision 2 requires every such obligation to be
proved before executable coverage or planning. Decision 4 records that ADR 0109
itself added no run-time check, fallback, or identity widening; it is evidence
that no such authority exists today, not a prohibition a future decision would
need to supersede. A host-side validation of one dispatch may prove that
dispatch's semantic precondition, but it is not timeless program proof and cannot
silently mint the existing executable coverage identity.

## Decision

**No index-layer representation is selected or admitted by this revision.** The
proposal is returned for a complete vertical comparison of two candidates:

1. a first-class, verified nested read/value expression that names its source
   tensor and source coordinates; and
2. an append-only tagged access representation whose new tag and framed payload
   denote the data-dependent read while preserving every previously encodable
   access byte-for-byte.

The comparison belongs to
[`revise-adr-0108-with-a-complete-data-dependent-index-vertical`](../../tickets/revise-adr-0108-with-a-complete-data-dependent-index-vertical.md).
It must carry each candidate through bounds and host validation, reachability,
typing, proof subjects, compaction, canonical identity, public authoring and
inspection, reference semantics, compiler explanation, `LogicalAccess`, and the
work graph. It may select a representation, defer with a non-circular trigger,
or show that neither candidate is yet supportable. It may not implement either.
Any candidate relying on host or per-dispatch validation must either prove how all
retained index-domain obligations are discharged before executable coverage under
ADR 0109, or return to Tom with an explicit proposal to supersede ADR 0109
decision 2. Decision 4 confirms that no present ADR 0109 authority supplies the
required run-time or identity contract; a future decision would have to add that
authority, not “supersede” the historical scope statement. The comparison may
not reinterpret a run-time observation as a timeless proof to avoid that
decision.

Until that comparison is decided, the exact five-node, three-class,
three-unknown-reason census is a useful negative boundary rather than evidence
for either candidate. The existing type-sized checks remain at 5/3/3 and make an
unreviewed widening loud. They do not reserve a fourth reason or promise what any
future widening looks like.

## Why the previous proposal was returned

### A data-dependent bound is not undecidable in principle

**Fact.** ADR 0107 states that the gather bound is proved statically or validated
at a named boundary. `decide_gather_index` is deliberately factored out of the
reference evaluator so a future host-side pre-dispatch validator can use the same
rule and diagnostic. The current shape-only index verifier cannot decide an
arbitrary tensor element, but that is a statement about the information at that
boundary, not an impossibility theorem.

**Fact.** The three current `IndexDomainUnknownReason` variants do not all mean
"a later environment will close this". `InsufficientFacts` says the admitted
facts permit models on both sides, `UnsupportedFragment` names the current proof
engine, and `ResourceLimit` records where a deterministic proof lane stopped.
They support typed fail-closed handling; they make no closure promise.

**Consequence.** A fourth reason naming "undecidable in principle" is not an
established prerequisite. The revised comparison must instead say which boundary
can establish a data-dependent bound, how that validation is represented, and
whether the existing residual vocabulary is the right carrier at all.

### The access route was dismissed on false identity and rank premises

**Fact.** `encode_region` already begins every access with an explicit tag for
`AccessMode`: existing reads write `1` and writes write `2`. A candidate using a
fresh tag `3` plus a framed payload can leave every old read and write byte
unchanged. Whether that is the right semantic representation remains undecided,
but an access-level representation does not inherently force an index-region
identity-domain step.

**Fact.** `IndexRegionBuilder::prepare_access` verifies that the coordinate count
equals the tensor rank before constructing `AccessData`. The seven later
coordinate/extent `zip` sites therefore consume an established same-length
invariant for the current representation; they do not silently truncate an
arbitrary direct access. A future representation must establish its own rank
invariant, but the `zip` census does not choose where indirection belongs.

### The proposed expression form was not a complete IR object

**Fact.** `IndexNode` is `pub(super)`, not public. The proposed list of four public
widenings therefore counted a private implementation enum while omitting the
public builder constructor and its errors. `IndexExprView` is the public
inspection surface; `IndexExprClass` and any obligation or evidence surface are
separate choices rather than automatic consequences.

**Fact.** A node that reads tensor data is a nested logical read, not ordinary
integer arithmetic. The draft did not define the nested source access's coordinate
bounds, tensor reachability, resolved value type, `u32` versus addressing-width
semantics, proof subject, compaction traversal, alpha-equivalence, canonical
identity, authoring and inspection surface, reference evaluation, or compiler
explanation. Existing construction and consumption sites are exhaustive over the
five current forms, so those omissions are correctness and identity questions,
not mechanical follow-up.

### The trigger and dependency pointed at one another

The returned draft waited for a physical route and named
`emit-the-indirect-gather-on-metal` as its trigger, while emission was blocked on
this decision and on an admitted IR representation. The revision/design decision
must precede any IR-admission implementation; the admitted IR and integer storage
carrier must in turn precede emission. A dependent implementation ticket cannot
be the event that authorizes its own prerequisite.

## Required revision

The replacement decision must answer, for both candidates and from exact current
source:

- where the nested source tensor, its coordinate tuple, and the outer gathered
  coordinate live;
- which bounds are static, which may be host-validated, and which object retains
  each result without treating a run-time observation as timeless program proof;
- how every retained index-domain obligation is proved before executable coverage
  as ADR 0109 decision 2 requires, or the exact supersession of decision 2 that
  must return to Tom before a run-time-validation route can be selected; ADR 0109
  decision 4 is the record that no present run-time or identity authority exists,
  not a second prohibition to supersede;
- the logical `tiler::u32@1` index contract, any conversion to physical address
  width, and the refusal of signed or lossy interpretations;
- source-tensor and expression reachability, rank equality, nested-access bounds,
  predicate ownership, and the subject named by proof or disproof;
- compaction, remapping, alpha-equivalence, canonical encoding, identity-domain
  consequences, and whether old bytes really remain unchanged;
- complete public authoring, view, error, validation, and explanation surfaces
  under [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md);
- the reference oracle and the compiler's fail-closed recognition, discharge,
  refusal, and explanation path;
- the relation to scheduled `LogicalAccess` without collapsing logical access
  meaning into storage addressing; and
- a dependency order in which a selected design is accepted, its IR form is
  separately admitted and verified, and only then can backend emission become
  ready.

## Consequences

- ADR 0108 remains `proposed`; no ADR 0108 public boundary is accepted or labelled
  as a draft by this outcome.
- ADR 0107 and ADR 0046 remain accepted and unchanged in authority. The current
  no-admission boundary and typed request refusal remain in force.
- ADR 0109 decision 2 remains accepted and governs every candidate's executable
  boundary. Decision 4 confirms that ADR 0109 supplied no run-time or identity
  authority, and this revision supplies none either.
- Q-SHAPE-007 remains open on both the index-layer design and the unfired scatter
  half.
- The access-record and nested-expression candidates both remain open until the
  complete vertical comparison is reviewed.
- `emit-the-indirect-gather-on-metal` stays structurally blocked on eventual ADR
  acceptance, a separately admitted IR representation, and the integer storage
  carrier. No form is implemented here.

## Alternatives considered

**Accept the previous expression-form decision and defer implementation.**
Rejected because its identity, rank, residual-reason, public-surface, and trigger
arguments are not supported by the current source, and because the proposed node
does not yet constitute a complete verified logical read.

**Select the tagged access representation in this correction.** Also rejected.
The audit reopens that candidate by disproving two reasons used to close it; it
does not establish the candidate's bounds, proof-subject, authoring, schedule, or
compiler contract. Selecting it without the vertical comparison would repeat the
same error in the other direction.

**Admit either form now.** Rejected. Research and a corrected proposed record do
not authorize implementation, and no reviewed representation yet satisfies ADR
0046 end to end.
