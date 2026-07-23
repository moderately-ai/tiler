---
id: decouple-status-prose-from-the-live-board
title: Decouple status prose from the live ticket board
status: todo
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, governance, maintenance]
---
`docs/status.md` names specific in-flight tickets as "the immediate compiler frontier" and `docs/roadmap.md` enumerates a dependency chain by ticket id. Both rot on every merge: within one working session the named frontier went stale twice, and today two of the three named authorities completed while a fourth listed as downstream also completed. `docs/status.md` currently carries nine ticket links.

This contradicts the repository's own stated authority split. `docs/work-tracking.md` says "Ticketsplease is the live work graph; Markdown status pages are not a duplicate board", and `tkt rollup` already reports the ready frontier and blocked set on demand. Enumerating dispatchable ticket ids in a governed contract duplicates that authority and guarantees drift.

Restructure both documents so they describe the durable phase, boundaries, and evidence state — which is genuinely theirs to own — and defer the dispatchable frontier to the board. Keep links to tickets that are durable references (accepted scope gates, milestone exits, deferred triggers); remove or generalize links whose only purpose is naming what happens to be dispatchable now. Where a frontier statement is genuinely useful to a reader, phrase it so it stays true across a wave, or point at the `tkt` commands work-tracking.md already documents.

As part of the same pass, correct the current staleness: the typed explain authority, the generic IndexRegion reference oracle, and bounded semantic normalization are complete and merged; generic region formation is in flight. Do not simply re-enumerate the new frontier, or this ticket will need filing again next wave.

Run the full documentation gate before completion.
