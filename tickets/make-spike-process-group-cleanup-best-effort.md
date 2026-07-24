---
id: make-spike-process-group-cleanup-best-effort
title: Make spike process-group cleanup best-effort so it cannot fail the gate
status: todo
priority: p1
dependencies: []
related: [record-source-verification-rule-in-agent-guidance]
scopes: [research/embedding, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness, gate-reliability]
---
The repository gate fails intermittently on `os.killpg`, and because it is the
**gate** that fails, every ticket on the host inherits the flakiness.

Five unguarded call sites: `spikes/extensions/run.py:55`, and
`spikes/embedding/measure.py:359,373,386,395`. Each raises out of the harness if
the process group cannot be signalled.

Two distinct triggers, both observed:

- **Already-exited child.** In `test_run_logged_enforces_output_limit` the child
  (`print('x' * 1000)`) usually exits before the capture-limit branch fires, so
  the group holds only an unreaped child and signalling it can return `EPERM`.
  Measured: of four gate invocations at one unmodified tree, two exited 0 and two
  exited 1 on exactly this test. It did **not** reproduce under narrow runs
  (12/12 for the test alone, 11/11 for the directory), which is why it reads as
  flake rather than defect.
- **Sandboxed execution.** A second agent independently hit the same syscall
  being refused outright in `spikes/extensions/run.py`, and had to run the gate
  outside its sandbox to get past it.

Both were provably unrelated to the changes under test — in both cases
`git diff --name-only <base> HEAD -- spikes/` was empty.

**Fix:** treat process-group termination as best-effort. Catch
`ProcessLookupError` and `PermissionError` around each `killpg` and continue;
cleanup that cannot signal a group is not a test failure, and a harness must not
fail the run it is tidying up after.

**Do this carefully — the calls are not all mere cleanup.** Some enforce a limit
(an output cap, a timeout), and there the kill is load-bearing: silently
swallowing its failure would let a harness report success while a runaway child
was never actually stopped. For each site decide which it is. Where the kill
enforces a limit, keep the assertion that the limit *was* enforced by an
independent observation (captured bytes, elapsed time, exit status) rather than
inferring it from the kill having been attempted.

While here, note a **host-local confound that is not this defect** and must not
be folded into it: `spikes/macro-environment/test_probe.py::test_overall_alarm_reaps_child_after_capture_pipes_close`
fails when run from an interactive shell because a bare `python3` resolves to a
pyenv shim (~0.76 s startup) and loses a 0.2 s `setitimer` race. The gate is
unaffected because `sanitized_environment()` puts `.venv/bin` first. That is an
invocation artifact, not a repository defect, and fixing it is not in scope here.

This matters beyond tidiness: an intermittently red gate erodes the signal the
whole review discipline rests on. A gate that fails at random teaches people to
re-run until green, which is exactly how a real failure gets waved through.
