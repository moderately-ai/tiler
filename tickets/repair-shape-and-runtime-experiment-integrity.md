---
id: repair-shape-and-runtime-experiment-integrity
title: Repair shape and runtime experiment integrity
status: done
priority: p1
dependencies: []
related: []
scopes: [research/shapes, research/runtime]
shared_scopes: [project/tickets]
paths: [spikes/README.md]
tags: [research, correctness, experiments]
---

Repair shape-evidence and runtime-validation experiment provenance found by the
fixed-point audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd`.

## Required outcome

- Make stable shape measurement entrypoints regenerate or independently verify
  the checked-in summaries through a governed transformation from raw output.
- Derive nightly measurement dates from the actual run and give every compiler
  subprocess an overall deadline. Reconcile stale toolchain text and make
  platform-specific `time`/`stat` prerequisites explicit.
- Publish the runtime semantic-validation benchmark command, mark it as a
  bounded measurement, and retain individual samples plus exact environment
  rather than only medians.
- Make the Candle source audit verify the exact expected commit/revision and
  source cleanliness before accepting the structural pattern.

## Acceptance

From a clean checkout, each documented command must either regenerate the cited
result or compare a fresh result against it, with exact run date, toolchain,
source revision, samples, and timeout behavior. Missing provenance must fail
closed.

## Outcome

Completed on 2026-07-21.

- Stable shape measurements now run through one governed Python transformation
  that retains raw streams and individual samples, derives both checked JSON
  summaries, records exact host/toolchain/input-tree provenance, normalizes
  local paths, and gives every subprocess group a 300-second deadline.
- The nightly measurement derives its UTC date from the run, hashes the exact
  measured input tree, rejects unsupported platforms or missing timer support,
  and kills the whole compiler subprocess group on timeout.
- The semantic-validation benchmark publishes a locked command and retains 76
  individual samples plus derived medians, exact source hash, base revision,
  host, compiler, measurement boundary, and deadline policy.
- The Candle audit now accepts a checkout root and rejects any revision other
  than `31f35b147389700ed2a178ee66a91c3cc25cc80d`, any tracked or untracked
  change, or a missing canonical source path before checking the function.

Verification:

- `uv run --locked python scripts/check_repository.py`
- both stable shape measurement entrypoints and the nightly measurement
- runtime semantic-validation and Candle transition Rust tests
- exact clean Candle source audit at the pinned revision
- `tkt lint` and `git diff --check`
