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

## Fact audit — 2026-08-22 at `1fb3675c`

Every Fact re-run at the dispatch base rather than relayed.

**Fact 1 (`system_profiler SPUSBDataType`) — verified.** `system_profiler SPUSBDataType | wc -c` returns `0` at exit 0; `system_profiler SPUSBHostDataType | wc -c` returns `345`. `system_profiler SPUSBDataType | grep -icE 'iphone|ipad'` also returns `0`, so the not-fired reading is the only reading available.

**Fact 2 (nine buried logs) — verified.** The stated command returns exactly the nine named tickets. It now returns none.

**Fact 3 (five more forms) — verified, with two corrections and one addition.**

- *Correction 1: "two censuses that match their own file" overcounts the deferred pool by one.* Only [`measure-complete-explain-demand-and-lossless-compaction-for-full-physical-provider-activity`](measure-complete-explain-demand-and-lossless-compaction-for-full-physical-provider-activity.md) is deferred. The second, [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](calibrate-the-physical-frontier-provider-and-outcome-budgets.md), reads `status: done` and its trigger is recorded **fired** on 2026-08-18, so it is outside this repair's population.
- *Correction 2: the `grep -v tests` defect is worse than reported.* The line filter is real — it strips 1 line of 57 and leaves two `RuleRef::builtin("test.root")` sites standing — but the command is additionally **blind to its own subject**: a semantic rewrite is registered as a `RewriteRuleIdentity`, not a `RuleRef::builtin`, and a fifth production rewrite added to a scratch tree produced `0` hits from it.
- *Addition — a sixth form the audit did not report.* [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md) reproduces through `cargo nextest` filters that match **zero tests**: both stated forms report `0 tests run: 0 passed, 987 skipped`, because nextest's `test()` predicate matches test names while the two names given are integration-test *binaries*.
- The remaining forms are verified as stated: the `target.rs` module-split trap, the unanchored `full (32|provider)` matching `full 32 bytes in the file`, the two rechecks delegating to logs whose last lines name no command, and the `head -3` truncating an 11-line population so that `fn prove_contraction` at line 4400 falls outside the window the claim rests on.

**Fact 4 (162 of 278 entries; 92 tickets) — imprecise.** Measured at this base over the 119 deferred tickets that carry a log, grouping each dated bullet with the indented and fenced lines beneath it and counting a "command" as a backticked span containing a shell verb: **281** dated entries, **160** carrying no command, **90** tickets whose most recent entry carries none. The claimed 278 / 162 / 92 are within one to two percent on each figure; the differences are definitional, not substantive, and the Fact's conclusion stands.

**Scope note on Fact 4's repair.** Historical entries are dated records of what was run on their date, so they are not retrofitted with commands run today. The live obligation is the *current* recheck, which is the 90 latest entries; each now carries a command that was run before it was written down, or an explicit statement that the condition is not mechanically checkable together with what a human must read instead.

## Findings reported, not acted on

Per this ticket's non-goals, these belong to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).

- **A recheck has changed its answer.** [`prove-partition-coverage-for-symbolic-extents`](prove-partition-coverage-for-symbolic-extents.md) records `rg -n 'symbolic_dimension|sourced_tensor' crates/tiler-compiler` as **empty**, and says a non-empty construction site would fire it. It now returns **three** lines, all in `crates/tiler-compiler/src/capability.rs`. Whether they emit a multi-root partition over undetermined extents is the reading that ticket owns.
- **A trigger already reads fired while its ticket is deferred.** [`scope-launch-granularity-optimization-for-the-decode-dominated-regime`](scope-launch-granularity-optimization-for-the-decode-dominated-regime.md) records **fired** on 2026-08-09.
- **Four stale counts and statuses**, each found by running a check rather than reading prose: the governed key census is **50**, not the **47** recorded in three `scope-the-*` entries; `StorageScalar` has **four** variants, not the three recorded in [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md); [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) is `blocked`, not the `awaiting-decision` recorded in [`reconsider-registered-quantitative-capability-axis-schemas`](reconsider-registered-quantitative-capability-axis-schemas.md); and the host OS build is `26A5416b`, not the `26A5406e` recorded in [`keep-the-first-macos-apple9-host-row-on-its-measured-os-build`](keep-the-first-macos-apple9-host-row-on-its-measured-os-build.md). None of the four changes its ticket's verdict.

## Non-goals

Re-deciding whether any trigger has fired — that is [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md). The hand-maintained key census, which is its own ticket. Any edit outside `tickets/`.

## Closes when

No deferred ticket's recheck command is structurally incapable of reporting a firing, each repair has been watched producing the firing answer on a constructed subject, every log sits at the end of its file, and every entry either carries a command that has been run or states why none exists.
