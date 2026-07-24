---
id: finish-spike-process-group-cleanup
title: Finish best-effort process-group cleanup across the remaining spike harnesses
status: done
priority: p1
dependencies: []
related: [make-spike-process-group-cleanup-best-effort]
scopes: [research/macro-environment, research/numerics, research/runtime, research/shapes, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness, gate-reliability]
---
`make-spike-process-group-cleanup-best-effort` fixed `spikes/embedding` and `spikes/extensions` and, in doing so, established that **its own scope was insufficient for its stated goal**. The gate is still intermittently red. This ticket finishes the job.

That ticket named five call sites. There are thirteen. Eight remain, and the pattern at each is identical: an unguarded `os.killpg` that raises out of the harness, followed by an unbounded `process.wait()`.

| Site | Scope | In the gate's `testpaths`? |
| --- | --- | --- |
| `spikes/macro-environment/probe.py:150,162,173,179` | `research/macro-environment` | **yes** |
| `spikes/numerics/sound_accuracy/daisy_runner.py:491` | `research/numerics` | **yes** |
| `spikes/runtime/measure_semantic_validation.py:39` | `research/runtime` | no |
| `spikes/shapes/shape-evidence/measure.py:54` | `research/shapes` | no |
| `spikes/shapes/nightly-dependent-static-shapes/measure.py:43` | `research/shapes` | no |

**Measurement — the gate still fails on this.** Across eight complete `check_repository.py` invocations at the fixed tree, one failed at `spikes/macro-environment/probe.py:162` in `test_command_capture_rejects_output_while_streaming` with `PermissionError: [Errno 1]` from `os.killpg`. That is the exact analogue of the embedding defect: a `print('x' * 1000)` child, the output-cap branch, a group whose only member is an exited-but-unreaped child. Its rate is lower than the embedding one — 0 in 40 standalone runs, 1 in 8 gate runs — which makes it harder to notice and no less real.

**Measurement — the trigger is deterministic once the state is reached.** `killpg` on a group whose only member is an exited-but-unreaped child raised `PermissionError` errno 1 in 200 of 200 attempts on macOS arm64 (Darwin 27.0.0, CPython 3.11.12), both when exit was observed via `wait` and via stdout EOF. Only the race that reaches that state is intermittent. So a regression test can be deterministic; `spikes/embedding/test_measure.py::test_kill_process_group_tolerates_unsignalable_group` is the worked example.

## Follow the shape already merged, do not reinvent it

`kill_process_group` in `spikes/embedding/measure.py` and in `spikes/extensions/run.py` are the reference implementations. Both tolerate `ProcessLookupError` and `PermissionError` from `os.killpg`, fall back to `process.kill()` on the child this process directly owns, and bound the reap at `CLEANUP_REAP_SECONDS = 5.0`.

**The bounded reap is load-bearing, not tidiness, and it is the part most likely to be dropped as incidental.** Suppressing the signal failure while leaving `process.wait()` unbounded trades a crash for a hang — and worse than a hang, it converts a hard failure into a silent false pass: the harness sits until the child's natural exit and *then* observes it "gone", so an assertion that a limit was enforced passes without the limit ever having been enforced. With the bound, a cleanup that never delivered a signal stays visible to the caller.

That failure mode is observed here, not hypothetical: under an unsanitized `PATH`, `spikes/macro-environment`'s `test_overall_alarm_reaps_child_after_capture_pipes_close` stopped progressing for **over nine minutes** against a child that sleeps ten seconds, precisely because `probe.py`'s reap is unbounded.

## Classify each site before changing it

Some of these kills are mere cleanup and some enforce a limit — an output cap, a timeout. Where the kill is load-bearing, **keep the assertion that the limit was enforced, and make it an independent observation** rather than an inference from the kill having been attempted. The merged work shows both forms: assert the bytes actually retained on disk for an output cap, and assert elapsed wall time under the grace period for a timeout. Do not assert merely that the harness raised.

## Scope note on a confound that is not this defect

`spikes/macro-environment/test_probe.py::test_overall_alarm_reaps_child_after_capture_pipes_close` also fails when run from an interactive shell because a bare `python3` resolves to a pyenv shim (~0.76 s startup) and loses a 0.2 s `setitimer` race. The gate is unaffected because `sanitized_environment()` puts `.venv/bin` first. That is an invocation artifact and out of scope — but note that bounding the reap fixes the *hang* that artifact produces even though the race itself remains, so do not conclude the two are the same problem or that fixing one fixes the other.

## Why this is p1

An intermittently red gate erodes the signal the whole review discipline rests on. ADR 0075 makes coordinator merge authority conditional on a green `check_repository.py`, so a gate that fails at random either blocks merges arbitrarily or teaches workers to re-run until green — which is exactly how a real failure gets waved through. Run the gate several times at the finished tree, not once, and report the count: a single green run is not evidence that an intermittent fault is fixed.

## Outcome

All eight remaining sites now route through a per-harness `kill_process_group` that tolerates `ProcessLookupError` and `PermissionError` from `os.killpg`, falls back to `process.kill()` on the child the harness directly owns, and bounds the reap at `CLEANUP_REAP_SECONDS = 5.0`. Five harnesses gained the helper: `spikes/macro-environment/probe.py`, `spikes/numerics/sound_accuracy/daisy_runner.py`, `spikes/runtime/measure_semantic_validation.py`, `spikes/shapes/shape-evidence/measure.py`, and `spikes/shapes/nightly-dependent-static-shapes/measure.py`. No unguarded `os.killpg` and no unbounded post-kill wait remains anywhere under `spikes/`; the exact check is `grep -rn killpg --include='*.py' spikes`, which now returns one call per harness, each inside its helper.

**One deliberate divergence from the merged reference.** `probe.py` drains and closes its own capture pipes, so its reap is `process.wait(timeout=…)`, byte-identical in shape to `spikes/embedding/measure.py`. The other four hand their pipes to `communicate`, so their reap is `process.communicate(timeout=…)`. Reaping those with a bare `wait` would leave the capture descriptors open and could block on a surviving grandchild that inherited the write end; `communicate` drains and closes them under the same bound.

### Site classification

| Site (pre-fix line) | Branch | Class | Independent observation that the limit was enforced |
| --- | --- | --- | --- |
| `probe.py:150` | deadline expires in the streaming loop | load-bearing (timeout) | `test_command_capture_enforces_deadline_while_streaming`: elapsed under deadline plus one grace period against a 30 s child, and `os.kill(child, 0)` raises `ProcessLookupError` |
| `probe.py:162` | output cap exceeded mid-stream | load-bearing (producer must stop) | `test_command_capture_stops_the_producer_it_rejects`: a child that prints forever is gone afterwards, plus a bounded elapsed time. The byte cap itself is enforced by the pre-extend check, so the kill's job is stopping the producer — that, not the cap, is what needed an observation |
| `probe.py:173` | deadline expires in the final `wait` | load-bearing (timeout) | `test_command_capture_enforces_deadline_after_capture_pipes_close`: both pipes reach end of file at once so this branch is reached rather than the loop; same elapsed and child-death observations |
| `probe.py:179` | `finally`, child outlived the body | load-bearing (no runaway child) | `test_overall_alarm_reaps_child_after_capture_pipes_close`, unchanged, already requires `ProcessLookupError` on the child's recorded pid. Not reclassified as cleanup: it is the only thing standing between an alarm-interrupted capture and an orphan |
| `daisy_runner.py:491` | analyzer exceeded its timeout | load-bearing (timeout) | `test_run_profile_kills_timed_out_process_group` now asserts elapsed under the deadline plus one grace period. Previously it asserted only `reason == "analyzer_timeout"`, which a 30-second wait-out also satisfies |
| `measure_semantic_validation.py:39` | command exceeded its deadline | load-bearing (timeout) | `test_run_enforces_its_deadline`: bounded elapsed and `ProcessLookupError` on the child |
| `shape-evidence/measure.py:54` | command exceeded its deadline | load-bearing (timeout) | `test_run_enforces_its_deadline`, same form |
| `nightly-dependent-static-shapes/measure.py:43` | command exceeded its deadline | load-bearing (timeout) | `test_run_enforces_its_deadline`, same form |

No site was classified as mere cleanup. The `finally` branch in `probe.py` is the only candidate, and it is the branch that produced the multi-minute stall recorded above, so it is treated as enforcing "no runaway child" and keeps its observation.

**No further sibling sites exist.** `spikes/numerics/sound_accuracy/daisy_runner.py:268` is a `multiprocessing.Process.kill()` on the provenance worker, not a group signal; `multiprocessing`'s own `_send_signal` already swallows `ProcessLookupError`, and `os.kill` to a directly-owned zombie does not answer `EPERM`. Every other subprocess use under `spikes/` and `scripts/` is `subprocess.run`, which owns no process group.

### Measurement — the defect made deterministic, before and after

The `EPERM` state is reached by a race, but the behaviour it produces is not. `spikes/macro-environment/cleanup_signal_demonstration.py` injects the two signal outcomes directly and drives any harness's bounded entry point by path, so the same driver runs against an earlier revision extracted with `git show`.

**Measurement** (macOS arm64, Darwin 27.0.0, CPython 3.11.12, this worktree, base `b9d9137` versus the fixed tree, one run per cell):

| Harness | Refused group signal (before → after) | Undelivered group signal (before → after) |
| --- | --- | --- |
| `probe.py` (`capture`) | crashed out of cleanup with `PermissionError` → reported its own `ProbeFailure` | 30.03 s, child gone, **silent false pass** → 11.01 s, child still alive, non-enforcement visible |
| `measure_semantic_validation.py` (`run`) | crashed with `PermissionError` → reported its own `RuntimeError` | 30.03 s, silent false pass → 6.00 s, non-enforcement visible |
| `shape-evidence/measure.py` (`run`) | crashed with `PermissionError` → reported its own `RuntimeError` | 30.03 s, silent false pass → 6.00 s, non-enforcement visible |
| `nightly-dependent-static-shapes/measure.py` (`run`) | crashed with `PermissionError` → reported its own `RuntimeError` | 30.03 s, silent false pass → 6.00 s, non-enforcement visible |
| `daisy_runner.py` (`run_profile`) | crashed with `PermissionError` → reported `Unknown(analyzer_timeout)` | 30.02 s, still reported `Unknown(analyzer_timeout)` — the false pass in its clearest form → 6.01 s |

The `daisy_runner` row is the argument for the bounded reap on its own. Before the fix, a cleanup that delivered no signal at all still produced exactly the `Unknown(analyzer_timeout)` the existing test asserted, 30 seconds later. The assertion passed; nothing had been enforced. The elapsed bound added here is what distinguishes the two.

The child in these cells sleeps for 30 seconds. `11.01 s` for `probe.capture` is two bounded reaps — the deadline branch and then the `finally`, which still sees a live child — and is correct rather than a doubled grace period being charged twice to one kill.

### Retained macro-environment evidence had to be re-measured

**Fact:** `PROVENANCE_INPUTS` in `probe.py` includes `probe.py`, and `verify_result` requires a retained result's `input_sha256` to equal the current digests. Editing the harness therefore invalidated both retained results, failing `spikes/macro-environment/test_probe.py` and the gate-run `scripts/tests/test_research_harnesses.py`. Verified at the base commit with `git stash push -- spikes/macro-environment/probe.py`, after which `probe.py verify …/native-2026-07-21.json` exits 0.

The results were re-measured on the fixed harness and renamed to `native-2026-07-24.json` and `family-cfg-2026-07-24.json`, superseding the 2026-07-21 pair. Re-labelling the old files was rejected: the binding exists precisely to stop a retained result claiming a harness that did not produce it. **Measurement:** the new pair reproduces the old field for field except `provenance.input_sha256` (the `probe.py` entry), `provenance.repository_revision`, and the scratch paths that appear in the trace and in the retained compiler diagnostic. The expansion matrix `1, 1, 1, 2, 2, 3, 4, 7`, the cache attribution sequence, and all four target `cfg` classifications are unchanged. `contracts/navigation` was added to this ticket's scopes for the one reference in `scripts/tests/test_research_harnesses.py`.

### Coverage that the gate actually runs

Collected by `check_repository.py`: the six new and one strengthened test in `spikes/macro-environment/test_probe.py`, and the one strengthened plus two new tests in `spikes/numerics/sound_accuracy/test_daisy_runner.py`.

**Not collected:** `spikes/runtime/test_measure_semantic_validation.py` and the two shapes test files. `spikes/runtime` and `spikes/shapes` are outside the canonical pytest `testpaths`, and expanding that contract needs `implementation/workspace` plus a decision about pytest walking the Cargo `target/` trees those spike directories keep. Filed as `collect-runtime-and-shapes-spike-tests`; the tests pass when run directly and each names its own invocation in its module docstring. Claiming otherwise would repeat this ticket's own complaint about a fix that stopped at a scope boundary.

### Verification

**Measurement** (same host, worktree `.worktrees/tiler/finish-spike-process-group-cleanup/edit`, shared with three other agents): `uv run --locked python scripts/check_repository.py` at the finished tree, 30 consecutive invocations, **30 green** — exit 0 and `complete repository validation passed` in all 30 logs. 66 s for the cold first run, 21–24 s thereafter. The predecessor ticket measured 1 failure in 8 gate invocations at `probe.py:162`, so this is roughly four times that exposure with no failure.

Those 30 were run before this outcome was written; the tree was then re-validated with 5 further invocations after it, also green, so 35 of 35 in total. That bounds the fault rate; it does not prove absence, and it is the weaker half of the evidence. The deterministic before/after table above is the stronger half, because it exhibits the behaviour the intermittent state produces instead of sampling for the state. Note also that this site never reproduced under narrow standalone runs — 0 in 40 in the ticket's own measurement — so a green `pytest spikes/macro-environment` says nothing about it and was not counted here.

### Deliberately not done

The pyenv `setitimer` confound in `test_overall_alarm_reaps_child_after_capture_pipes_close` is untouched, and bounding the reap did not fix the race. It removed the stall the artifact used to produce: under an ambient `PATH` the test now fails in about a second with `command exceeded deadline` instead of waiting. The race it loses is still there. Filed as `remove-bare-python3-from-macro-environment-tests`.
