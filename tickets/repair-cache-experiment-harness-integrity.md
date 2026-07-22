---
id: repair-cache-experiment-harness-integrity
title: Repair cache experiment harness integrity
status: done
priority: p1
dependencies: []
related: []
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [research, correctness, experiments]
---

Repair the cache crash/race experiment defects found by the fixed-point audit
at `ad6e9f463de6eabad44af47eaddad9317e0935fd`.

## Required outcome

- Give every worker child an overall deadline so a lock or publication
  regression is reported rather than hanging the harness indefinitely.
- Make the documented ten full repetitions at concurrency 32 executable from
  the published entrypoint and retain compact per-run evidence, or narrow the
  report to the one run the harness actually performs.
- Preserve the existing kill-point, corrupt-entry, reader/GC, and unwritable-
  root assertions while making timeout and child-attribution diagnostics
  deterministic.

## Acceptance

Inject a permanently blocked child and prove bounded failure. Run the exact
documented stress procedure and trace every published repetition/count claim to
tracked evidence.

## Outcome

The cache harness now owns every subprocess through a labeled overall deadline,
kills and reaps timed-out children, and verifies the exact diagnostic with an
injected permanently blocked child. Its published `--repetitions` and
`--evidence` options execute complete independent suites and synchronize compact
per-run evidence.

The documented stress-32 suite passed ten complete repetitions on 2026-07-21;
the tracked TSV fixture records all asserted counts and links from both the
spike README and cache research report. The complete repository validation gate
also passed after the repair.
