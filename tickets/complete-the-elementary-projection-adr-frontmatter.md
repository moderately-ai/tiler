---
id: complete-the-elementary-projection-adr-frontmatter
title: Complete the drafted elementary-projection ADR frontmatter to the decision schema
status: done
priority: p1
dependencies: []
related: [admit-the-registered-unary-families-at-the-compiler-request-boundary, land-the-elementary-family-projection-adr]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation, schema]
---
## User-visible outcome

The ADR body drafted inside [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) carries a frontmatter block that satisfies the governed decision schema, so [`land-the-elementary-family-projection-adr`](land-the-elementary-family-projection-adr.md) can transfer it byte-identically into `docs/decisions/` and write a catalog entry the same shape as its ninety-eight neighbours.

## Why this exists

The carrier stopped on 2026-08-04 rather than fork the draft. **Fact — the drafted block omits four fields the schema requires of every decision record and spells `id` in a form the schema forbids.** The draft's frontmatter is `schema`, `id`, `kind`, `title`, `topics`, `decision_status`. [The metadata contract](../docs/document-metadata.md) requires, beyond the common five, `decision_status`, `implementation_status`, `applies_to`, and `evidence` for kind `decision`, states that "Decision and research records also require `catalog_group`", fixes ADR IDs to "the fixed uppercase form `ADR-NNNN`", and forbids empty arrays ("Present arrays are nonempty"). The draft's `id` is `"tiler.decision.elementary-family-projection"`.

**Fact — the corpus is uniform, so this is not a convention the draft may simply differ from.** Reproduce with `cd docs/decisions && for f in 0*.md; do awk '/^---$/{n++; next} n==1{print $1}' "$f"; done | sort | uniq -c | sort -rn`: at `c4b4bdb9` all ninety-eight decision records carry `schema`, `id`, `kind`, `title`, `topics`, `catalog_group`, `decision_status`, `implementation_status`, `applies_to`, and `evidence`; ninety-six carry `ticket`. Every `id` matches `ADR-NNNN` (`grep -h '^id:' docs/decisions/0*.md | grep -cv '^id: "ADR-[0-9]\{4\}"$'` reports `0`). Every H1 is `# NNNN: <title>` (`for f in docs/decisions/0*.md; do grep -m1 '^# ' "$f"; done | grep -cv '^# [0-9]\{4\}: '` reports `0`).

**Inference — the catalog entry is unwritable without two of the missing fields, so landing the draft as it stands is half a step rather than a cheap one.** `docs/decisions/README.md` renders each entry as title, status, `contracts:` (from `applies_to`), and `evidence:` (from `evidence`) — see the 0095 through 0098 entries. A carrier holding a body with neither field cannot produce the shape the brief asks for, and the fields are needed before acceptance in any case: "An accepted decision has at least one `applies_to` contract and one `evidence` research record."

## Required delivery

Amend the fenced block under *Drafted ADR body, to be landed byte-identically* in the source ticket, in place, dating the amendment beside it so the draft stays attributable. The body's prose is not this ticket's to touch: only the frontmatter block and the H1's number prefix.

- **`catalog_group`.** One of the seven controlled values. `foundation-semantics-extensions` and `numerical-operations` are both defensible for a decision about projecting a registered family's per-point body into the physical node vocabulary; pick one and say why, because the value is a stable coarse catalog location rather than a topic.
- **`implementation_status`.** The decided route is implemented for one family (`tiler::silu-f32@1`, landed `3baa4718`) and not for the two structural families, which is `partial` on the face of it — confirm against the field's own definition ("the highest implementation maturity the record's own decided behaviour has reached") rather than assuming.
- **`applies_to`.** The normative contracts this decision governs, from the fifteen `tiler.contract.*` IDs in the corpus. The candidates worth testing are `tiler.contract.operation-extensions`, `tiler.contract.optimizer`, and `tiler.contract.ir`; a contract named here should have a sentence the decision actually binds.
- **`evidence`.** **This is the field that may not have an honest target, and it is why this is a ticket rather than a carrier's stated exception.** `evidence` admits only a `research` record. The decision's actual ground is the deriving ticket's implementation and its perturbation table, which is what `ticket:` is for. [The L3′ non-linear derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) and [the Metal elementary-function accuracy record](../docs/research/numerics/metal-elementary-function-accuracy.md) ground the *Context's* premise — the pinned `x / (1 + Exp(-x))` composition and which of its elements round — but neither reasons about projecting a body from one shared statement, which is the Decision. Either establish that one of them is genuine evidence for what this record decides, or record that no research record grounds it and take the schema question to Tom. Do not name a record the decision does not rest on: a fabricated traceability edge reads as authority to every later reader.
- **`ticket`.** `admit-the-registered-unary-families-at-the-compiler-request-boundary`, the deriving ticket. Optional by the schema and present on ninety-six of ninety-eight; there is no reason for this to be the third exception.
- **`id` and the H1 number.** Write the next free number at the time of the amendment (`ADR-0099` and `# 0099: …` as of `c4b4bdb9`). The carrier's licence to adjust a number it finds taken extends to both, and the carrier records the adjustment as a stated exception.

`decision_status` stays `proposed`. Nothing in the deriving work relayed an acceptance and this ticket relays none.

## Non-goals

Re-deriving or rewording the decision. The elimination is the deriving ticket's and the body states it; this ticket completes metadata and nothing else.

## Closes when

The fenced draft's frontmatter satisfies every rule in [the metadata contract](../docs/document-metadata.md) for `kind: decision`, each chosen value has its ground recorded beside the amendment, and `evidence` either names a research record the decision rests on or the schema question is with Tom with the derivation stated.

## Outcome, 2026-08-04

**The draft is schema-valid and landable, `evidence` names one research record the decision genuinely rests on, and nothing went to Tom.** The amendment is in [the deriving ticket](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) under *Amendment 2026-08-04 — the frontmatter block completed to the decision schema, field by field*, which carries every value's derivation, the reproduce command for each schema rule, and the failing control each check was shown against. The body's prose is untouched, so the carrier's byte-identical transfer is still a transfer.

The landed block is `schema`, `id: "ADR-0099"`, `kind`, `title`, `topics`, `catalog_group: "physical-planning-lowering"`, `decision_status: "proposed"`, `implementation_status: "partial"`, `applies_to: ["tiler.contract.optimizer", "tiler.contract.ir"]`, `evidence: ["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]`, `ticket`, with the heading `# 0099: …`.

### The evidence disposition, which is the one this ticket was filed for

**The carrier's elimination is upheld for the Metal record and refuted for the L3′ record, and both halves are derivations rather than preferences.** The [Metal elementary-function accuracy record](../docs/research/numerics/metal-elementary-function-accuracy.md) is **not** named: this record states no accuracy claim, and the machinery consuming Table 8.1 is `crates/tiler-compiler/src/target/accuracy.rs`'s obligation assessment, which the decision neither states nor governs. The [L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) **is** named, because the carrier's test — does the record reason about the projection? — is the test for `adopted_by` rather than for `evidence`. [The metadata contract](../docs/document-metadata.md#typed-relationships) states the predicates are independent, the corpus uses the edge that way (ADRs 0055, 0080, and 0095 cite `tiler.research.numerics.reduction-semantics-and-legality`, whose `adopted_by` names only 0012, 0013, 0014, 0022, and 0025), and the decision's rejection of a per-family node rests on a measurement the L3′ record carries and the implementing code cites at its exact argument, `0xc2b00000`.

**Candidate (b) was eliminated rather than skipped.** Authoring a research record to fill a required field, after the decision it would justify has landed, restates the deriving ticket's derivation under a different kind and adds an authority nobody consulted; it is also unnecessary once a genuine edge exists. No ticket is filed for it. The refutation that would reopen it is named in the amendment: a demonstration that the rounding-explicitness ground is not load-bearing for the Decision.

**One judgment departs from this ticket's own framing, deliberately.** Required delivery offered `foundation-semantics-extensions` and `numerical-operations` as the two defensible `catalog_group` values. Both are eliminated on what the record *decides* rather than what it is about — it pins no formula and admits no accuracy contract, and it touches no semantic registration and mints no extension seam — and `physical-planning-lowering` survives, which is where the corpus already files the four optimizer-governing decisions this one composes with (0007, 0043, 0069, 0073). The elimination is stated in the amendment so a reader can refute it rather than only the conclusion.

### Graph maintenance

- [`land-the-elementary-family-projection-adr`](land-the-elementary-family-projection-adr.md) moved `blocked` → `todo` and its stale claim was released; its *Unblocked 2026-08-04* section discharges each fact its stop recorded, preserves the stop's rationale, and marks the single inference the amendment refutes. It is **not** dispatchable yet and should not be: its dependency edge on this ticket holds it until this ticket is `done`, which `tkt ready` confirms by omitting it. One stale assertion in its Required delivery — a filename spelling the title's apostrophe as `-s-`, against the corpus's own handling in 0092 and 0098 — was corrected in the same edit.
- **No new ticket is filed, and the two candidates were tested.** The `evidence` edge is unqualified in the ADR body because the body is fixed; the qualification is the amendment note, reachable from the landed record through its `ticket:` field, which is what that field is for. And the optimizer contract's stale claim that all three families refuse is already owned by [`correct-the-optimizer-stage-generality-claims-for-the-admitted-activation`](correct-the-optimizer-stage-generality-claims-for-the-admitted-activation.md) at `todo` under `contracts/optimizer`; naming `tiler.contract.optimizer` in `applies_to` makes that correction more load-bearing but files nothing new.
