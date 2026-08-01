---
id: land-the-subgroup-execution-tier-adr
title: Land the subgroup execution tier ADR as proposed
status: todo
priority: p2
dependencies: [design-the-subgroup-execution-tier]
related: [design-the-subgroup-execution-tier, accept-the-subgroup-execution-tier-adr]
scopes: [contracts/decisions, contracts/navigation, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, subgroup, gpu]
---
## User-visible outcome

The subgroup execution tier's decision exists as a numbered ADR carrying `decision_status: proposed`, reachable from both catalogs, so a reader arrives at a decision record rather than at a research record that happens to contain one.

## Why this is a separate ticket

**Fact, reproducible in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and maps both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`. [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md) holds `research/scheduling` and shared `project/tickets` only, so writing the ADR file or either catalog row from that branch is a scope escape. This is the same split [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md), [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md), and [`land-the-backend-scoped-route-requirement-answer-adr`](land-the-backend-scoped-route-requirement-answer-adr.md) record.

## What to do

1. **Take the next free ADR number by reading `docs/decisions/` rather than by remembering one.** `0092` was the highest at `8252312` and `0093` was free, but a sibling may have landed since — and `land-the-cpu-vector-lane-tier-adr` is queued for a number too, so the two must not collide.
2. **Transfer the drafted body verbatim** from the `## Drafted ADR body, written to be landed verbatim` section of [the subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md). The section carries the title, the frontmatter key set, Context, nine numbered Decision items, Consequences, and six Alternatives-considered entries. Do not reword at landing; if something needs rewording, that is a finding to report, not an edit to make silently.
3. **The drafted span carries no traceability section and no relative links**, verified by counting them: zero local markdown links inside the span. The AGENTS.md drafted-body link tension therefore does not arise here and the byte-identical transfer is unconditional, as it was for the BF16 draft. If you add a traceability section at landing — worth doing, since the ADR depends on seven others — write it with `docs/decisions/`-relative paths and treat it as new text authored at the ADR, not as an edit to the transferred span.
4. **Add two catalog rows** — the decision row in `docs/decisions/README.md`, and a research row in `docs/research/README.md` for the new research record, which is new and has no row. The generated research-catalog block's row format is `- [Title](path.md) — <disposition>; <evidence classes>; informs: ...`.
5. **Leave `adopted_by` unset on the research record.** It stays `disposition: pending` while the decision is proposed; flipping it is acceptance work, not landing work.
6. **Rename the acceptance node.** [`accept-the-subgroup-execution-tier-adr`](accept-the-subgroup-execution-tier-adr.md) exists as the graph node standing for Tom's decision, with a placeholder id because the number was unknown when it was filed. Once the number is fixed, `tkt rename accept-the-subgroup-execution-tier-adr accept-adr-NNNN-subgroup-execution-tier`.

## Non-goals

Accepting the decision — that is Tom's and is the acceptance node's whole purpose. Any crate edit. Editing `docs/backends/metal.md` or `docs/artifact-abi.md`, which are other scopes and belong to the acceptance sweep.

## Closes when

The ADR file exists with a real number and `decision_status: proposed`, both catalog rows are added, the body is byte-identical to the drafted block modulo the frontmatter it gains, the acceptance node is renamed and its dependents still resolve, and `tkt lint` passes.
