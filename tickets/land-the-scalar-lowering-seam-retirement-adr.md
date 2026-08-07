---
id: land-the-scalar-lowering-seam-retirement-adr
title: Land the scalar-lowering seam retirement ADR
status: in-progress
priority: p1
dependencies: []
related: [resolve-or-retire-the-scalar-lowering-provider-seam, own-or-close-the-adr-internal-open-questions]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, extensions, capability]
claimed_from: todo
assignee: agent-scalar-adr
lease_expires_at: 1786066635
---
## User-visible outcome

The scalar-lowering seam's retirement exists as a numbered ADR carrying `decision_status: proposed`, reachable from both views of the decisions catalog, and ADR 0078's open question at its line 144 gains the answer its owner derived — so a reader arrives at a decision record rather than at a ticket that happens to contain one, and ADR 0078 stops carrying an unowned question whose owner has since answered it.

## Why this is a separate ticket

**Fact, reproducible in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md` to `contracts/navigation`. [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md) holds `implementation/compiler`, `contracts/optimizer`, `contracts/numerics`, and shared `project/tickets`, so writing the ADR file, the ADR 0078 amendment, or either catalog row from that branch is a scope escape. This is the same split [`land-the-two-level-reduction-adr`](land-the-two-level-reduction-adr.md) and [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md) record.

## What to do

1. **Read `docs/decisions/` and take the next free number.** The drafted body says `0103` because `0102` was the highest present at `eee734cf`, and the repository's carrier history shows records landing under a drafting ticket's nose more than once. If `0103` is taken, adjust the file name and the frontmatter `id` and change nothing else in the span.
2. **Transfer the drafted body byte-identically.** It is the span below the horizontal rule in the "The superseding record, verbatim for the carrier" section of [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md), from the `**Title:**` line to the closing rule. The frontmatter is given as a fenced block inside the span; lift it into real frontmatter and change nothing in it but the number, should step 1 require one. Change no other byte. A transfer that edits is a fork, and byte-identity is what makes "unreworded" checkable rather than asserted — normalize nothing else, `diff` the span against the landed `## Context`-through-alternatives range, expect no differences, and prove the check can fail by perturbing one word before believing it.
3. **Write the ADR's traceability, normative-owner, work-record, implementation-boundary, and open-questions sections fresh at the destination**, with `docs/decisions/`-relative links. The drafted span deliberately carries no links at all, so nothing has to be repointed — the check is that the `](` count inside the span's line range is zero while the count over the whole ticket is not.
4. **Amend ADR 0078's open question at `:144`** with the verbatim replacement recorded under "What is owed to scopes this ticket does not hold" in the same source ticket. It replaces that bullet's last two sentences and nothing else. Do **not** touch ADR 0078's item-2 inventory table or its `:63` absence-claim Fact: those are the supersession, and the supersession executes at acceptance, not at landing.
5. **Leave `decision_status: proposed`.** Nothing here is accepted, and acceptance is Tom's separate step.
6. **Add both catalog rows** — the theme view and the chronology view are separate blocks in `docs/decisions/README.md` and a decision needs a row in each — in the format the neighbouring rows use, and count the populations rather than asserting them.
7. **File the acceptance node**, `accept-adr-0103-retire-the-scalar-lowering-seam`, at `awaiting-decision`, following the `accept-adr-0100-multi-round-reduction-composition` shape. Its sweep, on acceptance, executes the item-2 supersession on ADR 0078, the `contracts/foundation` corrections to `docs/operation-extensions.md` at `:14`, `:58`, `:77`, `:85`, `:87`, and `:139`, and both catalog views. Give it `contracts/decisions`, `contracts/navigation`, and `contracts/foundation` so its sweep does not have to borrow scopes.
8. **File the removal ticket**, blocked on that acceptance node rather than on this carrier — the `ticketsplease.toml` workflow comment is explicit that a ticket conditional on an ADR being accepted depends on the acceptance node, never on the drafting or carrying ticket. It carries `implementation/compiler`, `contracts/optimizer`, `contracts/numerics`, and `contracts/foundation`, and it must carry the source ticket's normative finding that the ten registry-mechanics tests are **ported** to `register_index_access` rather than deleted, because deleting them would drop the surviving seam's collision, ambiguity, conflation, transactionality, and identity coverage — including the two-revisions case ADR 0078 item 3 cites as its own evidence.

## Non-goals

Accepting the decision. Editing the source ticket's derivation. Any crate change — the seam stays until Tom accepts the record. Touching ADR 0078's item-2 table or its `:63` Fact, which belong to the acceptance sweep.

## Closes when

The ADR file exists with the byte-identical span and a freshly written traceability section, ADR 0078:144 carries the verbatim amendment, both catalog rows resolve, the acceptance and removal tickets exist with the correct edge, `tkt lint` passes, and the byte-identity check has been run and shown able to fail.
