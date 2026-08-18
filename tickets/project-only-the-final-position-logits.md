---
id: project-only-the-final-position-logits
title: Project only the final position's logits
status: done
priority: p2
dependencies: [assemble-the-embedding-and-vocabulary-projection-programs, reclassify-language-model-work-as-a-conformance-track, admit-the-sub-tensor-selection-family]
related: [design-model-ingestion-and-complete-execution, own-operation-family-support-matrix, design-model-level-qualification-and-optimization, admit-a-position-selecting-slice-for-the-rotary-table]
scopes: [implementation/ir, implementation/reference, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, slice, logits, residency, language-model, class-conformance-fixture]
---
## User-visible outcome

A prefill pass that only needs its last token's logits produces `[1, 151936]` instead of `[T, 151936]`, which at the benchmark row's long end removes more peak residency than the model's entire weight set occupies.

## Why this exists

**Fact.** [L1](../docs/research/program-planning/first-metal-lm-workload.md) records that the pinned reference's `logits_to_keep=0` becomes `slice(0, None)` — the value that reads like "keep none" is the special case that keeps all — which is why the conformance row can retain every prefill logit.

**Inference, from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).** The logits contract therefore has two modes and they are not interchangeable. The conformance row needs every position; a benchmark row needs only the last. At B1-d prefill the difference is **4,978,634,752 bytes against 607,744** — a saving of **4,978,027,008 bytes (4.6361 GiB)**, larger than the 2,384,199,680-byte F32 weight budget. Under the D-B decomposition it moves the model-level prefill peak from 10,895,486,984 B to 5,917,459,976 B.

**Correction — 2026-08-10.** The absolute full-T logits figure and saving above inherit an off-by-4096 from L6 / the support-matrix trigger cell / the sequence-extending record. Recompute at B1-d `T = 8,192`: full-T is `8192 × 151936 × 4 = 4,978,638,848` bytes, one position is `151936 × 4 = 607,744`, saving is `4,978,031,104` bytes (still ≈4.6361 GiB). The all-positions D-B peak under the same other columns is `10,895,491,080` B; the final-position peak `5,917,459,976` B is unchanged because its logits cell was already the correct one-position size. The qualitative claim that the saving exceeds the 2,384,199,680-byte F32 weight budget still holds. Reproduce: `8192 * 151936 * 4` and `151936 * 4`.

**This fires the sub-tensor-selection row's *first* trigger**, the one [the support matrix](../docs/roadmap.md#operation-family-support-matrix) already states as "a prefill pass that needs only the final position's logits", and it fires it on residency rather than on convenience. The row's third trigger, the position-identity one, is [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md)'s only — that ticket stays `related`, not a dependency. The family this ticket depends on is [`admit-the-sub-tensor-selection-family`](admit-the-sub-tensor-selection-family.md) / `tiler::slice-f32@1`, already listed in frontmatter `dependencies`.

**Correction — 2026-08-10.** An earlier closing clause attached "and is the family this depends on" to the rotary third-trigger sentence; that collapsed consumer trigger, family owner, and dependency edge. Frontmatter was always correct: rotary is `related`; the sub-tensor family is the dependency.

## Required content

- Selecting row `T − 1` of a `[T, 1024]` residual stream is injective and not surjective, so it is outside `tiler::reindex-f32@1`'s admitted forms and outside `Broadcast`; the family that admits it is the dependency, and this ticket does not invent a second one.
- The two modes are two program shapes, not a runtime choice: a program that could return either would be two programs presented as one.
- The conformance mode stays, unchanged and default, because the oracle needs it.

## Exact-base Fact audit — 2026-08-18 at base `075d2d447b89d8f9b96fe6baa90157334a4359f6`

Every Fact above was re-read at this base before any edit.

- **Verified — L1's `logits_to_keep=0`.** `docs/research/program-planning/first-metal-lm-workload.md`, anchor `the value that reads like "keep none" is the special case that keeps all`. The ticket relays it exactly.
- **Verified as a relay, and the relayed source is still wrong — the L6 two-mode inference.** `docs/research/program-planning/complete-model-ingestion-and-execution.md`, anchor `the difference at B1-d prefill is`, still reads `4,978,634,752 bytes against 607,744` with a saving of `4,978,027,008`, and its residency table still carries `10,895,486,984` for the all-positions D-B peak. So the **Correction — 2026-08-10** above is live rather than discharged, and its arithmetic re-checks: `8192 * 151936 * 4 = 4,978,638,848`, `151936 * 4 = 607,744`, saving `4,978,031,104`, all-positions D-B peak `10,895,491,080`, final-position peak `5,917,459,976` unchanged, saving still larger than the `2,384,199,680`-byte F32 weight budget (whose own components sum exactly: `1,761,865,728 + 622,329,856 + 4,096`). Each stale figure is low by exactly 4,096.
- **Verified — the support matrix's first trigger, and imprecise in the same cell.** `docs/roadmap.md`, Sub-tensor selection row, anchor `A prefill pass that needs only the final position's logits`, states the trigger in the ticket's words and confirms that the rotary ticket carries the third, position-identity trigger. That same cell repeats the stale `4,978,634,752` and `10,895,486,984`.
- **Verified — the graph correction.** Frontmatter `dependencies` contains `admit-the-sub-tensor-selection-family`; `admit-a-position-selecting-slice-for-the-rotary-table` is in `related`, as the 2026-08-10 correction says.
- **Verified — the family admits the selection and this ticket needs no new one.** `crates/tiler-ir/src/semantic/slice.rs` registers `tiler::slice-f32@1` (anchor `The complete normative definition of`) over a total per-axis selection whose `window` relation takes a literal offset; `crates/tiler-reference/src/standard.rs` registers `SliceF32Reference` for it. The module's own doc states the Required-content claim directly: a non-surjective map `is a slice, a different family` from `tiler::reindex-f32@1`.
- **Imprecise, and repaired below — the "Closes when" clause naming the pinned reference's own last-position logits.**

**Repair — the close condition's reference-evaluation clause.** The pinned reference's *own* last-position logits are not reachable by reference evaluation, for the boundary [`assemble-the-embedding-and-vocabulary-projection-programs`](assemble-the-embedding-and-vocabulary-projection-programs.md) already recorded: `crates/tiler-reference/src/lib.rs`, anchor `MAX_REFERENCE_TENSOR_ELEMENTS`, bounds one tensor at `16 * 1024 * 1024` elements, and C1's `[151936, 1024]` matrix holds 155,582,464. The retained fixture is no way round it either — `spikes/program-planning/qwen3-conformance-fixture/results/2026-08-01-.../top32.tsv` keeps the top thirty-two logits per position, not the vocabulary. The reachable and delivered evidence is therefore: the exact C1 and B1-d final-position programs are constructed and shape-inspected without materializing anything, and at the extent-independent analogue the final-position program's result is bit-identical both to an independent literal oracle and to the last row of the all-positions program, which is the mode relation the claim rests on. Read the close condition as amended that way.

**Out-of-scope defect, filed rather than fixed here.** `grep -rln "4,978,634,752\|4978634752\|10,895,486,984" tickets/ docs/ spikes/` returns ten files at this base; four are dated audit or correction records that quote the retired figures deliberately, and seven live sites across `research/program-planning`, `research/shapes`, `contracts/navigation`, and `project/tickets` still state them as current. That population is outside this ticket's scopes and is owned by [`correct-the-b1d-prefill-logits-figures`](correct-the-b1d-prefill-logits-figures.md).

## Implementation evidence — 2026-08-18

`crates/tiler-reference/tests/language_model_boundaries.rs` gains the final-position mode beside the unchanged all-positions one. `build_final_position_projection_program` declares P3's same three inputs — `h [T, 1024]`, `w_norm [1024]`, `W_embed [151936, 1024]` — and writes four occurrences: `tiler::slice-f32@1`, `tiler::reindex-f32@1`, `tiler::rms-norm-f32@1`, `tiler::strict-tensor-contraction-f32@1`, for three inputs, four operations, seven values, one output, and derived buffers actual 7. The selection is `final_position_selection`, one entry per operand axis in axis order: a one-coordinate window at `T - 1` on the position axis and the hidden axis whole. Rank is preserved, so the result is `[1, 1024]` and no `remove-unit-axis` reindex follows — the extent-one axis the selection leaves behind is the position axis the logits declare. Nothing in the construction names 151,936 or 8,192; both are fixture constants read through `t`, `vocabulary`, and `hidden`.

**The correctness argument.** The selection is written *first*, so the `[T, 151936]` logits are never formed. That is sound because both later operations are row-independent: `tiler::rms-norm-f32@1` reduces over the hidden axis, so each position's result depends on that position's row alone, and `td,od->to` sums over `d` for each `(t, o)` pair independently. The widened weight is the same vector in both modes — replicated across `T` positions there, across the one selected position here — so the selected row meets identical operands either way. Because the selection reduces the stream to one position, the explicit weight widening in this mode is the decode-shaped `insert-unit-axis` reindex at every `T`, which is a consequence of the selection rather than a second mode.

**The two shapes, at both rows.** Anchor `the_two_logits_modes_are_distinguished_by_their_declared_output_shape` builds both modes at `T = 10` and at B1-d prefill `T = 8,192` and asserts identical declared input keys, shapes, and types against different declared results: `[T, 151936]` for the conformance mode, `[1, 151936]` for the final-position mode. There is no flag, no fourth input, and no attribute a caller sets — a consumer reads which program it holds off the result extent without running it.

**The residency figure, derived rather than written.** Anchor `the_benchmark_prefill_saving_follows_from_the_two_declared_shapes` reads both byte counts off the programs' own declared output shapes and asserts `4,978,638,848`, `607,744`, a saving of `4,978,031,104`, and that the saving exceeds the `2,384,199,680`-byte F32 weight budget. The corrected arithmetic is now a compiled fact that cannot silently revert to the stale figure.

**The evaluation.** Anchor `the_final_position_mode_returns_exactly_the_all_positions_mode_s_last_row` evaluates both modes at `V = 3`, `H = 2` over one shared binding set, at `T = 3` with the worked row last and at `T = 2` with the worked row first — so a program reading a fixed position fails one of the two whichever position it read. Each case asserts the independent literal oracle first (the retained RMS worked example `[3, 4]` with weight `[1, 2]` is `[0x3f593923, 0x4010d0c2]`, projected through rows `e0`, `e1`, `e0 + e1` to those two values and their one strict F32 addition `0x40471f0b`; an all-zero row stays positive zero), then the mode relation — the final-position result equals the all-positions result's last row bit for bit — then that the first and last rows differ, so neither comparison is vacuous.

**The unsupported case, stated by name.** Anchor `the_final_position_mode_is_not_statable_at_a_single_position`: at `T = 1` a one-coordinate window covers its axis, which *is* the `whole-axis` relation, so the occurrence is refused under `slice.selection.window-is-whole-axis` and the mode has no program. Nothing is lost — the decode program already declares `[1, 151936]`, which the same test asserts — and the refusal is what keeps one map from having two spellings. The mode distinction is a prefill one.

**Four subject perturbations, each restored before the next, each followed by a green run.** Run with `cargo nextest run -p tiler-reference -E 'binary(language_model_boundaries)'`.

- **Selection offset dropped** (`static_window(0, ...)` instead of `t - 1`): the evaluation test alone fails, `left: [0, 0, 0]` against `right: [1062811939, 1074843842, 1078402827]` — the literal oracle, not a cross-comparison, is what catches it.
- **Selection moved after the projection** (project all `T` positions, then select the final logits row): the *declared output shape, the operation count, the value count, the derived buffers, and the evaluated values are all unchanged*, and only the operation-key sequence fails — `[broadcast-f32@2, rms-norm-f32@1, strict-tensor-contraction-f32@1, slice-f32@1]` against the expected `[slice-f32@1, reindex-f32@1, rms-norm-f32@1, strict-tensor-contraction-f32@1]`. This is the load-bearing finding about the evidence: the byte assertion guards the *declared shape*, and only the occurrence ordering guards the *residency* — a program that forms the wide logits and then selects one row declares exactly the same result.
- **Weight widened against `T` rather than the selected position:** construction refuses under `rms-norm.f32.weight-shape`, message `the binary32 RMS normalization admits no implicit broadcasting; the weight operand must already have the normalized value's shape, which a tiler::broadcast-f32@2 occurrence produces`, reddening three tests.
- **Selection silently skipped at `T = 1`** (return the unsliced stream): the single-position test alone fails, `selecting the only position restricts no axis` — the refusal is reached by the composition rather than only by the family's own tests.

**Commands, all green after restoration.** `cargo fmt --all -- --check`; `cargo check -p tiler-reference --all-targets`; `cargo clippy -p tiler-reference --all-targets -- -D warnings`; `cargo nextest run -p tiler-reference` (324 tests run, 324 passed, 2 skipped — the four new tests above the package's prior 320); `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-reference`; `cargo test -p tiler-reference --doc`.

**What this does not claim.** No new operation family, key, attribute, identity, schema, diagnostic code, or public API. No compiler, lowering, artifact, runtime, or device support — a program stating `tiler::slice-f32@1` is still refused at the request boundary under `operation-set`, which the family's own support-matrix row records. No evaluator limit is widened and no C1 tensor is materialized. A *symbolic* `T` is not delivered: the final position of a sourced sequence length needs an offset the shape environment can prove equals `T - 1`, and these programs use literal shapes throughout.

## Closes when

A prefill program of the final-position shape verifies, reference-evaluates against a last-position oracle — amended under the repair above from "the pinned reference's own last-position logits", which the evaluator's tensor bound puts out of reach — and the two modes are distinguishable by their declared output shape rather than by a flag.
