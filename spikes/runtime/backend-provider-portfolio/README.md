---
schema: "tiler-doc/v1"
id: "tiler.spike.runtime.backend-provider-portfolio"
kind: "experiment"
title: "Standard Metal, custom Metal, and CPU in one portfolio"
topics: ["runtime", "backends", "pluggability", "metal", "cpu", "artifacts"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.extensions.backend-provider-composition"]
entrypoints: ["spikes/runtime/backend-provider-portfolio/src/main.rs"]
last_verified: "2026-08-12"
verified_at_commit: "61246804"
ticket: "exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio"
---

# Standard Metal, custom Metal, and CPU in one portfolio

One retained end-to-end proof that composes two physical authorities under the Metal family plus one CPU family, packages their valid alternatives, routes separate explicit Metal and CPU attempts, and matches `tiler-reference`.

There is no `BackendProvider` bundle and no family-fallback policy. Custom Metal is a physical-provider row on the Metal family. Each attempt states one `ExecutionEnvironment`.

## Run it

From this directory. `rust-toolchain.toml` is resolved by directory ancestry from the repository root, so no selector is passed and this spike carries no toolchain file of its own.

```sh
cd spikes/runtime/backend-provider-portfolio
CARGO_TARGET_DIR=./target cargo run
```

Passing a path records the result fixture instead of only printing the narrative:

```sh
CARGO_TARGET_DIR=./target cargo run -- results/2026-08-12-macos-arm64.json
```

CPU always runs. Metal payload production needs the system `xcrun` toolchain; Metal dispatch needs a bound device. Either absence is reported as `metal.unavailable` and is a non-zero-free skip of that leg only — the run does not silently pass a Metal claim. `CARGO_TARGET_DIR` is set explicitly because this is a nested workspace.

Spikes gate nothing. The `Makefile` has no target for this directory.

## What one run does, in order

1. **Builds one semantic program** — the smallest pointwise shape the compiler admits, `(input * 2.0) * 1.0` over a 4×3 `f32` tensor — and evaluates it through `tiler-reference`.
2. **Compiles twice against `BoundMetalCompileDeclaration::first_macos_apple9`** under `FLUSH_SUBNORMALS_TO_ZERO_F32`. Once with `CompileRequest::with_physical_providers(InstalledPhysicalProviders::installed([acme]))`, once with only the governed provider.
3. **Records offered and selected physical provenance.** `Compilation::offered_physical_providers` names governed then `acme::simdgroup-pointwise-metal@4` when installed. Two retained alternatives name those providers separately. Removing the custom provider leaves the governed alternative and does not name acme.
4. **Assembles CPU** through `assemble_plan_artifact` into `tiler.cpu.scalar` / `tiler.cpu.scalar-image-v1`.
5. **Assembles Metal** through `accept_or_publish_metal_plan` (which itself calls `assemble_plan_artifact`) into `tiler.metal` / `metallib`.
6. **Proves `check_subject`'s one-target pin:** pushing a second variant under a different descriptor refuses as `TargetProfileMismatch`.
7. **Packages one portfolio** whose members share that Apple variant-level `TargetProfileRef` and vary backend, representation, payload, and compilation subject.
8. **Cross-family preflight.** The Metal-only artifact under a CPU `ExecutionEnvironment`, and the CPU-only artifact under a Metal environment, refuse as `runtime.no-eligible-variant` / `UnsupportedRepresentation` before work. Against the combined portfolio the loader would select the matching family instead of refusing; that is eligibility, not fallback.
9. **Routes two explicit attempts** through `route_with_adapter`. CPU always. Metal only when a device binds.
10. **Compares twelve output bit patterns** against `tiler-reference`. Perturbs the envelope (flipped byte, truncation, foreign backend) and watches each refuse.

## Result

**Measurement**, Apple arm64 macOS, repository-pinned nightly, base `61246804`:

| Stage | Result |
| --- | --- |
| offered with custom | `tiler::prototype-serial-sum-physical@1`, `acme::simdgroup-pointwise-metal@4` |
| selected with custom | one alternative names acme, one names governed |
| offered without custom | governed only |
| CPU envelope | 52,734 bytes |
| Metal envelope | 58,149 bytes |
| shared portfolio | 94,717 bytes |
| CPU vs reference | 12/12 bits equal |
| Metal vs reference | 12/12 bits equal |
| mixed-target merge | `ArtifactBuildError::TargetProfileMismatch` |
| Metal under CPU env | `runtime.no-eligible-variant` / `UnsupportedRepresentation` naming `tiler.metal`/`metallib` |
| CPU under Metal env | `runtime.no-eligible-variant` / `UnsupportedRepresentation` naming `tiler.cpu.scalar`/`tiler.cpu.scalar-image-v1` |

The twelve output bit patterns are in [`results/2026-08-12-macos-arm64.json`](results/2026-08-12-macos-arm64.json). The NaN operand `0x7fc01234` is canonicalized to `0x7fc00000` on every path.

## Facts this run depends on

**Fact — `STRICT_F32` is infeasible against the Apple declaration.** `BoundMetalCompileDeclaration::first_macos_apple9` assesses `FLUSH_SUBNORMALS_TO_ZERO_F32`. Compiling the same program under `STRICT_F32` refuses as `NoFeasiblePlan`. The operand vector therefore omits subnormals, because a host-preserving CPU interpreter would disagree with Metal and with a strict reference evaluation of those values.

**Fact — a combined portfolio does not refuse a matching family.** Presenting the packaged portfolio under a CPU environment selects the CPU variant; presenting it under a Metal environment selects the Metal variant. The cross-family refusal the ticket names is therefore observed on each *family's own* assembled artifact, not on the combined envelope.

**Fact — the CPU variant of a Metal-assessed plan carries deferred predicates.** The Apple profile answers workgroup capacity as a prepared-entry query. `DecodedProgram::preflight` alone therefore refuses the CPU member as `runtime.deferred-predicates`. The CPU attempt uses `route_with_adapter` / `prepare` and answers that query with 1,024.

## What this spike does not claim

- It is not a production CPU backend and installs no `tiler-cpu` crate.
- It does not invent a composition facade, a family-fallback policy, or a production crate change.
- It does not compare the two Metal physical providers on cost: both survive under the same structural estimate. Cost-comparability stays the open question ADR 0090 recorded.
- Metal numbers are a fact about this host's device and toolchain in the interval the run was taken.
