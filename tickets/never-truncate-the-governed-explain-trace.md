---
id: never-truncate-the-governed-explain-trace
title: The governed compile path now truncates its explain trace
status: done
priority: p0
dependencies: []
related: []
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, explain, correctness]
---
**Fact — found while integrating two branches, on a merged tree, not hypothesised.** `pipeline::tests::every_wired_authority_emits_its_typed_explain_records` fails with one extra rule key in the actual set: `explain.retention` at count 1. That test's own comment calls itself an exhaustive snapshot where "a new authority that stays explain-silent, or one that becomes chatty, fails here", so it caught this by design.

**Fact — what that key means.** `ExplainWriter::append_truncation_summary` (`crates/tiler-compiler/src/explain.rs`) emits an `explain.retention` record carrying `omitted_records` and `omitted_bytes`, and it returns early when `self.omitted_records == 0`. So the key appears **only** when records were dropped. Its presence is the evidence of loss.

**Inference — the governed compile path is now losing explain output it previously retained.** Neither branch emitted this key alone; the combination does. The per-dimension numerical honourability work added `target.numerics.*` assessments to every region, which pushed the governed trace past its retention bound.

**This is the wrong behaviour and the bound is the thing to fix, not the test.** Tom's standing rule: never drop or silently truncate — fail loud, or remove a limit that is not needed. The truncation summary is a partial mitigation, not compliance: it says *that* something was lost and how much, and there is no way to recover *what*. On the one governed profile, compiling the one supported program, an explain trace that cannot hold its own records is a bound serving no purpose it was designed for.

**Do not close this by adding `explain.retention` to the expected set.** That records the loss as normal and is precisely the silent-degradation this rule forbids. The test is correct as written.

## Scope

Decide between, and implement, one of:

- **Remove or raise the retention bound** so the governed path retains every record. Preferred if the bound protects against nothing reachable — check what it was calibrated against and whether that case exists.
- **Make exceeding it a hard, typed compile failure** rather than a truncation, so a trace is either complete or refused. This keeps a bound for genuinely unbounded traces while removing the silent-loss path.

State which and why the other was rejected. If a bound is retained, state the reachable case it protects against, because a limit with no reachable trigger is one that should be removed.

Also record whether `omitted_bytes`/`omitted_records` should survive at all once truncation cannot happen on a governed path.

## Closes when

The governed compile path emits no `explain.retention` record, the exhaustive-snapshot test passes unmodified in that respect, the choice is stated with the rejected alternative, and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

**Decided: remove the soft retention budget; the retained hard ceiling now fails closed.** `ExplainLimits` (default 256 records / 64 KiB, per-writer configurable) was deleted. The detail bound is now the same `MAX_RECORDS` (4096) / `MAX_CANONICAL_BYTES` (1 MiB) ceiling that `MAX_TRACE_RECORDS`/`MAX_TRACE_CANONICAL_BYTES` are already derived from, and exceeding it returns the new typed `ExplainError::DetailCapacity` instead of dropping the record. A trace is therefore complete or the compilation is refused.

**Why the soft bound went rather than being raised.** It protected nothing the hard ceiling did not already protect. Memory is bounded by the byte ceiling, which is checked incrementally on every push, and `ExplainLimits::new` already rejected any budget above that ceiling — so the soft bound could only ever be *stricter* than a limit that was itself sufficient. Its sole production construction was `ExplainLimits::default()` at one call site (`pipeline.rs`), never tuned by a caller.

**Rejected alternative: raise `ExplainLimits::default()` to the hard ceiling and keep truncating.** That is nominally this ticket's first option and it would have made the failing test pass with a three-line change. It was rejected because it moves the threshold without removing the silent-loss path: any program large enough would still drop records and report only a count. The ticket's own standard is that a summary saying *that* something was lost is not compliance, and a bound whose behaviour on breach is a silent drop is wrong at every threshold, not just this one.

**The reachable case the retained bound protects against.** Candidate enumeration is combinatorial — the governed serial-sum program already emits 17 `region.candidate.v1` records and 12 `fusion.legality.v1` records — so record count grows with program size and is not caller-bounded. 1 MiB of canonical detail is a real ceiling on how much a single compilation may retain. It is kept, as a refusal.

**`omitted_records`/`omitted_bytes` do not survive, and neither does the machinery built around them.** Once a detail record cannot be dropped, each of these describes a state that cannot occur, so all were removed rather than left as unconstructible vocabulary implying a behaviour the compiler no longer has: `ExplainEvent::Truncated`, `ExplainDisposition::Truncated`, the `explain.retention` rule and `append_truncation_summary`, `TerminalCauseKind::Omitted`, and `ExplainEvent::CausalBridge` — the bridge existed only to re-materialize a cause whose detail record had been dropped. `push_detail` now returns `ExplainRecordId` rather than `Option<ExplainRecordId>`, and the `Option` threading that carried a possible drop through `pipeline.rs`, `normalize.rs`, and `region.rs` collapsed with it. Their canonical encoding tags (event 8 and 9, disposition 13) are left as gaps so no surviving record's encoding moved.

**One `Option` was deliberately kept.** `RegionFormationRecords::whole_program` and the alternative-id lookup in `record_cost_and_selection` stay optional: a program may genuinely have no whole-program candidate, and an id lookup may genuinely miss. Those `None`s are facts about the candidate set, not records that went missing — verified by reading each site rather than sweeping the type.

**Measured.** `pipeline::tests::every_wired_authority_emits_its_typed_explain_records` passes **unmodified** — confirmed by `git diff` over `pipeline.rs` showing no change within that test or to the string `explain.retention`. `explain::tests::terminal_ledger_rejects_duplicates_unknowns_and_max_detail_pressure` gained the regression assertion: pushing one detail past `MAX_RECORDS` returns `Err(ExplainError::DetailCapacity)`. Six truncation-dependent tests were rewritten to their surviving subject; two whose subject was the drop path itself (omitted-cause bridging) were replaced by tests that the cause is cited directly. 205/205 `tiler-compiler` tests pass.

**Gate.** `uv run --locked python scripts/check_repository.py` passes.
