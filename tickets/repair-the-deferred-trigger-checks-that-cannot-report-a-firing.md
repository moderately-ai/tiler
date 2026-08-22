---
id: repair-the-deferred-trigger-checks-that-cannot-report-a-firing
title: Repair the deferred trigger checks that cannot report a firing
status: in-progress
priority: p2
dependencies: []
related: [refresh-the-deferred-triggers-whose-stated-reason-is-now-false, size-the-governed-key-census-from-the-type-across-the-deferred-pool]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, deferred-triggers, unfireable-checks]
claimed_from: todo
assignee: worker-triggerfix
lease_expires_at: 1787435006
---
## User-visible outcome

Every deferred ticket's recheck command can return the answer that would reopen it, so "not fired" means the condition was tested rather than that the test was incapable of saying anything else.

## Why this exists

Filed 2026-08-22 from the deferred-pool trigger audit. AGENTS.md already names this class — *"Verify that a check reaches its subject at all"* and *"Before trusting a check, state what it would take for it to say no, and confirm that case is reachable"* — and the audit found the pool full of it.

**Fact — the sharpest one polls a data type macOS no longer has.** `measure-apple-numerics-on-physical-ios-device` gates on a physical iOS device being attached, and its only mechanical check is `system_profiler SPUSBDataType`. Verified by the coordinator at `754b63fb`: that command emits **0 bytes at exit 0**, because the data type was renamed — `system_profiler SPUSBHostDataType` emits **345**. Empty is exactly the not-fired reading, so **the trigger cannot report a device ever, no matter what is plugged in.**

**Fact — nine deferred tickets do not end with their trigger log**, contrary to the coordinator's briefed premise that the structural obligation was met. Each buries it behind a later `## Closes when` or `## Fact audit`. Two of the audit's ranked findings are among them, which is plausibly *why* they went stale: a sweep reading the file tail never sees their log. Verified: the command below returns **9**.

```sh
for f in $(grep -l '^status: deferred$' tickets/*.md); do awk '/^## Trigger check log/{p=1} p' "$f" | grep -q '^## [^T]' && basename "$f" .md; done
```

**Fact (reported by the audit, unverified by the coordinator) — the same shape recurs in at least five more forms.** A recheck grepping `crates/tiler-compiler/src/target.rs` for `CapabilityAxis`, which **moved to `target/feasibility.rs`** — the named file still exists, so a new axis leaves the grep empty (the module-split trap, applied to a check). A `grep -v tests` that filters *lines* rather than modules, letting `RuleRef::builtin("test.root")` survive because it says `test` not `tests` — a failure already documented inside another ticket's log and never propagated. Two censuses that **match their own file**. An unanchored `full (32|provider)` that matches a digest byte count. Two rechecks that delegate to logs naming no command at all. And a `head -3` that truncates the population it claims to bound.

**Fact (reported) — 162 of 278 dated entries carry no backticked command**, and **92 tickets' most recent entry has none**. AGENTS.md requires one.

## Required work

- Re-audit every Fact at your base with a per-Fact verdict, and **re-run each named command yourself**; the coordinator verified the first two and none of the rest.
- For each unfireable check: repair it so the firing case is reachable, and **prove it** by constructing the state that should fire it and quoting the output. A repair claimed without that demonstration has not been made — this is the whole subject of the ticket.
- Move the nine buried logs to the end of their tickets. Change no wording while moving; a move and an edit in one diff cannot be reviewed apart.
- Where an entry carries no command, either supply one **and run it before writing it down**, or record explicitly that the condition is not mechanically checkable and say what a human must read instead. An absent command silently defers the obligation; a stated one discharges it.

## Non-goals

Re-deciding whether any trigger has fired — that is [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md). The hand-maintained key census, which is its own ticket. Any edit outside `tickets/`.

## Closes when

No deferred ticket's recheck command is structurally incapable of reporting a firing, each repair has been watched producing the firing answer on a constructed subject, every log sits at the end of its file, and every entry either carries a command that has been run or states why none exists.
