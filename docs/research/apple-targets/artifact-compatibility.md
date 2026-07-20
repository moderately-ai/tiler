---
schema: "tiler-doc/v1"
id: "tiler.research.apple-targets.compatibility"
kind: "research"
title: "Apple Metal artifact compatibility"
topics: ["apple-targets", "metal", "artifacts", "compatibility"]
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.metal-backend", "tiler.contract.artifact-abi"]
adopted_by: ["ADR-0049", "ADR-0053"]
reproduced_by: ["tiler.spike.apple-targets"]
ticket: "apple-artifact-compatibility"
---

# Apple Metal artifact compatibility

**Status:** bounded research; compile and same-host reproducibility matrix
measured, runtime compatibility matrix still open

**Probe date:** 2026-07-20

## Result

The authorized Metal Toolchain 17F109 follow-up compiled all six tested
tuples. The trivial kernel produced three different final metallibs: one each
for macOS, iOS device, and iOS simulator. This directly supports treating those
as distinct artifact families. Mac Catalyst remains a fourth, deferred family.

Within each family, the two requested deployment minima produced identical AIR
and final metallib bytes for this kernel. That narrow result does not show that
deployment minima are interchangeable, absent from compatibility semantics, or
safe to omit from identity. The requested triple is still an input and the
declared lower runtime boundary; another source feature or compiler may encode
it differently.

Compiling identical source bytes from two absolute directories produced
different AIR bytes containing the respective source paths, but byte-identical
final metallibs. This supports using the final metallib as the embedded payload
and keeping absolute paths out of portable identity when equivalent inputs are
otherwise established. It does not establish reproducibility across machines
or toolchain builds.

No old macOS host, iOS device, or iOS simulator loaded these new artifacts.
Compile success is not runtime compatibility evidence, and library load would
still not prove function lookup or pipeline readiness.

## Evidence classification

This memo uses four labels:

- **Measured:** directly observed by the checked-in probes on the named host.
- **Source-backed:** stated by linked primary Apple documentation.
- **Inferred:** architectural conclusion from measured/source-backed facts.
- **Unmeasured:** requires a toolchain, OS, simulator, or physical device not
  exercised here.

## Host and installed tools

### Measured

```text
host: MacBook Pro Mac16,6, Apple M4 Max, arm64, 36 GB
macOS: 27.0 build 26A5378n
Xcode: 26.6 build 17F113
Metal Toolchain component: build 17F109
Metal Toolchain identifier: com.apple.dt.toolchain.Metal.32023.883
DEVELOPER_DIR: /Applications/Xcode.app/Contents/Developer
xcrun: 72
Apple clang: 21.0.0 (clang-2100.1.1.101)
Apple Swift: 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)
```

| SDK selector | Version | Build | SDK-settings target environment |
| --- | ---: | --- | --- |
| `macosx` | 26.5 | 25F70 | `macos`, empty environment |
| `iphoneos` | 26.5 | 23F81a | `ios`, empty environment |
| `iphonesimulator` | 26.5 | 23F81a | `ios`, `simulator` |
| Catalyst subprofile in macOS SDK | 26.5 | 25F70 | `ios`, `macabi` |

The SDK settings advertise defaults of 26.5. The Mac profile records an SDK
minimum of 10.13, iOS device/simulator 12.0, and the Catalyst subprofile 13.1.
Those are SDK metadata, **not** evidence that Tiler's selected MSL version,
operations, or generated metallibs support every OS back to those versions.

Tool discovery produced:

```text
metal launcher:
  /private/var/run/com.apple.security.cryptexd/mnt/
  com.apple.MobileAsset.MetalToolchain-v17.6.109.0.../
  Metal.xctoolchain/usr/metal/current/bin/metal
metal: Apple metal version 32023.883 (metalfe-32023.883)
metallib: AIR-LLD 32023.883 (metalfe-32023.883)
metal SHA-256:
  3457ceed7d7e9cd5ac4e96affa8726df72331e19f3a836ece5c0758e0ba79f54
metallib SHA-256:
  0f391fad93f257988128848b76eec747ddf6863cff871ee9b9518152a8db68ac
```

The initial probe, before the user-authorized Metal Toolchain installation,
resolved only Xcode's launcher; that launcher reported the missing component
and `metallib` was unavailable. The follow-up resolved the separately installed
MobileAsset toolchain shown above. This chronology matters: Xcode build alone
does not identify the compiler binaries actually used.

A read-only `xcrun simctl list devices available` during the initial probe
unexpectedly printed `Install Started` / `Install Succeeded` while initializing
Xcode simulator support, then listed only an iOS 26.0 simulator runtime. No
simulator was booted. This side effect was not requested and the command is not
part of the checked-in probe. `xcrun devicectl list devices` found no attached
physical device.

## Primary Apple contracts

### Source-backed

- Apple describes a metallib as Metal IR, a GPU-independent intermediate
  representation that the runtime may still compile into a GPU-specific binary
  during pipeline creation. See [Metal libraries](https://developer.apple.com/documentation/metal/metal-libraries)
  and [Target and optimize GPU binaries with Metal 3](https://developer.apple.com/videos/play/wwdc2022/10102/).
- `MTLDevice.makeLibrary(data:)` creates a library from precompiled bytes and
  throws on failure. `MTLLibrary.makeFunction(name:)` independently returns
  `nil` when a symbol is absent. `MTLDevice.makeComputePipelineState` is a
  separately fallible preparation step. See
  [library loading](https://developer.apple.com/documentation/metal/mtldevice/makelibrary%28data%3A%29),
  [function lookup](https://developer.apple.com/documentation/metal/mtllibrary/makefunction%28name%3A%29),
  and [pipeline creation](https://developer.apple.com/documentation/metal/mtldevice/makecomputepipelinestate%28function%3A%29).
- Apple's offline binary-archive example explicitly passes an iOS deployment
  triple such as `air64-apple-ios16.0`; app deployment minima define the lower
  OS range an application claims to support. See
  [custom offline archive compilation](https://developer.apple.com/documentation/metal/compiling-binary-archives-from-a-custom-configuration-script)
  and [deployment targets](https://developer.apple.com/documentation/xcode/running-code-on-a-specific-version/).
- Apple publishes availability by Metal/GPU family rather than promising every
  feature on every Metal device. See the
  [Metal feature tables](https://developer.apple.com/metal/capabilities/).

### Inferred contract

1. A metallib is source-level AOT, not proof of zero first-use compilation.
2. Platform family, deployment minimum, MSL language/features, and live GPU
   capabilities are independent compatibility dimensions.
3. SDK version/build is producer provenance. It does not mean “requires this
   SDK version at runtime,” nor does deployment minimum prove all required GPU
   features.
4. Simulator is not an iOS-device artifact selected by CPU architecture. Its
   `simulator` target environment makes it a separate artifact family.
5. Catalyst is likewise a distinct `ios` + `macabi` family. A macOS or iOS
   device artifact must not be relabeled as Catalyst-compatible.

## Compatibility matrix

| Family | SDK/compiler target probed | Artifact produced | Runtime exercised | Current conclusion |
| --- | --- | --- | --- | --- |
| macOS | `macosx`; `air64-apple-macos13.0`, `air64-apple-macos14.0` | Both compiled; identical final bytes | macOS 27 API-stage controls only; generated artifacts not loaded | Distinct final output; deployment-minimum runtime range unmeasured |
| iOS device | `iphoneos`; `air64-apple-ios16.0`, `air64-apple-ios17.0` | Both compiled; identical final bytes | No physical device attached | Distinct final output; all runtime behavior unmeasured |
| iOS simulator | `iphonesimulator`; `air64-apple-ios16.0-simulator`, `air64-apple-ios17.0-simulator` | Both compiled; identical final bytes | iOS 26.0 runtime listed but not booted | Distinct final output; runtime behavior unmeasured |
| Mac Catalyst | macOS SDK exposes `ios` + `macabi` | No command validated | Not exercised | **Deferred**, neither silently supported nor permanently rejected |

The identical-minima observation is specific to `copy.metal`, Metal 32023.883,
MSL 3.1, and the exact flags below. Internal AIR strings observed by the probe
named macOS 14.0 and iOS 17.0 even for the lower-minimum commands. That may be
toolchain normalization, an MSL/toolchain floor, or metadata unrelated to the
requested deployment contract; the probe does not distinguish those causes.
It is not evidence that Tiler may discard the requested minimum.

### Catalyst disposition

Catalyst should be deferred as a fourth optional Apple payload family. Treating
it as macOS is contradicted by the SDK's `ios`/`macabi` target environment;
treating it as iOS device is contradicted by its Mac execution environment.
Enabling it later requires a validated Metal compiler target, Rust target
mapping, deployment-minimum policy, a Catalyst runtime load/pipeline test, and
ordinary application integration coverage. Deferral does not require changing
the neutral envelope.

### Toolchain support policy

Metal 32023.883 with AIR-LLD 32023.883 is now qualified only for this bounded
compile and same-host reproducibility probe. It is not qualified for Tiler's
runtime support matrix or numerical conformance. “Xcode 26.6” remains an
insufficient identity because `xcrun` separately resolves the MobileAsset
compiler/linker component. Initial support should use a tested evidence table
keyed by Xcode build, SDK canonical version/build, `metal` and `metallib`
version/executable fingerprint, selected MSL standard, and exact flags. It
should not use an open-ended `Xcode >= N` claim.

Expansion fails with an actionable unsupported-toolchain diagnostic when no
evidence row realizes the requested numerical and compatibility contract. A
new patch build is admitted only after the family, metadata, numerical, and
reproducibility probes pass. This is an expansion-environment support policy,
not a statement that artifacts from older rows are runtime-incompatible.

## Runtime failure stages

### Measured control experiment

`runtime_failure_probe.swift` ran on the Apple M4 Max using the macOS 27 Metal
runtime:

```text
corrupt precompiled bytes -> MTLLibraryErrorDomain Code=1 "Invalid library file"
valid runtime source      -> library creation success
missing_entry             -> function lookup returned nil
valid compute function    -> pipeline creation success
vertex function as compute-> pipeline creation error (AGXMetalG16X Code=3)
```

This proves that the local API exposes distinct library-load, function-lookup,
and pipeline-creation stages. It does **not** prove at which stage a correctly
formed but wrong-platform, too-new-deployment, or unsupported-GPU metallib
fails. The new artifacts were compiled but not used for those runtime tests.

### Proposed classification

```text
neutral envelope integrity/schema
  -> Apple payload family + deployment compatibility preflight
  -> MTLDevice.makeLibrary(data:)             // library-load failure
  -> MTLLibrary.makeFunction(name:)            // manifest/symbol invariant
  -> MTLDevice.makeComputePipelineState(...)   // prepared-kernel feasibility
  -> RoutingCommit
  -> execution
```

- A declared platform/deployment mismatch is rejected before Metal API calls
  and may select another complete compatible payload or fallback.
- Corrupt bytes should already fail envelope digest validation. If Metal still
  rejects an integrity-valid payload, classify the concrete error before
  deciding whether another preflight alternative is legal; never call it an
  ordinary guard miss by default.
- A missing declared function is an artifact invariant failure, not device
  incompatibility.
- Pipeline failure can be a typed live-device/prepared-kernel incompatibility
  only when a governed provider classifies the error and another complete
  semantically equivalent route remains before `RoutingCommit`. Unknown or
  systemic errors fail closed.

## Neutral-envelope integration

The target-neutral envelope remains unaware of Apple spellings. Its existing
`BackendPayloadDescriptor` needs only `backend_key`, `representation_key`,
`target_compatibility_contract_ref`, section digests, producer provenance, and
required/optional payload semantics. The first `MetalPayloadMetadata` schema
resolves that compatibility reference with:

```text
AppleMetalCompatibility {
  schema_version,
  apple_platform: MacOS | IOSDevice | IOSSimulator | Catalyst,
  normalized_target_triple,
  deployment_minimum,
  msl_language_version,
  required_metal_and_gpu_features,
  sdk: { canonical_name, version, build },
  metal_compiler_fingerprint,
  metallib_linker_fingerprint,
  compile_and_link_flags,
  metallib_representation_version,
}
```

The artifact family, deployment minimum, language/features, compiler/linker
fingerprints, flags, and payload bytes participate in expansion-cache and
payload identity. Absolute SDK paths are local provenance unless their content
differs; SDK canonical version/build and any output-affecting header/library
digests provide the portable evidence.

One neutral envelope may carry several Apple payloads, but each payload retains
its own descriptor and digest. Required-payload policy must not make an iOS
artifact mandatory for a macOS-only consumer. Runtime selection occurs by
declared family and compatibility, never by trial-loading every metallib.

## Reproducibility

### Measured on one host and toolchain

The probe compiled identical source bytes from `src-a/copy.metal` and
`src-b/copy.metal`. Each family showed this pattern:

| Family | Requested minima | AIR across directories | Final metallib across directories | Final metallib SHA-256 |
| --- | --- | --- | --- | --- |
| macOS | 13.0, 14.0 | Different | Identical | `12bc2bc6771922c6d41a00b077111d8bcb3632f7196f386c63ab9332bbf114b8` |
| iOS device | 16.0, 17.0 | Different | Identical | `5410e94de593a21ea3190feb94c89b58cdb8cd3132dc1c64ed3495f856579262` |
| iOS simulator | 16.0, 17.0 | Different | Identical | `ede80a137044deed1d41efbd352fcf9fc8a136e332b8aebb3428ae5d63d0f9aa` |

Within a family, both tested minima also produced identical AIR and metallib
bytes. Across families, the final metallibs differed.

The AIR files contained their respective absolute `src-a` or `src-b` source
paths. The path is therefore a measured AIR input leak correlated with the AIR
digest change, although this probe does not prove it is the only differing
field. The linker removed or canonicalized the difference in the final
metallib for this case.

### Inferred cache consequences

- The final metallib, not AIR, is the initial product payload and payload
  digest. Persisting AIR is unnecessary for the proposed macro-local bundle.
- Absolute temporary paths should not enter the portable content key merely
  because the compiler records them in an intermediate. The key still includes
  every semantic and compiler input, including requested deployment minimum.
- Cache correctness must validate the stored final payload digest and never
  depend on a second compiler invocation reproducing the same bytes.

### Still unmeasured

- repeated compilation in the same absolute directory;
- a second machine with the same Xcode and MobileAsset toolchain builds;
- different Xcode or Metal Toolchain patch/build versions;
- whether richer kernels, debug/line-info flags, includes, or libraries retain
  paths, timestamps, UUIDs, or nondeterministic order in the final metallib.

Tiler therefore promises deterministic source/manifest/key construction, not
byte-identical Apple compiler output across machines or toolchains.

## Exact probes

Run the bounded family/reproducibility matrix:

```sh
spikes/apple-targets/compatibility_probe.sh
```

When tools are available it compiles each tuple twice with explicit target,
MSL, optimization, safe-math, precise-fp32, and no-contraction flags:

```text
macosx         air64-apple-macos13.0
macosx         air64-apple-macos14.0
iphoneos       air64-apple-ios16.0
iphoneos       air64-apple-ios17.0
iphonesimulator air64-apple-ios16.0-simulator
iphonesimulator air64-apple-ios17.0-simulator
```

The exact compile/link shape is:

```sh
ZERO_AR_DATE=1 xcrun --sdk <sdk> metal \
  -target <triple> -std=metal3.1 -O2 \
  -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise \
  -ffp-contract=off -c copy.metal -o copy.air
ZERO_AR_DATE=1 xcrun --sdk <same-sdk> metallib copy.air -o copy.metallib
```

The script records SDK/tool versions and hashes, compiles identical source bytes
from two different absolute directories, extracts target-like strings, hashes
both artifacts, and performs byte comparisons. It exits 4 when the Metal
Toolchain is unavailable and does not download anything. The follow-up run
completed successfully with all six tuples.

Run the API-stage control on a macOS Metal host:

```sh
xcrun --sdk macosx swiftc \
  spikes/apple-targets/runtime_failure_probe.swift \
  -framework Metal -o /tmp/tiler-apple-runtime-probe
/tmp/tiler-apple-runtime-probe
```

## Required follow-up matrix

The authorized toolchain follow-up completed the checked-in compile and
same-host path-variation probe. Remaining work is:

1. metallib metadata inspection with a format-aware tool rather than `strings`;
2. same-build second machine and at least two Xcode/Metal patch-build pairs;
3. minimum/equal/newer macOS runtime hosts for each chosen deployment minimum;
4. minimum/equal/newer physical iOS devices across admitted Apple GPU families;
5. minimum/equal/newer simulator runtimes;
6. deliberately wrong-family and too-new-minimum artifacts at every runtime;
7. library load, function lookup, and pipeline creation recorded independently;
8. richer kernels and compile modes that may retain paths or nondeterminism;
9. a Catalyst compile/load/pipeline experiment before changing its deferred
   status.

Old-device behavior remains explicitly unmeasured. Apple's forward-compatibility
statements about offline GPU binary archives do not establish universal
metallib compatibility across platform families, deployment minima, old OS
versions, or GPU features.

## Traceability

The measured family distinctions inform the [Metal backend](../../backends/metal.md)
and ADRs 0049 and 0053. The [Apple target spike](../../../spikes/apple-targets/README.md)
is reproducible; old-device and cross-machine runtime compatibility remain open.
