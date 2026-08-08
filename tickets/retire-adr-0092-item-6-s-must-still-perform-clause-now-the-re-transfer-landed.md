---
id: retire-adr-0092-item-6-s-must-still-perform-clause-now-the-re-transfer-landed
title: Retire ADR 0092 item 6 s must-still-perform clause now the re-transfer landed
status: in-progress
priority: p2
dependencies: []
related: [re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork, correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation, transfer]
claimed_from: todo
assignee: coord
lease_expires_at: 1786176329
---

ADR 0092's item-6 correction note ends with a clause naming work that has since been done. It went stale **the moment the re-transfer merged**, which is what makes it worth a ticket rather than a shrug: the note predicted an obligation, the obligation was discharged, and nothing brought the two back into agreement. That is the same decay pattern this ADR has now produced three times.

## Facts, coordinator-verified at the merge that discharged it

**Fact.** The clause is anchored by `which a branch holding` — the note ends by saying the re-transfer must still be performed by a branch holding `research/runtime`.

**Fact.** It has been performed. `re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork` spliced item 6 from the ADR into the span programmatically rather than retyping it, and `cmp` over the compared region moved from `differ: char 3051, line 19` to exit 0 — byte-identity restored. The failing `cmp` was taken on the real subject before the clean one was believed, so the comparison was shown capable of saying no.

**Fact.** The span's fence is intact and no in-fence link was re-rooted. The worker bounded the fence from **both** sides: a link broken after the closing fence fails (`no tracked file or directory at tickets/perturbation-no-such-ticket-after-the-fence.md`, exit 1), and a link broken inside it is silently suppressed (exit 0, zero mentions). A one-sided perturbation could not have shown that.

## What closes this

The clause retired — restated to record that the re-transfer landed on 2026-08-08 rather than that it remains owed. Keep the retired wording quoted inside a dated correction, this repository's convention, and note the hazard that creates: a retired sentence quoted verbatim **stays greppable**, so a later grep hit proves the string is present, not that the obligation stands.

**While you are in the note, check its siblings.** This ADR has now carried three claims about its source record that decayed the same way — the pre-rename spelling, the compiled-or-measured boundary, and this one. Two were repaired this week. **Enumerate every remaining sentence in ADR 0092 that asserts something about `docs/research/runtime/backend-scoped-route-requirement-answers.md` and give each a verdict**, naming the count so a clean result is distinguishable from an unexamined one. A prior worker enumerated 38 checkable claims and mechanically verified 21; start from that rather than repeating it, and say which of its verdicts you re-checked.

**Do not change what ADR 0092 decides.** The bolded decision clauses and "Every non-dispatching use is unaffected" have survived every repair in this chain byte-unchanged, and should survive this one.

## The general question underneath

An ADR that names a follow-up obligation has no mechanism telling it when the obligation is met. Nothing here checks it, and nothing could cheaply — `AGENTS.md` records that a mechanical check does not discharge a reading obligation. If you can state a convention that makes such clauses self-limiting — a date, a naming of the ticket that owes it, or a form that reads as history rather than as a live claim — say so; that generalization is worth more than this one repair. Do not build a checker for it.
