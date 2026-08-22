---
id: size-the-governed-key-census-from-the-type-across-the-deferred-pool
title: Size the governed key census from the type across the deferred pool
status: done
priority: p2
dependencies: []
related: [repair-the-deferred-trigger-checks-that-cannot-report-a-firing, refresh-the-deferred-triggers-whose-stated-reason-is-now-false]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, deferred-triggers, census]
---
## User-visible outcome

The governed-key count that roughly a fifth of the deferred pool triggers on is derived rather than transcribed, so it cannot drift again — and each ticket names the key its own family waits on, so the check can actually be applied.

## Why this exists

Filed 2026-08-22 from the deferred-pool trigger audit, which called this the pool's one **systemic** defect.

**Fact — a hand-maintained count is copy-pasted across twenty tickets and is wrong.** `grep -rl '46 governed' tickets/*.md` returns **20** files, 19 of them deferred. Verified by the coordinator at `754b63fb`. The audit reports the real count is **50** unique keys (and 19 `*_op` functions, not the 18 also asserted). Four of the twenty were corrected once to 47 and are stale again — so this has already drifted twice.

**Fact (reported by the audit, unverified by the coordinator) — seventeen of the nineteen state the verdict without naming the key.** They read as "the family's key is absent from that list" and never say *which* key. **That makes the check unapplicable mechanically at all**, independently of whether the count is right: a reader cannot grep for a key the ticket declines to name.

**This is precisely what AGENTS.md legislates against.** *"Size enumerations from the type, not by hand. `core::mem::variant_count` makes a widened vocabulary a build error at the enumeration rather than a population that silently shrinks while still reporting no collision."* A number transcribed into twenty markdown files is the worst case of the pattern: it drifts silently, and nothing fails when it does.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict. **Re-derive the true key count yourself** with an anchored pattern, and say which unit you report — `grep -c` counts lines, not occurrences, and an unanchored pattern over-matches.
- Decide **by reading** whether these triggers need the count at all. Most state a condition of the form "no named workload requires this family's key". If naming the key is sufficient, the count is decoration that has drifted twice and should go — deleting it is a better repair than correcting it a third time.
- Wherever a count genuinely earns its place, replace the transcription with a derivation a reader can run, and **run it before writing it down**.
- Make each of the seventeen name the key its family waits on. If a family has no key yet, say so explicitly rather than leaving the sentence to imply one exists.
- Preserve retired wording in dated corrections. Expect grep counts **not to shrink**; a shrinking count is a false progress signal, not evidence of success.

## Non-goals

Re-deciding any trigger's verdict; repairing unfireable check commands, which is [`repair-the-deferred-trigger-checks-that-cannot-report-a-firing`](repair-the-deferred-trigger-checks-that-cannot-report-a-firing.md); adding a gate over ticket prose; and any edit outside `tickets/`.

## Closes when

No deferred ticket transcribes a governed-key count it cannot derive, every family trigger names the key it waits on or states that none exists, and the derivation command has been run with its output recorded.
