---
schema: "tiler-doc/v1"
id: "ADR-0108"
kind: "decision"
title: "Choose how a data-dependent index coordinate enters the index layer"
topics: ["indexing", "semantics", "ir", "gather", "verification"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "none"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.transformer-operation-and-shape-surface"]
depends_on: ["ADR-0046", "ADR-0075", "ADR-0107", "ADR-0109"]
ticket: "revise-adr-0108-with-a-complete-data-dependent-index-vertical"
---

# 0108: Choose how a data-dependent index coordinate enters the index layer

**Status:** accepted on 2026-08-12.

**Acceptance provenance.** Tom accepted the revised tagged-access decision in the T3 Code orchestration conversation on 2026-08-12 with “okay agreeed, next decision”. This acceptance includes the gather-specific logical representation, conditional-coverage boundary, invocation-scoped validation authority, strict first-pass exclusions, identity rules, and reversal trigger below. It accepts no public Rust spelling and authorizes no implementation beyond the dependent tickets.

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

Represent the first data-dependent coordinate as an **append-only tagged gather access**, not as a tensor-reading `IndexNode`.

The logical access vocabulary becomes a closed sum. Existing direct reads and writes retain their current tensor, mode, domain, rank-equal coordinate list, proof rules, and canonical bytes. The new gather read names one F32 source tensor, one U32 index tensor, the gathered source axis, one shared output iteration domain, one direct coordinate per index-tensor axis, and one direct coordinate per non-gathered source axis. The gathered source coordinate is exactly the U32 value read from the index tensor at the stated index coordinates.

Construction proves the source rank is nonzero, the axis is in range, both tensors are program inputs in the first pass, both direct-coordinate runs have their required ranks and stay inside the shared domain, and source/index/result shapes satisfy the semantic gather relation. A U32 index widens losslessly into the verifier's unsigned coordinate space before ordinary checked address arithmetic. Signed reinterpretation, truncation to a target address width, clamp, wrap, inferred axes, recursive indirection, scatter, and data-dependent result shapes have no spelling.

Public authoring and inspection must expose this distinction as a truthful checked sum. Existing `TensorAccessRef::tensor` and `coordinates` cannot continue pretending every access names one tensor and one coordinate run. Exact Rust names remain an ADR 0075 decision for the implementation ticket.

Each gather access carries one intrinsic gather-index bounds requirement over the exact access, index tensor and type, source axis and extent, and semantic occurrence. Its resolution is total: a named timeless proof may establish `StaticallyProved`, or the host-visible first pass carries `InvocationValidationRequired`. The latter is a known mandatory execution precondition, not an `IndexDomainUnknownReason` and not a proof-engine miss.

Compilation may package conditional coverage only when the complete invocation requirement enters program and artifact identity. That does not grant dispatch authority. Runtime validates every value in the exact host-visible U32 binding against the exact gathered extent using the same semantic rule as `decide_gather_index`, copies the values into receipt-owned immutable storage, and binds the sealed receipt to the occurrence, type, extent, binding, snapshot, and invocation attempt. Only consumption of that exact receipt may mint the non-`Clone` preflight that reaches the infallible routing commit. Missing, stale, crossed, reused, unanswerable, or failed evidence refuses before commit.

This narrowly supersedes ADR 0109 decision 2: a named conditional requirement may reach packaged coverage, but one invocation becomes executable only after timeless proof or its exact invocation receipt. Arbitrary `Unknown` remains non-executable. The receipt and observed values are invocation evidence, never timeless proof or reusable artifact/cache identity.

At schedule level, the source read gains an append-only gather-source `LogicalAccess` relation carrying source shape, result shape, gathered axis, index-input ordinal, and index shape. The index input remains an ordinary U32 read. Both are derived from one checked realization law and cross-validated; a caller cannot author two independent, contradictory accounts. Physical lowering may map this relation to storage addressing but may not change its index interpretation or bounds rule.

Use fresh, framed tags for the index access and scheduled relation. Preserve access tags `0x01` and `0x02` and every old direct payload byte. Existing direct identities therefore remain byte-identical; new gather subjects receive new identities naturally. The conditional requirement is identity-bearing. Snapshot bytes, validation results, and receipt generation are ephemeral and excluded. The artifact carrier must include a compatibility fence that makes older readers refuse before dispatch; if a fresh tagged row plus required feature cannot establish that, the owning implementation takes the required major schema/domain step.

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

## Consequences

- The separate index-representation and invocation-receipt tickets own implementation. This ADR selects meaning and authority but adds no Rust surface or executable route.
- Static proof remains the zero-runtime-cost preferred lane. The host-visible immutable-copy lane is the only initial dynamic lane.
- Mutable zero-copy, device-resident or device-produced indices, inline-kernel validation, caller assertions, and generalization beyond gather remain explicitly deferred or unsupported.
- The expression vocabulary stays pure. A second accepted consumer requiring nested tensor-derived coordinates is the trigger to reconsider it.
- ADR 0107's semantic family and ADR 0046's logical-access/storage-addressing separation remain intact. ADR 0109 continues to refuse every unnamed or unresolved `Unknown` before execution.

## Alternatives considered

**Accept the previous expression-form decision and defer implementation.**
Rejected because its identity, rank, residual-reason, public-surface, and trigger
arguments are not supported by the current source, and because the proposed node
does not yet constitute a complete verified logical read.

**Use a broader generic nonrecursive indirect access.** Rejected for the first slice. It can be made sound, but freezes generalized source, value, and address semantics before a second consumer exists and adds invalid states without runtime benefit.

**Add a tensor-reading `IndexNode`.** Rejected. Its composability is real, but it merges typed logical effects into a pure coordinate algebra and recursively widens reachability, compaction, alpha identity, proof ownership, and reference evaluation. Reconsider only when a concrete second consumer cannot be represented without duplicating access meaning.

**Admit either form directly from this ADR.** Rejected. Acceptance selects the architecture; the separately scoped implementation and public-boundary tickets still own admission, exact Rust spelling, tests, and identity migration evidence.
