---
id: honor-the-precise-fp32-metal-compilation-requirement
title: Honor the precise FP32 Metal compilation requirement
status: done
priority: p0
dependencies: []
related: [decide-the-tiler-metal-public-facade-surface]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

An emitted Metal unit requiring precise FP32 elementary functions is admitted by `tiler-build` when the AOT request selects `Fp32Functions::Precise`, and is refused when it selects `Fast`. The build adapter no longer silently classifies that already-supported requirement through a wildcard.

## Exact-base discovery — 2026-08-17 at `73af3a9a484320891553d3d575926b349ecb6b93`

**Fact.** `tiler_metal::record::MetalNumericalRequirement` has three variants: `SafeMathMode`, `NoFloatingPointContraction`, and `PreciseFp32Functions`. `tiler_metal::golden_compilation::realization_honours` maps all three exhaustively and answers the last with `realization.fp32_functions == Fp32Functions::Precise`.

**Fact.** `tiler_build::metal_assembly::validate_numerical_selection` maps only the first two and uses `_ => false`. `tiler-metal` emits `PreciseFp32Functions` for `F32Exp` and `F32Rsqrt`, so the wildcard is live: it refuses a precise request as `UnsatisfiedNumericalRequirement`.

**Inference.** This repair is required independently of whether the whole `tiler-metal` facade is accepted. The facade decision additionally owns removing `#[non_exhaustive]` from the requirement vocabulary under ADR 0074 convention 5b; this ticket does not make that public change.

## Implementation keys

- Add the exact `PreciseFp32Functions => numerical.fp32_functions == Fp32Functions::Precise` arm. Do not parse `flag()`/`Display`, default a future requirement, or broaden `SafeMathMode`.
- Add a real elementary-function unit fixture. Prove `metal_compile_request` and `prepare_metal_payload` accept the precise selection and reject `Fast` with the exact `PreciseFp32Functions` cause.
- Perturb the selected `fp32_functions` subject and record the failing assertion. A test that only mutates its expectation is not evidence.
- Preserve target facts, emitted source, payload metadata, identities, schemas, and pins byte-for-byte.

## Stop boundary

No public type, variant, method visibility, module maturity statement, artifact byte, or identity version changes here. `decide-the-tiler-metal-public-facade-surface` owns the separate exhaustiveness/public-compatibility decision.

## Closes when

Both request derivation and prepared-payload validation accept the precise requirement only under the precise AOT selection; the fast neighbour refuses by the exact cause; the subject perturbation is recorded; focused build/Metal tests, Clippy, rustdoc, ticket lint, citation check, exact-base guard, and the proportional repository gate pass.

## Implementation and review — 2026-08-17

Exact worker commit `5eca21de2b13c6a3882213ee79bc83438b78cdbe` over published base `b085f9dcd95c77ecdf42e93d3e083f02a584a4a8` adds only the typed `PreciseFp32Functions` comparison and its real exponential-unit test. The test proves precise request derivation and prepared-payload validation, then independently proves `Fast` refuses both paths with the exact `PreciseFp32Functions` cause. Before the arm, the positive failed with that refusal; after the arm, changing only the selected FP32-functions subject to `Fast` failed the unchanged positive.

Independent exact-commit review read the complete changed file and all requirement/emission/AOT consumers, reproduced three independent production-subject perturbations, and found no implementation defect. Package test, nextest, check, Clippy with warnings denied, rustdoc with warnings denied, doctest, formatting, lint, citations, diff, and exact-base scope guard all passed. No public surface, emitted source, target fact, payload metadata, identity, schema, pin, Cargo manifest, or lockfile moved. The review separately exposed stale public accessor prose; `correct-metal-numerical-requirements-doc-after-precise-elementary-emission` owns that independent documentation repair rather than widening this P0.
