---
id: accept-adr-0105-retire-the-scalar-lowering-seam
title: Accept ADR 0105 retire the scalar-lowering seam
status: done
priority: p2
dependencies: []
related: [land-the-scalar-lowering-seam-retirement-adr, resolve-or-retire-the-scalar-lowering-provider-seam]
scopes: [contracts/decisions, contracts/navigation, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

[ADR 0105](../docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md) moves from `proposed` to `accepted`, or is rejected. **Only Tom closes this ticket.** Acceptance supersedes [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md)'s item-2 inventory row for `tiler_compiler::capability::ScalarLoweringProvider` and nothing else in that record, and the acceptance sweep executes that supersession explicitly, plus the `contracts/foundation` corrections to [the operation extension contract](../docs/operation-extensions.md) and both catalog views. The record self-accepts nothing and removes nothing from any crate: the two shapes falling out of the removal — a single-variant `LoweringImplementation` and a single-variant `LoweringFamily` — are reserved to Tom under ADR 0075 and enumerated in the record's own open questions.

This node exists as the record of the acceptance act rather than as a pending question. It is filed `done` because the acceptance had already happened when the carrier landed the record, and the [`ticketsplease.toml`](../ticketsplease.toml) convention it satisfies is that a ticket conditional on an ADR being accepted depends on the acceptance node rather than on the drafting or carrying ticket — a `done` node satisfies its dependents, which is exactly right once the decision is taken.

## Decided — accepted

**Accepted by Tom on 2026-08-06 at the live session's decision round**, presented by the orchestrator under explain-then-recommend and relayed to the carrier worker rather than witnessed by it. The presented packet carried the two-candidate elimination, the identity-preserving removal shape, and the ported-tests finding; the relay sources are [`land-the-scalar-lowering-seam-retirement-adr`](land-the-scalar-lowering-seam-retirement-adr.md) and the deriving ticket [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md).

**Sweep executed in the same change as the landing**, by the carrier, at the commit that ticket's Outcome records:

- `decision_status: "accepted"` on ADR 0105, with the provenance paragraph in the house form.
- ADR 0078's item-2 `ScalarLoweringProvider` row removed, and the `Fact` stating that row's absence claim removed with it and replaced by a `Superseded 2026-08-06` note — so the removal is stated on the superseded record rather than left as a silent absence, per the [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) precedent of naming the superseded item in prose on both records rather than edging the whole record.
- ADR 0078's scalar-lowering open question replaced by the answer, and its status paragraph, its item-4 current-refinement paragraph, its implementation boundary, and its open-questions preamble aligned with the one-open-question end state.
- [The operation extension contract](../docs/operation-extensions.md) corrected at six sites plus one population count: the status line, the three-claims installation paragraph, the seam table row, the "names five that are" count that the removed row made wrong, the rung-invariant paragraph, the two-halves paragraph, and the registry-lifecycle paragraph.
- Both views of [the decisions catalog](../docs/decisions/README.md) carry the 0105 row as accepted; 105 ADR files, 105 theme rows, 105 chronology rows.

**What acceptance did not do.** No crate file was touched. [`remove-the-scalar-lowering-family-from-the-compiler`](remove-the-scalar-lowering-family-from-the-compiler.md) owns the removal and is unblocked by this node being `done`. [The architecture contract](../docs/architecture.md) needed no edit and was checked rather than assumed — `grep -n 'scalar-lowering\|ScalarLowering' docs/architecture.md` returns nothing.

## Current implementation correction (2026-08-09)

Acceptance itself still touched no crate file, but its implementation follow-on
is no longer merely unblocked: `remove-the-scalar-lowering-family-from-the-compiler`
is `done`, and ADR 0105 records the executed removal at implementation status
`complete`. This note preserves the distinction between the acceptance act and
the repository's current delivered state.
