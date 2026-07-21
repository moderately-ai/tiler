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
entrypoints: ["spikes/numerics/sound_accuracy/run_daisy.sh", "spikes/numerics/sound_accuracy/daisy_runner.py", "spikes/numerics/sound_accuracy/observe.py"]
last_verified: "2026-07-21"
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
uv run --locked python spikes/numerics/sound_accuracy/observe.py
uv run --locked python -O spikes/numerics/sound_accuracy/observe.py
```

`/opt/homebrew/bin` is needed only for profiles that invoke the measured Z3
installation. The default runner uses interval ranges and affine errors.

The runner refuses a different or tracked-dirty Daisy revision and gives each
analyzer profile a 60-second deadline. Set `TILER_DAISY_TIMEOUT_SECONDS` to an
integer from 1 through 3600 to change that bounded deadline. Successful output
records the checked source revision, effective deadline, input digests, and
SHA-256 fingerprints of the generated launcher, selected Java executable, and
every file in the launcher's literal Scala classpath. Missing or excessively
large provenance inputs produce `Unknown`; a source revision alone is not
accepted as the identity of generated executable state. The runner independently
checks the checkout revision and tracked-clean state and requires the complete
fingerprint to remain equal after every profile.

On success, the runner emits normalized JSON with `status: "proved"` only after
it has parsed exactly one finite, nonnegative absolute-error bound and one
finite real range for every required function. It emits `status: "unknown"` to
standard error and exits 10 for a timeout, analyzer output/diagnostic, malformed
or duplicate result, or missing function result. This is required because the
pinned Daisy launcher can return zero through its `tee` pipeline after the
analyzer itself has failed. Exit status alone is not accepted as evidence.
The process tree has a one-MiB per-file ceiling; captured diagnostics and CSV
results are each limited to one MiB, CSV input to 64 rows and 4,096 characters
per field, and provenance traversal to 4,096 files, 4,096 directories, and 512
MiB. Each provenance collection has a separate 30-second wall deadline.
Crossing a resource or time limit produces `Unknown` rather than proof
evidence.

The pre/post identity check detects ordinary concurrent mutation but is not an
immutable execution snapshot: a hostile writer could change bytes and restore
them between checks. This is an explicit spike limitation. A production trusted
analyzer adapter must execute an immutable staged closure (or equivalently
strong content-addressed environment) before ingesting `SoundProof` evidence.

The shell entrypoint owns the pinned-checkout preflight. The separately tested
`daisy_runner.py` module owns bounded process execution and strict result
parsing. Its parser and timeout behavior can be exercised without Daisy:

```sh
uv run --locked pytest spikes/numerics/sound_accuracy/test_daisy_runner.py
```

### 2026-07-21 verification boundary

**Measurement:** the retained local Daisy checkout was still at the pinned,
tracked-clean revision, but its generated launcher referenced a removed
source-resource directory and removed temporary Coursier cache entries. The
wrapper returned `Unknown` with reason
`analyzer_provenance` and exit status 10 before execution. The fixture-driven
parser, provenance, resource-limit, and real process-group timeout tests passed
in the locked environment.

**Inference:** this verifies fail-closed adapter behavior, not a fresh Daisy
proof run. The certified bounds in `measurements.json` remain the earlier dated
measurements and were not regenerated. A fresh proof run must first rebuild the
launcher with every classpath input present and dependency storage that remains
available for execution.

The observation program applies a 100-digit `Decimal` context to every decimal
reference calculation. Its explicit-FMA candidate is computed without
`math.fma`: exact `Fraction` arithmetic forms `x * y + z`, then a local IEEE
binary32 round-to-nearest-ties-to-even oracle performs the single rounding.
Explicit checks cover lower-even and upper-even halfway cases, subnormal
underflow, and a fused cancellation that differs from multiply then add. The
equality-constrained ratio observation enumerates five named binary32 inputs
with `x == y` and records both its maximizing witness and sample count. Running
with `-O` must produce byte-for-byte identical output. The checked-in
[`observations.json`](observations.json) records the exact interpreter and host,
source/algorithm identity, numerical policy, complete finite domains, sample
counts, witnesses, and results. The aggregate checker requires both modes to
match each other and always requires the portable corpus/algorithm identity to
match the retained fixture. On the recorded interpreter and host it additionally
requires exact result replay; another supported host reports that the retained
measurement was not replayed rather than conflating a new environment with the
recorded observation.

## Traceability

- **Supported claim:** [Sound analyzer integration spike](../../../docs/research/numerics/sound-region-analyzer-spike.md).
- **Parent contract research:** [Region accuracy contracts](../../../docs/research/numerics/region-accuracy-contract.md).
- **Retained measurements:** [measurements.json](measurements.json),
  [observations.json](observations.json), and
  [unsupported_cases.json](unsupported_cases.json).
- **Work record:** [spike-sound-region-accuracy-analyzer-integration](../../../tickets/spike-sound-region-accuracy-analyzer-integration.md).
