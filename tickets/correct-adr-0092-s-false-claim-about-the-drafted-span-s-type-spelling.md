---
id: correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling
title: Correct ADR 0092 s false claim about the drafted span s type spelling
status: todo
priority: p2
dependencies: []
related: [repair-the-eight-dangling-links-in-the-runtime-route-answer-record, decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation, citations]
---

ADR 0092 states as fact something about its own source record that stopped being true when that record was re-transferred. The sentence is an accepted decision's description of another document, so it is exactly the kind of claim nobody re-reads.

## Facts, verified 2026-08-08 by the coordinator at `bdbeb2b5`

**Fact.** `docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md` asserts, in its item-8 correction note, anchored by the phrase *"The drafted body in the source record deliberately still carries the pre-rename spelling."*

**Fact.** It does not. `docs/research/runtime/backend-scoped-route-requirement-answers.md` carries `RouteResourceRequirement` **three** times and `ResourceFloor` once; reproduce with `grep -c`. The span reads the post-rename spelling.

**Fact.** The record's own dated note explains why: on 2026-08-06 both drifts were corrected **in ADR 0092 first** and the span was then re-transferred from it, so the two documents are byte-identical again across both passages. The ADR's sentence was written 2026-08-05 and the re-transfer superseded it a day later without anyone revisiting the sentence that described the old state.

**Inference.** This is the same class as the [architecture citation defect](correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier.md), reached from the other direction: there a pin resolved while misquoting; here a prose claim about a sibling document silently inverted when the sibling changed. Neither is reachable by any mechanical check the repository has, because both documents are well-formed and every link resolves.

## What closes this

The item-8 note restated to describe the tree as it is — the ADR was corrected first and the span re-transferred from it — rather than as it was for one day. Keep the retired sentence quoted inside a dated correction, as the sibling record does, so a reader who remembers the old claim can see it was withdrawn rather than wonder whether they misread.

**Check the rest of the note's factual claims while you are in it.** This one decayed because a correction landed in the other document and nothing brought the pair back into agreement; any neighbouring sentence describing the source record has the same exposure. Name the count you checked, so a clean result is distinguishable from an unexamined one.

Note that this sentence cites the convention that a drift is *recorded beside the span rather than repaired inside it* — the convention `decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span` settled by fencing. Read that ticket's outcome before restating the rationale, so the corrected note does not re-ground itself in a version of the convention that no longer holds.
