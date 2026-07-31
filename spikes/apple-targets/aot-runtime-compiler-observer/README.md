# AOT-compatible Metal runtime-compiler observer

This bounded experiment asks whether native metallib loading and compute-pipeline preparation expose an exact runtime-compiler identity without runtime source compilation. It deliberately accepts a negative answer: loaded-image evidence that cannot be attributed to the exact preparation remains weaker than a compiler identity and cannot satisfy host eligibility.

The harness compiles `kernel.metal` offline before launching `probe`. The probe accepts only those metallib bytes, snapshots compiler-related dyld images at process start and after device, library, function, and pipeline preparation, and scans each loaded image for embedded `metalfe-*` build strings. `validate.sh` rejects probe source containing the `newLibraryWithSource` selector, while the Objective-C program exposes no source-compilation input or mode, so source JIT cannot silently enter the measurement.

Run from the repository root:

```sh
spikes/apple-targets/aot-runtime-compiler-observer/run.sh \
  spikes/apple-targets/aot-runtime-compiler-observer/results/local
```

`run.sh` retains three clean-process observations, then preloads the first compiler-related image seen after pipeline creation and repeats the route. That negative control distinguishes “this image is loaded” from “this route caused this image to load.” It also records whether the separate `MTLCompiler.framework` is present on disk, compares repeated clean results byte for byte, and classifies the evidence independently of any particular image or build count. Before retaining `validation.tsv`, it proves rejection of a mutated result, absent stage metadata, an unavailable host, and the source-JIT sentinel, then proves that two synthetic plausible build strings still classify as unavailable rather than becoming an exact attribution.

The result applies only to the recorded host, OS, device, SDK, and native route. A path or build string present on disk, present in a process, or newly loaded during one stage is not by itself proof that the named compiler answered the preparation. Production use additionally requires a stable attribution contract; otherwise the exact runtime-compiler predicate remains unavailable.

## Result on 2026-07-31

The retained `results/2026-07-31-macos27-m4max/` run produced the negative outcome. Two `GPUCompiler.framework` library images were already loaded at process start, before the default device, native metallib, function, or pipeline existed. The same two images remained after every stage in three byte-identical clean processes; no new compiler-related image appeared, and neither loaded image exposed a `metalfe-*` build string. `MTLCompiler.framework` was present on disk and absent from the loaded-image population, independently reproducing why disk presence is not execution evidence.

This establishes only loaded-image membership and refutes stage attribution through dyld deltas on this route. It does not refute that private Metal components participate internally; it shows that the available observation cannot identify which compiler build, if any, answered native pipeline preparation. Even a newly loaded image or one or more embedded build strings would remain membership evidence unless an API associates that component with this preparation. The exact runtime-compiler predicate therefore remains unavailable for host eligibility. The policy must not substitute `metalfe-32023.921` from the source-JIT numerical measurement, the OS build, the framework directory name, or either preloaded library path.

**Inference — AOT applicability consequence.** The native route consumes the offline-produced metallib, the live OS/architecture/device environment, and Metal's library and pipeline APIs; it does not consume the source-JIT compiler measured by `newLibraryWithSource`. The first AOT profile must therefore retain the offline compiler in artifact provenance and qualify native execution by the measured OS, architecture, device name, and Apple family. The distinct source-JIT compiler identity remains `Unknown` and outside the AOT eligibility predicate unless a future route actually consumes it and supplies attributable evidence.
