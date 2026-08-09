---
id: repair-the-shape-records-after-sourced-semantic-result-shapes
title: Repair the shape records after sourced semantic result shapes
status: done
priority: p1
dependencies: [correct-the-symbolic-coefficient-era-index-vocabulary-claims]
related: [repair-the-records-the-sourced-semantic-shape-falsifies]
scopes: [research/shapes]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, shapes, correction]
---
## Why this exists

The earlier [`repair-the-records-the-sourced-semantic-shape-falsifies`](repair-the-records-the-sourced-semantic-shape-falsifies.md) ticket is complete and records an older transition. It cannot own newly discovered drift as an active work lane. The coefficient-era audit found present-tense research and roadmap claims that still say inferred semantic results or `ValueFact` are static after sourced result-shape inference landed.

## Starting evidence, stale until re-read at this ticket's base

- `docs/research/shapes/symbolic-semantic-extents.md`, anchors `The Fact's narrow half survives`, `no symbol reaches an *inferred result*`, and `keeping \`ValueFact\` on a fixed \`Shape\``, retains the former boundary. These are literal source fragments, verified with fixed-string search rather than copied rendered wording.
- `docs/roadmap.md`, anchor `a symbolic contracted extent is not reached either`, derives a current contraction limitation from a static value fact.
- `crates/tiler-ir/src/semantic/operation.rs`, anchor `pub struct ValueFact`, stores `SourcedShape`.
- `crates/tiler-ir/src/semantic/program.rs`, anchor `fn push_operation`, carries operation inference results into those facts.
- [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md) records the implemented transition and its maturity boundary; it is evidence to verify, not a substitute for reading source.

The worker's first deliverable is a per-Fact verdict from the exact base. Read both documents in full, the complete implementation sites, the completed predecessor ticket, and governing accepted shape decisions. Re-derive the affected population rather than assuming these anchors are exhaustive.

## Per-Fact audit at base `96e867a3ec2d370ccac42ebb1273073d45f1effa`, before any edit

| Starting Fact | Verdict | Evidence read at this base |
| --- | --- | --- |
| The three named anchors in `symbolic-semantic-extents.md` retain the former fixed-result boundary. | **verified as drift** — each anchor occurs once, and each is false as a present-tense claim. `ValueFact` is source-bearing, not fixed-shape. | `docs/research/shapes/symbolic-semantic-extents.md`; `crates/tiler-ir/src/semantic/operation.rs` anchors `pub struct ValueFact` and `pub const fn shape(&self) -> &SourcedShape`. |
| The contraction row derives its live limitation from a static semantic value fact. | **verified as drift** — `a symbolic contracted extent is not reached either` is live and names that retired premise. | `docs/roadmap.md` anchor `a symbolic contracted extent is not reached either`; `crates/tiler-ir/src/semantic/contraction.rs` anchor `Extent agreement, through the accepted three-outcome path. A semantic`. |
| `ValueFact` stores `SourcedShape`. | **verified.** | `crates/tiler-ir/src/semantic/operation.rs` anchor `pub(super) shape: SourcedShape`. |
| `push_operation` carries inferred result shapes into value facts. | **verified.** It submits cloned source-bearing operand facts to `infer_operation_with_extent_sources` and stores `shape: fact.shape` for every inferred result. | `crates/tiler-ir/src/semantic/program.rs` anchor `fn push_operation` and its `shape: fact.shape` insertion. |
| `resolve-semantic-shape-inference-over-symbolic-extents` is usable evidence for the transition and its boundary. | **imprecise.** Its status is `awaiting-decision`, but its delivered record and the source agree that the elementwise transition landed. The current boundary is family-specific, not a `ValueFact` or result-storage limit. | `tickets/resolve-semantic-shape-inference-over-symbolic-extents.md`; `crates/tiler-ir/src/semantic/registry.rs` anchor `pub(crate) fn elementwise_binary_shape`; family inferencers calling `static_operand_shape`. |
| `transformer-operation-and-shape-surface.md` anchor `What survives is the half this Fact was offered for` correctly retained a fixed-result boundary, its former typed refusal, and row 3 `in-progress`. | **false as live correction.** `ValueFact` and `push_operation` preserve `SourcedShape`; the shared elementwise rule constructs sourced results; literal-only families decline through `static_operand_shape` with `ExtentSourceError::SymbolicExtentUnsupported`; row 3 is `awaiting-decision`. | `docs/research/shapes/transformer-operation-and-shape-surface.md`; `crates/tiler-ir/src/semantic/{operation,program,registry}.rs`; `tickets/resolve-semantic-shape-inference-over-symbolic-extents.md`. |
| `docs/roadmap.md` anchor `Fact — the symbolic-offset boundary` correctly identifies a divisor-only index vocabulary and static semantic value fact as Slice's current blockers. | **false as live correction.** Source-bearing linear-combination addends/coefficients reach a coordinate, and `ValueFact` is source-bearing. Slice still refuses `symbolic-window`, but at its literal-only selection grammar and its family-owned bounds rule. | `docs/roadmap.md`; `crates/tiler-ir/src/index/{builder,model}.rs`; `crates/tiler-ir/src/semantic/{operation,program,slice}.rs`. |
| `docs/roadmap.md` Softmax row anchor `The half that landed is not the half this row needs` correctly retains a fixed inferred-result boundary, `SymbolicOperandUnsupported`, and resolve `in-progress` as its live blockers. | **false as live correction.** Softmax's current refusal is `SoftmaxF32::infer` calling `static_operand_shape`, and its focused sourced-operand test observes the typed refusal. The C1 conclusion remains, but the family rule — not `ValueFact` — supplies it. | `docs/roadmap.md`; `crates/tiler-ir/src/semantic/softmax.rs` anchor `let input = request.static_operand_shape(0)?`; `crates/tiler-ir/src/semantic/softmax/tests.rs` anchor `a_symbolic_reduced_extent_is_refused_and_every_literal_one_infers`. |

The audit does not change this ticket's purpose. It narrows the correction: `add-f32`, `multiply-f32`, `add-bf16`, and `multiply-bf16` construct sourced results through the shared elementwise rule; its left boundary is retained after `ExtentSources::proves_equal`. `strict-serial-sum-f32`, strict tensor contraction, broadcast, concatenate, gather, reindex, slice, softmax, RMS normalization, SiLU, and strict-affine quantization still decline a symbolic operand through `static_operand_shape`, at their family inference/bounds/schema boundary rather than in `ValueFact`. Constants are rank-zero and therefore static by construction. Compiler and physical planning remain a later boundary for every symbolic interface, independently of which semantic family constructs a sourced result. The strict-sum source anchor is `impl OperationInferencer for StrictSerialSumF32` in `semantic/registry.rs`; the focused `a_literal_only_family_declines_a_symbolic_operand_by_name` test constructs its sourced operand and asserts the resulting typed refusal, so the negative-control enumeration reaches the registry rather than omitting it.

## Population and remainder re-derived on 2026-08-08

The seven owned live claims are the three anchors in `symbolic-semantic-extents.md`, the transformer-surface `What survives is the half this Fact was offered for` correction, and the three roadmap corrections for contraction, Slice, and Softmax. They are corrected in this ticket. The following current claims share the same retired premise but are outside this ticket's declared paths and remain mapped, not edited:

- `docs/research/program-planning/flash-class-capability-set.md`, `the growing context axis is a symbolic extent`: false where it says a `ValueFact` is static and all three delivery links are `todo`. The planning boundary remains later, but the semantic-result representation is already source-bearing and the resolve record is `awaiting-decision`.
- `docs/research/program-planning/complete-model-ingestion-and-execution.md`, `The count survives because the C1 row's shapes are inferred results` and `still thirteen as of 2026-08-08`: false where they retain fixed `ValueFact`, `SymbolicOperandUnsupported`, and `in-progress` claims. Its broadcast attribute/identity argument is a separate, potentially remaining family boundary and must be re-read rather than discarded with the correction.
- `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md`, `the_reduced_extent_is_always_literal_so_no_symbolic_refusal_can_fire`: false as a general representation claim. The present softmax family still declines a sourced operand through its own `static_operand_shape`/reduction-boundary path; any follow-up must preserve that distinction and replace its unreachable-check rationale.

Two implementation records also need separate ownership: `crates/tiler-ir/src/semantic/concatenate.rs`, `Every extent a semantic occurrence can carry`, generalizes the retired static-occurrence claim; and `crates/tiler-ir/src/semantic/slice.rs`, `carries static extents`, occurs inside the old normative-definition text. Slice's literal selection identity remains current — `SliceAxisSelection::Window` has a literal offset and `decode_axis` refuses `symbolic-window` before inference — but the quoted old index/value premises are retired and conflict with the current module-level correction. A follow-up ticket must own that source/document population before this ticket can close.

`crates/tiler-ir/src/semantic/contraction/tests.rs`, `The unresolved outcome remains unreachable`, is the sixth out-of-scope implementation record. Its `ValueFact`/static-`Extent` explanation is false: the unresolved equality outcome remains unreachable for this family because `StrictTensorContractionF32::infer` calls `static_operand_shape` before it can bind or compare indices, and it has no symbolic equality or unresolved-requirement rule. The test comment belongs with the contraction implementation scope and is not edited here.

## Follow-up ownership recorded at integration

- [`repair-downstream-records-after-sourced-semantic-results`](repair-downstream-records-after-sourced-semantic-results.md) owns the three research records under `research/program-planning` and `research/numerics`.
- [`correct-static-valuefact-premises-in-semantic-family-comments`](correct-static-valuefact-premises-in-semantic-family-comments.md) owns the Concatenate and Contraction implementation comments.
- [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md) already owns the identity-bearing Slice definition and downstream compiler-pin recomputation.

## Outcome

The owned seven-claim correction landed in `cb21e6a6` (`docs: correct sourced semantic result-shape records`, 2026-08-08) and the ticket closed in `4aef7812`. Dated corrections in `symbolic-semantic-extents.md`, `transformer-operation-and-shape-surface.md`, and the contraction, Slice, and Softmax roadmap rows now distinguish the old static-result premise from the current source-bearing `ValueFact` and `push_operation` path. They name the four elementwise families that construct sourced results and place each surviving refusal at its family-specific schema, inference, bounds, or later lowering boundary.

The change was documentation-only: it introduced no operation schema, inference rule, compiler capability, public API, or identity change, and it preserved historical measurements and conclusions not derived from the retired premise. `make citations`, `tkt lint`, `git diff --check`, and the exact-base scope guard passed before closure.

All mapped remainders have since completed. [`repair-downstream-records-after-sourced-semantic-results`](repair-downstream-records-after-sourced-semantic-results.md), [`correct-static-valuefact-premises-in-semantic-family-comments`](correct-static-valuefact-premises-in-semantic-family-comments.md), and [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md) are `done`; they own the research, implementation-comment, and identity-bearing Slice populations respectively. This ticket therefore has no still-unowned six-site remainder.

## Closes when

Every live research/navigation claim in this ticket's scopes derived from fixed `ValueFact::shape` is classified and corrected or supported; the contraction, Slice, and Softmax rows no longer call a general static-value or divisor-only boundary their blockers; related historical tickets are not presented as active owners; the six mapped out-of-scope implementation/research records have a follow-up owner; `make citations`, `tkt lint`, and `git diff --check` pass; and `tkt guard` shows no undeclared scope.
