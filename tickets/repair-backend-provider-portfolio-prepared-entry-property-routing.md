---
id: repair-backend-provider-portfolio-prepared-entry-property-routing
title: Repair backend-provider portfolio prepared-entry property routing
status: in-progress
priority: p1
dependencies: []
related: [replace-flat-selected-lowering-capability-keys-with-structured-subjects]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, evidence, runtime]
claimed_from: todo
assignee: repair_backend_portfolio
lease_expires_at: 1786921403
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
git rev-parse HEAD
git merge-base --is-ancestor 98669e8ea9cafc91b3a9139ff821781560c526bd HEAD
git show --name-only --format= b23f0722 -- spikes/runtime/backend-provider-portfolio
rg -n -F 'answers that query with 1,024' spikes/runtime/backend-provider-portfolio/README.md
rg -n -F 'This scalar host owns no prepared-entry property' spikes/runtime/backend-provider-portfolio/src/cpu.rs
rg -n 'prepared_entry_target_requirements|TargetPropertyRequest \{|resolve_target_properties' spikes/runtime/backend-provider-portfolio/src/portfolio.rs crates/tiler-runtime/src/load.rs crates/tiler-runtime/src/load/route.rs
```

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
