---
id: correct-the-flash-class-rule-table-rows-this-record-refutes
title: Correct the flash-class rule table's R1 and R3 rows
status: review
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

## Outcome

**Both rows are corrected in place, in the corpus's dated-correction convention**: the proposed content stays visible as what was proposed and the correction is appended in the cell it falsifies, citing the refuting record at the row rather than in a footnote. Nothing was re-derived; every corrected claim is a citation of a merged record read at that record's own base.

- **R1 — all three cells.** The rule cell records that R1 is not a rule object over the registered vocabulary, quoting `crates/tiler-ir/src/semantic/softmax.rs:6` through [the rule-object record's](../docs/research/numerics/online-softmax-rule-object.md) Part 2 and naming [the elementary-identity record's](../docs/research/numerics/elementary-identity-rewrite-dimension.md) Part 6 as the check that had already run, and states the condition under which it becomes one — a registered general `Exp` key. The consumes cell keeps `elementary-function identity` and records that it holds unconditionally *and at every shape* under that registration, which makes R1 the one rule in the table with no shape dependence. The owner cell records that R1 owns no bound: the telescoping step is charged once inside R2's, so a separate R1 bound composed with R2 over one program would double-charge the same evaluations.
- **R3 — consumes and owner.** The consumes cell corrects "the same two" to **three**, citing [the tree-fold record's](../docs/research/numerics/tree-fold-online-softmax-bound.md) Part 1 derivation, `SOFTMAX_F32_FACT_SUM_FOLD_ORDER`, the reduction contract's allowed-trees table, and that record's Outcome sentence, and records that reassociation is separately grantable — a conjunct to check, not a third blocked door — and why R2's row is unchanged (Algorithm 3 is the left-deep tree, whose expansion is the canonical grouping). The owner cell corrects **underived** to derived and merged, carrying `P(D, h2)`, its first-order form, the shape-matched-baseline requirement, and the sequential specialization `D = h2 = V − 1`.
- **The axis-5 prose, swept.** The stop-condition paragraph's (b) verdict is **narrowed, not withdrawn**: R2's and R3's statements stay algebraic, which is what (b) asks, but [the rule-object record's](../docs/research/numerics/online-softmax-rule-object.md) Part 3 derives that their consumed dimension set is a function of the scheduled merge tree — `(D, h2)` does not decide it — and its Part 4 obligation 3 records that no schedule type carries that tree, so **R3 joins R4 as a rule to watch**, for the dimensions recorded beside it rather than for its statement. A second paragraph states what the corrections do and do not change for the probe's input: the row count and (a)'s answer are unchanged; the set is four rules plus one conditional; and R2 and R3 are one rule object at the admission layer (Part 3 eliminated the per-shape split) while remaining two term rewrites at the probe's, because the probe's specification records dimensions *beside* the rules and does not encode them.
- **Four further sentences the merged records falsify, corrected with them.** The record's Outcome and axis 1 both said prerequisite (1) of ADR 0095's reopening condition "has no owner anywhere in the graph"; it is owned, `done`, and delivered by the rule-object record, and both places now say so while recording that **the condition still has not fired** because prerequisite (2) is open — plus that record's finding that obligation 1 refuses independently of both permissions. Axis 1's outcome line and the four-outcome roll-up's axis-1 row carried `todo` for a ticket that is `done`. The closing "five-rule table is a Proposal" bullet said R3's bound is underived; it is derived, R4's is not, and the bullet now also names the five refusing admission obligations as the table's real incompleteness. A Traceability bullet records the two refuting records, their bases, and why `depends_on` is **not** extended to the rule-object record — it lists this record in its own, and asserting both edges would state a cycle.
- **The probe deferral's trigger check log carries a dated line**, because the corrections change what its 2026-08-06 entry reads: five rules become four plus one conditional, (b)'s reading narrows and gains R3 as a rule to watch, and "R3's bound is underived" is retired while R4's stands. **Verdict unchanged: not fired.** The entry also corrects a reproducing command in the line above it — `grep -rn 'RewriteRuleIdentity::new' crates/ --include='*.rs' | grep -v test` returns 22 lines at this base rather than the four it claims, because `grep -v` filters lines and the test identities carry no `test` on the line — and replaces it with `grep -rn 'RewriteRuleIdentity::new("tiler' crates/ --include='*.rs'`, which returns exactly the four production identities by name.

**Checks.** `tkt lint` after each ticket edit; `git diff --check` clean; 96 local link targets across the three touched files resolved with zero missing, and the checker was watched reporting `MISSING` on a fabricated target. **No cargo gate was run and none is owed**: the diff touches `docs/research/program-planning/flash-class-capability-set.md` and two files under `tickets/`, and nothing under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` — verified with `git diff --name-only` against the branch base.

**Not done, deliberately.** No bound re-derived, no `docs/research/numerics/**` or `docs/decisions/**` file edited, no permission admitted, and the probe left `deferred`. The table's five rows are unchanged in count, so the earlier log entry's `grep -c '^| R[0-9]'` control still returns `5`.
