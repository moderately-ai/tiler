---
id: repair-backend-provider-portfolio-prepared-entry-property-routing
title: Repair backend-provider portfolio prepared-entry property routing
status: done
priority: p1
dependencies: []
related: [replace-flat-selected-lowering-capability-keys-with-structured-subjects]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, evidence, runtime]
---
## User-visible outcome

The backend-provider portfolio again completes its documented CPU route through the shared artifact and compares all twelve output bit patterns with `tiler-reference`, while continuing to fail closed for genuinely unowned prepared-entry properties.

## Exact-base evidence

**Fact audit repaired at implementation base `e2522345d571d5088ce47039e4399b7247e7bc47`.** The earlier subject base `98669e8ea9cafc91b3a9139ff821781560c526bd` is an ancestor, not the exact implementation base. The failure remains reproducible on the implementation base.

- **Verified.** The complete spike README says the CPU attempt "answers that query with 1,024" (source anchor: `The CPU attempt uses route_with_adapter / prepare`), but `CpuAdapter::observe_prepared_entry` returns `PreparedEntryObservation::Unrecognized` for every request (source anchor: `This scalar host owns no prepared-entry property`).
- **Verified.** `portfolio::push_plan_variant` forwards each compiler-minted `prepared_entry_target_requirements` record with its entry into `DeferredPredicateSpec`; `DecodedProgram::prepare` constructs `TargetPropertyRequest` from the decoded requirement; and `RoutePreparation::resolve_target_properties` maps `Unrecognized` to `UnownedPreparedEntryProperty` while retaining the comparison itself.
- **Imprecise, repaired.** The related structured-capability commit `b23f0722` changes 41 files overall. Within this spike it changes only `spikes/runtime/backend-provider-portfolio/src/portfolio.rs`, projecting the selected compiler subject into the artifact-owned subject; it does not change the runtime adapter or its property authority.

Reproduce the audit from the repository root:

```sh
git rev-parse e2522345d571d5088ce47039e4399b7247e7bc47
git merge-base --is-ancestor 98669e8ea9cafc91b3a9139ff821781560c526bd e2522345d571d5088ce47039e4399b7247e7bc47
git show --name-only --format= b23f0722 -- spikes/runtime/backend-provider-portfolio
git show e2522345d571d5088ce47039e4399b7247e7bc47:spikes/runtime/backend-provider-portfolio/README.md | rg -n -F 'answers that query with 1,024'
git show e2522345d571d5088ce47039e4399b7247e7bc47:spikes/runtime/backend-provider-portfolio/src/cpu.rs | rg -n -F 'This scalar host owns no prepared-entry property'
git grep -n -E 'prepared_entry_target_requirements|TargetPropertyRequest \{|resolve_target_properties' e2522345d571d5088ce47039e4399b7247e7bc47 -- spikes/runtime/backend-provider-portfolio/src/portfolio.rs crates/tiler-runtime/src/load.rs crates/tiler-runtime/src/load/route.rs
git diff --name-only e2522345d571d5088ce47039e4399b7247e7bc47..HEAD
```

The first six commands read the historical implementation base; the final command separately enumerates the repaired tip's exact changed-path population.

Reproduce from the repository root:

```sh
cd spikes/runtime/backend-provider-portfolio
CARGO_TARGET_DIR=./target cargo run
```

Observed after the program compiles and its CPU/Metal artifact members are assembled:

```text
cpu.load: runtime.unowned-prepared-entry-property: no adapter decided variant 1's predicate 0 for prepared entry 0, tiler.target.prepared-entry.max-threads-per-workgroup.v1 from tiler::prepared-entry-properties@1 required 1 observed-at-least-required
```

The live run exits 1 before the documented `cpu.route_with_adapter` bit comparison. A standalone `cargo check --manifest-path spikes/runtime/backend-provider-portfolio/Cargo.toml` passes, so compilation alone does not expose the evidence drift.

## Implementation evidence

**Measurement — repaired implementation subject `e0e4580d1c2f3df4cc6b6c2fd4c5472080bd474b`, macOS arm64, repository-pinned toolchain.** The documented non-recording run exits zero. It reports both `cpu_only.route_with_adapter 12 elements agree with tiler-reference` and `cpu.route_with_adapter 12 elements agree with tiler-reference`; this host also reports the same 12-element agreement for Metal. The retained cross-family controls still refuse as `runtime.no-eligible-variant` / `UnsupportedRepresentation`.

The same run perturbs each prepared-entry subject independently and reports:

```text
probe.prepared_entry_unknown_provider  runtime.unowned-prepared-entry-property: ... tiler.target.prepared-entry.max-threads-per-workgroup.v1 from acme::prepared-entry-properties@1 required 1 observed-at-least-required
probe.prepared_entry_unknown_property  runtime.unowned-prepared-entry-property: ... tiler.target.prepared-entry.unknown-property.v1 from tiler::prepared-entry-properties@1 required 1 observed-at-least-required
probe.prepared_entry_required_above_observed  runtime.unsatisfied-deferred-predicate: ... tiler.target.prepared-entry.max-threads-per-workgroup.v1 from tiler::prepared-entry-properties@1 required 1025 observed-at-least-required observed 1024
```

Each probe's check was made deliberately red by restoring only its subject to the owned or satisfied value. All three runs exited 1 with `cpu.load: the perturbed prepared-entry request routed successfully`, proving the negative controls reach the requests rather than merely exercising their assertions.

**Gate repair — exact-base comparison.** `cargo clippy --all-targets -- -D warnings` reports ten diagnostics at exact base `e2522345`: one assigning clone, five bounded Metal style diagnostics, two pre-existing portfolio arity diagnostics, one needless profile ownership diagnostic, and the fixture writer's arity diagnostic. The first routing repair added exactly three more: probe-assembler arity, needless CPU-profile ownership, and needless owned packaging. The final tree fixes all thirteen without lint suppression: it groups the paired portfolio plans, borrows profiles and built artifacts, groups fixture-record fields, and applies the direct mechanical Clippy suggestions in the CPU and Metal spike adapters. The warnings-denied Clippy command now exits zero; the dependency-only `block v0.1.6` future-incompatibility notice remains informational and is not a Clippy diagnostic.

**Negative control — unavailable Metal toolchain, not a hardware measurement.** Running the already-built executable with `PATH=/nonexistent` exits zero, reports `metal.unavailable` with `could not run xcrun`, completes `cpu_only.route_with_adapter` with 12 agreeing elements, and executes all three prepared-entry probes against the CPU-only artifact. This changes no host component and supports only the missing-toolchain branch; it does not claim how a host with no Metal device behaves after payload production.

## Required delivery

- Re-audit the complete deferred-predicate construction, runtime ownership declaration, adapter query, and refusal paths at the implementation base.
- Establish the narrow authority that lets the CPU adapter answer this governed maximum-threads-per-workgroup request without inventing answers for unknown providers or property keys.
- Align the README claim and reproducible run with the implemented authority.
- Re-run the CPU-only and shared-portfolio routes and retain their fail-closed cross-family and unknown-property controls.
- Perturb the request provider, property key, and required value independently and quote the resulting refusal or comparison failure for each.

## Non-goals

Changing compiler lowering-capability identity, artifact schema, target-profile authority, Metal device policy, or introducing a general backend-family fallback.

## Closes when

The documented non-recording run completes the CPU comparison on the current tree; a Metal-unavailable host still reports only the Metal leg as unavailable; unknown provider/property requests remain unowned; the exact governed request is answered under a source-backed authority; targeted checks and `tkt lint` pass; and an evidence-sensitive review reports no findings.
