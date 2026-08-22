---
schema: "tiler-doc/v1"
id: "ADR-0108"
kind: "decision"
title: "Choose how a data-dependent index coordinate enters the index layer"
topics: ["indexing", "semantics", "ir", "gather", "verification"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
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

## Implementation boundary

**Vocabulary correction — 2026-08-19 by [`reconcile-the-three-adr-implementation-statuses-outside-the-metadata-vocabulary`](../../tickets/reconcile-the-three-adr-implementation-statuses-outside-the-metadata-vocabulary.md), on the field's spelling and not on the maturity it claims.** This record carried `implementation_status: "none"`, a value [the document metadata contract](../document-metadata.md) does not define — its four are `not-started`, `spike-only`, `partial`, and `implemented` — and the retired token is quoted here on one line so a grep hit lands inside this note: `implementation_status: "none"`. The replacement coincides with the obvious translation, and it is recorded here because the derivation was not free: one accepted successor packet *has* written bytes into the tree for this decision, and the reading had to establish that those bytes do not advance the field. `decide-the-data-dependent-index-representation-public-surface` (2026-08-18, `done`) reserved a logical-access tag for this record's gather source, and `crates/tiler-ir/src/schedule/model.rs` states its own status exactly: the tag is `reserved-and-unwritten at this base`, recorded in the derivation comment above `const TAG_PARTITIONED_COPY_SOURCE: u8 = 0x0D;`. A reserved-and-unwritten identity assignment is a reserved type, not implemented support, so it leaves this field at its floor rather than lifting it to `partial`.

**Nothing this record decides is realized, and each half was checked rather than assumed.** The logical access vocabulary has not become a closed sum: `crates/tiler-ir/src/index/model.rs "pub enum AccessMode"` still holds `Read` and `Write` alone, and `pub(super) enum IndexNode {` still carries exactly the five forms the Context names. The resolution vocabulary does not exist — `grep -rn 'InvocationValidationRequired\|StaticallyProved\|GatherIndexBounds' crates/` returns nothing — so neither the intrinsic gather-index bounds requirement nor its two-way total resolution is spelled anywhere. No invocation receipt, sealed snapshot, or preflight exists, and the implementation ticket [`admit-the-selected-data-dependent-index-representation`](../../tickets/admit-the-selected-data-dependent-index-representation.md) is still `todo`. The `decide_gather_index` hits under `crates/tiler-ir/src/semantic/gather.rs` and `crates/tiler-reference/src/structural.rs` are [ADR 0107](0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md)'s semantic and reference rule, which this record's Context already cites as pre-existing; they are not this record's decided behaviour and must not be read as partial realization of it. No spike exercises the decided representation either — `spikes/indexing/index-access-model/` tests the *rejection* of a data-dependent coordinate under ADR 0046, which is the fail-closed status quo this record proposes to replace rather than a reproduction of it — so `spike-only` would be wrong for the same reason `partial` is.

**Status advanced to `partial`, and the paragraph above is superseded in tense rather than deleted — 2026-08-22 by [`admit-the-selected-data-dependent-index-representation`](../../tickets/admit-the-selected-data-dependent-index-representation.md).** The reading recorded on 2026-08-19 was correct when written and is now false in three of its four checks, so it is kept for its derivation and corrected here rather than edited in place.

- The resolution vocabulary now exists. That paragraph's own command, `grep -rn 'InvocationValidationRequired\|StaticallyProved\|GatherIndexBounds' crates/`, returned nothing at its base and matches 72 lines across ten files at this one — **lines, not occurrences**, because `-c` counts matching lines and several of these carry the token twice. `GatherIndexBoundsProof`, `GatherIndexValidationRequirement`, `GatherIndexBoundsResolution`, and the verifier-private deriver that mints them are all in `crates/tiler-ir/src/index/`, and the two-way total resolution this record decided is what they spell.
- The logical access vocabulary has become a checked sum. The sentence that checked it remains literally true — `pub enum AccessMode` does still hold `Read` and `Write` alone — but it no longer supports the claim, because the sum landed on `AccessData`/`VerifiedAccessData` as `Direct`/`GatherRead` rather than on `AccessMode`. A reader re-running that check should not conclude the vocabulary is unchanged.
- The implementation ticket is no longer `todo`.
- The schedule relation is no longer reserved-and-unwritten. `crates/tiler-ir/src/schedule/model.rs` now defines `const TAG_GATHER_SOURCE: u8 = 0x0C;` beside `LogicalAccess::GatherSource` and `BoundsProofKind::GatherSource`, so the derivation above `TAG_PARTITIONED_COPY_SOURCE` that described `0x0C` as reserved describes a gap that has since closed.

What has **not** landed, and is why the field is `partial` rather than `implemented`: no invocation receipt, sealed snapshot, or preflight exists; no gather reaches a kernel, artifact, cache, or dispatch route; and the compiler layer that would recognize a gather program is unwritten. The one check in that paragraph that still holds exactly is `IndexNode`'s five forms — and it holds because this record *chose* the tagged access over a tensor-reading node, so it is the decision working rather than an absence.

**One clause of the paragraph above went false on 2026-08-22, and the field stays `partial` — corrected by [`carry-the-gather-relation-through-the-compiler-vertical`](../../tickets/carry-the-gather-relation-through-the-compiler-vertical.md), in tense rather than by deletion.** The clause is `the compiler layer that would recognize a gather program is unwritten`. A gather program is now recognized: `crates/tiler-compiler/src/request/recognize.rs` carries `fn recognize_gather`, `NormalizedOutput::Gather` is one of six recognized shapes, and the compiler request subject projects it under its own `gather-f32.v1` output sub-tag with access-relation tag `0x06`. The whole-program arithmetic walk exempts a gather's address operand by *operand position* rather than admitting U32 generally, so `recognized_arithmetic` still names exactly the two widths it did. Every other clause of that sentence is untouched and remains exactly true, and `IndexNode`'s five forms still hold.

The field stays `partial` because the vertical still stops — two layers later than it did. On the governed target a gather is refused for its exact U32 index before recognition, unchanged. On a U32-capable profile it now advances past arithmetic recognition and stops at `phase: "lowering", rule: "missing-capability"`, because the governed lowering registry carries no gather capability row. Behind that row sits a second named stop, `RegionVocabularyWall::GatherProofUnavailable`: a scheduled `BoundsProofKind::GatherSource` carries a `GatherIndexBoundsProof`, that proof is minted only by the index layer's verifier-private deriver and binds a `CanonicalIndexRegionIdentity`, and no seam carries one from index refinement to a physical region builder. Neither stop grants a schedule, kernel, artifact, cache, or dispatch route.

**Schedule-clause amendment — 2026-08-22, and it narrows one clause without touching the decision.** The Decision above states that the scheduled gather relation carries an "index-input ordinal". The accepted public-surface packet [`decide-the-data-dependent-index-representation-public-surface`](../../tickets/decide-the-data-dependent-index-representation-public-surface.md) amends exactly that clause, and this note records the amendment where a reader of the ADR will find it. A *declared-program* ordinal in shared schedule identity would alias reusable computation with program-interface position, and the compiler's `DeclaredInputOrdinal` is deliberately compiler-private, so `LogicalAccess::GatherSource` carries a region-local `AccessOrdinal` instead. The checked semantic association — which declared input is the source and which is the index — lives in the compiler's retained request subject, stage binding, and whole-program identity, where it is not shared between regions. Preserving the literal older spelling would require superseding a later accepted layer boundary and adding a new public declared-input coordinate to shared IR; that is worse on identity canonicality, public surface, and maintenance, and returns only if Tom reopens that boundary. Nothing else in the Decision moves: the relation still carries source shape, result shape, gathered axis, and index shape, and the index input is still an ordinary U32 read.

**What is unchanged.** The decision, its supersession of ADR 0109 decision 2, its identity rules, its strict exclusions, and its reversal trigger are untouched. This correction states the field's value at this base and claims nothing about when the dependent tickets land.
