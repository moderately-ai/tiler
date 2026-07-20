---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.sound-accuracy"
kind: "experiment"
title: "Sound accuracy probe"
topics: ["numerics", "accuracy", "proof"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["sound-proof", "bounded-measurement"]
supports: ["tiler.research.numerics.sound-region-analyzer-spike", "tiler.research.numerics.region-accuracy-contract"]
ticket: "spike-sound-region-accuracy-analyzer-integration"
---

# Sound accuracy probe

This corpus invokes a pinned Daisy trusted-analyzer profile and separately
produces adversarial observations. The observations are not proof.

## Prepare the pinned analyzer

Clone with the workspace helper and detach at the measured revision:

```sh
zsh -ic 'gwc https://github.com/malyzajko/daisy.git'
git -C /path/to/daisy checkout --detach \
  38a0f33915dde03eeadd34786a920e834c1d9110
```

The measurement host had no global `sbt`. It used a temporary launcher and
temporary dependency caches:

```sh
probe_tmp=$(mktemp -d /tmp/tiler-daisy.XXXXXX)
curl -L --fail -o "$probe_tmp/sbt-launch.jar" \
  https://repo1.maven.org/maven2/org/scala-sbt/sbt-launch/1.9.9/sbt-launch-1.9.9.jar

env COURSIER_CACHE="$probe_tmp/coursier" /path/to/java17/bin/java \
  -Dsbt.ivy.home="$probe_tmp/ivy" \
  -Dsbt.global.base="$probe_tmp/global" \
  -Dsbt.boot.directory="$probe_tmp/boot" \
  -jar "$probe_tmp/sbt-launch.jar" clean compile script
```

Daisy's generated runner at this revision includes
`-XX:+UseConcMarkSweepGC`, so the measured invocation used an installed Java 8
runtime. No host package or toolchain was installed by the spike.

## Run

From the Tiler checkout:

```sh
PATH="/path/to/java8/bin:/opt/homebrew/bin:/usr/bin:/bin" \
  spikes/numerics/sound_accuracy/run_daisy.sh /path/to/daisy
python3 spikes/numerics/sound_accuracy/observe.py
```

`/opt/homebrew/bin` is needed only for profiles that invoke the measured Z3
installation. The default runner uses interval ranges and affine errors.

The runner refuses a different Daisy revision. Treat any diagnostic, missing
function result, parse failure, or timeout as `Unknown`, even when Daisy exits
with status zero.

## Traceability

- **Supported claim:** [Sound analyzer integration spike](../../../docs/research/numerics/sound-region-analyzer-spike.md).
- **Parent contract research:** [Region accuracy contracts](../../../docs/research/numerics/region-accuracy-contract.md).
- **Retained measurements:** [measurements.json](measurements.json) and [unsupported_cases.json](unsupported_cases.json).
- **Work record:** [spike-sound-region-accuracy-analyzer-integration](../../../tickets/spike-sound-region-accuracy-analyzer-integration.md).
