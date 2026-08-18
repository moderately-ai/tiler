---
id: correct-the-b1d-prefill-logits-figures
title: Correct the B1-d prefill logits figures the off-by-4096 seeded
status: in-progress
priority: p2
dependencies: []
related: [project-only-the-final-position-logits, admit-a-position-selecting-slice-for-the-rotary-table, scope-the-sequence-extending-tensor-family, design-model-level-qualification-and-optimization]
scopes: [research/program-planning, research/shapes, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, documentation, program-planning, residency]
claimed_from: todo
assignee: worker-b1d-figures
lease_expires_at: 1787062049
---
## User-visible outcome

The B1-d prefill logits figure reads the same everywhere, and it is the arithmetic's answer. A reader who takes `4,978,634,752` or `10,895,486,984` from any live record today is taking a number that is low by exactly 4,096 bytes and that no reproduction of `8192 * 151936 * 4` will confirm.

## Why this exists

**Fact — the corrected arithmetic.** At B1-d prefill `T = 8,192` with vocabulary 151,936 in F32: full-T logits are `8192 * 151936 * 4 = 4,978,638,848` bytes, one position is `151936 * 4 = 607,744`, and the saving is `4,978,031,104` (≈4.6361 GiB). The all-positions D-B model-level peak under L6's other three columns (`2,384,199,680 + 1,879,048,192 + 1,653,604,360`) is `10,895,491,080` B, and the D-A unfused peak is `28,074,311,688` B. The final-position D-B peak `5,917,459,976` B is **unchanged**, because its logits cell was already the correct one-position size. Reproduce with `python3 -c "print(8192*151936*4, 151936*4)"`.

**Fact — the population, enumerated at base `075d2d447b89d8f9b96fe6baa90157334a4359f6`.** `grep -rln "4,978,634,752\|4978634752\|10,895,486,984" tickets/ docs/ spikes/` returns ten files. Three are dated `docs/research/documentation/ticket-audit-2026-08-10/` records and one is [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md), which already carries its own dated correction quoting the retired figures; by this corpus's convention a dated record that quotes retired text stays as written, so those four are **not** in scope. The seven live sites, each with a searchable anchor:

- `docs/research/program-planning/complete-model-ingestion-and-execution.md` — the origin. Anchor `the difference at B1-d prefill is` for the inline figures, plus the two residency-table rows anchored `D-A unfused` and `B1-d prefill, D-B`.
- `docs/roadmap.md` — the Sub-tensor selection row's trigger cell, anchor `A prefill pass that needs only the final position's logits`, which repeats both the stale byte figure and the stale D-B peak.
- `docs/research/shapes/sequence-extending-tensor-family.md` — anchor `family now, with a stated trigger`.
- `docs/research/program-planning/model-level-qualification.md` — anchor `L6 supplies the figures and labels every one an Inference`, which also carries the D-A `28,074,307,592`.
- `tickets/design-model-level-qualification-and-optimization.md` — the same sentence, same anchor.
- `tickets/admit-a-position-selecting-slice-for-the-rotary-table.md` — anchor `bytes against 607,744. [Rung L5]`.
- `tickets/scope-the-sequence-extending-tensor-family.md` — anchor `F32 bytes at B1-d against 607,744 for one position`.

Re-enumerate at your own base rather than trusting this list.

**Fact — the C1 rows are correct and must not be touched.** C1 prefill's `6,077,440` is `10 * 607,744` and C1 decode's `607,744` is one position. Only the two B1-d prefill rows and the sentences quoting them are wrong.

**Fact — the saving already has a compiled guard, and it does not reach the prose.** `crates/tiler-reference/tests/language_model_boundaries.rs`, anchor `the_benchmark_prefill_saving_follows_from_the_two_declared_shapes`, derives all three corrected figures from the two programs' own declared output shapes and asserts them, so the corrected numbers cannot drift again without a red test. This ticket is the sweep that the guard does not cover.

## Required content

- Correct each live site to the recomputed figure, and where a record states the D-A or D-B peak, correct that too.
- Attach a dated correction where the corpus convention calls for one, rather than silently substituting a number inside a sentence labelled **Fact** with a source read.
- Do not edit the four dated audit and correction records named above.
- Recompute rather than copy from this ticket; a relayed figure is how the original error reached seven places.

## Closes when

Every live site states the recomputed figures; `grep -rn "4,978,634,752\|10,895,486,984\|28,074,307,592" docs/ tickets/` returns only the dated correction and audit records that quote them deliberately; and `make citations` and `tkt lint` pass.
