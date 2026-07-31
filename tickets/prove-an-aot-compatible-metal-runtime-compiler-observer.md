---
id: prove-an-aot-compatible-metal-runtime-compiler-observer
title: Prove an AOT-compatible Metal runtime-compiler observer
status: done
priority: p0
dependencies: []
related: [validate-macos-metal-profile-host-applicability, record-metal-runtime-compiler-provenance-gap]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Tiler can identify, or explicitly decline to identify, the Metal runtime compiler relevant to native AOT library/pipeline preparation without compiling source at runtime, guessing from the OS build, or treating any loaded framework with a plausible name as the compiler that answered the route.

## Facts and eliminated shortcuts

**Fact:** `metal` 0.33.0 exposes device name, family support, and resource limits but no runtime-compiler identity query. The retained Apple numerical harness obtains exact `metalfe-32023.921` evidence by runtime-compiling source, serializing an `MTLBinaryArchive`, and scanning the archive bytes. That is valid measurement evidence and is forbidden as a normal Tiler runtime route because the accepted developer experience is AOT-only.

**Fact:** the same harness enumerates loaded `GPUCompiler`/`MTLCompiler` image paths, but its record treats this as weaker evidence: an image being loaded does not by itself prove that image or build answered a particular compilation or pipeline preparation. The earlier inference from a framework merely present on disk was already corrected as false.

**Inference:** substituting OS build, Xcode build, a hard-coded framework path, or an arbitrary loaded-image string would silently certify a compiler identity the observation did not establish. Those options fail correctness and are eliminated rather than offered as cheaper alternatives.

## Implementation keys

Build a bounded preserved experiment under `spikes/apple-targets/` that starts from a native offline-compiled metallib and exercises the exact library and compute-pipeline preparation route used by the serial-sum runner. Inventory every safe and foreign observation point available before and after each preparation stage. Determine whether the responsible runtime compiler build can be associated with that exact AOT preparation without `newLibraryWithSource`, source JIT, ambient Xcode selection, or producer-owned artifact/profile bytes.

Prefer a safe platform API when one exists. If the only surviving probe requires a foreign dyld or framework API, the spike must name the exact call, returned population, bounds, ownership, and safety invariant; production admission remains a separate ADR 0079 case-by-case review. If exact association is impossible, define the strongest truthful evidence class and keep exact compiler identity unavailable rather than widening the measured applicability policy.

The experiment must separate three claims: a framework image is present on disk, an image is loaded in the process, and a particular compiler build is attributable to the AOT preparation. Agreement of their strings on one host does not collapse those evidence classes.

## Required evidence

- A retained harness and fixture run the native metallib/pipeline path with no runtime source compilation; a source-JIT sentinel proves that calling `newLibraryWithSource` makes the run fail.
- Before/after observations identify which images or metadata change at library load and pipeline creation, with exact macOS/device/compiler environment recorded.
- Positive attribution, if any, is reproduced from a clean process and is stable across repeated runs; an unrelated preloaded compiler image does not satisfy it.
- Negative fixtures cover absent metadata, multiple plausible images/build strings, framework-present-but-not-loaded, loaded-before-the-route, and an unavailable qualified host.
- Every validator/check is perturbed once and observed failing before restoration.

## Closes when

The retained experiment ends in one of two concrete outcomes: a reproducible AOT-compatible observation contract with exact evidence strength and production requirements, or a demonstrated inability to attribute an exact runtime compiler build through the native route plus a fail-closed contract that leaves the predicate unavailable. The result updates the Apple numerical/runtime provenance research without overstating one-host evidence, `tkt lint` and `git diff --check` pass, and the spike's exact manual invocation passes on an available qualified host or records the exact unavailable predicate.

## Graph maintenance

- Block `validate-macos-metal-profile-host-applicability` until the observer knows which runtime-compiler evidence class it may accept.
- Keep `record-metal-runtime-compiler-provenance-gap` related and update its conclusion from this result rather than duplicating the experiment.
- If a reusable observer requires unsafe foreign access, file a separately reviewed implementation ticket carrying ADR 0079's four conditions; this spike does not admit the site.
- If no exact AOT-compatible observer exists, revise the first authoritative profile's applicability contract to retain `Unknown` or another independently justified predicate rather than treating the measurement's source-JIT compiler identity as runtime-route evidence.

## Outcome (2026-07-31)

**Measurement:** `spikes/apple-targets/aot-runtime-compiler-observer/run.sh spikes/apple-targets/aot-runtime-compiler-observer/results/2026-07-31-macos27-m4max` passed on arm64 macOS 27.0 build 26A5388g and an Apple M4 Max reporting Apple9 support. It offline-compiled one MSL 4.0/macOS 26 metallib, loaded the native library, resolved its function, and prepared its compute pipeline without a source-compilation input or selector. Three clean processes were byte-identical. Two GPUCompiler library images were already loaded at process start and the population did not change through any route stage. Both direct file scans were unavailable, so no build string was recovered and absence inside the mapped image is not claimed. `MTLCompiler.framework` was present on disk but absent from the loaded-image population.

**Fact:** dyld loaded-image membership, membership deltas, readable image strings, and unavailable scans do not associate a translator or compiler with a particular native library or pipeline preparation. The classifier therefore reports only `loaded-image-membership-and-image-byte-scan` evidence and exact attribution `unavailable`; a structurally coherent fixture carrying two plausible synthetic compiler builds produces the same unavailable result. The validator was observed rejecting an altered result status, missing stage metadata, an unavailable-host status, and a compiled binary carrying a real `newLibraryWithSource` call. The byte-stability and observation-drift checks were each perturbed and observed refusing, and an unrelated synthetic GPUCompiler image and build string remained non-attributable.

**Inference:** `metalfe-32023.921` remains valid provenance for the numerical experiment's distinct source-JIT comparison and is not evidence about the private translator/compiler that native pipeline preparation may consume. Exact native translation identity remains `Unknown`. The dependent applicability and parent profile tickets now fail closed pending `authorize-macos-environment-identity-for-native-metal-translation`, which must decide whether the exact measured OS, architecture, device name, and Apple family are a sufficient applicability authority. No unsafe production observer is justified by this result.
