---
id: refresh-the-l2-derivation-operation-family-standing
title: Refresh the L2 derivation's operation-family standing against the current support matrix
status: done
priority: p2
dependencies: []
related: [refresh-the-l1-operation-family-standing]
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
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

## Outcome — 2026-08-06

**Fact.** [The L2 derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) is the only file edited besides this ticket and one new one; no `crates/` path is touched, so the workspace gate is untouched and the latest green gate carries.

**Every roadmap cell was read in full before its bound was written**, not copied from L1's table: the contraction, activation, normalization, softmax, structural, concatenate, slice, `Select`, general-transcendental, and cast-and-convert rows of [the family-state table](../docs/roadmap.md#family-state-and-reconsideration-triggers). L1's ten-row bounds survived re-verification with one exception, recorded below.

### Each site, old to new

| Site | Old | New |
| --- | --- | --- |
| *Status* paragraph | "Every family it names sits where the support matrix already places it — at R1 or R2 except the residual addition and the attention scale, which were already at R6" | Dated **Correction** quoting that clause; the surviving clause "this record moves no row" kept as the paragraph's own point, with the movement attributed to tickets elsewhere and the "not delivered support for this workload" half stated |
| *Disposition*/*Rung today* column note | "*Rung* is the family's current position on the support matrix, restated rather than changed" | Same, plus the re-read date, the statement that each moved cell now carries its bound, and a pointer to the quoting correction; *Disposition* explicitly marked as a claim about what a family **is**, which nothing here moves |
| Tensor contraction *Rung* | R1 | **R6** whole-program, R5 met, R7 bounded to two prototype rows, `tiler::strict-tensor-contraction-f32@1` under ADR 0087, all three structures admitted as structure *values*. Bound: `direct` not `tiled`; six shapes reach a selected plan, exactly one dispatched (`[1024, 1024]`, decode extent); a fused region reaches no plan |
| Softmax *Rung* | "R2 for its constituent reductions and R2 for `Exp`" | **R5**, `tiler::softmax-f32@1` with its four pinned decisions. Bound: no registered `IndexRealizationLaw` (thirteen laws, none this key), so the request boundary refuses under `operation-set`. General `Exp` half kept |
| RMS normalization *Rung* | R2 | **R5**, `tiler::rms-norm-f32@1`. Bound: compiles end to end and bit-agrees, but that dispatch evidence is the structured-kernel interpreter's; no compiler-derived region has been through a backend emission |
| SiLU *Rung* | R2 | **R6**, `tiler::silu-f32@1`. Bound: offline translation and linking on Xcode 27.0 / Metal `32023.921`, excluded by name from the compile-profile authority ledger under ADR 0086 item 4; R7 unmet |
| `Reindex` *Rung* | R2 | **R6**, `tiler::reindex-f32@1`. Bound: same toolchain row; R7 unmet **and unowned**; three bit-compared `compile()` programs; `structural-operand` still refuses a computed operand |
| `Broadcast` *Rung* | R2 | **R6**, `tiler::broadcast-f32@1`, same bound; one of the three programs is this record's own `[1024]`-against-`[T, 1024]` broadcast |
| GQA repetition | decision-dependent disposition, no rung | *Resolved to the free branch* by ADR 0087; the conditional derivation kept in tense as the evidence the decision was weighed against |
| KV append *Rung* | `absent` | "absent when L2 looked; `tiler::concatenate-f32@1` is **R5** today", with a note explaining why this cell no longer keeps a convention its neighbours dropped |
| Slice/concatenate paragraph | "Neither appears as a row on the support matrix, which means neither is even at R2" | Dated **Correction** quoting it: `Concatenate` **R5**, `Slice` **R4** literal-offset with R5 awaiting a fusion role and the strided and symbolic forms at **R1**; bound is that nothing lowers, fuses, or emits either |
| Closing **Inference** | "Nothing moved. Every family this workload needs remains at R1 or R2 except…" | Split — see below |

### The line-189 split

"Nothing moved" is kept as the paragraph's first sentence and defended: it is a claim about what **L2 itself** delivered, and no registration, admission, lowering, or measurement in the corpus is attributable to this rung. The clause after it was a claim about the **corpus**, and a dated correction quotes it, states that five family groups have moved, and points at the quoting correction for each bound. A third paragraph then states what the corrected standing does not license — one weight shape dispatched at the decode extent, nothing composed compiling or running, no rung above L3 delivering any part of its named capability. The paragraph is not deleted and neither half is asserted without the other.

### Beyond the nine named sites, and why

Three further groups were corrected because leaving them would have created the same self-contradiction this ticket exists to remove.

- **The keyed-family question**, asserted open at four sites (the shape-class count, the non-`[M, K] × [K, N]` derivation, the *does not decide* bullet, and L3's activation trigger). ADR 0087 settled it on 2026-07-31 and cites this record as evidence; correcting only the GQA row would have left the record answering a question it elsewhere says is reserved. Each is corrected in tense and none is deleted. ADR 0087 settles the first of Q-SEM-015's three choices only; the multi-operand bullet is untouched and still reserved.
- **The *What each family owes* table's absence findings.** Six cells asserted an absent lowering capability, fusion role, or structured-kernel construct for the contraction, softmax, normalization, and SiLU; all six are discharged by the same landings that moved the *Rung today* column, and a dated correction beneath the table quotes each and states its bound. The gather's and the predicated selection's cells stand verbatim and are named as the two rows for which nothing landed.
- **Two filed-ticket outcomes** that asserted standing rather than filing: "two families with normative semantics, no key, and no delivery owner" and "two candidate mechanisms are both absent". Both now carry a delivery note.

### What survived, checked rather than assumed

Line 77's BF16→F32 ingestion recommendation (cast-and-convert still R2, no `Cast` key; ADRs 0091 and 0102 fix the family shape and register nothing), line 65's indirect gather at R1 with no row of its own, and line 128's `Select` at R1 with its trigger explicitly recorded as *not* fired. Each is restated as unchanged in *The Rung column restated* so it reads as rechecked rather than unexamined. The *Disposition* column and the derivation-versus-trace argument are untouched.

### Two divergences found and not silently absorbed

**Fact — the roadmap's softmax cell is stale where this record now is not.** It names two remaining prerequisites, a governed maximum scalar key and a multi-reader handed value; both tickets are `done` and `tiler.scalar::maximum-f32@1` is among the standard scalar keys at `crates/tiler-ir/src/index/scalar.rs`. The L2 cell states the accurate bound — the law's own registration is what is left — and says the stale clause is the roadmap's and outside this record's scopes. **The rung is unaffected**: no `IndexRealizationLaw` is registered for `tiler::softmax-f32@1` (thirteen laws at `crates/tiler-ir/src/semantic/registry.rs`, none this key), so R5 and the refusal both stand.

**Fact — "only the sum has a registered key" needed disambiguating rather than negating.** It stays true of *semantic operation families* and is false of the governed *scalar* profile, which now carries `maximum`, `exp`, and `divide` beside `add` — four of the composition's five constituents, only the subtraction absent. The correction says which registry each reading is about and shows the derivation surviving on a moved ground: the four constituents lack an *identity* rather than lacking anything that can compute them.

### One defect found, filed rather than absorbed

**Fact.** Every source claim in the *Extent classes* section's public-index-boundary paragraph has gone stale — `extent_sources` is `pub` at `crates/tiler-ir/src/index/model.rs:285`, the two named accessors do not exist under those names, no `#[allow(dead_code)]` marks them, and the quoted doc comment is nowhere in `crates/` — because [`promote-the-symbolic-index-profile-to-a-public-boundary`](promote-the-symbolic-index-profile-to-a-public-boundary.md) is `done`. Correcting it needs a full read of `crates/tiler-ir/src/index/` rather than the grep that found it, so it is filed as [`refresh-the-l2-derivation-s-symbolic-index-profile-source-claims`](refresh-the-l2-derivation-s-symbolic-index-profile-source-claims.md) and the record carries a **Known stale** note naming that ticket and each clause.

### L1's forward reference

**Fact.** L1's *What remains open* names this ticket as the owner of L2's superseded standing. That standing is now corrected, so the reference is dischargeable; L1 lives under `research/program-planning`, which this ticket does not hold, so no edit to L1 was made and the coordinator owns whether L1's sentence is rewritten or left as the pointer it is.

### Checks

`tkt lint`, `git diff --check`, and `tkt guard` against the true base. No `crates/`, `prototypes/`, `Cargo.*`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` path is touched, so the workspace gate is untouched by this delta.
