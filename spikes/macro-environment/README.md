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
entrypoints: ["spikes/macro-environment/run.sh", "spikes/macro-environment/run-target.sh", "spikes/macro-environment/run-family-cfg.sh", "spikes/macro-environment/probe.py", "spikes/macro-environment/cleanup_signal_demonstration.py", "spikes/macro-environment/alarm_landing_site.py"]
last_verified: "2026-07-24"
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

Retained results from the 2026-07-24 macOS run are
[native-2026-07-24.json](results/native-2026-07-24.json) and
[family-cfg-2026-07-24.json](results/family-cfg-2026-07-24.json). They supersede
the 2026-07-21 pair, which they reproduce field for field apart from harness
identity and scratch paths: a retained result binds the exact digest of every
harness input, so changing `probe.py` requires re-measuring rather than
re-labelling. Verify that their raw/decoded traces, predicates, and source
digests remain internally consistent with the checkout:

```sh
python3 spikes/macro-environment/probe.py verify \
  spikes/macro-environment/results/native-2026-07-24.json
python3 spikes/macro-environment/probe.py verify \
  spikes/macro-environment/results/family-cfg-2026-07-24.json
uv run --locked pytest spikes/macro-environment/test_probe.py
```

`cleanup_signal_demonstration.py` drives a harness's bounded entry point with a
refused or an undelivered group signal, so the cleanup contract that
`kill_process_group` implements can be observed deterministically, including
against an earlier revision extracted with `git show`.

`alarm_landing_site.py` takes a census of where the overall alarm interrupts
`capture`, which is what decides whether an expired deadline surfaces as the
harness's own `overall deadline` or as the streaming loop's rewritten
`command exceeded deadline`. `--construction pre-armed` arms a fixed timer
before the child is spawned and therefore races the child interpreter's startup;
`--construction drain-armed` arms it from the `unregister` that empties the
selector map, after which no `selector.select` call remains for the signal to
land in. Both report the spawn-to-drain latency a pre-armed margin has to
out-run, and `--interpreter` selects the child, so an ambient-`PATH` interpreter
is reproducible directly:

```sh
spikes/macro-environment/alarm_landing_site.py \
  --module spikes/macro-environment/probe.py --construction pre-armed \
  --trials 5 --interpreter "$(command -v python3)"
```

The malformed-output tests reject missing, duplicate, invalid-hex, unknown
version, wrong-fingerprint, unexpected-environment, malformed-cfg, and invalid
result-schema inputs. The harness does not measure rust-analyzer. See the
[research report](../../docs/research/macro-environment/proc-macro-build-environment.md).
