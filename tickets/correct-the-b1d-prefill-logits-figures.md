---
id: correct-the-b1d-prefill-logits-figures
title: Correct the B1-d prefill logits figures the off-by-4096 seeded
status: done
priority: p2
dependencies: []
related: [project-only-the-final-position-logits, admit-a-position-selecting-slice-for-the-rotary-table, scope-the-sequence-extending-tensor-family, design-model-level-qualification-and-optimization]
scopes: [research/program-planning, research/shapes, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, documentation, program-planning, residency]
---
## User-visible outcome

The B1-d prefill logits figure reads the same everywhere, and it is the arithmetic's answer. A reader who takes `4,978,634,752` or `10,895,486,984` from any live record today is taking a number that is low by exactly 4,096 bytes and that no reproduction of `8192 * 151936 * 4` will confirm.

## Why this exists

**Fact — the corrected arithmetic.** At B1-d prefill `T = 8,192` with vocabulary 151,936 in F32: full-T logits are `8192 * 151936 * 4 = 4,978,638,848` bytes, one position is `151936 * 4 = 607,744`, and the saving is `4,978,031,104` (≈4.6361 GiB). The all-positions D-B model-level peak under L6's other three columns (`2,384,199,680 + 1,879,048,192 + 1,653,604,360`) is `10,895,491,080` B, and the D-A unfused peak is `28,074,311,688` B. The final-position D-B peak `5,917,459,976` B is **unchanged**, because its logits cell was already the correct one-position size. Reproduce with `python3 -c "print(8192*151936*4, 151936*4)"`.

**Fact — the population, enumerated at base `075d2d447b89d8f9b96fe6baa90157334a4359f6`.** *Corrected 2026-08-18 by the worker, at base `a9efd708438b20e8454eafdd26b95de8a65fffab`: this read "returns ten files" and "the seven live sites". Both counts were low by one, in two different ways, and the enumerating grep is itself the reason for the second.* The quoted grep returns **eleven** files at the base this Fact names — reproduce with `git grep -l "4,978,634,752\|4978634752\|10,895,486,984" 075d2d447b89d8f9b96fe6baa90157334a4359f6 -- tickets/ docs/ spikes/` — and twelve at `a9efd708`, the extra being this ticket. The stated ten never agreed with the Fact's own breakdown, which enumerates three plus one plus seven. Three are dated `docs/research/documentation/ticket-audit-2026-08-10/` records and one is [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md), which already carries its own dated correction quoting the retired figures; by this corpus's convention a dated record that quotes retired text stays as written, so those four are **not** in scope. The seven live sites that grep finds, each with a searchable anchor:

- `docs/research/program-planning/complete-model-ingestion-and-execution.md` — the origin. Anchor `the difference at B1-d prefill is` for the inline figures, plus the two residency-table rows anchored `D-A unfused` and `B1-d prefill, D-B`.
- `docs/roadmap.md` — the Sub-tensor selection row's trigger cell, anchor `A prefill pass that needs only the final position's logits`, which repeats both the stale byte figure and the stale D-B peak.
- `docs/research/shapes/sequence-extending-tensor-family.md` — anchor `family now, with a stated trigger`.
- `docs/research/program-planning/model-level-qualification.md` — anchor `L6 supplies the figures and labels every one an Inference`, which also carries the D-A `28,074,307,592`.
- `tickets/design-model-level-qualification-and-optimization.md` — the same sentence, same anchor.
- `tickets/admit-a-position-selecting-slice-for-the-rotary-table.md` — anchor `bytes against 607,744. [Rung L5]`.
- `tickets/scope-the-sequence-extending-tensor-family.md` — anchor `F32 bytes at B1-d against 607,744 for one position`.

Re-enumerate at your own base rather than trusting this list.

**Fact — an eighth live site, found by re-enumerating and invisible to the grep above.** `tickets/design-model-ingestion-and-complete-execution.md`, `## Roadmap edits`, anchor `the ticket that owns it` — deliberately not the byte figure itself, which this sweep moves and which would therefore rot the anchor into a false absence — stated the stale **saving** `4,978,027,008`, and stated it as a description of what the roadmap trigger cell says. The enumerating grep matched only the full-T figure and the D-B peak, so no site carrying the saving alone could appear in it; widening it to `grep -rln "4,978,634,752\|4978634752\|10,895,486,984\|28,074,307,592\|4,978,027,008" tickets/ docs/ spikes/` returns **thirteen** files at `a9efd708` against the narrow grep's twelve there, and the single addition is that ticket. It is corrected with a dated note under the same convention as the other seven. Its owning ticket is `done`, so the note explains the repair rather than reopening it.

**Fact — every GiB rendering in L6's residency table is unchanged at four decimals**, so no argument drawn from the 26.1462 → 10.1472 → 5.5111 GiB comparison moves: `28,074,311,688 / 2**30 = 26.14624024` and `10,895,491,080 / 2**30 = 10.14721680`.

**Escalated, not decided — the saving's GiB parenthetical.** The corrected saving is `4,978,031,104 / 2**30 = 4.63615274` GiB, which renders as **4.6362** under round-to-nearest and **4.6361** under truncation; the stale saving rendered as 4.6361 under both, so the off-by-4096 is exactly what moved this figure across the boundary. Three live sites carry `(4.6361 GiB)` — L6's logits contract, the support-matrix trigger cell, and this ticket's own arithmetic Fact — and the excluded 2026-08-10 dated correction states "still ≈4.6361 GiB" for the corrected byte figure. The corpus's own convention is not self-consistent: L6's C1 prefill row renders `2,394,286,488` B as 2.2299 GiB, which is reachable only by rounding (`2.22985306`), while `4.6361` is reachable only by truncation. All three live `4.6361` renderings are therefore **left as written** and flagged here rather than changed, because substituting 4.6362 would settle a rendering convention against two standing records on a worker's judgement. The compiled guard asserts only the three integers and no GiB value, so nothing mechanical depends on the answer.

**Fact — the C1 rows are correct and must not be touched.** C1 prefill's `6,077,440` is `10 * 607,744` and C1 decode's `607,744` is one position. Only the two B1-d prefill rows and the sentences quoting them are wrong. *Re-derived 2026-08-18 rather than assumed, because a "no change needed" leaves no diff to review: the B1-d decode and `D-B and final-position logits` rows carry `607,744` too, and `2,384,199,680 + 1,879,048,192 + 1,653,604,360 + 607,744 = 5,917,459,976` confirms the final-position peak. What the off-by-4096 did to that row is understate the projection's saving by 4,096 B, not misstate the row.*

**Fact — the saving already has a compiled guard, and it does not reach the prose.** `crates/tiler-reference/tests/language_model_boundaries.rs`, anchor `the_benchmark_prefill_saving_follows_from_the_two_declared_shapes`, derives all three corrected figures from the two programs' own declared output shapes and asserts them, so the corrected numbers cannot drift again without a red test. This ticket is the sweep that the guard does not cover.

## Required content

- Correct each live site to the recomputed figure, and where a record states the D-A or D-B peak, correct that too.
- Attach a dated correction where the corpus convention calls for one, rather than silently substituting a number inside a sentence labelled **Fact** with a source read.
- Do not edit the four dated audit and correction records named above.
- Recompute rather than copy from this ticket; a relayed figure is how the original error reached seven places.

## Closes when

Every live site states the recomputed figures; `grep -rn "4,978,634,752\|10,895,486,984\|28,074,307,592\|4,978,027,008" docs/ tickets/` returns only records that quote the retired figures deliberately — the four excluded dated records, the eight dated correction notes this sweep attached, and this ticket; and `make citations` and `tkt lint` pass. *Corrected 2026-08-18: the condition previously omitted the stale saving `4,978,027,008` from its grep, which is the same omission that hid the eighth live site. It cannot be read as "the grep is empty", because attaching a dated correction is what the corpus convention requires and a dated correction quotes what it retires — so the check is that every hit is accounted for by name, and the accounting is below.*

## Worker sweep — 2026-08-18, at base `a9efd708438b20e8454eafdd26b95de8a65fffab`

**Fact — per-Fact verdict, each read at this base before any edit.** *Corrected arithmetic*: **verified** independently — `python3 -c "print(8192*151936*4, 151936*4)"` prints `4978638848 607744`, the saving is `4,978,031,104`, and summing L6's other three columns gives `2,384,199,680 + 1,879,048,192 + 1,653,604,360 + 4,978,638,848 = 10,895,491,080` and `2,384,199,680 + 1,879,048,192 + 18,832,424,968 + 4,978,638,848 = 28,074,311,688`. *Population*: **false in its count and imprecise in its scope** — repaired in place above; eleven files at the base it names, and an eighth live site the enumerating grep could not match. *C1 rows*: **verified** for L6's table, with its final clause — "only the two B1-d prefill rows and the sentences quoting them are wrong" — **imprecise**: four of the ten corrected sites quote no logits cell at all but a figure *derived* from one — the saving or a peak — so "quoting them" has to be read through the derivation or the sweep stops four sites short. *Compiled guard*: **verified** — the named test asserts `4_978_638_848`, `607_744`, and `4_978_031_104`, and no GiB value; it was not touched.

**Fact — the ten corrected sites across eight live records, by anchor.**

| File | Anchor | What moved |
| --- | --- | --- |
| `docs/research/program-planning/complete-model-ingestion-and-execution.md` | `the difference at B1-d prefill is` | full-T and saving; dated note in place |
| `docs/research/program-planning/complete-model-ingestion-and-execution.md` | table rows `D-A unfused` and `B1-d prefill, D-B` | both logits cells and both peaks; dated correction paragraph under the table |
| `docs/research/program-planning/complete-model-ingestion-and-execution.md` | `removing 4,978,031,104 bytes of peak residency` | ticket-table row 12's saving, enumerated in that same paragraph |
| `docs/roadmap.md` | `A prefill pass that needs only the final position's logits` | full-T, saving, and D-B peak — both places the cell states them; dated parenthetical in the cell |
| `docs/research/shapes/sequence-extending-tensor-family.md` | `family now, with a stated trigger` | full-T; dated note |
| `docs/research/program-planning/model-level-qualification.md` | `L6 supplies the figures and labels every one an Inference` | D-A peak, D-B peak, and the saving below it; dated correction paragraph |
| `tickets/design-model-level-qualification-and-optimization.md` | `L6 supplies the figures and labels every one an Inference` | same three; dated correction paragraph |
| `tickets/admit-a-position-selecting-slice-for-the-rotary-table.md` | `bytes against 607,744. *(` | full-T; dated parenthetical |
| `tickets/scope-the-sequence-extending-tensor-family.md` | `F32 bytes at B1-d against 607,744 for one position` | full-T; dated parenthetical |
| `tickets/design-model-ingestion-and-complete-execution.md` | `the ticket that owns it` | saving; dated parenthetical. The eighth site, not in the enumeration above |

**Fact — the accounting the close condition asks for**, from `grep -rc` over the four retired figures across `docs/` and `tickets/`. In the eight corrected records every remaining occurrence is inside a dated correction note attached by this sweep, one occurrence per retirement, and there are no others:

| Record | `4,978,634,752` | `4,978,027,008` | `10,895,486,984` | `28,074,307,592` |
| --- | --- | --- | --- | --- |
| `docs/research/program-planning/complete-model-ingestion-and-execution.md` | 2 | 2 | 1 | 1 |
| `docs/roadmap.md` | 1 | 1 | 1 | — |
| `docs/research/program-planning/model-level-qualification.md` | — | 1 | 1 | 1 |
| `tickets/design-model-level-qualification-and-optimization.md` | — | 1 | 1 | 1 |
| `docs/research/shapes/sequence-extending-tensor-family.md` | 1 | — | — | — |
| `tickets/admit-a-position-selecting-slice-for-the-rotary-table.md` | 1 | — | — | — |
| `tickets/scope-the-sequence-extending-tensor-family.md` | 1 | — | — | — |
| `tickets/design-model-ingestion-and-complete-execution.md` | — | 1 | — | — |

The L6 record carries two of the first two figures because it took two notes — one at the logits contract and one under the residency table — and each retires the pair it substituted. Every other hit in `docs/` and `tickets/` is in one of the four excluded dated records or in this ticket, whose own counts are deliberately not pinned here because stating them would change them.

**Not done, deliberately.** The four excluded dated records are untouched, including `tickets/project-only-the-final-position-logits.md`, whose `## Out-of-scope defect` paragraph relays the same "ten files" count this ticket carried; that ticket is `done` and is excluded by name, so the count is noted here rather than repaired there. The `(4.6361 GiB)` rendering question is escalated above rather than settled.

**Coordinator decision — 2026-08-18, at closure.** The escalated GiB rendering question is settled without a sweep: in this corpus the exact byte figure is the authoritative value and a four-decimal GiB parenthetical is a non-normative reader convenience, not a Fact a check or decision may rest on — the compiled guard deliberately asserts only the integers. No rounding-versus-truncation convention is imposed retroactively; the three live `(4.6361 GiB)` renderings stand as approximations of 4.63615274 GiB (off by less than one part in 46,000), and neither dated nor live records are churned over the fourth decimal. A future record that needs a normative derived unit must state the derivation beside the bytes, as the residency tables already do. This is an internal documentation convention with one dominant answer, not a reserved decision.
