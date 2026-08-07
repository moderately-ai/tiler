---
id: refresh-the-l1-operation-family-standing
title: Refresh the L1 workload profile's operation-family standing against the current support matrix
status: in-progress
priority: p2
dependencies: []
related: [audit-the-l1-workload-records-evidence-classes]
scopes: [research/program-planning]
shared_scopes: []
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-l1-refresh
lease_expires_at: 1786077002
---
## User-visible outcome

`docs/research/program-planning/first-metal-lm-workload.md`'s statement of where this workload's operation families stand matches the roadmap's current support matrix, so L2 and L8 derive from a true capability picture.

## The finding, from the L1 evidence-class audit

**Fact.** [`audit-the-l1-workload-records-evidence-classes`](audit-the-l1-workload-records-evidence-classes.md) read L1 in full and found its operation-family standing stale in three places. The record says every family this workload needs sits at R1 or R2 with no registered key; the roadmap's family-state table no longer says that for five of them.

| L1's claim | Roadmap's current row |
| --- | --- |
| Contraction "sits at R1 with no registered key" | **R6** for a whole-program contraction occurrence, R5 met for the F32 family, `tiler::strict-tensor-contraction-f32@1` registered under ADR 0087 |
| "Softmax, SiLU, `rsqrt` ... at R2 with no operation, evaluator, or structured-kernel construct" | `tiler::silu-f32@1` **R6**; `tiler::rms-norm-f32@1` **R5**; `tiler::softmax-f32@1` **R5**, each with a registered key, a reference evaluator, and an ADR 0042 accuracy contract |
| "Reindex, broadcast, transpose, slice, concatenate ... structural families at R2 with no registered key" | `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` **R6**; `Concatenate` **R5** for the F32 family; `Slice` **R4** for F32 literal-offset semantics; views and bit-preserving copies stay R2 |

**Fact — what is still true and must survive the correction.** Reductions beyond the strict serial sum are still R2, so L1's claim that RMSNorm's mean-reduction and softmax's max-and-sum resolve to no fusion legality needs rechecking against the `PrologueCarryingOrderedReduction` and `ExtremumShiftedOrderedReduction` roles the two registrations added rather than being deleted. The cast-and-convert row is still R2, so L1's BF16-to-F32 ingestion paragraph stands.

**Fact — three sites carry the falsity.** The status line ("no rung of the ladder is built"), the closing **Inference** of *Operation and shape surface handed to L2* ("Every family this workload needs is at R1 or R2 today; nothing in the ladder is partially built"), and the last bullet of *What remains open* ("Every operation family this workload needs is at R1 or R2"). L1's remark that the roadmap's absence check 1 "returns no output at all" is from the same reading and needs rerunning.

## The work

Read L1 in full and every roadmap family cell it names in full — the cells are long and each states its own bound, so a rung number alone is not the claim. Every one of the moved rows is bounded (one measured toolchain row, prototype execution rows, R7 unmet), and the honest correction states both that the families moved and that the delivered support does not cover this workload's six weight shapes. Follow the record's own dated-**Correction** convention rather than silently rewriting, as its 2026-08-02 BF16 correction does.

Check whether [Transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) and [L8](../docs/research/program-planning/model-level-qualification.md) restate the same standing; if they do, they move with it or get their own tickets.

## Closes when

L1's operation-family standing agrees with the roadmap's family-state table, verified by a full read of both, with each moved row's bound stated rather than only its rung.
