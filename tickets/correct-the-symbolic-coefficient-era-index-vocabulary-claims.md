---
id: correct-the-symbolic-coefficient-era-index-vocabulary-claims
title: Correct the symbolic-coefficient-era index vocabulary claims
status: done
priority: p2
dependencies: []
related: [admit-symbolic-index-expression-coefficients, repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them, admit-a-position-selecting-slice-for-the-rotary-table]
scopes: [research/shapes, research/indexing, implementation/ir]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [docs, doc-drift, indexing]
---
## What is stale

`admit-symbolic-index-expression-coefficients` admitted a declared `ShapeSymbol` as a `LinearCombination` coefficient and as the sourced constructor's additive argument, so a bound symbol now reaches an index expression in a *coordinate* position and not only as a `FloorDiv` or `Modulo` divisor. A symbolic additive argument normalizes to the term `symbol * 1`; the stored and viewed `LinearCombination` constant remains exact. Verified at `cd86cac1`:

- `LinearTermData::coefficient` is `SourcedIndexInteger` (`crates/tiler-ir/src/index/model.rs:100-103`).
- `IndexRegionBuilder::sourced_linear_combination` takes a `SourcedIndexInteger` constant and `SourcedIndexInteger` coefficients (`crates/tiler-ir/src/index/builder.rs`, anchor `pub fn sourced_linear_combination`).
- [`docs/ir.md`](../docs/ir.md)'s implemented-extent paragraph states the coefficient and addend correctly. Its separate slice and semantic-shape paragraphs do not, so this ticket records them as an out-of-scope contract remainder rather than treating the whole contract as current.

Several records still state the divisor-only bound as current fact, and each states it as the ground for a reserved capability rather than in passing. This was found while repointing the 2026-08-07 relocation under [`repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them`](repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them.md), which deliberately did **not** repair it: the cause is different, the correction takes a design position about what the sub-tensor selection family's symbolic-offset trigger now blocks on, and the owning ticket was `in-progress` at the time.

## Sites, each verified at `cd86cac1`

| Site | Scope | What it claims |
| --- | --- | --- |
| `docs/roadmap.md:481` | `contracts/navigation` | "`SourcedExtent` is the only carrier of a symbolic extent and appears in no variant except the `FloorDiv` and `Modulo` divisors, so a literal-offset selection is expressible today and a symbolic-offset one is not", and the reserved trigger "an `IndexNode` variant carrying a `SourcedExtent` in a coordinate position". |
| `docs/open-questions.md:275` (Q-SHAPE-006) | `contracts/navigation` | "`SourcedExtent` is the only `IndexNode` variant that carries a possibly-symbolic extent and it appears in no other position, so `t + k` for a literal `k` is expressible and `t + C` for a bound symbol is not." |
| `docs/research/shapes/sequence-extending-tensor-family.md:74-78` | `research/shapes` | Reproducible check 3's headline, "No coordinate expression carries an extent symbol", and its comment "LinearCombination's constant is a literal `IndexInteger`". |
| `docs/research/shapes/transformer-operation-and-shape-surface.md:122` | `research/shapes` | "*First, a symbol reaches an index expression only as a floor-division or modulo divisor.*", with `builder.rs:1019` and `builder.rs:974` beneath it — both ordinals also drifted, and they were deliberately left unrepointed so the citation would not read as freshly verified. |
| `docs/research/indexing/sub-tensor-selection-fusion-role.md:117` | `research/indexing` | States the reserved symbolic relation's trigger as "an `IndexNode` variant carrying a `SourcedExtent` in a coordinate position". |
| `docs/research/indexing/concatenate-fusion-role-and-lowering.md:103` | `research/indexing` | Lower confidence: its claim about `IndexNode::LinearCombination { constant: IndexInteger, … }` is still literally true of the *node*, but its `index/model.rs:97-100` and `:101-108` ranges have drifted to `:108-111` and `:112-119` and it elides `LinearTermData`. |
| `docs/research/indexing/concatenate-fusion-role-and-lowering.md`, anchor `A concatenate occurrence carrying a symbolic extent` | `research/indexing` | A separate live contingency says a symbolic concatenate would reopen Q-SHAPE-006's coordinate carrier gap. It is false after the coefficient landing even though its static-range proof question remains consequential. |
| `crates/tiler-ir/src/semantic/slice.rs:340` | `implementation/ir` | The `SymbolicOffsetUnsupported` doc comment carries the same divisor-only sentence. A comment is a claim about current behaviour, so it is in the population. |
| `crates/tiler-ir/src/semantic/slice/tests.rs`, anchor `the refusal states the delivered half` | `implementation/ir` | The reserved-symbolic relation test asserted the retired `bound extent symbol` diagnostic wording. It must pin the corrected literal-selection grammar boundary exactly. |

Do not trust this list to be exhaustive; re-derive it, and read each file in full rather than editing around a grep hit.

## The judgement this ticket owes, and must not skip

**The literal wording survives for `SourcedExtent` and the claim it supports does not.** `SourcedExtent` really does appear in no `IndexNode` variant outside the two divisors — it is `SourcedIndexInteger` that reaches a coefficient or an additive term — so a find-and-replace produces sentences that are true and still mislead. What each site actually asserts is that *a bound symbol cannot reach a coordinate position*, and that is false.

The consequential half is what it does to two reserved triggers. `docs/roadmap.md`'s sub-tensor-selection row and Q-SHAPE-006 both cite the divisor-only bound as the reason a symbolic-offset slice is not expressible. The family's refusal under `slice.selection.symbolic-offset-unsupported` stands either way; what needs re-deriving is whether [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) still owes the *index vocabulary* it is recorded as owing, or now owes the literal-only semantic selection grammar. Decide that from the source, state it, and correct the trigger wording to match; do not restate the bound with a new type name and leave the trigger reading as it did.

## Fact audit at `7134a7322b217b3149bbf39c3f976233c96b257e`

- **Imprecise:** the opening claim that a symbol was admitted to the `LinearCombination` "constant term" overstates the stored representation. `crates/tiler-ir/src/index/model.rs`, anchors `LinearCombination {` and `Additive constant, always exact`, keeps that field as `IndexInteger`; `crates/tiler-ir/src/index/builder.rs`, anchor `pub fn sourced_linear_combination`, accepts the sourced argument and normalizes a symbol into a `symbol * 1` term. The text above is repaired rather than silently relied on.
- **Verified:** `LinearTermData::coefficient` is `SourcedIndexInteger` in `crates/tiler-ir/src/index/model.rs`, anchor `pub(super) struct LinearTermData`.
- **Verified:** the sourced builder accepts `SourcedIndexInteger` for both its `constant` argument and every term coefficient in `crates/tiler-ir/src/index/builder.rs`, anchor `pub fn sourced_linear_combination`.
- **False:** `docs/ir.md`, anchors `the offset is a literal` and `What it does not reach is an operation`, says a semantic occurrence or `ValueFact` has static extents and names `BuildError::SymbolicOperandUnsupported`. `crates/tiler-ir/src/semantic/operation.rs`, anchor `pub struct ValueFact`, stores `SourcedShape`; `ValueFact::shape` returns it, and `BuildError::SymbolicOperandUnsupported` has been removed. This contract-file remainder is outside the declared scopes and is reported below.
- **Verified as stale live claim:** `docs/roadmap.md`, anchor `SourcedExtent is the only carrier`, derives the sub-tensor trigger from the divisor-only premise. Its dated correction now names the literal-only `SliceAxisSelection::Window` grammar and `decode_axis` as the remaining boundary.
- **Verified as stale live claim:** `docs/open-questions.md`, anchor `The sub-tensor selection family's`, derives Q-SHAPE-006's near miss from the divisor-only premise. Its dated correction retains the unfired verdict while locating the grammar refusal.
- **Verified as stale live claim:** `docs/research/shapes/sequence-extending-tensor-family.md`, anchor `No coordinate expression carries`, supplied a now-false reproducible check. The check and its dated correction now name the coefficient and symbolic-addend construction.
- **Verified as stale live claim:** `docs/research/shapes/transformer-operation-and-shape-surface.md`, anchor `First, a symbol reaches`, was true when its dated correction was written but was superseded by the coefficient landing. Its new correction distinguishes the implemented index expression from the unconstructible semantic selection.
- **Verified as stale live claim:** `docs/research/indexing/sub-tensor-selection-fusion-role.md`, anchor `an IndexNode variant carrying`, preserved the former reconsideration trigger. Its dated correction now names the selection grammar.
- **Imprecise as a current conclusion:** `docs/research/indexing/concatenate-fusion-role-and-lowering.md`, anchor `IndexNode::LinearCombination { constant: IndexInteger, terms }`, correctly describes the stored node but elides `LinearTermData`; its dated correction distinguishes concatenate's rule-specific `static_operand_shape` use from the false claim that every semantic occurrence is static.
- **False live contingency:** the same record, anchor `A concatenate occurrence carrying a symbolic extent`, said that a future symbolic concatenate would move its offsets into Q-SHAPE-006's carrier gap. Its new dated correction preserves the live proof question — source-bearing ranges would require a new joint-coverage derivation — while separating it from the closed coordinate-vocabulary gap. `Concatenate` currently calls `OperationInferenceRequest::static_operand_shape`, so no source-bearing concatenate occurrence is constructible.
- **Verified as stale live claims:** `crates/tiler-ir/src/semantic/slice.rs` repeats the former ground in five non-identity-bearing locations: anchors `The refusal stands at this layer's`, `The current inferencer deliberately asks`, `Reserved name of the symbolic-offset relation`, `The selection states a symbolic offset`, and `source-bearing offset field`. They are corrected in place. The sixth live copy, `SLICE_F32_NORMATIVE_DEFINITION`, is identity-bearing and remains mapped below.
- **False test assertion:** `crates/tiler-ir/src/semantic/slice/tests.rs`, anchor `the refusal states the delivered half`, still required `bound extent symbol` in `SymbolicOffsetUnsupported`'s message. It now compares the entire corrected diagnostic, including `literal offsets` and `no source-bearing offset field`, so a change to either premise fails. A deliberate diagnostic-subject perturbation to `source-bearing selection field` produced `assertion \`left == right\` failed`: left was `symbolic-window is reserved and not admitted: this family selects at literal offsets, and its current selection grammar has no source-bearing selection field`; right ended `no source-bearing offset field`. The assertion rather than its expectation remained unchanged.
- **Verified:** `crates/tiler-ir/src/semantic/program.rs`, anchor `fn push_operation`, mints each operation operand `ValueFact` from the stored `SourcedShape`; `crates/tiler-ir/src/semantic/standard_operations.rs`, anchor `pub struct F32Slice`, accepts only `&SliceSelection`; `crates/tiler-ir/src/semantic/slice.rs`, anchors `offset: u64`, `SLICE_RELATION_SYMBOLIC_WINDOW`, and `match (name, fields)`, show the exact current grammar rejection. `SliceF32::infer`, anchor `request.static_operand_shape`, is a later, independent literal-operand bounds restriction.

## Additional-site classification and identity remainder

The trigger tickets and records `admit-the-sub-tensor-selection-family`, `admit-a-position-selecting-slice-for-the-rotary-table`, `reclassify-language-model-work-as-a-conformance-track`, `lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`, and `scope-the-sequence-extending-tensor-family` repeat the former index-vocabulary ground. They are reported here rather than rewritten on this ticket branch: each is a historical delivered-ticket account or a separately owned live implementation ticket. The open rotary ticket remains the consumer path that needs a symbolic selection; this correction does not decide the new relation's attribute-versus-operand public boundary, identity, or bounds rule.

`docs/ir.md` is a second mapped remainder outside this ticket's declared `contracts/navigation` scope: its anchors `the offset is a literal`, `What it does not reach is an operation`, and `a semantic value's shape is still a Shape` retain the false static-`ValueFact`/removed-`BuildError` account. Its implemented-extent paragraph remains a valid source for the coefficient work, but the three source anchors require a separate `contracts/foundation` correction.

The broader semantic-shape sweep also found `docs/research/shapes/symbolic-semantic-extents.md`, anchors `The Fact's narrow half survives`, `No symbol reaches an INFERRED RESULT`, and `keeping ValueFact on a fixed Shape`, plus `docs/roadmap.md`, anchor `a symbolic contracted extent is not reached either`. These are independent stale accounts of the already-landed general `ValueFact` transition, not the coefficient-era slice trigger this ticket corrects. The former repair track is already complete, so [`repair-the-shape-records-after-sourced-semantic-result-shapes`](repair-the-shape-records-after-sourced-semantic-result-shapes.md) now owns this newly discovered drift rather than pretending the historical ticket remains active.

A renewed corpus search after this correction used the anchors `carrier gap Q-SHAPE-006`, `No coordinate expression carries an extent symbol`, and `ValueFact on a fixed Shape`. Apart from the newly classified concatenate contingency, its current-looking hits are the mapped `docs/ir.md` contract remainder and the independently owned semantic-shape records above; the other matches quote retired wording inside dated corrections or live only in historical ticket accounts. No additional unclassified live carrier-gap or static-`ValueFact` premise is left in this ticket's owned document population.

`crates/tiler-ir/src/semantic/slice.rs`, anchor `SLICE_F32_NORMATIVE_DEFINITION`, is a further live stale site that this ticket cannot safely edit in isolation. `OperationDefinition::new` registers it; `encode_operation_definition` in `crates/tiler-ir/src/semantic/registry.rs` frames it into both the reached-definition projection and the frozen registry snapshot. Changing that string therefore moves identities for slice-using semantic programs and the standard-registry subject consumed by compiler pins. This branch leaves the identity-bearing string unchanged; [`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md) owns the coherent definition-and-pin recomputation.

## How to repair

Follow each file's existing convention. `docs/roadmap.md`, `docs/open-questions.md`, and the two shapes records all use dated corrections that quote the superseded sentence rather than rewriting it away. `docs/ir.md`'s implemented-extent paragraph is useful source evidence, but its stale slice and semantic-shape wording is an out-of-scope `contracts/foundation` remainder, not a current statement to copy.

## Closes when

Every declared non-identity site above is classified as a live claim repaired with a dated correction, or as an already-recorded correction needing none, with the classification stated per site rather than counted; the corrected symbolic-window diagnostic is pinned by its complete test assertion and the recorded subject perturbation; the two reserved triggers are re-derived from source and their wording states that the remaining symbolic-slice boundary is the literal-only selection grammar; and the `docs/ir.md` contract, broader semantic-shape records, and `SLICE_F32_NORMATIVE_DEFINITION` identity-bearing remainder have explicit follow-up owners rather than being claimed fixed.

## Outcome and completed remainders — 2026-08-09

Commit `c7ffe174` corrected the non-identity source and document population and
its exact symbolic-window diagnostic. Commit `83bb7839` closed this ticket after
filing the three separately scoped remainders. All three have since landed:
[`correct-the-ir-contract-after-sourced-semantic-result-shapes`](correct-the-ir-contract-after-sourced-semantic-result-shapes.md),
[`repair-the-shape-records-after-sourced-semantic-result-shapes`](repair-the-shape-records-after-sourced-semantic-result-shapes.md),
and
[`correct-the-slice-normative-definition-and-recompute-compiler-identities`](correct-the-slice-normative-definition-and-recompute-compiler-identities.md).

The remaining symbolic Slice admission is still owned by
[`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md);
this completed correction did not decide its attribute-versus-operand boundary
or its identity consequences. No stale vocabulary site discovered by this
ticket remains unowned.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The Outcome sentence "No stale vocabulary site discovered by this ticket remains unowned" is true only for the ticket's enumerated owned population and the three search anchors it re-ran (`carrier gap Q-SHAPE-006`, `No coordinate expression carries an extent symbol`, `ValueFact on a fixed Shape`). It is false as corpus-wide completeness: `docs/glossary.md` Slice row still asserts in present tense that "the offset is a literal because no index expression carries an extent symbol in a coordinate position." The offset *is* still a literal (window grammar admits only `offset: u64`); the *ground* is the retired coordinate-carrier claim. The glossary wording does not match this ticket's renewed-search anchors, so it was not discovered when the Outcome was written, and none of the three named remainders owns that row. Status stays `done` for the declared population; a narrow `contracts/navigation` (glossary-owning) remainder still needs filing to replace that ground with the literal-only selection-grammar boundary and optionally re-scan for other glossary-class hits of the same conclusion.
