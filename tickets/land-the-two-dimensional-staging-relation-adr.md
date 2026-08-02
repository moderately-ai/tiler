---
id: land-the-two-dimensional-staging-relation-adr
title: Land the two-dimensional staging relation ADR
status: in-progress
priority: p2
dependencies: [admit-a-two-dimensional-cooperative-staging-relation]
related: [admit-a-two-dimensional-cooperative-staging-relation]
scopes: [contracts/decisions, contracts/navigation, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, ir, identity]
claimed_from: todo
assignee: agent-adr-carrier
lease_expires_at: 1785688837
---
## User-visible outcome

The two-dimensional cooperative staging relation's decision exists as a numbered ADR carrying `decision_status: proposed`, reachable from both catalogs, and [a two-dimensional cooperative staging relation](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) has its row in the research catalog — so a reader arrives at a decision record rather than at a research record that happens to contain one, and reaches the research record from the index rather than by knowing its path.

## Why this is a separate ticket

**Fact, reproducible in one line.** `ticketsplease.toml:103` maps `docs/decisions/[0-9]*.md` to `contracts/decisions`, and `:89-102` maps both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`. [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) holds `implementation/ir` and `research/scheduling` with shared `project/tickets` only, so writing the ADR file or either catalog row from that branch is a scope escape. This is the same split [`land-the-two-level-reduction-adr`](land-the-two-level-reduction-adr.md), [`land-the-subgroup-execution-tier-adr`](land-the-subgroup-execution-tier-adr.md), and [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md) record.

## The transfer is byte-identical, and that is the whole obligation

The research record carries the drafted ADR body verbatim-landable between two horizontal rules, and states its own line range beside it. **Re-read the rule positions with `grep -n '^---$'` before trusting any line number in that record**, because any edit above the span moves them; at the record's landing commit the span was lines 241-286, beginning at `**Title:**` and ending at the last alternatives-considered bullet.

Transfer that range with `### ` mapped to `## ` and change nothing else. Check it by diffing the two ranges after that normalization, and **perturb one word and watch the check fail before believing it** — a diff that reports no differences because it compared the wrong ranges is indistinguishable from a correct transfer.

**The span carries no traceability section and therefore no relative links**, which is checked rather than assumed: `sed -n '<span>p' | grep -c ']('` returns `0` while the same command over the record's first hundred lines returns `3`. So there is no link to repoint and no fork risk from repointing one. Cross-references inside the span are by ADR number and contract name in prose, which resolve from either location.

**Verified at the carrier's base `6f2601a`, because the numbers above had already moved once.** `grep -n '^---$'` on the record gives rules at 1, 15, 243, and 292, so the span is lines **245-290**, not the 241-286 this ticket was written with. Within that span, lines 245 and 247 are the `**Title:**` and `**Frontmatter:**` *directives* — they are consumed into the ADR's `# NNNN:` heading and its YAML frontmatter rather than landed as body prose, which is what the three sibling carriers did and what ADR 0096's own status line records ("diffing the source span against this record's `## Context`-through-alternatives range"). **The byte-identical body is therefore lines 249-290**, transferred with `### ` mapped to `## `. The check was watched failing on a one-word perturbation and on a one-line range misalignment before it was believed, and the link-free check reproduces at this base: `0` over 249-290 against `3` over lines 1-100.

## What the carrier writes fresh at the destination

The traceability section, the normative-owner paragraph, the work-record paragraph, the implementation boundary, and the open questions — none of which exists in the span, and all of which must be written against the tree the ADR lands into rather than copied from a sibling.

**The number is read from the directory, not taken from the draft.** `0096` was the highest ADR present at the research record's base `54833c9`, so the draft says `0097`; three records drafting against a number have had it move underneath them. Read `docs/decisions/` again and take the next free number. Nothing in the span depends on it.

**The implementation boundary must state the tree state at the carrier's own base**, not the research record's. At `54833c9` the facts to re-check are: `StagedSpan` has exactly three fields and `LocalCoordinateSource` exactly one variant, so every construct the ADR names is a type-system reservation that does not compile; `crates/tiler-ir/src/schedule/model.rs:1878` still writes `tiler.schedule.v4`; and no pinned identity has moved.

## Catalog rows

Two, and the first is the one most likely to be forgotten because nothing checks it:

- `docs/research/README.md`, under the scheduling group beside the two-level reduction's row, for the research record — which has never had one.
- `docs/decisions/README.md`, for the new ADR, in its `catalog_group: "physical-planning-lowering"` section.

## What this ticket does not own

The public boundary the ADR states — Tom's acceptance of it is a separate act with its own graph node, exactly as [`accept-adr-0096-two-level-reduction`](accept-adr-0096-two-level-reduction.md) is for ADR 0096. And the identity step: this ticket lands a document and moves no encoding, no version string, and no pinned value.

## Closes when

The ADR exists at its own number with `decision_status: proposed`, its body is byte-identical to the drafted span under the stated normalization with the check watched failing first, both catalog rows resolve, and the research record's `adopted_by` is left unset because a proposed decision has adopted nothing.
