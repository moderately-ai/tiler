---
id: remove-bare-python3-from-macro-environment-tests
title: Remove the bare python3 invocation from the macro-environment probe tests
status: closed
priority: p3
dependencies: []
related: [finish-spike-process-group-cleanup]
scopes: [research/macro-environment]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness]
closed_reason: duplicate
closed_note: Duplicate of resolve-macro-environment-alarm-path-dependence, which is more thorough and higher priority. Its unique fact — a second bare-python3 site in test_command_capture_rejects_output_while_streaming — was folded into that ticket before closing.
---
`spikes/macro-environment/test_probe.py` spawns children as bare `python3` in `test_command_capture_rejects_output_while_streaming` and `test_overall_alarm_reaps_child_after_capture_pipes_close`. That resolves through the ambient `PATH`.

**Measurement (macOS arm64, Darwin 27.0.0, CPython 3.11.12, 2026-07-24, interactive zsh):** `which -a python3` resolves first to `/Users/tsanterre/.pyenv/shims/python3`. Under that resolution `test_overall_alarm_reaps_child_after_capture_pipes_close` fails, because the shim's startup cost loses the test's 0.2-second `setitimer` race and the harness reaches its deadline in the streaming loop rather than the final wait; the raised message is then `command exceeded deadline` instead of `overall deadline`. Under the gate's `sanitized_environment()`, which puts `.venv/bin` first, the same test passes. Measured at commit-under-test on branch `tkt/finish-spike-process-group-cleanup`: 20 passed / 2 unrelated failures with a sanitized `PATH`, and the same suite with the ambient `PATH` additionally fails that one test.

**Inference:** the test is correct about the harness and wrong about how it selects an interpreter. Every other child-spawning test in the repository uses `sys.executable`, including the tests added by `finish-spike-process-group-cleanup` in this same file.

This is an invocation artifact and not the process-group cleanup defect: bounding the reap removed the multi-minute hang the artifact used to produce, but the race it loses is still there. Do not describe fixing this as fixing that.

Done when neither test depends on `PATH` resolution for its interpreter, and the file's tests pass with a deliberately hostile `PATH` (for example with a pyenv shim directory first).
