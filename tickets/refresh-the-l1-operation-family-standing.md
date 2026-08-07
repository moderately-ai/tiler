---
id: refresh-the-l1-operation-family-standing
title: Refresh the L1 workload profile's operation-family standing against the current support matrix
status: done
priority: p2
dependencies: []
related: [audit-the-l1-workload-records-evidence-classes]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
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

## Outcome — 2026-08-06

Delivered on `tkt/refresh-the-l1-operation-family-standing` from base `428d201d`. Scopes: `research/program-planning` (exclusive) and `project/tickets` (shared) — the shared scope was **absent when the work started** and was added with `tkt set --add-shared-scope project/tickets` before any ticket file was edited. No `crates/` path is touched, so the workspace gate is untouched and the latest green gate carries.

### The three sites, old claim to new

1. **The status line.** It read "Nothing here authorizes implementation, and no rung of the ladder is built." The quoted clause is now carried in a dated **Corrected 2026-08-06** paragraph beneath it, which records that L3's named capability has been delivered — a `td,od->to` toy on 2026-08-02 and this workload's own `w_decode_kv` (`1 x 1024 x 1024`) on 2026-08-05, both on one host row under the `direct` realization — while [the roadmap's ladder](../docs/roadmap.md#the-ladder) deliberately holds L3's `Maturity today` cell by Tom's 2026-08-06 decision. What survives is the narrower clause: every rung above L3 still owes its named composed capability entirely, and no moved family row is delivered support for this workload's shapes.

2. **The closing Inference of *Operation and shape surface handed to L2*.** The trace enumeration is kept verbatim as an **Inference**; the four standing clauses and the closing "Every family this workload needs is at R1 or R2 today; nothing in the ladder is partially built" are **quoted inside a dated Correction rather than left in place**, following this record's own 2026-08-02 BF16 convention. The Correction carries a ten-row table stating each family's rung *and its bound*:

   | Family | Rung | Bound stated |
   | --- | --- | --- |
   | Tensor contraction | **R6** whole-program, R5 met, R7 bounded | `direct` and not the `tiled` realization L3 selects on cost; two prototype execution rows on one host row; the six L3 correctness cells cover five of the six weight shapes exactly and `[151936, 1024]` only as its first 8,192 rows; **all six reach a selected physical plan (compile-phase) and exactly one — `[1024, 1024]` at the decode extent `M = 1` — has dispatched through the accepted route**; no prefill extent has executed; a *fused* contraction region reaches no plan |
   | SiLU | **R6** | offline translation and linking of one golden on the Xcode 27.0 / Metal `32023.921` row, which ADR 0086 item 4 excludes from the compile-profile authority ledger by name; R7 unmet — nothing dispatched, no compiler-derived elementary region through `emit` |
   | RMS normalization | **R5** | compiles end to end and bit-agrees with `tiler-reference`, but that dispatch is the structured-kernel *interpreter's*; no backend emission of a compiler-derived region |
   | Softmax | **R5** | no registered `IndexRealizationLaw`, so the request boundary still refuses under `operation-set`; two named prerequisites remain |
   | Reindex, broadcast, transpose | **R6** | same toolchain-row bound; R7 unmet **and unowned**; `structural-operand` still refuses an occurrence over a computed value |
   | Concatenate | **R5** F32 | no index-access lowering, no backend emission |
   | Slice | **R4** literal-offset; strided and symbolic **R1** | nothing lowers, fuses, or emits it; this trace needs none |
   | GQA 8→16 repetition | **not a family** | free under the one general keyed contraction — present in the query operand and result, absent from the key operand and the contracted set of `grtd,gsd->grts` |
   | Views, bit-preserving copies | **R2** | unmoved, no key and no contract |
   | RoPE `cos`/`sin`, general `Exp`/`Rsqrt`/`Sqrt`/`Log`/`Sin`/`Sigmoid`/`Gelu` | **R2** | unmoved; `rsqrt` reaches a backend only as the normalization's subordinate; the rotary composition is a checked program, the tables are host inputs |

3. **The last bullet of *What remains open*.** Its heading read "Every operation family this workload needs is at R1 or R2." Rewritten to the moved standing with a dated Correction quoting the old heading, restating what remains open in a checkable form (one of six weight shapes dispatched, nothing composed executes, the transcendental general keys and standalone reducers still R2), and naming the out-of-scope L2 ticket.

### (a) The fusion-legality recheck — premise survives, conclusion does not

The clause "A general mean-reduction for RMSNorm and a max-and-sum reduction for softmax … resolve to no fusion legality at all" is **false as a conclusion**. Read at the construction site, `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs:334`–`:352` maps `tiler::rms-norm-f32@1` to `FusionOperationRole::PrologueCarryingOrderedReduction` and `tiler::softmax-f32@1` to `FusionOperationRole::ExtremumShiftedOrderedReduction`, so a cover region holding either derives legality instead of failing closed to `Unknown` with `unsupported-operation-capability`; the contraction took the *same* prologue-carrying role at `:465`–`:468` on 2026-08-06 rather than a seventh variant. **Two further errors beyond the rung**, both now recorded: the normalization's fold is a strict ordered sum over an elementwise squaring prologue with the division by the static axis extent inside its law's `Rsqrt(a / N + eps)` chain, so no *mean* reducer is needed or registered; and the softmax's first fold is the NaN-propagating `Maximum` admitted as an embedded form under an identity-less empty-domain **refusal** rather than as a seeded reduction. **The premise survives:** no standalone reducer beyond the strict serial sum is registered — product, a standalone extrema reduction, a non-identity seed, and the variadic question are all still R2, held by `no_general_exponential_maximum_reduction_or_log_softmax_key_is_registered`. Legality comes from the family's registered role, not from the reducer's registration.

### (b) The absence-check rerun

Ran the check as the roadmap currently spells it, at base `428d201d`:

```sh
grep -rniE '\b(exp|sin|cos|tanh|sqrt|rsqrt|gelu|erf|sigmoid)\b' crates/ --include='*.rs' | wc -l   # 303
grep -rlniE '\b(exp|sin|cos|tanh|sqrt|rsqrt|gelu|erf|sigmoid)\b' crates/ --include='*.rs' | wc -l  # 45
```

**303 lines across 45 files, not "no output at all".** Thirteen of the forty-five are operation-bearing: the semantic, reference, and test modules of `silu`, `rms_norm`, and `softmax`, plus `crates/tiler-ir/src/kernel/model.rs` and `crates/tiler-metal/src/emit.rs`. The roadmap's own comment block already records that the emptiness claim was false when L1 wrote it (`762ba34`) and has been corrected twice since; the check's instruction is now to read the hits rather than count them, and absence rests on check 3's registry enumeration. L1's remark is corrected in full.

### (c) What is still R2 and survives as stated

Verified against the roadmap rows and left untouched: the reductions-beyond-strict-sum row (product, a standalone extrema reduction, a non-identity seed, the variadic multi-input question) is **R2**; the cast-and-convert row is **R2** over one realized construct that is not a dtype conversion (`ConvertOp::CanonicalizeF32Nan`), and [ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md), accepted 2026-08-06, fixes the family *shape* while registering nothing. **L1's BF16-to-F32 ingestion paragraph and its 2026-08-02 Correction therefore stand unedited** — no `Cast` key exists, so whether the conversion is a Tiler operation or a host ingestion step still changes whether that row is triggered, and that remains L6's question.

### Filed, and checked-but-not-owed

- **Filed:** [`refresh-the-l2-derivation-operation-family-standing`](refresh-the-l2-derivation-operation-family-standing.md) (`todo`, p2, scope `research/shapes`), naming every stale site by line — the status line at 19, the *Rung* cells at 61–67, the decision-dependent GQA disposition at 73, the slice/concatenate "not even at R2" at 118, and the closing "Nothing moved" at 189 — with what must survive (the BF16 ingestion recommendation at 77, the gather at 65, `Select` at 128, and the whole *Disposition* column). L2 contradicts itself today: line 76 was half-corrected on 2026-08-04 and line 118 was not.
- **Checked and owes nothing: L8.** [`model-level-qualification.md`](../docs/research/program-planning/model-level-qualification.md) was read for restatements and carries none. Its lines 19 and 260 claim only that *it* moves no support-matrix row and that no family moved a rung on *its* evidence — true claims about what L8 delivers, not restatements of L1's standing.

### Checks

- `tkt lint` — clean.
- `git diff --check` — clean.
- `tkt guard tkt/refresh-the-l1-operation-family-standing --format json` — no scope escape.
- **No `crates/` path is touched**, so the workspace gate is untouched: the diff is confined to `docs/research/program-planning/first-metal-lm-workload.md` and two files under `tickets/`.
