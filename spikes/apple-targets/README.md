---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets"
kind: "experiment"
title: "Apple Metal target compatibility spikes"
topics: ["apple-targets", "metal", "compatibility"]
experiment_status: "reproducible"
implementation_status: "spike-only"
supports: ["tiler.research.apple-targets.compatibility"]
entrypoints: ["compatibility_probe.sh", "runtime_failure_probe.swift"]
last_verified: "2026-07-20"
ticket: "apple-artifact-compatibility"
---

# Apple Metal target compatibility spikes

The compile probe records exact SDK/tool versions and compares explicit macOS,
iOS-device, and iOS-simulator artifacts. It requires an installed Apple Metal
toolchain and downloads nothing.

```sh
spikes/apple-targets/compatibility_probe.sh
```

On a macOS Metal host, the Swift control distinguishes library, function, and
pipeline failure stages:

```sh
xcrun --sdk macosx swiftc spikes/apple-targets/runtime_failure_probe.swift -framework Metal -o /tmp/tiler-apple-runtime-probe
/tmp/tiler-apple-runtime-probe
```

Old OS/GPU devices and cross-machine reproducibility remain unmeasured. See the
[compatibility report](../../docs/research/apple-targets/artifact-compatibility.md).
