---
id: decide-shared-spike-process-cleanup-helper
title: Decide whether the duplicated spike kill_process_group helper should be shared
status: todo
priority: p3
dependencies: []
related: [finish-spike-process-group-cleanup, make-spike-process-group-cleanup-best-effort]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [harness, duplication]
---
Seven spike harnesses now carry a near-identical `kill_process_group`: `spikes/embedding/measure.py`, `spikes/extensions/run.py`, `spikes/macro-environment/probe.py`, `spikes/numerics/sound_accuracy/daisy_runner.py`, `spikes/runtime/measure_semantic_validation.py`, `spikes/shapes/shape-evidence/measure.py`, and `spikes/shapes/nightly-dependent-static-shapes/measure.py`. Each tolerates `ProcessLookupError` and `PermissionError`, falls back to `process.kill()`, and bounds the reap at `CLEANUP_REAP_SECONDS = 5.0`. They differ only in whether the bounded reap spells `wait` or `communicate`, which follows from whether the harness drains its own capture pipes.

**The duplication may be correct.** Spikes are self-contained reproducible experiments; a shared module would introduce a cross-spike import that every harness would have to resolve from its own working directory, and would couple an experiment's reproducibility to a file outside it. Two independent copies also caught this defect twice rather than once.

**Against:** the defect this pattern fixes was found and fixed three separate times, across two tickets, and the count of affected sites grew from five to thirteen because nobody had one place to look.

Decide explicitly, record the decision, and if duplication stays, say where a future harness author is expected to copy it from. Scope depends on the outcome; add the spike scopes actually touched before editing.
