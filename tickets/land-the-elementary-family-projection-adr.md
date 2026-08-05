---
id: land-the-elementary-family-projection-adr
title: Land the elementary-family projection ADR
status: in-progress
priority: p1
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary, complete-the-elementary-projection-adr-frontmatter]
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, decisions, carrier]
claimed_from: todo
assignee: agent-adr-carrier2
lease_expires_at: 1785904792
---
## User-visible outcome

The route the compiler took to make a registered elementary family reachable is an accepted decision in `docs/decisions/`, so the next reader of `crates/tiler-compiler/src/elementary.rs` finds the derivation behind it rather than a module comment asserting one.

## Why this is a carrier ticket

**Fact — the deriving ticket's scopes cannot reach the decision record.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md` to `contracts/navigation`. [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) declares `implementation/compiler` exclusively and `project/tickets` shared, so writing the ADR or editing the decision catalog from that branch would be a guard escape. Its body therefore holds the ADR **drafted verbatim-landable**, in the "Drafted ADR body, to be landed byte-identically" section.

**The transfer is byte-identical.** A transfer that edits is a fork. The drafted body carries no traceability section and no `docs/decisions/`-relative links, so it has none of the link tension a drafted body with one would have; copy the fenced block, allocate the next number, and rename the file accordingly.

## Required delivery

- The drafted body landed byte-identically at the next free `docs/decisions/NNNN-…md`, with only the numeric prefix, the `id` frontmatter field, and the H1's number prefix adjusted to the allocated number if the catalog's convention requires it — and if any adjustment is needed, it is recorded here as a stated exception rather than made silently. **The H1 was added to that list on 2026-08-04**, because every one of the ninety-eight records spells its heading `# NNNN: <title>` and the draft's heading carries no number, so an allocation that moved the filename and the `id` but not the heading would leave the record spelling two different identities. **Both now carry `0099` already** — see the resolution section below — so at the time of writing this bullet authorizes an adjustment that is not needed.

  **Correction 2026-08-04 — the filename this bullet stated does not follow the corpus's own handling of a possessive.** It read `NNNN-project-an-elementary-family-s-per-point-body-from-one-shared-statement.md`, spelling the apostrophe as `-s-`. The corpus drops the apostrophe and keeps the `s`: [0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md) renders "the owning backend's vocabulary" as `the-owning-backends-vocabulary`, and [0098](../docs/decisions/0098-state-an-inline-regions-delivery-policy-with-a-named-profile-or-a-family-list.md) renders "an inline region's delivery policy" as `an-inline-regions-delivery-policy`. The mechanical slug is therefore `0099-project-an-elementary-familys-per-point-body-from-one-shared-statement.md`. **The slug is not a strict transform of the title and the carrier may shorten it** — [0088](../docs/decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) drops "consumer" from its own title — so this correction fixes the apostrophe convention rather than fixing the name.
- The catalog block in `docs/decisions/README.md` updated in the same commit as the metadata behind it. That file maps to `contracts/navigation`, so this ticket declares both scopes.
- `decision_status` left at `proposed`. **Acceptance is Tom's and nothing in the deriving work relayed one.** Moving it to accepted, updating the catalog views, and correcting every contract sentence whose truth depended on the old status is a separate step with its own acceptance provenance — who accepted, the date, and the venue.

## Non-goals

Re-deriving the decision. The elimination is recorded in the deriving ticket and the ADR states it; this ticket transfers, it does not re-argue.

## Closes when

The ADR file exists at an allocated number with the drafted body byte-identical, the catalog names it, and `decision_status` is `proposed` with no acceptance claimed.

## Unblocked 2026-08-04 — the draft is schema-valid and the transfer is mechanical again

**The block below is preserved because its facts were true when written and its stop was correct; every one of them is now discharged.** [`complete-the-elementary-projection-adr-frontmatter`](complete-the-elementary-projection-adr-frontmatter.md) amended the fenced draft in place at `116f11ad`, and the amendment note beside the draft records each value's ground. This ticket returns to `todo`; the dependency edge on that ticket is what holds it until the amendment lands, so nothing here shortens the graph.

- **The five absent or malformed fields are present and well-formed.** The block now carries `schema`, `id`, `kind`, `title`, `topics`, `catalog_group`, `decision_status`, `implementation_status`, `applies_to`, `evidence`, and `ticket` — the corpus's ten plus the optional `ticket` — with `id: "ADR-0099"` and the heading `# 0099: …`. Reproduce over the extracted block exactly as the block below does over the corpus; the amendment note carries the extraction one-liner and each check's failing control.
- **`applies_to` is `["tiler.contract.optimizer", "tiler.contract.ir"]`** and `evidence` is `["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]`, so the catalog entry this ticket owes — title, status, `contracts:`, `evidence:` — is writable in the shape its ninety-eight neighbours have.
- **The number question is settled, not deferred.** `0099` was free at `116f11ad` (`ls docs/decisions/0*.md | tail -1` reports 0098, and `grep -rn 'id: "ADR-0' tickets/` reports no other drafted body claiming a number). The licence in Required delivery to adjust a number found taken is unspent and still stands: if `0099` is allocated before the transfer, move the filename, the `id`, and the H1 together and record it as a stated exception.
- **One of this ticket's own facts is refuted rather than discharged, and only one.** The block below states, correctly, that no research record reasons about projecting an elementary body from one shared statement — **that Fact stands and the amendment does not contradict it**. What the amendment refutes is the inference drawn from it, that naming a record would therefore be a false authority edge: `evidence` does not require the record to derive the decision, and [the metadata contract](../docs/document-metadata.md#typed-relationships) states the three predicates are independent, with ADRs 0055, 0080, and 0095 citing a record whose `adopted_by` names none of them. The L3′ record is named because the Decision's rejection of a per-family node rests on a measurement it carries; the Metal record is **not** named, exactly as the carrier's elimination concluded. The full derivation, including the boundary the edge does not cover, is in the amendment note.

## Blocked 2026-08-04 — the draft is not schema-valid, and conforming it is a fork

**Nothing was landed under `docs/decisions/`.** The transfer stops on the dispatch's third stop condition — "anything that would require editing the draft to land it" — because the drafted frontmatter block cannot be written into `docs/decisions/` as it stands without producing the corpus's only schema-invalid decision record. [`complete-the-elementary-projection-adr-frontmatter`](complete-the-elementary-projection-adr-frontmatter.md) carries the amendment and this ticket now depends on it.

**Fact — four required fields are absent and `id` is spelled in a form the schema forbids.** The draft's frontmatter is `schema`, `id`, `kind`, `title`, `topics`, `decision_status`. [The metadata contract](../docs/document-metadata.md) requires `decision_status`, `implementation_status`, `applies_to`, and `evidence` beyond the common five for `kind: decision`; adds "Decision and research records also require `catalog_group`"; fixes ADR IDs to "the fixed uppercase form `ADR-NNNN`" against the draft's `"tiler.decision.elementary-family-projection"`; and forbids the empty-array escape ("Present arrays are nonempty"). So `implementation_status`, `applies_to`, `evidence`, and `catalog_group` are missing and cannot be written as placeholders.

**Fact — the corpus is uniform on all ten fields, so the draft differs from a rule rather than from a habit.** `cd docs/decisions && for f in 0*.md; do awk '/^---$/{n++; next} n==1{print $1}' "$f"; done | sort | uniq -c | sort -rn` at `c4b4bdb9` reports 98 each for `schema`, `id`, `kind`, `title`, `topics`, `catalog_group`, `decision_status`, `implementation_status`, `applies_to`, and `evidence`, and 96 for `ticket`. `grep -h '^id:' docs/decisions/0*.md | grep -cv '^id: "ADR-[0-9]\{4\}"$'` reports `0`, and `for f in docs/decisions/0*.md; do grep -m1 '^# ' "$f"; done | grep -cv '^# [0-9]\{4\}: '` reports `0`.

**Inference — the catalog deliverable is unwritable from this body, so landing it would be half a step.** Every entry in `docs/decisions/README.md` renders `contracts:` from `applies_to` and `evidence:` from `evidence`; a body carrying neither cannot produce the shape. And the fields are required before acceptance regardless — "An accepted decision has at least one `applies_to` contract and one `evidence` research record" — so landing now buys no earlier acceptance, only a defect for the acceptance step to trip over.

**Why the carrier did not simply fill them in.** Required delivery above pre-authorizes exactly two mechanical adjustments, the number and the `id`, and a third was added today on the same mechanical ground. `catalog_group` and `implementation_status` are arguably that class too. `applies_to` and `evidence` are not: they are the record's traceability edges, and `evidence` admits only a `research` target while the decision's actual ground is the deriving ticket's implementation and perturbation table — which is what `ticket:` is for. **Fact — no research record in the corpus reasons about projecting an elementary body from one shared statement.** [The L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) and [the Metal elementary-function accuracy record](../docs/research/numerics/metal-elementary-function-accuracy.md) ground the Context's premise — the pinned `x / (1 + Exp(-x))` and which of its elements round — and neither reasons about the Decision. Naming one anyway would plant a traceability edge that reads as authority; that is a judgment for the drafting side or for Tom, not a carrier's stated exception.

## Reviewer context — the drafted body's facts still hold at `c4b4bdb9`

Checked because three implementation waves landed after the draft was written at `3baa4718`. **No delta requires an edit, and none was made.**

- **`PointwiseF32Node` is still closed and still carries the three nodes the Consequences section names.** Its variants at this commit are `Input`, `Constant`, `Add`, `Multiply`, `Divide`, `Exp`, `Rsqrt` — so "`Exp`, `Divide`, and `Rsqrt` were already nodes" reads true, and the closedness the second alternative is eliminated on is intact.
- **The stage-8 provider generalization (`51042613`) moved no wall.** Its own message states "This widens nothing. The three walls still refuse the same programs; what changed is that each refusal is now a statement a reader can act on." `physical::spell_region` now returns a typed `RegionVocabularyWall` (`FusedPrologueUnspellable`, `PartialFusedProgram`, `PartialCoverage`) for every cover region rather than an empty offer. That is a better-attributed refusal for regions the recognizer did not match, not a new spelling for an access relation, so the Consequences paragraph on reindex and non-scalar broadcast is unaffected.
- **The stage-11 assembly generalization (`9659033b`) and the slice family (`18977fe9`, `277adb02`) likewise touch nothing the body asserts.** `tiler::slice-f32@1` gained a governed identity and was seated in the compiler; the body's claim is about `tiler::reindex-f32@1` and `tiler::broadcast-f32@1`, which still refuse — `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs:329` still carries "`tiler::reindex-f32@1` still refuses: `LogicalAccess` has no reindex map".
- **`ElementwiseFamily` is still `Add`, `Multiply`, `Silu`**, so the one admitted family the `implementation_status` question turns on has not grown.
