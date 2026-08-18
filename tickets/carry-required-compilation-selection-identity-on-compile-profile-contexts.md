---
id: carry-required-compilation-selection-identity-on-compile-profile-contexts
title: Carry required compilation-selection identity on compile-profile contexts
status: blocked
priority: p1
dependencies: [record-the-compilation-selection-in-target-measurement-provenance, refuse-unknown-fact-source-provenance-schemas-in-artifact-decode, decide-the-compilation-selection-provenance-public-and-wire-surface, resolve-the-retained-metal-profile-measurement-invocation-authority]
related: [split-metal-profile-measurement-sources-by-compilation-selection]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, provenance, identity, numerics, public-boundary, fail-closed]
---
## User-visible outcome

Every compile-profile measurement context states exactly which backend compilation selection produced its facts. Missing, empty, malformed, or mismatched selection evidence is rejected; no profile, backend, or governed default is inferred.

## Accepted boundary

The semantic decision is recorded in `record-the-compilation-selection-in-target-measurement-provenance`. The exact Rust/wire surface, schema retirement, Metal grammar, adapter branch, and identity cascade remain Tom-gated in `decide-the-compilation-selection-provenance-public-and-wire-surface`. Implement only the packet Tom accepts. Runtime/device contexts remain separate.

For Metal, derive the identity from the same `tiler-metal-aot::CompileRequest` authority that emits the SDK selector, target/platform selection, ordered compiler flags, and ordered linker flags. Exclude source and resolved toolchain facts. Do not duplicate the selection spelling in `tiler-build`.

## Required delivery

- Audit and repair every Fact in this ticket at its implementation base before editing.
- Introduce the smallest public opaque type and required compile-profile constructor/accessor; no `Option`, `Default`, empty sentinel, implicit governed selection, or conversion from a target profile.
- Retain exact bytes, not only a digest. Reject empty input and enforce the exact 64-KiB complete-descriptor ceiling before proportional allocation; do not add a smaller unexplained cap.
- Prove the compiler/IR surface contains no Metal type or flag interpretation.
- Derive Metal bytes beside the invocation authority. Perturb the production request's platform/target, language standard, optimization, and each numerical flag independently. SDK selector is derived from `ApplePlatform::sdk`, so pin the type-sized `ApplePlatform::ALL` mapping rather than inventing an override. Each reachable change must move selection identity and profile descriptor.
- Follow the accepted linker-control rule from the decision prerequisite. The driver selects `metallib` with `xcrun --sdk <sdk> --find metallib`, then passes the AIR input and `-o` output to the resolved binary; the current additional-linker-flag run after tool/SDK selection is always empty. Do not fabricate a helper-only flag and call it a production-subject perturbation.
- Keep source and resolved toolchain perturbations in their existing provenance fields; they must not be duplicated into selection.
- Step `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` and every unframed owner domain that actually changes. Re-derive outer-domain steps rather than copying this ticket's expectation.
- Update artifact encode/decode/read views, explanation, contracts, domain ledgers, all pins, and public API docs coherently.
- Close the current governed/external phase-laundering route: exact triples are validated for every evidence basis and the public raw provenance assemblers are narrowed or removed as the accepted packet requires.
- Apply exactly the adapter branch Tom selects in the accepted packet. Retention preserves the caller-vouched transactional `declare_metal_f32_subnormal_behaviour(builder, facts, source)` surface and its non-authenticating ADR/contract language; retirement performs the packet's exact public/error/ADR/contract removal. Generic `TargetProfileBuilder::declare_measured_*` routes remain caller-authored in either branch and make no Metal-production authentication claim.
- Make malformed, empty, missing, phase-incoherent, and facts-versus-selection mismatch cases fail with typed errors before descriptor or artifact construction.
- Add the packet's type-sized `MetalProfileMeasurementPopulation::ALL` census and the required unconditional `#![feature(variant_count)]` gate in `tiler-build`; do not substitute a hand-sized list.
- Repair checked and complete source-table construction so structurally equal `FactSourceProvenance` references are deduplicated before canonical encoding, every structurally unique source is encoded exactly once, canonical byte ordering/collision collapse stays byte-identical, and row loops use precomputed source indexes. One source reused by nineteen scalar rows must not allocate nineteen canonical copies before the descriptor limit is checked.
- Partition the authoritative `tiler-build` grid, saturated cost, workgroup-tree-width, dispatchability, and numerical sources in this same compiling change. Carry each retained population's independently derived expected selection, construct its source contexts from that identity, and compare it to the production `CompileRequest`; generic IR/compiler code cannot interpret opaque Metal selection bytes. Any differing recorded-invocation disposition must first amend the accepted packet with both the sealed Metal authority and an enforced population-specific transfer/applicability rule. Tree-width and dispatchability/numerical may share only after exact equality. Grid and cost follow the accepted dispositions in `resolve-the-retained-metal-profile-measurement-invocation-authority`. This atomic migration owns the production portion originally deferred to `split-metal-profile-measurement-sources-by-compilation-selection` so no intermediate revision misattributes a row.

## Performance boundary

This adds one bounded linear encode/compare at profile construction and identity hashing. It does not run in kernel execution or physical-plan search. Measure only if profiling shows this small provenance record is material.

## Closes when

Two otherwise equal compile-profile contexts with different exact backend selections have different canonical provenance and descriptors, while no caller can construct a compile-profile context without choosing one selection explicitly.
