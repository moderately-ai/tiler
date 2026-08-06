---
id: land-the-conversion-pair-decomposition-adr
title: Land the conversion pair decomposition ADR
status: done
priority: p2
dependencies: [test-the-directional-conversion-pair-generalization]
related: [scope-the-in-type-precision-reduction-family, derive-the-operation-family-and-signature-delivery-graph, preserve-the-float-to-integer-conversion-precedent-sources]
scopes: [contracts/decisions, contracts/navigation, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, numerics, conversion, dtype]
---
## User-visible outcome

The rule that governs every future conversion registration — that a family is keyed by the ordered `(source, destination)` pair and a mode, and that its owed field set is *derived* from containment predicates over the pair rather than declared on it — exists as a numbered `proposed` ADR under `docs/decisions/`, listed in the decision catalog. Today it lives only inside a `docs/research/numerics/` record, so a reader arriving at the accepted-decision index to ask "what shape does a conversion family take?" finds ADR 0091's BF16/binary32 answer and nothing saying how far it reaches.

## Why this is a separate ticket and not an omission

**Fact — the scope map, read from the config rather than remembered.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md` to `contracts/navigation`:

```sh
rg -n 'contracts/decisions|contracts/navigation' -A 14 ticketsplease.toml
```

**Fact.** [`test-the-directional-conversion-pair-generalization`](test-the-directional-conversion-pair-generalization.md) held `research/semantic-graph`, `research/numerics`, and `contracts/navigation`, and did not hold `contracts/decisions`. Writing an ADR file from that branch would have been a guard escape.

**Inference.** That ticket's own closes-when asked for the answer recorded and the taxonomy row updated, and both are delivered. Carrying the answer into a decision record is the separately-schedulable half, and it is the half the [delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md) argues is time-sensitive: "Whichever shape the first registered conversion takes becomes the precedent every later pair is read against."

## What already exists and must be used rather than rewritten

[Conversion family decomposition across pairs](../docs/research/numerics/conversion-family-decomposition-across-pairs.md) carries a **Drafted ADR body** section written to be landed verbatim: context, five numbered decisions, five consequences, and three alternatives-considered entries each with the elimination stated. It also carries the twenty-pair enumeration, the worked `bf16`/`f16` walk, the double-rounding counterexample, the `n²` cost statement, two acquisition requests, and three deferred questions with triggers. Do not re-derive any of it.

**The transfer is byte-identical.** A transfer that edits is a fork.

**Two links inside the drafted body are already spelled for the destination and must not be repointed.** Its Context paragraph links `0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md` and `0041-separate-float-to-integer-conversion-families.md` as `docs/decisions/`-relative siblings, so they are broken where they currently sit and correct where they are going. The record states that beside the span rather than repointing, exactly as ADR 0092's source record does. Verify after the transfer that both resolve from `docs/decisions/`; a link check run over the research record before the transfer will report them and that report is expected, not a defect.

## Implementation keys

- Create `docs/decisions/0NNN-<slug>.md` with `decision_status: proposed`, `implementation_status: not-started`, `applies_to: ["tiler.contract.numerical-semantics"]`, `evidence: ["tiler.research.numerics.conversion-family-decomposition-across-pairs"]`, and `catalog_group: numerical-operations` — matching ADRs 0010, 0041, and 0091, the three conversion decisions it sits beside. Take the next free number by reading the directory, not by remembering one; 0100 was the highest when this ticket was filed and that is a claim to re-check, not to trust.
- Transfer the five decisions, the consequences, and the alternatives-considered entries verbatim from the drafted body. Add `depends_on` naming the accepted authorities the record preserves: ADR 0010, ADR 0018, ADR 0041, ADR 0075, and ADR 0091.
- Add the traceability block the sibling ADRs carry — normative owner [Numerical semantics](../docs/numerical-semantics.md), evidence the research record, work record this ticket — as new text rather than as part of the transfer.
- Add the row to `docs/decisions/README.md` in the same change, per the docs-maintenance rule that a catalog is edited in the change that moves the metadata behind it. **Corrected: that file has no "proposed" section.** It carries two hand-maintained views — a theme index grouped by `catalog_group` and sorted by title, and a chronological index — and each row states its own status suffix; the row goes into **both**, marked `— proposed`, which is the shape [`land-the-elementary-family-projection-adr`](land-the-elementary-family-projection-adr.md) used at `60221911` when it landed ADR 0099, and the reason the row count is two rather than one.
- ~~Add a `related` link from the research record's frontmatter to the new ADR id.~~ **Corrected before execution — see "Correction — the reciprocal edge is prose, not frontmatter" below.**
- Leave the research record's `disposition` at `pending` and set no `adopted_by`. A proposed ADR is not an adoption.

## Correction — the reciprocal edge is prose, not frontmatter

**Fact — a `research` record may not carry `related`.** [Documentation metadata](../docs/document-metadata.md)'s required-fields table gives each kind an *exhaustive* optional-typed-field licence, and the `Research` row reads `adopted_by`, `ticket`. The paragraph under it states the rule directly: "`related` is not among them; the optional column above is its exhaustive licence", and separately that `related` is "licensed only for the navigational kinds" — `Portal`, `Roadmap`, `Questions`, and `Prior art`. Reproduce with `sed -n '127,140p' docs/document-metadata.md`.

**Fact — the corpus agrees, exhaustively.** `grep -rl '^related:' docs/` returns exactly three files — `docs/dtype-support.md`, `docs/open-questions.md`, `docs/status.md` — and none of them is a research record. The check is fault-provable in one line: the same grep with `'^informs:'` returns the research population instead.

**Inference — and there is no substitute frontmatter edge at the `proposed` stage.** The typed research-to-ADR predicate is `adopted_by`, which asserts adoption; this ticket's own next key forbids setting it, correctly, because a proposed ADR is not an adoption. So executing the key as written would have added a field the metadata contract forbids, and the only conforming alternative would have asserted something false.

**What was done instead, following the one precedent for this exact situation.** [ADR 0092's source record](../docs/research/runtime/backend-scoped-route-requirement-answers.md) landed `proposed` at `6c652632` with its frontmatter unmoved — `disposition` pending, `adopted_by` unset — and its reciprocal written as a prose annotation on the drafted-body heading plus a paragraph beneath it. That commit's own message records the reasoning: "the disposition stays pending, adopted_by stays unset", and "the reach is prose only and the record's frontmatter does not move". This carrier did the same. One consequence is recorded rather than left to be discovered: annotating the heading moves its anchor, and the record's line 149 was the one referrer to `#drafted-adr-body`, so it is repointed in the same commit.

## Explicit non-goals

- Accepting the ADR. Acceptance is Tom's, and an acceptance executed on a relay must name who accepted, the date, and the venue.
- Registering any conversion key, choosing any Rust spelling, or moving the `Cast and convert` support-matrix rung. All are reserved to Tom under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md), and ADR 0091 already reserved the same list for its own two families.
- Deciding any specific pair's *contract* — the rounding rule, overflow destination, underflow behaviour, or NaN pattern of `bf16 → f16` or any other pair. The record derives which fields are owed and deliberately fixes no value.
- Editing `docs/numerical-semantics.md`. Its widening-and-narrowing section is titled "derived at the BF16/binary32 pair" and states that the derivation does not transfer, so it was already scoped correctly; propagating a `proposed` decision into a contract would be exactly the silent proposal-to-fact conversion the corpus forbids.

## Closes when

The ADR exists at `proposed` with the body transferred byte-identically, the decision catalog carries its row, the research record carries the reciprocal edge — **as prose, per the correction above, because the frontmatter form the original key named is invalid for a `research` record** — and no rung, key, or contract sentence moved.

## Outcome

**[ADR 0102](../docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md) exists at `proposed`, and nothing was accepted.** "Key a conversion family by the ordered pair and a mode and derive its owed fields", `implementation_status: not-started`, `catalog_group: numerical-operations`, `applies_to: ["tiler.contract.numerical-semantics"]`, `evidence: ["tiler.research.numerics.conversion-family-decomposition-across-pairs"]`, `depends_on: ["ADR-0010", "ADR-0018", "ADR-0041", "ADR-0075", "ADR-0091"]`.

**The number was read, not remembered, and the reading is reproducible.** At base commit `b689ea00`, `git ls-tree --name-only b689ea00 docs/decisions/ | grep -Eo '^docs/decisions/[0-9]+' | cut -d/ -f3 | sort | tail -1` prints `0101` over a population of 101 numbered files — so the ticket's filing-time claim that 0100 was highest was already stale, exactly as the key anticipated. 0102 was confirmed free against the directory and against both catalog views, and against every branch in the checkout: `for b in $(git branch --format='%(refname:short)'); do git ls-tree -r --name-only $b -- docs/decisions/ | grep -E '010[2-9]'; done` matched nothing across all 57 branches, so no in-flight branch had taken it.

**The transfer is byte-identical and the check was watched failing three ways.** The source span is `sed -n '173,199p'` of the research record — `### Context` through the last alternatives-considered entry — and the destination span is `sed -n '24,50p'` of the ADR; mapping `## ` back to `### ` on the destination and diffing produces empty output, and both files hash to `6f71a1976c5af61588e6f1b682c1901613de698b`. Heading promotion by one level is the only change, and it is the transfer convention every prior carrier used: the same extraction over ADR 0092 and ADR 0101 against their own source records reports the heading lines and nothing else. The check was fault-proven before being believed — against ADR 0101's body, against a one-line-shifted range, and against a one-character perturbation — and printed a difference every time, so its empty output is a match rather than a checker that did not run.

**Both destination-relative links inside the transferred body resolve from `docs/decisions/`.** `0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md` and `0041-separate-float-to-integer-conversion-families.md` both exist as siblings; all twelve links in the file were resolved with `[ -e ]` from `docs/decisions/`, all twelve passed, and the resolver was fault-proven against two invented paths that it reported broken.

**Four files moved, in three scopes this ticket already held.** `docs/decisions/0102-…md` (`contracts/decisions`, new); `docs/decisions/README.md` (`contracts/navigation`, two rows — the theme index under "Numerical operations", placed between 0012 and 0010 by the section's title sort, and the chronology); `docs/roadmap.md` (`contracts/navigation`, one clause: the `Cast and convert` row's next-step cell named this carrier and a future `proposed` ADR, so it now names the landed record — **the rung did not move and the cell says so**); and `docs/research/numerics/conversion-family-decomposition-across-pairs.md` (`research/numerics`, prose only — the drafted-body heading, its lead paragraph, the disposition bullet, and the one internal anchor the heading change moved). No scope needed adding.

**Nothing else moved, and the negatives are checked rather than asserted.** The research record's frontmatter is untouched: its terminator is line 14, and `git diff -U0` on that file reports hunks at lines 24, 149, 165, and 167 — every one below it. `disposition` stays `pending`, and no `adopted_by` field exists. `docs/numerical-semantics.md` is unmodified. No conversion key, name, version, or Rust spelling was chosen, and no support-matrix rung moved.

**One staleness was found and deliberately not filed, because a prior worker already weighed it.** [The minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) still says "`RQ-OP-04` leaves conversion's family decomposition open". [`test-the-directional-conversion-pair-generalization`](test-the-directional-conversion-pair-generalization.md) recorded that clause as "a one-clause edit rather than a ticket", and it remains defensible while ADR 0102 is `proposed` — the *decision* is genuinely still open. It becomes plainly wrong at acceptance, which is why the acceptance node below carries it rather than a new ticket.

**Acceptance is Tom's and now has a node**: [`accept-adr-0102-conversion-pair-decomposition`](accept-adr-0102-conversion-pair-decomposition.md), filed at `awaiting-decision` in the shape [`accept-adr-0099-elementary-family-projection`](accept-adr-0099-elementary-family-projection.md) established, because a carrier goes terminal the moment the file exists and cannot distinguish "written" from "decided".
