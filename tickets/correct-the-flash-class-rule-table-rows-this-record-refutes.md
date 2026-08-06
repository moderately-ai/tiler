---
id: correct-the-flash-class-rule-table-rows-this-record-refutes
title: Correct the flash-class rule table's R1 and R3 rows
status: in-progress
priority: p3
dependencies: []
related: [derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels, probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, program-planning, optimizer, numerics]
claimed_from: todo
assignee: agent-flash-rows
lease_expires_at: 1786026692
---
## User-visible outcome

[The flash-class capability record](../docs/research/program-planning/flash-class-capability-set.md)'s five-rule table states two rows that later records refute, and the table is a filed probe's declared input — so a reader reaching it for that purpose reads the corrected rows rather than re-deriving the corrections.

## Why this exists

**Fact — the R3 row is refuted by a merged record.** The table lists R3, the tree merge of `(m, d)` pairs, as consuming "the same two" dimensions as R2. [The tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md)'s Part 1 derives that a tree form consumes those two **and reassociation**, because reaching the pinned strict left fold from a tree grouping is the move the reduction contract's allowed-trees table governs and `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` pins the left fold. Its Outcome states the consequence directly: "the parallel form consumes **three** dimensions, not two."

**Fact — the R1 row is refuted by [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md)'s Part 2.** The table lists R1 with an owner for its bound, alongside R2 and R3, which reads as a rule with an independently matchable subject. It has none: `crates/tiler-ir/src/semantic/softmax.rs:6` records that the graph "admits none of a `Maximum` reduction, a general `Exp`, or a general `Divide` as a semantic key", so there is no pair of `exp` occurrences to fold. [The elementary-identity record](../docs/research/numerics/elementary-identity-rewrite-dimension.md)'s Part 6 had already checked exactly this candidate and found it not statable. R1 is a step *inside* R2's derivation and its price is already charged once in R2's bound; a separate rule composing with R2 over one program would charge the same evaluations twice.

**Inference — this matters because the table is an input, not a summary.** The record's axis 5 supplies it as "the missing input" to [`probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary`](probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md)'s stop condition (a), so a probe dispatcher reads the rows as the declared rewrite set. A row naming a rule with no matchable subject and a row understating a rule's consumed dimensions are both defects in that input.

## What this ticket must produce

- The two corrected rows, with the refuting records cited at the row rather than in a footnote, and the conditional restated where it belongs: R1 becomes a rule object exactly when a general `Exp` key is registered, at which point it consumes elementary-function identity alone and is the one rule in the table with no shape dependence.
- Whatever nearby sentences the corrections falsify — the record's axis 5 inference that "none of R1, R2, R3, or R5" needs a schedule-space concept is worth re-reading against R3's third dimension and against [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md)'s finding that R2's *dimension set* is a function of the scheduled fold tree, which is a schedule-derived input even though the rule's statement is algebraic.
- A dated line on the probe deferral's trigger check log if the corrected table changes whether stop condition (a) is answered.

## Non-goals

Re-deriving either bound; editing `docs/research/numerics/**` or `docs/decisions/**`; admitting a permission; reactivating the probe.

## Closes when

Both rows state what the merged records derive, every sentence the corrections falsify has been swept, and a reader of the table can act on it without consulting either refuting record first.
