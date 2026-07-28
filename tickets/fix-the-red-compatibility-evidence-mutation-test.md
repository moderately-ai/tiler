---
id: fix-the-red-compatibility-evidence-mutation-test
title: Fix the red compatibility-evidence mutation test in the apple-targets spike
status: done
priority: p3
dependencies: []
related: []
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
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

## Outcome — the fixture was stale, not the validator (2026-07-27)

**Decided by reading the producer, as this ticket required.** `probe.project_sha256` and `probe.lock_sha256` hashed `pyproject.toml` and `uv.lock` as measurement inputs — stated in `e197176`'s own commit message, which deleted both files with the rest of the Python tooling. `spikes/apple-targets/compatibility_probe.sh` writes neither key today (it writes `source`, `harness`, `validator`, and `input_manifest` digests only), and neither file exists.

So the validator is right to not require them: a record produced now *cannot* carry either, and requiring them would reject every future record while satisfying nothing. The two keys survived only in `test_probes.py`'s `valid_record()` fixture, and the mutation loop then demanded the validator enforce fields no producer supplies. Both are removed from the fixture with that reasoning recorded beside them, including that the retained `2026-07-21` record legitimately still carries them — it was measured while the files existed.

**The mutation loop's failure path needs no synthetic demonstration: it fired on exactly this.** Before the fix it reported `validator accepted missing provenance field: probe.project_sha256`, naming the key. That is the check saying no, with the right message, about a real discrepancy — it was simply right about a fixture that had gone stale rather than about a validator gap.

**One attempted negative check was invalid and is recorded so it is not repeated.** Deleting the `require(values, "probe.harness_sha256", SHA256)` call at line 84 and re-running left the suite green, which reads as "the loop cannot say no". It is not: the same key is required a second time at line 163, where the retained input manifest is checked against the producer fields, so removing the first call changes nothing. Editing the validator is also self-defeating in general, because the record pins `probe.validator_sha256` against that file's own digest. A negative check on this test has to change the *record*, not the validator.

`uv run --with pytest pytest spikes/apple-targets` now reports **82 passed, 0 failed** — the whole spike suite green, where it had 1 failure before this change and 44 mid-session.
