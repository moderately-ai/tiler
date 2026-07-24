---
id: resolve-macro-environment-alarm-path-dependence
title: Make the macro-environment overall-alarm test independent of ambient PATH
status: done
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

## Absorbed from a duplicate

`remove-bare-python3-from-macro-environment-tests` was filed independently by `finish-spike-process-group-cleanup` for the same defect and is closed as a duplicate of this ticket. It carried one fact this one did not: **there is a second bare-`python3` site in the same file**, `test_command_capture_rejects_output_while_streaming`, so fix both rather than only the alarm test. It also stated a caution worth keeping — bounding the reap in `make-spike-process-group-cleanup-best-effort` removed the multi-minute *hang* this artifact used to produce, but the `setitimer` race it loses is untouched. Do not describe fixing this as fixing that; they are different defects that happened to compound.

That two agents found the same defect from opposite directions — one while fixing process-group cleanup, one while widening the gate's test collection — is itself evidence it sits on a path contributors actually walk.

## Outcome

Both bare-`python3` sites in `spikes/macro-environment/test_probe.py` now spell the child `sys.executable`, and the alarm test no longer races a timer at all: it synchronizes the alarm on the pipe closure it is about. The exact check for the first half is `grep -rn '"python3"' --include='*.py' spikes scripts` together with its single-quoted form, both of which now return nothing. `probe.py` is untouched, so no retained result moved. The other changed files are the new `alarm_landing_site.py` driver that preserves the measurements below and the `README.md` entry describing it.

**Site one — `test_command_capture_rejects_output_while_streaming`.** `probe.capture(["python3", …])` became `probe.capture([sys.executable, …])`. This site never had a timing exposure: its deadline is the full 60 s `start_deadline()` budget, which dwarfs any interpreter startup. What it had was an interpreter-identity dependency. A bare name can resolve to an interpreter too old for the harness, to a shim that fails, or to nothing at all, and `capture` would then raise `cannot execute …` rather than the `stdout exceeds 64 bytes` the test exists to assert.

**Site two — `test_overall_alarm_reaps_child_after_capture_pipes_close`. The landing site was made deterministic rather than the margin widened.** `sys.executable` was treated as necessary and not sufficient, exactly as the proposal above suspected. The test's subject is *where* the SIGALRM lands, and any pre-armed `setitimer` makes that a race against child interpreter startup however fast the interpreter is; a faster race is still a race, and this repository has just spent two tickets removing an intermittent gate failure. The alarm is now armed from the observable the assertion depends on. A `DrainArmedSelector` subclass of `selectors.DefaultSelector`, patched over `probe.selectors.DefaultSelector` for the duration of the test — the same idiom the file already uses for `monkeypatch.setattr(probe.os, "killpg", …)` — calls `setitimer(ITIMER_REAL, ALARM_AFTER_DRAIN_SECONDS)` from the `unregister` that empties the selector map, that is, at the instant both capture pipes have reached end of file.

**Inference — why that is deterministic and not merely faster.** `capture` calls `selector.select` only from inside `while selector.get_map():`. Arming on the `unregister` that empties the map means the loop condition is already false, so no further `select` call exists for the signal to land in, and the `except ProbeFailure` that rewrites the handler's message into `command exceeded deadline` is unreachable. Every statement that remains — the loop exit, `remaining_timeout()`, and `process.wait(...)` — propagates the handler's `ProbeFailure` unmodified, so the assertion holds wherever in that window the signal arrives. The 0.2 s is therefore no longer a margin against anything the environment controls; it is only how long the test waits with the child already in the state under test. A new `assert len(armed) == 1` fails the test if the synchronization never engaged, so the construction cannot silently decay back into the pre-armed race, and patching a `probe.selectors` attribute that stopped existing would raise rather than pass.

The assertion was not weakened. The test still requires a `ProbeFailure` matching `overall deadline`, still requires the child's pid file to exist, and still requires `ProcessLookupError` on the recorded pid — so it still proves that the overall alarm reaps a child whose capture pipes have already closed. The child program is now the module-level `CLOSE_PIPES_AND_SLEEP`, shared with `test_command_capture_enforces_deadline_after_capture_pipes_close`, which keeps the wall-clock and signal paths through the same landing site provably comparable instead of separately maintained.

**Measurement — environment.** macOS arm64, Darwin 27.0.0, 14 cores, CPython 3.11.12, uv 0.11.28, 2026-07-24, in the ticket worktree at base `82254ff`. Ambient login `PATH` puts `~/.pyenv/shims` ahead of `/opt/homebrew/bin`, so bare `python3` is the shim.

**Measurement — interpreter startup, best of 5, `subprocess.run([exe, "-c", "pass"])`:** `~/.pyenv/shims/python3` 477.9 ms; `.venv/bin/python3` 21.3 ms. Taken with sibling worktrees building concurrently, load average about 26, which is the plausible reason it exceeds the 281 ms recorded above under the same procedure. That the same measurement on the same host ranges from 281 ms to 478 ms depending on what else is running is itself the point: the quantity the old test raced is bounded by nothing the test controls.

**Measurement — before, from a pyenv-first shell.** The one-line reproduction above is 3/3 red at `82254ff`, each run raising `ProbeFailure: command exceeded deadline: ['python3', …]` from `probe.py:172` — the `selector.select` landing site, not the final wait. **After, same shell and same ambient `PATH`:** 10/10 green in 0.24 s each, and the whole file 22/22 green in three consecutive runs of `uv run --locked pytest spikes/macro-environment`. Under the gate's sanitized environment the whole gate is green, see below.

**The landing-site evidence is checked in.** `spikes/macro-environment/alarm_landing_site.py` takes a census of the frame chain the alarm actually interrupted, for either construction and any child interpreter, and reports the spawn-to-drain latency a pre-armed margin has to out-run. It loads the harness by path in the same way `cleanup_signal_demonstration.py` does, so it also runs against an earlier revision extracted with `git show`. The three measurements below are its output, and `README.md` documents it.

**Measurement — the shipped defect, reproduced through the driver.** `--construction pre-armed --interpreter "$(command -v python3)"`, 5 trials: site `probe.py:capture` 5/5, message `command exceeded deadline` 5/5. The alarm never reaches the wait.

**Measurement — the fix is path-independent, not merely faster.** `--construction drain-armed --interpreter "$(command -v python3)"`, 5 trials, that is the *same pyenv shim*: site `probe.py:capture -> subprocess.py:wait -> subprocess.py:_wait -> probe.py:overall_timeout_handler` 5/5, message `overall deadline` 5/5, spawn-to-drain 277.1 / 277.9 / 285.6 ms. A drain latency 39% beyond the 200 ms margin no longer changes the outcome, which is the claim `sys.executable` alone cannot make. With `--interpreter .venv/bin/python`, 30 trials: same site 30/30, same message 30/30, spawn-to-drain 19.2 / 26.4 / 41.8 ms.

**Measurement, and its boundary — the counterfactual was not falsifiable on this host.** The rejected alternative, `sys.executable` with the original pre-armed 0.2 s timer, is green wherever it was pushed: 30/30 through the driver, and 70/70 in earlier ad-hoc runs of which 40 were under 24 spinning processes and 30 under 56 on 14 cores, where the spawn-to-drain maximum still only reached 57.2 ms. That does not qualify it. It shows only that synthetic CPU load on this host could not push a 20 ms startup past 200 ms, while resolving the interpreter name differently on the same host pushed it to 277 ms; startup on a contributor's host is further governed by `sitecustomize`, `.pth` scanning, a network home, endpoint security scanning, and page-cache state, none of which the test bounds. The drain-armed construction removes the quantity from the correctness condition instead of betting on a bound for it. The driver also exposes the relevant asymmetry: late *timer* delivery was observed (0.264 s for a 0.2 s timer under load) and is harmless, because it only grants the child more time, whereas a slow *child* flips the assertion.

**Fact — retained results were not invalidated and were not re-measured.** `PROVENANCE_INPUTS` in `probe.py` lists `probe.py`, the three `run*.sh` wrappers, the two `.rs` sources, and six fixture files; `test_probe.py` is not among them. With the change applied, `python spikes/macro-environment/probe.py verify` exits 0 for both `results/native-2026-07-24.json` and `results/family-cfg-2026-07-24.json`, and the gate-run `scripts/tests/test_research_harnesses.py` passes. Confining the fix to the test file was a deliberate condition on the design, not a happy accident: any repair inside `probe.py` would have forced a re-measurement of both retained results.

**Scope decision — the `run*.sh` wrappers were examined and deliberately left alone.** `run.sh`, `run-target.sh`, and `run-family-cfg.sh` each `exec python3 "$root/probe.py"`, and `README.md` documents `python3 spikes/macro-environment/probe.py verify`. These are entry points where the operator chooses the interpreter, which is the ordinary convention and not the defect class this ticket describes: no assertion's outcome depends on which one wins. They are also all three inside `PROVENANCE_INPUTS`, so editing them would invalidate both retained results for a change with no correctness content. The residual sharp edge is that `probe.py` needs Python 3.10 or newer for `zip(..., strict=True)` and would fail opaquely under an older ambient `python3`; that is recorded here rather than filed, and the trigger for reconsidering it is a harness edit that requires re-measurement anyway.

**Measurement — gate.** `uv run --locked python scripts/check_repository.py` is green in 6 of 6 runs: three on the test-only change and three more after `alarm_landing_site.py` and the `README.md` entry were added. 204 pytest items collected and passed every run; the driver is not a test module and is not collected, and Ruff discovery accepts it because `source_files(".py")` already covers everything under `spikes/`. `git diff --check` clean; `ticketsplease lint` and `ticketsplease guard` against `82254ff` both clean, with `conflict: false` and no under-declared scopes.
