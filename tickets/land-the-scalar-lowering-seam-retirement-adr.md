---
id: land-the-scalar-lowering-seam-retirement-adr
title: Land the scalar-lowering seam retirement ADR
status: done
priority: p1
dependencies: []
related: [resolve-or-retire-the-scalar-lowering-provider-seam, own-or-close-the-adr-internal-open-questions]
scopes: [contracts/decisions, contracts/navigation, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, extensions, capability]
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

**Amended 2026-08-06 by the coordinator, before the work ran, and the amendment is what makes the non-goals above stale.** Tom accepted the retirement on 2026-08-06 at the live session's decision round, so the acceptance had already happened when this ticket was dispatched. The carrier therefore lands the record `accepted` rather than `proposed`, executes the acceptance sweep in the same change — ADR 0078's item-2 row and its `:63` Fact included, which the non-goals had reserved for a later ticket — files the acceptance node `done` rather than `awaiting-decision`, and files the removal ticket unblocked rather than dependent. `contracts/foundation` was added to this ticket's scopes for the sweep's `docs/operation-extensions.md` edits, because the sweep now runs here instead of behind an acceptance node.

## Outcome (2026-08-06)

### The number taken, and why the drafted one was wrong

**0105.** `0103` and `0104` landed under the drafting ticket's nose while this carrier was being written, exactly the hazard step 1 anticipated: the identity campaign took both while the drafted span still said `0103`. `ls docs/decisions/[0-9]*.md | wc -l` returns 105 with no gap, so 0105 was the next free number. The file is [`docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md`](../docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md), and the frontmatter `id` is `ADR-0105`. **The drafted span itself needed no number edit at all** — `grep -c '0103'` over its line range returns zero, because the span carries no self-reference; the only `0103` occurrences in the source ticket are in the frontmatter block and in the two owed-text sections outside the span.

### Byte-identity, run and shown able to fail

The landed `## Context`-through-`## Alternatives considered` range is lines 20–82 of the ADR, 63 lines and 13,202 bytes, against the source span at lines 130–192 of [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md).

```sh
diff <(sed -n '20,82p' docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md) <(sed -n '130,192p' tickets/resolve-or-retire-the-scalar-lowering-provider-seam.md)
```

**No differences**, and `cmp` over the same two streams reports byte-identical. **The check was proved able to fail before the clean result was believed**: one word in the landed file's first Context sentence was changed from `classified` to `classifies`, the same `diff` reported `3c3` with both variants, and the word was reverted. The `](` count over the landed span range is **zero**, against **five** over the whole source ticket — the step-3 check that the freshly written sections carry the links and the transferred body carries none.

### The acceptance sweep, site by site

**[ADR 0105](../docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md)** carries `decision_status: "accepted"` and a status paragraph in the house form: who (Tom), when (2026-08-06), venue (the live session's decision round), relay (presented by the orchestrator under explain-then-recommend, relayed rather than witnessed by this worker, with this ticket and the deriving ticket as the packet). It states the supersession as item-2-row-scoped in prose on both records, per the [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) precedent.

**[ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md)**, five sites. The item-2 `ScalarLoweringProvider` row is removed and the `Fact` stating its absence claim is removed with it, replaced by a `Superseded 2026-08-06` note that quotes the removed row and states the ground — so the supersession is legible on the superseded record rather than a silent absence. The scalar-lowering open question carries the verbatim amendment. Three further sites were corrected because the row's removal made them false, and leaving them would have been an incoherent sweep rather than a narrow one: the status paragraph's "two questions still open" count, the item-4 current-refinement sentence whose "so its row is unchanged" pointed at a row that no longer exists, and the implementation boundary's "two open maturity questions" plus its item-1-without-exception claim, which was false for exactly one row until this acceptance and is now true. The open-questions preamble's disposition count moved with them.

**Three adjustments to the owed verbatim amendment, each reported rather than silent.** The coordinator's brief directed adjusting only words that state the decision as pending. Two did: "it is now a **proposed** decision rather than an open question" became "**accepted**", and "ADR 0103 **proposes retiring** the family and **superseding** this record's item-2 row, and **only Tom's acceptance of that record settles it**" became "ADR 0105 **retires** the family and **supersedes** this record's item-2 row, **accepted by Tom on 2026-08-06**". The third is formatting rather than tense: `ADR 0103` became a markdown link to the landed record, because every sibling ADR reference in that document is linked and an unlinked one would read as an error. Nothing else in the replacement moved, and the two sentences it replaces are exactly the two the source ticket names.

**[`docs/operation-extensions.md`](../docs/operation-extensions.md)**, the six owed sites plus one the owed list did not name. `:14` status line, `:58` three-claims installation paragraph, `:77` seam table row (removed), `:85` rung-invariant paragraph, `:87` two-halves paragraph, and `:139` registry-lifecycle paragraph. **The seventh is a population count**: the paragraph below the table read "the surrounding table names **five** that are", which the removed row made wrong; it now reads four. That is the site a sweep working from a line list is likeliest to miss, and it is the kind of claim AGENTS.md asks be counted rather than asserted. `docs/architecture.md` was re-checked rather than trusted — `grep -n 'scalar-lowering\|ScalarLowering' docs/architecture.md` returns nothing, confirming the source ticket's claim on this base.

**[The decisions catalog](../docs/decisions/README.md)**, both views. Counted, not asserted: 105 ADR files on disk, 105 rows in the theme view, 105 rows in the chronology view, and the chronology passes `sort -c` ascending. The theme row sits in `foundation-semantics-extensions` in that block's title order, between "Name the intended public extension seams" and "Separate extent symbols from typed root bindings".

Every markdown link in all six touched files was resolved against the filesystem; none is broken.

### The two tickets filed

[`accept-adr-0105-retire-the-scalar-lowering-seam`](accept-adr-0105-retire-the-scalar-lowering-seam.md) at `done`, following the [`accept-adr-0100-multi-round-reduction-composition`](accept-adr-0100-multi-round-reduction-composition.md) shape, carrying the provenance and the executed sweep. It is `done` rather than `awaiting-decision` because it records an acceptance that has happened; a `done` node satisfies dependents, which is what unblocks the removal.

[`remove-the-scalar-lowering-family-from-the-compiler`](remove-the-scalar-lowering-family-from-the-compiler.md) at `todo`, unblocked, with `implementation/compiler`, `contracts/optimizer`, `contracts/numerics`, and `contracts/foundation` and a `related` edge to the acceptance node rather than a dependency. It carries **three** findings verbatim from the deriving ticket rather than the one required: the ported-tests finding, the identity-preserving finding, and the full removal inventory — each is normative and each is a line a paraphrase would damage. Verbatim carriage was verified with `grep -Fxf` against the source lines rather than by reading. It also names what it must not do — collapse either reserved type, recompute any pin — and states its own line numbers as drifting, with the two re-derivation commands, because the deriving ticket records that its numbers had already drifted once.

### Scope and checks

**Docs and tickets only.** No crate, prototype, spike, `Cargo.*`, `.config/`, `Makefile`, or toolchain file is touched, so no build, test, or lint gate applies and the delta is eligible for the latest green gate under AGENTS.md's carry rule. `tkt lint`, `git diff --check`, and `tkt guard` against the true base are the applicable checks and all pass.

## Current implementation correction (2026-08-09)

The removal ticket this carrier filed as `todo` is now `done`.
`remove-the-scalar-lowering-family-from-the-compiler` removed the retired
family, ported the ten registry-mechanics tests to `register_index_access`,
left the two reserved single-variant types standing, and preserved canonical
registry identity. ADR 0105 now correctly carries
`implementation_status: "complete"`. The carrier's Outcome above remains the
history of the acceptance/landing change, not the current implementation state.

## Fact audit — 2026-08-10

**User-visible outcome and original What-to-do `proposed` wording are pre-Amendment plan text.** The UV outcome still says the record lands with `decision_status: proposed`, and steps 5–8 still describe an awaiting-decision acceptance node and a removal blocked on acceptance. The **Amended 2026-08-06** paragraph already supersedes that plan (land `accepted`, run the sweep here, file the acceptance node `done`, file removal unblocked). Status `done` and the Outcome match delivered work; do not read the UV outcome or pre-amendment steps as live board state.

**Byte-identity recipe in the Outcome is historical line numbers, not the durable check.** Outcome cites ADR lines `20–82` and source `130–192` with `13,202` bytes. Those exact `sed` ranges no longer bound the transferred body: at the 2026-08-10 audit base the heading-bounded span is ADR `## Context` through the end of `## Alternatives considered` at lines **22–84** against the deriving ticket at **127–189** (63 lines, 13,202 bytes each; `diff` empty). Prefer locating by those headings over the frozen line recipe. Outcome's ADR-file population **105** and catalog 105/105/105 counts are likewise landing-time history (ADR count has grown since).

**Acceptance-time present tense on ADR 0078 is live residual after removal.** The 2026-08-06 sweep correctly removed the item-2 row, answered the open question, and aligned maturity counts. Three clauses written when the family still compiled were not rewritten when [`remove-the-scalar-lowering-family-from-the-compiler`](remove-the-scalar-lowering-family-from-the-compiler.md) landed: the Superseded note still says the family, trait, and registration are **still in the tree**; item-4's current refinement still says `` `ScalarLoweringProvider` installation is reachable and has no compile-path caller ``; the implementation boundary still says crate-level execution is a **scheduled removal**. ADR 0105 and `docs/operation-extensions.md` already state the end state (`implementation_status: "complete"`, family gone from the crate). Repair is docs-only on [`docs/decisions/0078-name-the-intended-public-extension-seams.md`](../docs/decisions/0078-name-the-intended-public-extension-seams.md) under `contracts/decisions` — not a false claim about what this carrier's acceptance sweep did, and not work this ticket re-opens (status stays `done`).
