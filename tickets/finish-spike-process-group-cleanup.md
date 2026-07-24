---
id: finish-spike-process-group-cleanup
title: Finish best-effort process-group cleanup across the remaining spike harnesses
status: in-progress
priority: p1
dependencies: []
related: [make-spike-process-group-cleanup-best-effort]
scopes: [research/macro-environment, research/numerics, research/runtime, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness, gate-reliability]
claimed_from: todo
assignee: agent-finish-spike-process-group-cleanup
lease_expires_at: 1784919183
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
