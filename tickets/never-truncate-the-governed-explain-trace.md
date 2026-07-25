---
id: never-truncate-the-governed-explain-trace
title: The governed compile path now truncates its explain trace
status: in-progress
priority: p0
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler, explain, correctness]
claimed_from: todo
assignee: agent-integrate
lease_expires_at: 1785000222
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
