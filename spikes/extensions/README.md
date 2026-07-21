---
schema: "tiler-doc/v1"
id: "tiler.spike.extensions"
kind: "experiment"
title: "Operation-extension experiments"
topics: ["extensions", "proc-macro", "rust"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api", "tiler.research.extensions.proc-macro-extension-visibility"]
entrypoints: ["spikes/extensions/run.py"]
last_verified: "2026-07-21"
ticket: "operation-extension-surface"
---

# Operation-extension experiments

The `operation-api` crate compile-checks the proposed capability boundary. The
`proc-macro-visibility` workspace demonstrates which providers a stable proc
macro can observe across host and consumer crate boundaries.

Run from the repository root:

```sh
python3 spikes/extensions/run.py
```

The runner gives the complete suite a five-minute deadline, runs the
proc-macro observation twice, rejects missing success markers or the wrong
cycle diagnostic, bounds each command's combined output to four MiB, and
records source/toolchain provenance plus command output
in the ignored `spikes/extensions/proc-macro-visibility/target/` directory.
Source provenance includes full tracked/untracked status and bounded digests of
every Rust, Python, shell, and Cargo fixture input.
Run `python3 spikes/extensions/run.py --self-test` to exercise malformed-output
and timeout handling without invoking Cargo.
It requires Python 3.11 or newer and POSIX process-group behavior on the
repository's supported macOS and Debian-family development hosts.

The API names remain experimental. The visibility result is bounded to the
recorded Rust/Cargo compilation model and does not establish a plugin ABI.
