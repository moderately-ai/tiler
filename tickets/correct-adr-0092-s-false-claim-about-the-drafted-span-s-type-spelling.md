---
id: correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling
title: Correct ADR 0092 s false claim about the drafted span s type spelling
status: in-progress
priority: p2
dependencies: []
related: [repair-the-eight-dangling-links-in-the-runtime-route-answer-record, decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation, citations]
claimed_from: todo
assignee: coord
lease_expires_at: 1786169827
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

## Per-Fact audit at `0f319ec8`, by the worker

The coordinator verified the Facts above at `bdbeb2b5`. Re-read here at `0f319ec8`; the substance of all three survives and two carry a reproduction that does not do what it says.

**Fact 1 — imprecise anchor, true claim.** ADR 0092 did assert it, but the anchor as quoted — *"The drafted body in the source record deliberately still carries the pre-rename spelling"* — never appeared in the file as a contiguous string, because the ADR spells it with a link in the middle: `The drafted body in [the source record](../research/…) deliberately still carries…`. A grep for the quoted phrase returns nothing and reads as absence rather than as a citation that needs re-locating. The searchable anchor is `deliberately still carries the pre-rename spelling`. Retired by `accaed84`, so the anchor now resolves only in that commit's dated correction, where it is quoted as withdrawn.

**Fact 2 — verified in substance; its reproduction is imprecise.** `grep -c` counts *matching lines*, not occurrences, so "carries `RouteResourceRequirement` **three** times" is a line count that coincides with the occurrence count here rather than being measured. Counted with `grep -o … | wc -l`: 3 and 1, so the numbers hold. What the numbers do not show, and what the claim turns on, is *where*: the text fence added by `91f67cc5` opens at line 356 and closes at line 412, so the span's item 8 (line 383) reads `RouteResourceRequirement`, while the record's one `ResourceFloor` (line 352) is outside the fence in the dated note that records the retired spelling. Reproduce with `grep -n 'ResourceFloor\|RouteResourceRequirement' docs/research/runtime/backend-scoped-route-requirement-answers.md`, read against the fence bounds from a grep for the fence markers in the same file. Item 8 is `cmp`-identical between the two documents.

**Fact 3 — verified.** The record's dated paragraph, anchored `were settled together by one re-transfer on 2026-08-06`, states both drifts were corrected in ADR 0092 first and the span re-transferred. Dates confirmed from history rather than from the prose: the ADR's item-8 correction landed at `21485eed` on 2026-08-05 and the alternatives repair at `67abe1da` on 2026-08-06, which is the commit that also re-transferred the span.

**Inference — verified, and it recurred once more in the same file.** See the neighbour census below.

## Neighbour census — 17 assertions checked, 2 false

ADR 0092 asserts 17 distinct things about `docs/research/runtime/backend-scoped-route-requirement-answers.md`, across 9 passages: the landing paragraph (2), the acceptance-scope paragraph (3), the derivation paragraph (3), the item-8 note (3), the alternatives note (1), Traceability (2), Open questions (2), and the open-questions correction (1). Fifteen verified against the record and the tree at this base. Two were false and both are repaired:

1. The item-8 spelling claim, this ticket's subject — repaired at `accaed84`.
2. **The derivation paragraph repeated the record's measurement boundary in a form the record no longer carries** — "nothing in this design was compiled or measured … none compiles". The record has read "at `6f7caf3`" plus "Two of the shapes stopped being reservations at `662d9be`" since `b8dfd7e4`, which `git merge-base --is-ancestor b8dfd7e4 7db46ae9` shows is not an ancestor of the paragraph's own acceptance edit. False at this base, not merely stale: `crates/tiler-metal/src/applicability.rs` publishes `AppleGpuFamilyConstant` and `observe_highest_gpu_family`. The ADR's own implementation boundary, re-read at `50409b9`, already said so, so the record contradicted itself. Repaired at `6228b085`, in its own commit so it can be dropped independently of this ticket's stated outcome.

The remaining fifteen include the two most load-bearing: the span is byte-identical to the ADR's Context-through-Traceability once its `###` headings are demoted to `##`, so "transferred verbatim … unreworded" holds across all nine decisions, the consequences, all six alternatives entries, and the traceability paragraph; and the record does list exactly seven public-boundary items under the heading the ADR names, matching the ADR's enumeration item for item.

**The ADR does not carry the false AGENTS.md grounding that `91f67cc5` struck from the record.** Its note attributes the convention to "That record's own convention" and never to canonical guidance. `grep -c 'span\|repoint' AGENTS.md` returns 0, confirmed at this base, and nothing in the ADR depends on it.
