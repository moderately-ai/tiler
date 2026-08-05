---
id: refresh-adr-0090-source-anchors-after-the-seams-moved
title: Refresh ADR 0090's source anchors after the seams moved
status: todo
priority: p3
dependencies: []
related: [audit-backend-authoring-against-all-thirteen-responsibilities, specify-the-consumer-neutral-backend-provider-composition-contract]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, decisions, backend-providers]
---
## User-visible outcome

ADR 0090's reproductions resolve against the tree a reader runs them on, so a reader checking the record's Facts gets the evidence rather than an unrelated line or an empty result.

## Why this exists

Carrier ticket. The [backend-authoring audit](audit-backend-authoring-against-all-thirteen-responsibilities.md) re-ran ADR 0090's own reproductions at base `51e9374a` and found three anchors that no longer resolve. The audit's scopes reach `docs/research/extensions/` but not `docs/decisions/`, so the research record carries the corrections and the ADR does not. Each item below was verified by reading the cited file, not by search alone.

**The row 4 and row 5 reproduction returns nothing, and the sentence it supports is half stale.** ADR 0090:33 says to reproduce with `grep -n "dyn PhysicalImplementationProvider; 1\|OpaqueCallRegistry::new()" crates/tiler-compiler/src/pipeline/planning.rs`. That command now exits 1 with no output. The hardcoded one-element provider array and the inline empty registry were replaced by `PhysicalAuthorities` (`crates/tiler-compiler/src/frontier.rs:2893`), constructed as `PhysicalAuthorities::governed()` at `crates/tiler-compiler/src/pipeline.rs:604` and consumed at `pipeline/planning.rs:292`. **Row 4's claim survives** — nothing still installs a physical implementation from outside the crate, and `PhysicalImplementationProvider` is `pub(crate)` at `frontier.rs:1234` behind a private `mod frontier` at `lib.rs:24`. **Row 5's claim does not**: `PhysicalAuthorities::composed` (`frontier.rs:2920`) is driven from the compile path at `crates/tiler-compiler/src/pipeline/tests.rs:3449`, and [`register-opaque-calls-on-the-compile-path`](register-opaque-calls-on-the-compile-path.md) is `done`. Production still installs an empty registry, which the type's own documentation records as the ordinary case rather than a gap.

**Item 9's loader citation moved.** ADR 0090:97 cites `crates/tiler-runtime/src/load.rs:309-310` for `refuse_route_requirements` followed by `refuse_deferred`. Those calls are at `load.rs:410-411` at this base, in the same order and with the same comment giving the same reason; `load.rs:309` is now `DecodedProgram::payloads`. The finding is unchanged and only the anchor moved.

**Item 5's population site moved.** ADR 0090:77 cites `crates/tiler-compiler/src/session.rs:1513` for `Compilation::offered_providers` being populated from the lowering registry alone. That population is at `session.rs:2092-2093` at this base. The claim is unchanged and still true.

## Implementation keys

- Re-verify each anchor by reading the file at the base the edit lands on rather than trusting this ticket's line numbers, which are facts about `51e9374a`.
- Correct row 5's sentence rather than only its anchor: "nothing registers one on the compile path" is the part that is now false, and an anchor refresh that left it standing would preserve the wrong claim.
- Prefer an anchor that survives movement where one exists — an item name plus a reproducing command beats a bare line number, which is the pattern the record already uses for its greps.
- Do not restate the audit's maturity table here; the research record owns it and duplicating it would create a second authority over one subject.

## Closes when

Every reproduction in ADR 0090 runs and returns what the sentence beside it claims; row 5's stale sentence is corrected; and no anchor this ticket names still points at an unrelated line.
