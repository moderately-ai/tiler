---
id: refresh-the-l2-derivation-operation-family-standing
title: Refresh the L2 derivation's operation-family standing against the current support matrix
status: in-progress
priority: p2
dependencies: []
related: [refresh-the-l1-operation-family-standing]
scopes: [research/shapes]
shared_scopes: []
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-l2-refresh
lease_expires_at: 1786077743
---
## User-visible outcome

`docs/research/shapes/transformer-operation-and-shape-surface.md`'s *Rung* column and its standing prose match the roadmap's current family-state table, with each moved row's **bound** stated rather than only its rung, so nothing downstream of L2 derives from a superseded capability picture.

## Why this is a separate ticket

**Fact.** [`refresh-the-l1-operation-family-standing`](refresh-the-l1-operation-family-standing.md) corrected the same standing in [L1](../docs/research/program-planning/first-metal-lm-workload.md) on 2026-08-06 and could not reach this record: L1 lives under `research/program-planning` and this one under `research/shapes`, which that ticket does not hold. L1's *What remains open* now names this ticket as the owner, so the two records disagree until this lands.

**Fact — L8 was checked in the same pass and owes nothing.** [`model-level-qualification.md`](../docs/research/program-planning/model-level-qualification.md) states only that it *moves* no support-matrix row and that no operation family moved a rung *on its own evidence*, both of which are true claims about what L8 delivers rather than restatements of L1's standing.

## The stale sites, each read in full on 2026-08-06

L2's *Rung* column says it restates the matrix "rather than changed", so every cell below is a restatement that has gone stale rather than an independent claim.

| Line | What it says | What the [family-state table](../docs/roadmap.md#family-state-and-reconsideration-triggers) says |
| --- | --- | --- |
| 19 (Status) | "Every family it names sits where the [support matrix] already places it — at R1 or R2 except the residual addition and the attention scale, which were already at R6" | False for five family groups |
| 61 | Tensor contraction *Rung* cell reads **R1** | **R6** for a whole-program occurrence, R5 met, R7 bounded to two prototype execution rows; `tiler::strict-tensor-contraction-f32@1` registered under [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md). The cell's own "unsettled keyed-family question" is settled |
| 62 | Softmax *Rung* cell reads "R2 for its constituent reductions and R2 for `Exp`" | `tiler::softmax-f32@1` is **R5**. The prose "only the sum has a registered key" is false; the general `Exp` half stands |
| 63 | RMS normalization *Rung* cell reads **R2** | `tiler::rms-norm-f32@1` is **R5** |
| 64 | SiLU *Rung* cell reads **R2** | `tiler::silu-f32@1` is **R6**, bounded to offline translation and linking on one measured toolchain row that is not the compile-profile authority ledger's, with R7 unmet |
| 66, 67 | `Reindex` and `Broadcast` *Rung* cells read **R2** | Both **R6**, on the same toolchain-row bound, with R7 unmet **and unowned** |
| 73 | The GQA 8→16 repetition is "free under a general contraction; an explicit `Broadcast` plus `Reindex` under fixed-arity matmul keys", left decision-dependent | The decision landed: one general keyed family, so the repetition is **free**, in the query operand and the result and in neither the key operand nor the contracted set of `grtd,gsd->grts` |
| 118 | Slice and concatenate "Neither appears as a row on the support matrix, which means neither is even at R2" | `tiler::concatenate-f32@1` is **R5** for the F32 family and `tiler::slice-f32@1` is **R4** for the literal-offset form, R5 awaiting a fusion role, with the strided and symbolic forms at R1. Line 76 was already half-corrected on 2026-08-04 and line 118 was not, so the record contradicts itself today |
| 189 | Closing **Inference**: "Nothing moved. Every family this workload needs remains at R1 or R2 except the residual add and the attention scale" | The first two words are the load-bearing error |

**Fact — what must survive the correction.** Line 77's BF16→F32 ingestion recommendation stands unchanged: the cast-and-convert row is still R2, no `Cast` key exists, and [ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md) fixes the family *shape* while registering nothing. Line 65's indirect gather stands: no gather key is among the registered keys. Line 128's `Select` at R1 stands. Line 34's derivation-versus-trace argument and the whole *Disposition* column are untouched by this — what moved is where each family stands, never whether it is atomic.

## The work

Read L2 in full and every roadmap family cell it names in full; the cells are long and each states its own bound, so a rung number alone is not the claim. Follow L1's own dated-**Correction** convention as applied on 2026-08-06 rather than silently rewriting: quote the stale clause, state what is now true, and state the bound. The honest correction says both that the families moved and that none of the movement is delivered support for this workload — exactly one of the six weight shapes has been dispatched through the accepted route, at the decode extent, and nothing composed from these families compiles or runs.

Line 189's "Nothing moved" needs the most care. It is a claim about what **L2 itself** delivered, and in that reading it is still true and should stay; what is false is the clause after it, which asserts the corpus-wide standing. Separate the two rather than deleting the paragraph.

## Closes when

L2's *Rung* column and its standing prose agree with the roadmap's family-state table, verified by a full read of both, with each moved row's bound stated; and L1's forward reference to this ticket is discharged.
