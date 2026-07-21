---
schema: "tiler-doc/v1"
id: "tiler.spike.macro-environment"
kind: "experiment"
title: "Proc-macro environment and artifact-family spikes"
topics: ["proc-macros", "cargo", "cross-compilation"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.macro-environment.build-environment"]
entrypoints: ["spikes/macro-environment/run.sh", "spikes/macro-environment/run-target.sh", "spikes/macro-environment/run-family-cfg.sh", "spikes/macro-environment/probe.py"]
last_verified: "2026-07-21"
ticket: "macro-build-environment"
---

# Proc-macro environment and artifact-family spikes

Run the isolated fixture for native freshness, an installed explicit Rust
target, and generated consumer-family `cfg` behavior:

```sh
spikes/macro-environment/run.sh --output /tmp/tiler-macro-native.json
spikes/macro-environment/run-target.sh <installed-target-distinct-from-host> \
  --output /tmp/tiler-macro-target.json
spikes/macro-environment/run-family-cfg.sh \
  --output /tmp/tiler-family-cfg.json
```

The target argument is mandatory and must differ from the rustc host. The probe
rejects an unavailable target and lists installed targets rather than installing
one. Each whole harness run has a 60-second overall deadline, configurable from
1 through 600 seconds with `TILER_PROBE_TIMEOUT_SECONDS`; every child process
receives only the remaining portion. Standard output and standard error are
read incrementally and capped at one MiB each before they enter memory.

The native probe requires the complete expansion-count sequence
`1, 1, 1, 2, 2, 3, 4, 7`. It parses every trace field and verifies invocation
tokens, both explicit fingerprints, miss/hit attribution, the consumer package
identity, and the measured absence of every reported implicit target/build
variable. Its result preserves both the encoded raw trace and its decoded form.
The family probe compiles and executes fallback on every host; it requires the
macOS diagnostic only when the host itself matches `target_os="macos"`, and
requires successful nonmatching compilation on Debian-family Linux.

Retained results from the 2026-07-21 macOS run are
[native-2026-07-21.json](results/native-2026-07-21.json) and
[family-cfg-2026-07-21.json](results/family-cfg-2026-07-21.json). Verify that
their raw/decoded traces, predicates, and source digests remain internally
consistent with the checkout:

```sh
python3 spikes/macro-environment/probe.py verify \
  spikes/macro-environment/results/native-2026-07-21.json
python3 spikes/macro-environment/probe.py verify \
  spikes/macro-environment/results/family-cfg-2026-07-21.json
uv run --locked pytest spikes/macro-environment/test_probe.py
```

The malformed-output tests reject missing, duplicate, invalid-hex, unknown
version, wrong-fingerprint, unexpected-environment, malformed-cfg, and invalid
result-schema inputs. The harness does not measure rust-analyzer. See the
[research report](../../docs/research/macro-environment/proc-macro-build-environment.md).
