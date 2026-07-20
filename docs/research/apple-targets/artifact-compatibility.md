# Apple Metal artifact compatibility

**Status:** bounded research; compilation matrix blocked by absent Metal
Toolchain component

**Probe date:** 2026-07-20

## Result

The portable contract can be tightened even though this host cannot produce a
new metallib. macOS, iOS device, iOS simulator, and Mac Catalyst are distinct
artifact families. Each needs an explicit platform/environment and deployment
minimum in the Metal payload descriptor. SDK identity and compiler provenance
do not substitute for that compatibility contract, and a successful metallib
load does not prove function or pipeline readiness.

The installed Xcode contains macOS, iPhoneOS, and iPhoneSimulator 26.5 SDKs but
does not contain the separately downloadable Metal Toolchain. The `metal`
launcher fails before compilation and `metallib` cannot be resolved. Per the
research constraint no Metal Toolchain component was downloaded or installed.
Therefore this run does **not** establish AIR/metallib target metadata, output-byte
reproducibility, or old/new OS compatibility.

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
  /Applications/Xcode.app/Contents/Developer/Toolchains/
  XcodeDefault.xctoolchain/usr/bin/metal
launcher SHA-256:
  1705fe2424223a740e3ec0a419ffce6568610cc62156d60df475a1a67afbe0d1
metallib: not found by xcrun
metal --version exit: 1
metal --version error:
  cannot execute tool 'metal' due to missing Metal Toolchain;
  use: xcodebuild -downloadComponent MetalToolchain
```

No download command was run. A read-only `xcrun simctl list devices available`
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
| macOS | `macosx`; intended `air64-apple-macos<min>` | No: missing Metal Toolchain | macOS 27 API-stage controls only | Family required; deployment minimum explicit; real artifact compatibility unmeasured |
| iOS device | `iphoneos`; intended `air64-apple-ios<min>` | No: missing Metal Toolchain | No physical device attached | Separate family required; all load/pipeline/old-device behavior unmeasured |
| iOS simulator | `iphonesimulator`; intended `air64-apple-ios<min>-simulator` | No: missing Metal Toolchain | iOS 26.0 runtime listed but not booted | Separate family required; runtime behavior unmeasured |
| Mac Catalyst | macOS SDK exposes `ios` + `macabi` | No command validated | Not exercised | **Deferred**, neither silently supported nor permanently rejected |

The word “intended” is deliberate: the exact target triples are inputs to the
checked-in matrix, consistent with installed SDK target metadata and Apple's
iOS example, but this host could not ask the Metal compiler to validate them.

### Catalyst disposition

Catalyst should be deferred as a fourth optional Apple payload family. Treating
it as macOS is contradicted by the SDK's `ios`/`macabi` target environment;
treating it as iOS device is contradicted by its Mac execution environment.
Enabling it later requires a validated Metal compiler target, Rust target
mapping, deployment-minimum policy, a Catalyst runtime load/pipeline test, and
ordinary application integration coverage. Deferral does not require changing
the neutral envelope.

### Toolchain support policy

No complete Metal AOT toolchain version was qualified by this run. “Xcode
26.6” is insufficient because its separately resolved Metal compiler/linker
component is absent. Initial support should use a tested evidence table keyed
by Xcode build, SDK canonical version/build, `metal` and `metallib`
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
fails. No such artifacts could be created on this host.

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

### Unmeasured

AIR and metallib byte reproducibility could not run because compilation failed
at toolchain preflight. No conclusion is available for:

- repeated output in one directory;
- identical sources in different absolute directories;
- different machines with the same Xcode build;
- Xcode patch/build changes;
- embedded timestamps, UUIDs, source paths, or nondeterministic section order.

Until measured, Tiler promises deterministic source/manifest/key construction,
not byte-identical Apple compiler output across machines or toolchains. A cache
hit requires the complete compiler/SDK/flag identity and stored payload digest;
cache correctness never depends on independently reproducing the same bytes.

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
Toolchain is unavailable and does not download anything.

Run the API-stage control on a macOS Metal host:

```sh
xcrun --sdk macosx swiftc \
  spikes/apple-targets/runtime_failure_probe.swift \
  -framework Metal -o /tmp/tiler-apple-runtime-probe
/tmp/tiler-apple-runtime-probe
```

## Required follow-up matrix

After installing a Metal Toolchain through a user-approved maintenance action,
repeat the checked-in probe and add:

1. metallib metadata inspection with the toolchain-provided inspection tool;
2. same-build second machine and at least two Xcode patch/build versions;
3. minimum/equal/newer macOS runtime hosts for each chosen deployment minimum;
4. minimum/equal/newer physical iOS devices across admitted Apple GPU families;
5. minimum/equal/newer simulator runtimes;
6. deliberately wrong-family and too-new-minimum artifacts at every runtime;
7. library load, function lookup, and pipeline creation recorded independently;
8. a Catalyst compile/load/pipeline experiment before changing its deferred
   status.

Old-device behavior remains explicitly unmeasured. Apple's forward-compatibility
statements about offline GPU binary archives do not establish universal
metallib compatibility across platform families, deployment minima, old OS
versions, or GPU features.
