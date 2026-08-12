---
id: revise-adr-0108-with-a-complete-data-dependent-index-vertical
title: Revise ADR 0108 with a complete data-dependent index vertical
status: awaiting-decision
priority: p1
dependencies: [accept-adr-0109-fail-closed-on-unknown-index-domain-proof]
related: [accept-adr-0108-data-dependent-index-coordinate-siting, admit-the-indirect-access-class-into-the-index-layer, emit-the-indirect-gather-on-metal, admit-a-storage-carrier-for-integer-program-inputs]
scopes: [contracts/decisions, contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, indexing, gather, verification, design, needs-tom]
---
## User-visible outcome

ADR 0108 is revised from a complete, source-audited vertical comparison of a
first-class verified nested read/value expression and an append-only tagged access
representation. The result either selects one coherent logical representation or
defers with a non-circular trigger; it does not implement either form.

## Facts at the creating base

- ADR 0107 admits `tiler::gather-f32@1` only at the semantic layer. The index
  language has five `IndexNode` forms, three `IndexExprClass` members, and three
  `IndexDomainUnknownReason` variants, and no form reads tensor data.
- `decide_gather_index` defines the exact data-dependent bounds check and is
  factored for reuse by a future host-side pre-dispatch validator. The current
  reference evaluator is the only named enforcement boundary.
- `encode_region` already tags existing read and write accesses separately. A
  fresh access tag can be append-only and preserve old bytes, subject to a
  complete representation and injectivity proof.
- `IndexRegionBuilder::prepare_access` enforces coordinate-count/rank equality
  before the verifier's coordinate/extent `zip` consumers run.
- `IndexNode` is private. Public construction and inspection run through
  `IndexRegionBuilder`, builder errors, and `IndexExprView`.
- No `LogicalAccess`, realization law, compiler recognition route, or backend
  construct realizes an indirect gather today.
- ADR 0109 decision 2 requires every retained index-domain obligation to be
  proved before executable coverage. Decision 4 records that ADR 0109 added no
  run-time validation, fallback, or identity widening; it documents the absence
  of current authority rather than a prohibition a future ADR must supersede. A
  host/per-dispatch result is not timeless program proof.

## Required comparison

For each candidate, derive and compare the complete contract for:

- the outer gathered coordinate, the nested source tensor, and every source
  coordinate expression;
- static proof versus named host validation, including what evidence belongs to
  timeless program identity and what belongs only to one dispatch;
- conformance to ADR 0109's refusal-before-coverage boundary: each candidate must
  prove every retained obligation before executable coverage, or explicitly
  return decision 2 to Tom for supersession before selecting a route that depends
  on run-time validation; decision 4 instead establishes that a new decision must
  supply the run-time and identity authority absent today;
- the semantic `tiler::u32@1` index value, conversion to any target address
  width, overflow behavior, and refusal of signed or lossy interpretations;
- bounds for both the outer access and nested source read, rank equality,
  reachability, aliasing, and the exact subject of every predicate, proof,
  disproof, or residual obligation;
- expression and tensor reachability, compaction, remapping, alpha-equivalence,
  canonical ordering, canonical encoding, and identity-domain consequences;
- public authoring, view, typed errors, validation, and the exact labelled-draft
  boundary under ADR 0075;
- reference construction and evaluation, including reuse of
  `decide_gather_index` and agreement diagnostics;
- compiler recognition, IR-owned discharge or host validation, typed refusal,
  explanation, feasibility, and lowering;
- the relation to scheduled `LogicalAccess` while keeping logical access meaning
  separate from physical storage addressing; and
- graph sequencing: design acceptance, a separately scoped IR-admission ticket,
  the integer storage carrier, and only then Metal emission.

The expression candidate must treat a tensor read as a nested logical read/value,
not as an untyped arithmetic leaf. The access candidate must state the exact fresh
tag and framing, demonstrate old-byte preservation, and show how predicates and
proof subjects name an indirect axis without weakening existing direct accesses.

## Deliverables

- A revised, still-`proposed` ADR 0108 body that removes the returned premises,
  states the strongest case and counterpoint for each candidate, and contains a
  verbatim-landable decision section for Tom.
- A source-first Fact audit citing searchable symbols or source-safe phrases for
  every construction, validation, proof, identity, compaction, reference,
  compiler, schedule, and graph claim.
- Contract updates to `docs/ir.md` and Q-SHAPE-007 that match the revised proposal
  without presenting it as accepted.
- If the proposal selects a form, a separate implementation/admission ticket with
  the owning code scopes and an explicit dependency added to the Metal emitter.
  This design ticket itself admits and implements nothing.

## Non-goals

Implementing an index form, a schedule relation, host validation, compiler
lowering, or backend emission. Scatter and data-dependent output shapes. Choosing
a public boundary without Tom's separate ADR 0075 decision.

## Stop conditions

Stop and return to Tom if the comparison exposes a consequential public-boundary
choice, a change to ADR 0046's accepted guarantee, an identity-domain step, or an
evidence claim that cannot be represented without changing what a verified region
means. Also stop if a candidate cannot satisfy ADR 0109 before executable coverage:
host validation cannot silently become timeless proof, and superseding ADR 0109
decision 2 is Tom's decision. Decision 4 requires no supersession, but it confirms
that a new run-time or identity contract would also need Tom's authority. Do not
disguise either decision as implementation detail.

## Decision packet — 2026-08-09

The stop condition fired before the vertical comparison could select a representation. Both candidates require consequential public authoring, inspection, validation, and identity choices. More importantly, neither candidate can turn an arbitrary dispatch-time `u32` tensor value into timeless index-domain proof under current ADR 0109 decision 2. A host check can establish one dispatch's semantic precondition, but the repository has no accepted object that carries that result into executable coverage without mislabelling it as program proof.

Tom chooses the authority under which the revision continues:

1. **Keep ADR 0109 decision 2 unchanged and constrain the revision to static proof (recommended).** Complete the two-representation comparison, but select neither form unless it can prove every retained obligation before executable coverage. If neither can, ADR 0108 remains proposed with a non-circular trigger naming the first statically provable indirect-access vertical.
2. **Authorize a dispatch-bound validation design.** Expand the revision to propose the run-time evidence object, its lifetime and identity relation, the exact point before executable coverage where it is consumed, and the explicit supersession of ADR 0109 decision 2. This is an architecture decision, not an implementation shortcut.

The choice does not accept either nested-expression or tagged-access public spelling. Their exact public surfaces still return separately after the comparison. **Strongest counterpoint to option 1:** it may leave gather permanently semantic-only even though `decide_gather_index` was deliberately factored for host validation; option 2 can unlock the real workload, but only by adding the missing run-time and identity authority honestly.

After Tom answers, return this node to `todo` under the selected authority. The dependent ADR 0108 acceptance node remains blocked until the revised proposal is complete.

## Direction accepted for the revision — 2026-08-11

Tom selected option 2 in the T3 Code orchestration conversation, conditional on a deliberately strict narrow first pass and on every excluded lane remaining explicit in tickets and documentation. This acceptance authorizes the revision to design **invocation-bound preflight validation**; it does not accept either logical representation or any public Rust spelling.

The revised proposal must preserve static proof as the zero-runtime-cost first lane. Its only initial dynamic lane is a host-visible `tiler::u32@1` program input whose exact values are validated by Tiler against the exact gathered extent before routing commit. Successful validation mints an invocation-scoped, sealed receipt over an immutable snapshot owned by the validated binding. The receipt is bound to the gather occurrence, extent, value type, and snapshot; it cannot be reused after a value or binding changes and it is consumed by the exact attempt it authorizes.

This is a narrow supersession proposal for ADR 0109 decision 2, not a general permission to execute `Unknown`. Compilation may carry only the selected gather representation's **mandatory named validation obligation** toward preflight; no invocation receives executable authority until that obligation has a matching receipt. An absent, stale, mismatched, unanswerable, or failed validation refuses. An out-of-range value remains a semantic failure naming position, value, and extent; it never becomes a plan miss, clamp, wrap, alternate route, reference execution, or backend fallback.

Mutable zero-copy inputs, device-resident or device-produced indices, validation inside the gather kernel, caller assertions, and generalization beyond gather are excluded from the first pass and now have explicit follow-up tickets or rejection text. The representation comparison must still choose between the complete nested-read and tagged-access candidates, account for every identity-domain consequence, and return every consequential public surface separately under ADR 0075.

Acceptance provenance: Tom's direct 2026-08-11 message, “okay so long as we are strict with our narrow first pass... and have proper tickets/comments/documentation for future work and are not leaving work missing from the tree”.

## Source-first Fact audit at `4c9742df2c9fc41d2d2c68de5606d42d627cac8c`

- **Verified — the negative vocabulary census remains exact.** `IndexNode` in `crates/tiler-ir/src/index/model.rs` still has five forms, `IndexExprClass` still has three members, and `IndexDomainUnknownReason` in `crates/tiler-ir/src/index/predicate.rs` still has three variants. None reads tensor data.
- **Verified — expressions and accesses currently have different dependency algebras.** `mark_expr`, `remap_node`, `encode_index_node`, the structural and alpha keys, and the finite-domain evaluator traverse only dimensions, index expressions, and sourced extents. `AccessData` separately owns the tensor, mode, domain, and coordinate expressions. `CompactionOrder` orders tensors before expressions and accesses after expressions. A tensor-reading `IndexNode` would therefore create new expression-to-tensor and expression-to-access edges, not append one more arithmetic leaf.
- **Verified — existing access construction proves rank before every later coordinate/extent traversal.** `IndexRegionBuilder::prepare_access` checks `coordinates.len() == tensor_data.shape.rank()`, validates every coordinate against the access domain, and only then constructs `AccessData`. The current proof consumers do not receive arbitrary unequal runs.
- **Verified — the access encoding has an append-only lane.** `encode_region` writes access tag `1` for a direct read and `2` for a direct write before the existing tensor/domain/coordinate payload. A fresh tag `3` can encode a gather read while preserving every previously encodable direct-access byte. The tag must frame the source tensor, index tensor, gathered axis, shared domain, direct index coordinates, and direct source coordinates; omitting or inferring one would create either an ambiguous relation or a second authority.
- **Verified — the public access view is currently singular and would become false if left unchanged.** `TensorAccessRef::tensor` and `TensorAccessRef::coordinates` promise one tensor and one coordinate per tensor axis. A gather relation reaches two tensors and has two direct-coordinate runs plus one value-derived source coordinate. The public inspection surface must become a checked sum rather than keep those accessors with partial or guessed meanings.
- **Verified — the existing residual predicate path cannot honestly carry invocation validation.** `UnknownIndexDomainPredicate` names one direct access and an `IndexDomainPredicate` whose expression must be among that access's coordinates. `PendingIndexRefinementReceipt` deliberately has no executable-coverage identity, and `discharge_pending_index_refinement` either completes every residual through finite proof or refuses. A tensor element is not an `IndexExprId`, and “validation required” is a known execution precondition rather than a fourth reason why the proof engine returned `Unknown`.
- **Verified — executable program coverage currently has no conditional spelling.** `CoveredOccurrence::from_receipt` accepts only a completed `IndexRefinementReceipt` and stores its `IndexRefinementExecutableCoverageIdentity`. A dynamic gather therefore needs a separately tagged conditional-coverage subject carrying the exact invocation requirement; it cannot be smuggled through `CoveredOccurrence` as if proof had already happened.
- **Verified — the semantic rule is exact and reusable.** `decide_gather_index` rejects every `u32` value greater than or equal to the gathered extent and reports the exact value and extent. The reference gather uses that rule. The initial host validator can share it without inventing clamp, wrap, signed conversion, or backend-specific semantics.
- **Verified — schedule access is already the semantic relation layer.** `LogicalAccess` distinguishes linear, broadcast, reindex, reduction, contraction, and packed relations from storage addressing. An indirect source read belongs as a new relation on the source access, with the exact index input ordinal and shapes/axis it consumes; the index input remains an ordinary read. The schedule verifier must cross-check the pair and derive one bounds/validation subject rather than accept two independently authored descriptions.
- **Verified — the current artifact route-requirement family is not the right carrier unchanged.** `RouteRequirement` is explicitly live-device evidence and contains only quantitative device resources and backend-owned device features. Invocation input validation is neither. The receipt implementation must add a separately typed invocation requirement and make old readers fail closed, rather than misclassify host data as a device property.
- **Imprecise — “no identity-domain step” is true only for the representation append.** The new index access and schedule relation can use fresh, framed tags and leave all old bytes unchanged. The conditional-coverage and artifact invocation-requirement grammars do not exist yet; their owning tickets must derive whether their tagged additions preserve old encodings or require domain/schema steps. The invocation receipt and copied values remain ephemeral and must never enter artifact or cache identity.

## Representation decision packet — 2026-08-12

Choose the **append-only tagged access representation**, with a narrow gather-specific first variant. Do not add a tensor-reading `IndexNode`.

The exact logical form is a closed access-kind sum. Existing direct accesses retain their current tensor, mode, domain, and rank-equal coordinate list and their exact bytes. A new `GatherRead` names one F32 source tensor, one U32 index tensor, one gathered source axis, one shared output iteration domain, one direct coordinate per index-tensor axis, and one direct coordinate per non-gathered source axis. The gathered source coordinate is exactly the U32 value loaded from the named index tensor at the named index coordinates. Source rank must be nonzero; the gathered axis must be in range; both tensors must be inputs in the first pass; coordinate counts and domains must match their respective ranks; source/index/result shapes must satisfy the semantic gather relation; and every checked size or address conversion must refuse overflow. A U32 index widens losslessly to the verifier's unsigned coordinate space and then enters ordinary checked storage-address arithmetic; no signed, truncating, target-width-dependent, clamp, or wrap interpretation exists.

Public construction and inspection must expose the sum truthfully, for example as private `AccessKind::{Direct, GatherRead}` storage plus nonconstructible `TensorAccessView::{Direct, GatherRead}` borrowed views. A direct view keeps one tensor and one rank-equal coordinate run. A gather view exposes both tensors, the axis, and both direct-coordinate runs. There is no `Option` whose absence means direct, no generic tensor-valued expression callback, and no recursive source access in the first version.

The access carries one intrinsic `GatherIndexBoundRequirement` over the exact gather access, index tensor/type, source axis/extent, and semantic occurrence. Resolution is a total sum: `StaticallyProved` carries named timeless proof authority, while `InvocationValidationRequired` carries the exact immutable preflight requirement. The initial public producer supports the latter only for a host-visible U32 program input; a future constant/range authority may mint the former without changing gather meaning. This requirement is not an `IndexDomainUnknownReason` and is never treated as a proof-engine miss.

Compilation may package a conditionally covered gather route only by carrying the complete requirement in program/artifact identity and by marking the artifact with a decoder-visible invocation-validation capability. It still has no dispatch authority. Runtime validates the exact bound input, copies the checked U32 values into receipt-owned immutable storage, and binds the receipt to occurrence, type, extent, program binding, snapshot content, and attempt. Only consuming that exact receipt may produce the non-`Clone` preflight whose commit is infallible. Missing, stale, crossed, already-consumed, or failed evidence refuses before commit. This is the narrow supersession of ADR 0109 decision 2: named conditional coverage may reach packaging, but executable invocation coverage still requires proof or the exact receipt.

At schedule level, add an append-only `LogicalAccess::GatherSource` relation to the source read. It carries source shape, result shape, gathered axis, index-input ordinal, and index shape. The corresponding index input is an ordinary U32 read. The schedule builder derives both from one index-law subject and verifies that the ordinary read and gather-source relation name the same input and shape; callers do not author two independent authorities. Physical lowering may then map the relation to target addressing, but cannot change its bounds rule or index interpretation.

The nested-expression alternative is rejected for the first vertical. Its strongest case is future composition: gather-of-gather, tensor-valued coordinates, and `gather_nd` could reuse one expression tree. That flexibility is also its present defect. It introduces recursive reachability and cycle questions, typed value reads inside a previously pure integer algebra, tensor/access dependencies in expression compaction and alpha identity, and proof subjects that no current second consumer needs. Those costs buy unsupported meanings and make direct proof maintenance harder. Reconsider it only when a second accepted semantic family requires nested composition that the tagged access sum cannot state without duplicating meaning.

### Ranked alternatives

1. **Gather-specific tagged access plus typed conditional coverage and invocation receipt.** Best correctness and maintainability; old direct bytes and proof rules remain unchanged; compile validation is O(rank), host validation is O(index elements), and the immutable copy is exactly four bytes per index. The selected workload's maximum 8,192 indices therefore copies 32 KiB.
2. **A broader nonrecursive indirect-access sum.** Can be equally sound, but freezes generalized source/value/address semantics before a second consumer exists and increases verification and public-surface burden with no current runtime benefit.
3. **Tensor-reading `IndexNode`.** More composition in principle, but merges pure coordinate expressions with typed logical effects and recursively widens compaction, identity, reachability, reference evaluation, and proof ownership. Dominated for the admitted gather vertical.
4. **Backend-only indexing, caller assertions, inline bounds checks, clamp/wrap, or treating validation as `Unknown`.** Rejected: each either bypasses the logical proof boundary, makes a semantic error backend-dependent, or grants execution without the exact authority ADR 0109 requires.

### Strongest counterpoint and reversal trigger

The access sum is intentionally less universal. If a second accepted operation requires a tensor-derived coordinate to feed another tensor-derived coordinate, or requires several indirect axes whose semantics cannot be represented as one checked nonrecursive relation without duplicating source reads, reopen the expression decision with that concrete consumer. Backend preference, code reuse, or hypothetical scatter is not enough.

### Identity and compatibility boundary

Use index access tag `0x03` and a separately assigned schedule access tag, with every variable run length-framed. Preserve tags `0x01` and `0x02` and every old direct payload byte. Existing direct index-region, schedule, kernel, artifact, and cache identities remain byte-identical. New gather subjects receive new identities naturally. The conditional coverage requirement is timeless identity; the observed values, immutable snapshot, receipt nonce/generation, and validation result are invocation evidence and never enter reusable identities. The receipt implementation must prove old binaries reject the new requirement before interpreting or dispatching it; if the existing artifact grammar cannot provide that fence through a fresh tagged row and required feature, it must take the appropriate major schema/domain step rather than rely on lockstep deployment prose.

## Closes when

The complete comparison and revised proposal are coherent, source-audited, and
ready for the acceptance ticket's decision. Closure does not satisfy or unblock
backend emission by itself.
