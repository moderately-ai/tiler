---
id: land-the-two-level-reduction-adr
title: Land the two-level subgroup-then-workgroup reduction ADR
status: todo
priority: p2
dependencies: [compose-the-two-level-subgroup-and-workgroup-reduction]
related: [compose-the-two-level-subgroup-and-workgroup-reduction]
scopes: [contracts/decisions, contracts/navigation, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, subgroup, gpu]
---
## User-visible outcome

The two-level reduction's decision exists as a numbered ADR carrying `decision_status: proposed`, reachable from both catalogs, and [the two-level subgroup-then-workgroup reduction](../docs/research/scheduling/two-level-subgroup-workgroup-reduction.md) has its row in the research catalog — so a reader arrives at a decision record rather than at a research record that happens to contain one, and reaches the research record from the index rather than by knowing its path.

## Why this is a separate ticket

**Fact, reproducible in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and maps both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`. [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) holds `research/scheduling` and shared `project/tickets` only, so writing the ADR file or either catalog row from that branch is a scope escape. This is the same split [`land-the-subgroup-execution-tier-adr`](land-the-subgroup-execution-tier-adr.md), [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md), and [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) record.

**This carrier takes two catalog rows rather than one**, because the research record is new: the ADR's row under `docs/decisions/README.md` and the research record's row under `docs/research/README.md`. The subgroup and CPU lane-tier carriers each took only the decision rows, because their research records were already catalogued by the tickets that produced them; this one was produced under a `research/scheduling`-only ticket that could not reach `docs/research/README.md` either.

## What to do

1. **Read `docs/decisions/` and take the next free number.** The drafted body says `0096` because `0095` was the highest present at `2aa0824`, and the warning has earned its keep three times: `0093`, `0094`, and `0095` each landed while a record drafting against them was open. If `0096` is taken, adjust the file name and the frontmatter `id` and change nothing else in the span.
2. **Transfer the drafted body byte-identically.** It is the span below the horizontal rule in the "Drafted ADR body, to be landed verbatim by a carrier ticket" section of [the research record](../docs/research/scheduling/two-level-subgroup-workgroup-reduction.md), from the `**Title:**` line to the closing rule. Map `### ` headings to `## ` and change nothing else — a transfer that edits is a fork, and byte-identity is what makes "unreworded" checkable rather than asserted. Verify it the way the subgroup landing did: normalize the heading level and `diff` the span against the ADR's `## Context`-through-alternatives range, expecting no differences, and prove the check can fail by perturbing one word before believing it.
3. **Write the ADR's traceability, normative-owner, work-record, implementation-boundary, and open-questions sections fresh at the destination**, with `docs/decisions/`-relative links. The drafted span deliberately carries no links at all, so nothing has to be repointed — the check is that the `](`-count inside the span's line range is zero while the count over the whole record is not, and both were run at drafting.
4. **Leave `decision_status: proposed`.** The span's frontmatter line says `proposed` and that is what it means: nothing here is accepted, and acceptance is Tom's separate step with its own ticket, following the `accept-adr-0094-subgroup-execution-tier` shape.
5. **Add both catalog rows** in the same change as the metadata behind them, in the format the neighbouring rows use.
6. **Set the research record's `disposition`** — it is `pending` today. It moves to `adopted` with `adopted_by` naming the new ADR only when the ADR is accepted, not when it lands as proposed; check what the subgroup record's carrier did rather than guessing, because that record moved in two steps.

## Non-goals

Accepting the decision. Editing the research record's body. Any crate change. Any of the seven public-boundary items the research record enumerates — each arrives at Tom under ADR 0075 with the implementation ticket that reaches it, and landing a proposed ADR accepts none of them.

## Closes when

The ADR file exists with the byte-identical body and a freshly written traceability section, both catalog rows resolve, `tkt lint` passes, and the byte-identity check has been run and shown able to fail.
