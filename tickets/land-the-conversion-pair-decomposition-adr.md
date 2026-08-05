---
id: land-the-conversion-pair-decomposition-adr
title: Land the conversion pair decomposition ADR
status: todo
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
- Add the row to the **proposed** section of `docs/decisions/README.md` in the same change, per the docs-maintenance rule that a catalog is edited in the change that moves the metadata behind it.
- Add a `related` link from the research record's frontmatter to the new ADR id.
- Leave the research record's `disposition` at `pending` and set no `adopted_by`. A proposed ADR is not an adoption.

## Explicit non-goals

- Accepting the ADR. Acceptance is Tom's, and an acceptance executed on a relay must name who accepted, the date, and the venue.
- Registering any conversion key, choosing any Rust spelling, or moving the `Cast and convert` support-matrix rung. All are reserved to Tom under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md), and ADR 0091 already reserved the same list for its own two families.
- Deciding any specific pair's *contract* — the rounding rule, overflow destination, underflow behaviour, or NaN pattern of `bf16 → f16` or any other pair. The record derives which fields are owed and deliberately fixes no value.
- Editing `docs/numerical-semantics.md`. Its widening-and-narrowing section is titled "derived at the BF16/binary32 pair" and states that the derivation does not transfer, so it was already scoped correctly; propagating a `proposed` decision into a contract would be exactly the silent proposal-to-fact conversion the corpus forbids.

## Closes when

The ADR exists at `proposed` with the body transferred byte-identically, the decision catalog carries its row, the research record carries the reciprocal `related` edge, and no rung, key, or contract sentence moved.
