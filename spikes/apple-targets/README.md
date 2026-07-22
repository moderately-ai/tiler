---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets"
kind: "experiment"
title: "Apple Metal target compatibility spikes"
topics: ["apple-targets", "metal", "compatibility"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.apple-targets.compatibility"]
entrypoints: ["spikes/apple-targets/compatibility_probe.sh", "spikes/apple-targets/runtime_failure_probe.swift", "spikes/apple-targets/validate_compatibility_record.py", "spikes/apple-targets/test_probes.py"]
last_verified: "2026-07-21"
ticket: "apple-artifact-compatibility"
---

# Apple Metal target compatibility spikes

The compile probe records exact SDK/tool versions, commands, logs, artifact
digests, and byte comparisons for explicit macOS, iOS-device, and iOS-simulator
artifacts. It requires an installed Apple Metal toolchain and downloads
nothing. Its optional argument is the result directory; omitting it preserves
the run in a newly created operating-system temporary directory.

```sh
spikes/apple-targets/compatibility_probe.sh \
  spikes/apple-targets/results/<yyyy-mm-dd>-<toolchain>
```

Success means the complete line-oriented `record.tsv` passed
`validate_compatibility_record.py`; compile-matrix success without valid host,
SDK, compiler, and linker provenance fails closed. Preserve `record.tsv`, SDK
settings, `input-manifest.tsv`, and command logs for any published measurement.
Schema v2 binds the repository base and exact harness, validator, kernel,
project, lockfile, and manifest digests. AIR and metallib
files are regenerable and ignored in the checked-in result area; their digests
remain in the record.

The retained 2026-07-21 local run is
[`results/2026-07-21-xcode26.6-metal32023.883/record.tsv`](results/2026-07-21-xcode26.6-metal32023.883/record.tsv).
Its SDK extracts and command logs are checked in beside it.

On a macOS Metal host, the Swift control distinguishes library, function, and
pipeline failure stages:

```sh
xcrun --sdk macosx swiftc spikes/apple-targets/runtime_failure_probe.swift -framework Metal -o /tmp/tiler-apple-runtime-probe
/tmp/tiler-apple-runtime-probe
```

The control exits nonzero for every unexpected library, function, or pipeline
outcome. Run its portable record-mutation tests and, on macOS, its compiled
runtime-stage injections with:

```sh
uv run --locked python spikes/apple-targets/test_probes.py
```

Old OS/GPU devices and cross-machine reproducibility remain unmeasured. See the
[compatibility report](../../docs/research/apple-targets/artifact-compatibility.md).
