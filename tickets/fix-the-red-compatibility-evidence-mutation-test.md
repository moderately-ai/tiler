---
id: fix-the-red-compatibility-evidence-mutation-test
title: Fix the red compatibility-evidence mutation test in the apple-targets spike
status: todo
priority: p3
dependencies: []
related: []
scopes: [research/apple-targets]
shared_scopes: []
paths: []
tags: [spike, testing]
---
`spikes/apple-targets/test_probes.py::test_compatibility_evidence_mutations` fails, and has been failing without anyone noticing.

## Fact — reproducible in one command

```sh
uv run --with pytest pytest spikes/apple-targets/test_probes.py -q -k compatibility_evidence_mutations
```

fails with `AssertionError: validator accepted missing provenance field: probe.project_sha256` at `test_probes.py:166`.

**Verified pre-existing.** It fails identically from a clean detached worktree at `origin/main`, so it is not a consequence of any branch in flight. Found while running the suite for `widen-the-f16-operation-vocabulary-to-contraction-and-reassociation`, which is why that ticket does not own it.

## Why it went unnoticed

No `make` target reaches `spikes/`, by the deliberate design `AGENTS.md` records: a spike is a retained measurement, run by hand when someone is working on it. The stated cost of that trade is exactly this — a retained assertion drifts from the source beside it and only running the spike says so. This is the first observed instance, so it is worth keeping the concrete case beside the rule.

## What the test is checking

The negative half of the compatibility-evidence validator: for each provenance key (`probe.*`, `host.*`, `sdk.*`, `tool.*`), delete it and require the validator to reject. It rejects for the others and accepts when `probe.project_sha256` is missing.

## Closes when

Either the validator rejects a record missing `probe.project_sha256` and the test passes, or the field is deliberately optional and the test is corrected to say so with the reason — decided by reading the validator rather than by making the assertion match current behaviour, since the test is the thing claiming the field is load-bearing. Confirm the failure path is still reachable afterwards by deleting a different provenance key and watching it fail.
