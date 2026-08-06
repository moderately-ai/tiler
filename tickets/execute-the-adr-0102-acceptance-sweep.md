---
id: execute-the-adr-0102-acceptance-sweep
title: Execute the ADR 0102 acceptance sweep
status: done
priority: p2
dependencies: []
related: [accept-adr-0102-conversion-pair-decomposition, land-the-conversion-pair-decomposition-adr]
scopes: [contracts/decisions, contracts/navigation, contracts/numerics, research/numerics, research/program-planning, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [acceptance-sweep, adr, conversion]
---

## The acceptance this applies

**Tom accepted ADR 0102 on 2026-08-06 at the live session's decision round** (provenance on [`accept-adr-0102-conversion-pair-decomposition`](accept-adr-0102-conversion-pair-decomposition.md)). A decision recorded is not a decision applied; this ticket is the whole application, in one change, because an acceptance applied in halves is how a draft gets read as settled.

## The sweep, enumerated

1. `docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md`: `decision_status: proposed` → `accepted`, with the acceptance sentence (who, date, venue) added per the corpus convention.
2. `docs/decisions/README.md`: both catalog rows' `— proposed` suffix → `— accepted` (theme row under Numerical operations, chronology row after 0101).
3. `docs/roadmap.md`: the Cast-and-convert row's trigger cell sentence ("sits at `proposed`, so the decision is Tom's and this row's rung moves on neither outcome") corrects to the accepted state — the rung still does not move (the ADR registers nothing), but the pending-decision framing is now false.
4. `docs/numerical-semantics.md`: the widening-and-narrowing section titled "derived at the BF16/binary32 pair" gains the accepted general rule's statement (or a pointer to the ADR as its owner), and the document's `evidence` frontmatter array gains the research record id if the convention requires it — read the sibling acceptances (ADR 0091's sweep) for the exact shape rather than inventing one.
5. The stale "`RQ-OP-04` leaves … open" clause in the minimum-correct-physical-realization profile record: plainly wrong at acceptance per the carrier's own analysis — correct in tense. **This item's file is `docs/research/program-planning/minimum-correct-physical-realization-profile.md`, not anything under `docs/research/numerics/`** — the filing line guessed the directory and [`accept-adr-0102-conversion-pair-decomposition`](accept-adr-0102-conversion-pair-decomposition.md) had it right; `grep -rn 'RQ-OP-04' docs/` returns the clause in that one file and nowhere in `docs/research/numerics/` outside the evidence record's own annotation of it.
6. Sweep for any other sentence whose truth depended on the proposed status (`grep -rn '0102' docs/ tickets/` and read each hit).

## Why it is held rather than dispatched

`contracts/navigation` is live-claimed by the navigation cell batch at filing time. TRIGGER: that claim releases — dispatch or execute coordinator-inline immediately after its merge.

## Closes when

All six items land in one change, every 0102 mention agrees with the accepted status, and `tkt lint` passes.

## Outcome

**The acceptance is applied, in one change, docs-only.** Eleven files moved: nine under `docs/`, this ticket, and one new ticket for an out-of-scope defect the sweep found. No file under `crates/` or `prototypes/` was touched, so the delta is a gate carry rather than a gate input.

**Two scopes were added, and each is the file an enumerated item names rather than new work.** `research/program-planning` carries item 5's file, which the filing line placed under `docs/research/numerics/` and which is actually `docs/research/program-planning/minimum-correct-physical-realization-profile.md` — the acceptance node named it correctly and the sweep item did not. `research/semantic-graph` carries two sentences item 6 reaches by its own words but not by its stated `grep '0102'`, because neither sentence spells the number. Both scopes were free at claim time: `tkt list --status in-progress` returned four tickets and the other three hold `implementation/ir`, `implementation/compiler`, and `implementation/metal-aot` only.

### The six items

1. **[ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md).** `decision_status: proposed` → `accepted`. The status block's first paragraph now records who, when, where, and the relay route, and names the three flagged items as presented and weighed; its third paragraph said Numerical semantics was unedited and why, and now records what the sweep moved there and that the `mixed`-contract rule is why it could not have moved earlier. Traceability's normative-owner paragraph moved with it. The second status paragraph — the record fixes the family's *shape* and no pair's contents — needed no edit and is unchanged.
2. **[The decisions index](../docs/decisions/README.md).** Both rows, `— proposed` → `— accepted`: the theme row under "Numerical operations" at `:54` and the chronology row after 0101 at `:247`.
3. **[The roadmap](../docs/roadmap.md).** The `Cast and convert` row's trigger cell said the record "sits at `proposed`, so the decision is Tom's and this row's rung moves on neither outcome". It now records the acceptance, states that the rung did not move *and could not have* because the record registers nothing, and says what the acceptance does change for the row: the next step is an admission under a stated pair rather than a decision about the family model. **No rung moved and no other cell moved.**
4. **[Numerical semantics](../docs/numerical-semantics.md).** A new `### A conversion family is keyed by the ordered pair and a mode, and its owed fields are derived` subsection under `## Casts`, stating clauses 1 through 5 normatively with the four containment predicates (including the second clause of the rounding predicate the decisive pair turns on), the two-way construction refusal, the incomparable-pair case with both `bf16`/`f16` field sets, the constructibility-and-legibility merge test, the identities-not-schemas growth statement, the double-rounding counterexample, and the evidence record's stated boundary. `evidence` gains `tiler.research.numerics.conversion-family-decomposition-across-pairs`, appended, following ADR 0101's sweep at `5add0046` rather than re-sorting an array that acceptance already left unsorted. The existing BF16/binary32 subsection keeps its title and its four Facts; its lead sentence claimed the pair's asymmetry was "enough to fix the family's shape", which clause 3 falsifies, and now places the pair as the *comparable* case whose behaviour the general rule names.
5. **[The minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md).** The admission rule's third condition cited `RQ-OP-01` and `RQ-OP-04` as open questions that do not bear on the route. `RQ-OP-01` is untouched; the `RQ-OP-04` half now records that it closed — answered 2026-08-05, decided 2026-08-06 — and that the route moved on neither the answer nor the acceptance, which is what the condition predicted. The family classification does not move and the table is unchanged.
6. **The sweep.** Five further sites, each read in full before it was edited or kept.
   - [Conversion family decomposition across pairs](../docs/research/numerics/conversion-family-decomposition-across-pairs.md): `disposition: pending` → `adopted` and `adopted_by: ["ADR-0102"]` added, which is exactly the pair the acceptance node predicted would move together; the disposition and normative-destination bullets; the Part-3 label paragraph; the drafted-body heading and the one internal anchor that heading change moves; the heading's two lead paragraphs, one of which argued that no frontmatter reciprocal could exist because `adopted_by` would assert an unreceived adoption; and both bullets of "Where this record differs from the corpus", whose two staleness notes the sweep discharged.
   - [The research catalog](../docs/research/README.md) at `:50`: `— pending` → `— adopted (ADR 0102)`, in the shape ADR 0101's sweep gave the row above it. **This row spells no `0102` and the ticket's stated grep does not reach it**; it was found by grepping the record's id and title instead, which is the check the stated one needs beside it.
   - [The operation taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md) at `:342`: the `RQ-OP-04` row ended "that rule is a Proposal drafted for an ADR and carried by [the carrier]", which acceptance makes false. It now names ADR 0102 and the contract section that states the rule normatively. The F-18 and F-19 rows were read and need nothing: they attribute the per-ordered-pair key to `RQ-OP-04`, which is where it was derived and remains true.
   - [The delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md) at `:209`: track O-22's row said the rule "is carried to an ADR"; it now says the ADR was accepted, and repeats that the acceptance moved no rung either, for the same reason the roadmap cell gives.
   - Ticket bodies were deliberately **not** edited, following the precedent at `5add0046`, which touched no landing ticket when it applied ADR 0101. [`land-the-conversion-pair-decomposition-adr`](land-the-conversion-pair-decomposition-adr.md) states at `:69` that the record "exists at `proposed`, and nothing was accepted" and at `:81` that the profile clause "becomes plainly wrong at acceptance"; both are that ticket's dated account of what *it* delivered and both were true when written. [`accept-adr-0102-conversion-pair-decomposition`](accept-adr-0102-conversion-pair-decomposition.md) carries the acceptance provenance and its `## Accepted 2026-08-06` section describes this sweep as pending dispatch; that node is the coordinator's and its closure is the coordinator's step.

### The 0102 population, counted

`grep -rn '0102' docs/ tickets/ spikes/ crates/ prototypes/ *.md` returns **62 lines** excluding this ticket. Every one was read. **Eighteen are coincidental digit strings and carry no reference**: five hex identity literals in `crates/tiler-compiler/src/frontier.rs` and one in `physical.rs`, two in `crates/tiler-ir/src/schedule/builder.rs`, two rendered contract keys in `crates/tiler-ir/src/schedule/numerics.rs`, one in `crates/tiler-ir/src/semantic/registry.rs`, a WWDC video id `10102` in the Apple artifact-compatibility record, two vendored third-party source hits (`LangRef.rst`, ONNX `Operators.md`, the latter the SELU coefficient `1.05070102214813232421875`), two `Cargo.lock` checksums, and the same SELU coefficient quoted in `land-the-elementary-family-projection-adr`. **Twenty more are TSV measurement cells** under `spikes/program-planning/qwen3-*/results/`. That leaves **twenty-four real references**: two in the ADR's own file, two catalog rows, one roadmap cell, four in the evidence record, one in the delivery graph, one in the taxonomy, one in the profile record, one in the contract, one in the research catalog, five in `land-the-conversion-pair-decomposition-adr`, and six in `accept-adr-0102-conversion-pair-decomposition`. Eleven of the twenty-four were edited, and the remaining thirteen are the ADR's id and title lines, the ticket work records above, and prose that acceptance leaves true.

### One out-of-scope defect found and filed rather than fixed

[Numerical semantics](../docs/numerical-semantics.md)'s ownership-boundary paragraph says "The accepted decisions are ADRs 0009–0042 together with ADRs 0055, 0059, 0060, 0062, and 0066." Forty-five accepted decisions carry `applies_to: tiler.contract.numerical-semantics`, over a population of 102 numbered decision files; thirty-nine fall inside the stated ranges and six do not — ADRs 0076, 0080, 0091, 0095, 0101, and 0102. **Five of the six predate this acceptance**, so the enumeration was already wrong independently of it and appending 0102 alone would have turned a visible defect into an invisible one. [`repair-the-numerical-semantics-accepted-decision-list`](repair-the-numerical-semantics-accepted-decision-list.md) owns it, with the reproducing loop.

### Checks

- `tkt lint` — pass.
- `tkt guard --base c22d4b24` — no scope escape.
- `git diff --check` — clean.
- `git diff --name-only` — `docs/` and `tickets/` only; no `crates/`, `prototypes/`, `Cargo.*`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh`, so the delta may carry the latest green gate under the rule in `AGENTS.md`.
- **The transferred span is still byte-identical after the sweep**, re-checked rather than assumed because two of the edits sit immediately above it: the evidence record's `### Context`-through-alternatives range with `### ` mapped to `## `, compared against the ADR's `## Context`-through-alternatives range, is 27 lines on each side and `cmp` reports no difference. The check was proved able to say no first — substituting "keyed by the UNORDERED pair" for "keyed by the ordered `(source, destination)` pair" in a copy of the source side makes it report the differing line.
