---
id: land-the-cpu-vector-lane-tier-adr
title: Land the CPU vector-lane tier ADR as proposed
status: todo
priority: p2
dependencies: [design-the-cpu-vector-lane-tier]
related: [design-the-cpu-vector-lane-tier, accept-the-cpu-vector-lane-tier-adr]
scopes: [contracts/decisions, contracts/navigation, research/scheduling, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, cpu, simd]
---
## User-visible outcome

The CPU vector-lane tier's decision exists as a numbered ADR carrying `decision_status: proposed`, reachable from both catalogs, so a reader arrives at a decision record rather than at a research record that happens to contain one.

## Why this is a separate ticket

**Fact, reproducible in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and maps both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`. [`design-the-cpu-vector-lane-tier`](design-the-cpu-vector-lane-tier.md) holds `research/scheduling`, `research/target-profiles`, and shared `project/tickets` only, so writing the ADR file or either catalog row from that branch is a scope escape. This is the same split [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) and [`land-the-backend-scoped-route-requirement-answer-adr`](land-the-backend-scoped-route-requirement-answer-adr.md) record, and the idiom is copied deliberately.

## What to do

1. **Take the next free ADR number by reading `docs/decisions/` rather than by remembering one.** `0091` was the highest at `cb5d86a` and a sibling may have landed since.
2. **Transfer the drafted body verbatim** from the `## Drafted ADR body, written to be landed verbatim` section of [the CPU vector-lane tier](../docs/research/scheduling/cpu-vector-lane-tier.md). The section carries the title, the frontmatter key set, Context, seven numbered Decision items, Consequences, and five Alternatives-considered entries. Do not reword at landing; if something needs rewording, that is a finding to report, not an edit to make silently.
3. **Add three catalog rows** — the decision row in `docs/decisions/README.md`, and a research row in `docs/research/README.md` for **each** of the two new research records, since both are new and neither has a row. The generated research-catalog block's row format is `- [Title](path.md) — <disposition>; <evidence classes>; informs: ...`.
4. **Set `adopted_by` on neither research record.** Both stay `disposition: pending` while the decision is proposed; flipping them is acceptance work, not landing work.
5. **Rename the acceptance node.** [`accept-the-cpu-vector-lane-tier-adr`](accept-the-cpu-vector-lane-tier-adr.md) exists as the graph node standing for Tom's decision, with a placeholder id because the number was unknown when it was filed. Once the number is fixed, `tkt rename accept-the-cpu-vector-lane-tier-adr accept-adr-NNNN-cpu-vector-lane-tier`, which moves the file, rewrites the id, and repoints its three dependents.

## Non-goals

Accepting the decision — that is Tom's and is the acceptance node's whole purpose. Editing `docs/backends/cpu.md`, which is `contracts/artifacts` and whose move off proposed is part of the acceptance sweep. Any crate edit.

## Closes when

The ADR file exists with a real number and `decision_status: proposed`, all three catalog rows are added, the body is byte-identical to the drafted block modulo the frontmatter it gains, the acceptance node is renamed and its dependents still resolve, and `tkt lint` passes.
