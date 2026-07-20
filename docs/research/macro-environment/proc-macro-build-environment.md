# Proc-macro build environment and freshness

**Status:** measured contract; rust-analyzer and a genuinely different installed
Rust target remain unmeasured on this host

## Question

Which target and freshness facts can a stable inline procedural macro actually
observe, and how can Tiler select Apple artifact families without a consumer
build script, source scan, registry, or prepare step?

## Result

Artifact-family selection is an explicit, canonical input to an inline
invocation. A macro must not infer the consumer target from its host process or
from build-script variables that Cargo does not promise to procedural macros.
The selection may name one or several governed families, such as `macos`,
`ios-device`, and `ios-simulator`; each family produces a separate artifact
identity and target manifest.

A Metal toolchain change is a rebuild boundary. If expansion runs, Tiler
fingerprints the selected compiler/SDK and cannot reuse an incompatible cache
entry. Cargo does not track that external state, however, so changing Xcode or
deleting Tiler's cache does not itself make an otherwise fresh macro expansion
run. The user or CI must cause a rebuild after changing the selected Apple
toolchain. This limitation is diagnosed and documented, not hidden behind an
undocumented analysis-mode heuristic.

## Reproducible fixture

[`spikes/macro-environment/run.sh`](../../../spikes/macro-environment/run.sh)
checks no-change freshness, unrelated consumer edits, macro-crate edits, cache
deletion, a simulated compiler-fingerprint change, and `cargo test` expansion.
[`run-target.sh`](../../../spikes/macro-environment/run-target.sh) performs the
same environment capture for an explicitly selected installed Rust target.

The proc macro records only a fixed allowlist of non-secret environment names.
It simulates a content-addressed entry using the canonical invocation tokens and
an explicit `TILER_TOOLCHAIN_FINGERPRINT`; the simulation is evidence about
freshness, not the cache protocol.

## Measurement environment

- macOS on Apple silicon
- `rustc 1.97.0 (2d8144b78)` and Cargo 1.97.0
- Xcode 26.6, build 17F113; macOS SDK reported as 26.5
- installed Rust target: `aarch64-apple-darwin` only
- rust-analyzer component: not installed; the rustup proxy alone is not an
  executable analyzer

No toolchain component was installed or mutated for this experiment.

## Observations

### Environment

During native and explicit `--target aarch64-apple-darwin` expansion, these
were absent:

- `HOST`, `TARGET`, and `CARGO_BUILD_TARGET`;
- all probed `CARGO_CFG_TARGET_*` variables;
- `OUT_DIR`, `PROFILE`, `OPT_LEVEL`, `DEBUG`, and `RUSTC`;
- `SDKROOT`, `MACOSX_DEPLOYMENT_TARGET`, and
  `IPHONEOS_DEPLOYMENT_TARGET`.

`CARGO_MANIFEST_DIR` and `CARGO_PKG_NAME` described the consumer package.
Invocation tokens remained observable and contained the explicit family list.
These are host measurements, not a claim that Cargo can never expose another
variable. Correctness therefore relies only on documented macro inputs and
Tiler-owned configuration.

### Freshness

| Change | Did expansion run? | Simulated cache result |
|---|---:|---:|
| first `cargo check` | yes | miss |
| identical `cargo check` | no | n/a |
| fingerprint environment only | no | n/a |
| unrelated consumer source edit | yes | miss under new fingerprint |
| cache deletion only | no | n/a |
| next consumer source edit | yes | miss |
| proc-macro crate source edit | yes | hit |
| `cargo test` after check | yes, for additional compilation contexts | hit |

Thus the external cache is load-bearing when rustc chooses to expand, but it is
neither a generated-code dependency nor an input Cargo tracks for freshness.

### Unavailable measurements

A truly different cross target could not be executed because only the native
Rust standard library is installed. `run-target.sh` fails closed and lists the
installed targets rather than downloading one. rust-analyzer cold/warm behavior
could not be measured because the component is absent. Neither gap weakens the
contract: Tiler does not consume implicit target variables or depend on an IDE
mode. They remain useful performance measurements when a suitable environment
already exists.

## Contract

1. `ArtifactFamilySelection` is a typed compiler request field serialized into
   explain output, manifests, and cache identity.
2. Each selected Apple family has an explicit platform, SDK identity,
   deployment minimum, Metal language standard, compiler flags, and payload.
3. No family defaults from the proc-macro host. A frontend may provide a
   documented literal default profile, but that profile is still canonical
   invocation/configuration input rather than inferred process state.
4. Generated Rust selects only among embedded compatible families and retains
   a semantically compatible fallback where the integration contract allows
   one.
5. On every actual expansion, Tiler resolves and fingerprints the toolchain
   before cache lookup. A changed fingerprint cannot hit the old entry.
6. After changing Xcode, its selected developer directory, SDK contents, or
   Tiler's explicit toolchain configuration, users and CI must force the
   affected consumer crate to rebuild. A clean build is the portable recovery
   operation.
7. Cache deletion alone never invalidates already emitted Rust or compiled
   binaries. The next expansion reconstructs a valid entry or reports a hard
   compiler/artifact error.
8. No correctness or diagnostic behavior depends on undocumented
   rust-analyzer environment variables or on detecting `cargo check` versus
   code generation.

## Options rejected

- **Infer the consumer target from `TARGET` or `CARGO_CFG_*`.** Those are
  build-script contracts and were absent in the measured macro process.
- **Treat host macOS as consumer macOS.** This silently emits the wrong family
  for iOS, simulator, Catalyst, or a non-Apple target.
- **Generate every family unconditionally.** Correct but needlessly expensive,
  and it can require SDKs the consumer never selected.
- **Use a required build script, registry, scan, or prepare command.** These
  violate the accepted inline developer experience.
- **Rely on a rust-analyzer analysis stub.** No stable analysis-mode contract
  exists, and divergent expansion would risk type and diagnostic drift.

## Primary documentation

- [Rust procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Cargo environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- [Experimental tracked proc-macro inputs](https://doc.rust-lang.org/proc_macro/tracked/index.html)
