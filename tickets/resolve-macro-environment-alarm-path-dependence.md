---
id: resolve-macro-environment-alarm-path-dependence
title: Make the macro-environment overall-alarm test independent of ambient PATH
status: todo
priority: p2
dependencies: []
related: [collect-runtime-and-shapes-spike-tests, finish-spike-process-group-cleanup, make-spike-process-group-cleanup-best-effort]
scopes: [research/macro-environment]
shared_scopes: [project/tickets]
paths: []
tags: [testing, gate-reliability, harness]
---
`collect-runtime-and-shapes-spike-tests` found this while widening the gate's pytest `testpaths`. It is not caused by that change and is not fixed by it: the test is already collected by the gate today.

**Fact.** `spikes/macro-environment/test_probe.py:242` `test_overall_alarm_reaps_child_after_capture_pipes_close` arms a 0.2 s `setitimer(ITIMER_REAL, 0.2)` and then spawns a child spelled `"python3"` — a bare name resolved through `PATH` (line 247). Every other subprocess test in the same file spells the interpreter `sys.executable`.

**Fact.** The test's expectation depends on where the SIGALRM lands. The child writes a pid file, closes fds 1 and 2, then sleeps 10 s, so both capture pipes reach EOF quickly and `probe.capture` falls through to `process.wait(timeout=remaining_timeout())`, where the handler's `ProbeFailure("... exceeded its overall deadline")` propagates unmodified. If the child has not yet closed its pipes when the alarm fires, the same `ProbeFailure` is raised inside `selector.select(...)` at `probe.py:169`, which catches it and re-raises `ProbeFailure(f"command exceeded deadline: {command!r}")`. The `pytest.raises(..., match="overall deadline")` then fails.

**Measurement** (macOS arm64, this host, 2026-07-24, base `ffbfce1`): interpreter startup, best of 5, `subprocess.run([exe, "-c", "pass"])`:

| `python3` resolves to | startup | test result |
| --- | --- | --- |
| `~/.pyenv/shims/python3` (ambient login `PATH`) | 281 ms | 3/3 red |
| `<checkout>/.venv/bin/python3` (gate `sanitized_environment()`) | 16 ms | 5/5 green |

The pyenv shim is a shell script that re-execs, and 281 ms exceeds the 200 ms margin deterministically. Reproduce the failure in one line from the repository root:

```sh
PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 .venv/bin/python -m pytest -c pyproject.toml -q \
  'spikes/macro-environment/test_probe.py::test_overall_alarm_reaps_child_after_capture_pipes_close'
```

**Inference.** `scripts/check_repository.py:sanitized_environment` pins `PATH` to `<checkout>/.venv/bin` first, so CI and `uv run --locked python scripts/check_repository.py` are green and this never reddens the gate. It reddens the *contributor* invocation `uv run --locked pytest spikes/macro-environment` for anyone whose `python3` is slower than the margin, which a stock pyenv install is. That is the inverse of the defect class `make-spike-process-group-cleanup-best-effort` chased: not a flaky gate, but a test that only passes because one governed harness hides an ambient dependency. A contributor who hits it has no way to tell a real regression from this.

**Proposal.** Spell the child `sys.executable`, matching the sibling tests at lines 197 and 224 of the same file. That removes the `PATH` dependency and cuts the observed startup from 281 ms to 16 ms, restoring most of the 200 ms budget. Consider separately whether 0.2 s is enough margin on a loaded CI host even at 16 ms startup, or whether the test should assert the landing site directly rather than race for it.

Done when the test passes from a pyenv-first login shell as well as under the gate's sanitized environment, and the repository gate still passes.
