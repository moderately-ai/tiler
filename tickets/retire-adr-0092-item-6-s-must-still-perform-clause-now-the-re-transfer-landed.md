---
id: retire-adr-0092-item-6-s-must-still-perform-clause-now-the-re-transfer-landed
title: Retire ADR 0092 item 6 s must-still-perform clause now the re-transfer landed
status: done
priority: p2
dependencies: []
related: [re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork, correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation, transfer]
---

ADR 0092's item-6 correction note ends with a clause naming work that has since been done. It went stale **the moment the re-transfer merged**, which is what makes it worth a ticket rather than a shrug: the note predicted an obligation, the obligation was discharged, and nothing brought the two back into agreement. That is the same decay pattern this ADR has now produced three times.

## Facts, coordinator-verified at the merge that discharged it

**Fact.** The clause is anchored by `which a branch holding` — the note ends by saying the re-transfer must still be performed by a branch holding `research/runtime`.

**Fact.** It has been performed. `re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork` spliced item 6 from the ADR into the span programmatically rather than retyping it, and `cmp` over the compared region moved from `differ: char 3051, line 19` to exit 0 — byte-identity restored. The failing `cmp` was taken on the real subject before the clean one was believed, so the comparison was shown capable of saying no.

**Fact.** The span's fence is intact and no in-fence link was re-rooted. The worker bounded the fence from **both** sides: a link broken after the closing fence fails (`no tracked file or directory at tickets/perturbation-no-such-ticket-after-the-fence.md`, exit 1), and a link broken inside it is silently suppressed (exit 0, zero mentions). A one-sided perturbation could not have shown that.

## What closes this

The clause retired — restated to record that the re-transfer landed on 2026-08-08 rather than that it remains owed. Keep the retired wording quoted inside a dated correction, this repository's convention, and note the hazard that creates: a retired sentence quoted verbatim **stays greppable**, so a later grep hit proves the string is present, not that the obligation stands.

**While you are in the note, check its siblings.** This ADR has now carried three claims about its source record that decayed the same way — the pre-rename spelling, the compiled-or-measured boundary, and this one. Two were repaired this week. **Enumerate every remaining sentence in ADR 0092 that asserts something about `docs/research/runtime/backend-scoped-route-requirement-answers.md` and give each a verdict**, naming the count so a clean result is distinguishable from an unexamined one. Start from the prior census rather than repeating it, and say which of its verdicts you re-checked. **Corrected 2026-08-08 by the worker:** this sentence read "A prior worker enumerated 38 checkable claims and mechanically verified 21", and neither number is supported anywhere in the tree or in any commit body — see the per-Fact audit below. The prior census is **17 assertions, 2 false**, in [`correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling`](correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling.md).

**Do not change what ADR 0092 decides.** The bolded decision clauses and "Every non-dispatching use is unaffected" have survived every repair in this chain byte-unchanged, and should survive this one.

## The general question underneath

An ADR that names a follow-up obligation has no mechanism telling it when the obligation is met. Nothing here checks it, and nothing could cheaply — `AGENTS.md` records that a mechanical check does not discharge a reading obligation. If you can state a convention that makes such clauses self-limiting — a date, a naming of the ticket that owes it, or a form that reads as history rather than as a live claim — say so; that generalization is worth more than this one repair. Do not build a checker for it.

## Per-Fact audit at `68ba010a`, by the worker

**Fact 1 — verified.** The anchor `which a branch holding` occurred exactly once under `docs/` at this base, at the end of ADR 0092's item-6 correction note, and `grep -rn 'must still perform' docs/` returned the same single line. Both anchors were run before being relied on. After the repair each returns one hit, inside the new dated note.

**Fact 2 — verified, and re-established independently rather than carried from the worker's report.** `23746b12` ("Re-transfer the ADR 0092 span after the item 6 restatement") is an ancestor of this base. Byte-identity was re-derived here rather than believed: the ADR's `## Context` through its Traceability paragraph, with the four dated correction notes dropped and blank runs collapsed, is character-identical to the record's `### Context` through its Traceability paragraph with `###` mapped to `##` — 51 lines each. The comparison was shown capable of saying no by perturbing the *subject*, appending one character inside decision item 1 of the span, which turned the result to DIFFER; the assertion was not touched.

**Fact 3 — verified in substance, not re-perturbed.** The fence opens at the record's line 362 (` ```text `) and closes at 418, and the transferred region 367–417 lies wholly inside it. The eight in-fence links are still spelled for `docs/decisions/`, and `./check-citations.sh` exits 0 tree-wide. The two-sided fence perturbation the previous worker ran was not repeated; nothing in this ticket touched the fence, and the diff is confined to `docs/decisions/`.

**The ticket's dispatch claim about the prior census is unsupported and is corrected here.** The body above states that "a prior worker enumerated 38 checkable claims and mechanically verified 21". No such census exists anywhere in the tree or in any commit message: `grep -rn '38 checkable\|checkable claim'` over `*.md` returns only this ticket's own sentence, and a scan of every commit body matching `0092` finds neither number. The census that does exist is **17 assertions across 9 passages, 2 false**, recorded at `580ad377` in [`correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling`](correct-adr-0092-s-false-claim-about-the-drafted-span-s-type-spelling.md) under the heading `Neighbour census — 17 assertions checked, 2 false`. The census below starts from that one.

## Census — 33 assertions about the source record, 2 false, 2 imprecise

ADR 0092 asserts **33** distinct checkable things about `docs/research/runtime/backend-scoped-route-requirement-answers.md`, across **10** passages. Twenty-nine verified at this base. Two were false and both are repaired in this ticket's commit; two are imprecise and neither is repaired, for the reasons given.

| # | Passage | Assertion | Verdict |
|---|---|---|---|
| 1–2 | Status, landing | the b1/b2 derivation is that record; the nine decisions are transferred verbatim and unreworded | verified — the record carries `## The elimination: b1 against b2`, and the transfer is character-identical as above |
| 3–6 | Status, "what is still Tom's" | the record lists seven public-boundary items under the named heading; ADR 0075's mechanical test fires for the first alone and AGENTS.md's clause carries the other six; item 2 is the acceptance act for decision item 6 | verified item for item — with the nuance that the heading numbers **nine**, items 8 and 9 sitting under `**No approval required.**`, so seven is the count of items that are Tom's rather than the count of numbered rows |
| 7–10 | derivation paragraph | the witnesses, eliminations, both worked examples, the b1a/b1b/b1c split, the 5b/5c defect and the measurement boundary live there; nothing was compiled or measured when derived; the cited prototype proves decision logic and not reachability; two of the four shapes have stopped being reservations | verified against the record's own sections and its measurement-boundary bullet |
| 11–14 | 2026-08-08 measurement-boundary note | the paragraph repeated a form the record no longer carries; the record's bullet has carried the base qualifier and the exception since `b8dfd7e4`; `git merge-base --is-ancestor b8dfd7e4 7db46ae9` exits non-zero; the tree publishes two of the four items and none of the other two | verified — both quoted strings are present at `b8dfd7e4` and the first is absent at `b8dfd7e4^`; the merge-base call exits 1; `applicability.rs` declares `AppleGpuFamilyConstant` and `observe_highest_gpu_family`, and the decision-half grep is empty |
| 15–16 | item-8 note | the drafted body carries the corrected spelling because the repair landed here first; the convention is the record's own, stated beside its span | verified |
| 17 | 2026-08-08 note on item 8 | the retired sentence "read" `The drafted body in the source record deliberately still carries the pre-rename spelling` | **imprecise** — the retired sentence carried an inline link mid-phrase, so this is the link-stripped reading rather than the bytes. Harmless as prose, fatal as an anchor; not repaired, because rewriting a quotation of withdrawn text to restore its markup makes it less readable without making it more true |
| 18–21 | 2026-08-08 note on item 8 | it was true on 2026-08-05; the 2026-08-06 re-transfer settled it with the alternatives drift; the record's one remaining `ResourceFloor` is the dated note rather than the span; the convention did not move | verified — `21485eed` wrote the note on 2026-08-05, `67abe1da` re-transferred on 2026-08-06 and is the single commit carrying both drifts; `grep -n 'ResourceFloor\|RouteResourceRequirement'` returns lines 247, 358 and 389, the sole `ResourceFloor` at 358, outside the fence that opens at 362 |
| 22–23 | item-6 note | the record's public-boundary item 2 was corrected the same day on a parallel branch; the trap is named in the record's own analysis | verified — both anchors resolve under `./check-citations.sh` |
| 24 | item-6 note | the re-transfer "must still perform" | **false** — performed at `23746b12`. Repaired: the clause now names the ticket, the date and the commit, and the retired wording is quoted in a dated note that says so |
| 25 | Traceability | the record is the derivation, the eliminations, both worked examples, the public-boundary list, and the measurement boundary | verified |
| 26 | Traceability | "Three links the transferred paragraph does not carry" | **imprecise** — the transferred paragraph carries none of them, which is the load-bearing half and is true; but five links follow, across three bolded groups. Not repaired: "three" is defensible if the two acceptance-provenance citations are not among the additions the clause counts, and choosing between the readings is the author's call rather than a worker's guess |
| 27–31 | Open questions | five deferrals with closing evidence and triggers; one now closed, filed as `close-the-metal-gpu-family-out-of-crate-total-map`, `done`, repaired at `662d9be`; the remaining four restated subject for subject; the landing-order framing; the 2026-08-05 note's claim that the paragraph is this record's own text and outside the transfer | verified — five bullets under `## Deferrals, each with its closing evidence and trigger`, the ticket reads `status: done`, `662d9be` is "Make a new Apple GPU family stop the build rather than a device", and the span holds no Open questions section |
| 32 | Implementation boundary | `AppleGpuFamilyConstant` carries an `isize` rather than the record's sketched `i64` | verified — the record's sketch line still spells `i64` and carries its own supersession comment; the crate spells `isize` |
| 33 | Implementation boundary | the record's open questions "record as *live today and independent of it*" | **false** — a fourth instance of the identical decay, repaired here |

**Which of the prior census's verdicts were re-checked: all seventeen.** Its landing pair, acceptance-scope trio, derivation trio, item-8 trio, Traceability pair, Open-questions pair and open-questions-correction singleton map onto rows 1–2, 3–6, 7–10, 15/16/20, 25–26, 27–29 and 31 above, and each was re-read at this base rather than carried. Its two false findings are confirmed repaired: the item-8 spelling claim now describes the tree, and the measurement-boundary repetition now agrees with the bullet it repeats. One disagreement: the prior census counted the 2026-08-06 alternatives note as a passage asserting something about the record. Re-read in full here, that note asserts things about `prototypes/candle-metal-adapter`, `prototypes/serial-sum-run` and convention 5b, and nothing about the record, so it is outside this population.

**Why the prior census missed row 33, which matters more than the row.** Its nine passages did not include the Implementation boundary at all — the enumeration was built by walking the passages a reader associates with the source record, and a whole section fell outside that association. Row 33 is the same fork as row 11: written accurately at `7db46ae9`, where the record's deferral bullet still read "is live today and independent of this design"; falsified by `b8dfd7e4` on a parallel branch the same day; and left standing when `6228b085` repaired the sibling sentence two sections away. It also contradicted this record's own Open questions section, which has called the defect closed since acceptance. **A passage-based enumeration cannot report the sections it never listed**, which is why this census states its population as a whole-file walk and names its count.

## The convention this generalizes to

**An ADR clause naming work still owed must name the ticket that owes it, by link, and must never name a scope, a branch or a capability.**

The mechanism of the defect just repaired is exactly that the clause named `research/runtime`. A scope is a permission, not a work item: it has no state, no owner and no terminal transition, so no reading of the tree can make a clause conditioned on one become false. It could only ever be discharged by someone remembering. A linked ticket has all three properties, and it moves the "is this still owed" question to the one place the repository already keeps work state, where `tkt` answers it in a command.

Three consequences, and the third is why this is a convention rather than a checker.

1. **The clause reads as a citation instead of an instruction.** "…, owed by [`ticket`](…)" makes the record point at work; "a branch holding X must still perform" makes the record issue an order to whoever is reading, which is the form that cannot expire. On landing the same clause becomes history — "which [`ticket`](…) performed on <date> at `<sha>`" — and history does not rot, because it was never a claim about now.
2. **`make citations` keeps the pointer valid and deliberately says nothing about the obligation.** A ticket link must resolve or the gate fails, so the pointer cannot decay into a bare name; the checker asserts nothing about whether the ticket is open, which is the correct division of labour. The mechanical floor keeps the citation honest and the reading decides whether the obligation stands.
3. **No checker should be added, and this is a reason rather than a concession.** `AGENTS.md` records that a mechanical check does not discharge a reading obligation, and a check that tried to read obligation state across two documents would be one more artifact that quietly stops working — the failure mode this corpus has already met in a fixture that went inert when its host ticket was closed. What makes the convention self-limiting is not enforcement but the form: a citation of a work item cannot masquerade as a live claim.

**The retirement half is inseparable from it.** Because this corpus quotes retired wording rather than deleting it, every repair permanently increases the number of greppable false strings. A dated note must therefore state, in itself, that a grep hit for its quotation lands inside the note. Both notes added by this ticket do, and so does [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md) where it carries the same retired sentence. Without that sentence the convention manufactures a fresh false positive with every repair it performs.

**Applied here.** ADR 0092's one remaining live obligation — decision item 6's amendment to `docs/architecture.md`, public-boundary item 2 — now names `accept-the-public-route-requirement-answer-boundary` as the node that owes it, in the implementation boundary, instead of leaving a reader to infer the owner from the scope that could perform the edit.
