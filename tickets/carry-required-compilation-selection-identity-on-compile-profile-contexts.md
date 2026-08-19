---
id: carry-required-compilation-selection-identity-on-compile-profile-contexts
title: Carry required compilation-selection identity on compile-profile contexts
status: in-progress
priority: p1
dependencies: [record-the-compilation-selection-in-target-measurement-provenance, refuse-unknown-fact-source-provenance-schemas-in-artifact-decode, decide-the-compilation-selection-provenance-public-and-wire-surface, resolve-the-retained-metal-profile-measurement-invocation-authority]
related: [split-metal-profile-measurement-sources-by-compilation-selection]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, provenance, identity, numerics, public-boundary, fail-closed]
claimed_from: todo
assignee: worker-selection-carrier
lease_expires_at: 1787151418
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

## Exact-base Fact audit — 2026-08-19 at `350a367e1672d7925c477f6b349af0662d8e4b1a`

Worker `worker-selection-carrier` on branch `tkt/carry-required-compilation-selection-identity-on-compile-profile-contexts`, clean tree verified before any edit. Read in full before this audit: repository `AGENTS.md`; this ticket; the accepted packet `decide-the-compilation-selection-provenance-public-and-wire-surface` (including §4's reservation, §5's construction rules, the adapter RETIREMENT acceptance, and both reviews); the (R, R) record and closure in `resolve-the-retained-metal-profile-measurement-invocation-authority`; `record-the-compilation-selection-in-target-measurement-provenance`; `reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records`; `crates/tiler-ir/src/numerics.rs`; `crates/tiler-compiler/src/target.rs` and all nine `target/` submodules; `crates/tiler-artifact/src/program/realization.rs` and `realization/codec.rs`; `crates/tiler-metal-aot/src/input.rs` and `identity.rs`; `crates/tiler-build/src/lib.rs`, `metal_profile.rs`, `metal_declaration.rs`, and `metal_subgroup_declaration.rs`; and both 2026-08-18 record directories.

### Verdicts

1. **Verified — schema constant.** `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` is 3 (anchor `This constant was introduced at 3`, `crates/tiler-ir/src/numerics.rs`). `RETIRED_FACT_SOURCE_PROVENANCE_SCHEMAS` is the empty slice in `crates/tiler-artifact/src/program/realization/codec.rs`, whose dispatch reads `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION => decode_provenance_v3` — the constant-dispatch hazard the accepted packet's "dispatch literally `4 => decode_provenance_v4`" instruction closes.
2. **Verified — the adapter still exists and its retirement is this ticket's work.** `pub fn declare_metal_f32_subnormal_behaviour` at `crates/tiler-build/src/metal_profile.rs`, re-exported from `crates/tiler-build/src/lib.rs`; `BoundMetalDeclarationError::SubnormalProjection(MetalF32TargetProfileError)` is the production caller's error route; `declare_metal_bf16_subnormal_behaviour` is already private with the `UnstatedBf16SubnormalBehaviour` / `Bf16SubnormalProjection` shape the retirement branch mirrors for F32.
3. **Verified — Metal derivation authority.** `CompileRequest::compile_flags` emits `-target`, `-std=`, one `-O` flag, then the three `NumericalRealization::flags()`; `link_flags` returns `Vec::new()` unconditionally (anchor `the vector is the reserved seam`, `crates/tiler-metal-aot/src/input.rs`); `ApplePlatform::sdk` derives the selector; `ApplePlatform::ALL` and `MslVersion::ALL` are `variant_count`-sized; `#![feature(variant_count)]` is unconditional in `crates/tiler-metal-aot/src/lib.rs` and absent from `crates/tiler-build/src/lib.rs` (only `cfg_attr(test, …)` gates exist in ir/compiler/artifact). The `crates/tiler-metal-aot/src/` census is exactly seven Rust files.
4. **Verified — the three unframed owner domains at their pre-step values.** `tiler.artifact-program.delivered-realization.v2\0` (`realization.rs`, `TargetEvidence`'s encoder ends `source.encode(bytes)`); `tiler.target-profile.descriptor.v10\0` (`target/feasibility.rs`, reached through `encode_honourability_facts`); `tiler.compiler.selected-physical-plan.v2\0` (`selection.rs`, `encode_honoured` raw-appends `HonouredDimension::canonical_key`). The complete declaration `tiler.target-profile.declaration.v11\0` frames each source with `push_slice` and stays. `EXPLAIN_RENDERER_VERSION` is 9, `EXPLAIN_SCHEMA_VERSION` 11.
5. **Imprecise — "nineteen scalar rows" is twenty-one at this base.** `ae927f28` added the reciprocal-transform and approximate-intrinsics rows before this ticket's dispatch. Recounted from `BoundMetalCompileDeclaration::declare` in full: **eighteen** measured declaration operations producing **twenty-six** canonical rows — grid 1, cost 1, tree-width 1, dispatchability 2, four subnormal-dimension operations expanding to 12 rows, nine remaining F32 numerical operations — of which **twenty-one** are scalar honourability rows. This matches the (R, R) ticket's corrected census and changes nothing load-bearing: the encode-once contract ("exactly once per structurally unique source") is population-independent. Both `encode_declaration_table` (`target/honourability.rs`) and `complete_descriptor` (`target/descriptor.rs`) still call `source.canonical_bytes()` once per declaration during collection and once more per row lookup, confirming the allocation-multiplier defect.
6. **Verified — the (R, R) records exist with production selection.** `spikes/target-profiles/metal-grid-axis-extent/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/` pins `selection.compile_flags -target air64-apple-macos26.0 -std=metal4.0 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`, `selection.link_flag_count 0`, `ladder.max_extent 268435456`, harness and result hashes, and execution row macOS 27.0 `26A5406e` / Apple M4 Max. `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/calibration.txt` pins `parallel_threads 1.280001e3` with held-out separation retained; `environment.tsv` pins the same execution row, harness hashes, and sole-occupancy loads.
7. **Verified — scope note.** `crates/tiler-compiler/src/target.rs` is the facade plus tests; the nine submodules named in the brief exist; `request.rs` was split into `request/`.

### Boundary finding — the source partition cannot land truthfully without a minimal row reseat

The brief carves ledger third-environment tables and profile-row reseating out to `reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records`, unless the partition cannot land truthfully without them. It cannot, and the reasons are mechanical:

- The old grid record's invocation (`xcrun --sdk macosx metal -std=metal4.0 -target air64-apple-macos26.0 -c`, no optimization or numerical flag) is not representable by `CompileRequest`, and the old cost record's harness is unrecoverable — both established in the (R, R) packet. Under the accepted construction rules a grid or cost population citing the old `26A5388g` contexts has no constructible expected selection: the declaration would refuse as `CompilationSelectionMismatch` and `first_macos_apple9()` would return `Err`, a noncompiling partition this ticket forbids.
- Citing the 2026-08-18 records is the accepted (R, R) disposition and gives both populations the production selection — but their execution row is `26A5406e`, so the grid and cost source contexts must move to that environment, and the cost record's fit is `parallel_threads = 1,280`, not the currently declared `1_056`. Attributing 1,056 to a record that fits 1,280 would be a value the cited evidence does not state.

**Disposition taken, loudly rather than silently:** this migration carries the minimal forced reseat — the grid and cost populations cite the 2026-08-18 records (new execution contexts; grid value unchanged at 268,435,456, which the new record verifies; cost value moved to 1,280) — isolated in its own commit for coordinator review. The authority ledger's third-environment tables, the old rows' disposition prose, and the remaining doc mirrors stay with `reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records`. The alternative dispositions are all worse: an untruthful value, a noncompiling declaration, or a temporary (W, W) withdrawal contradicting the accepted (R, R).

### Discovered population outside the packet's enum — the M3 Pro subgroup evidence fixture

`crates/tiler-build/src/metal_subgroup_declaration.rs` (crate-private since the 2026-08-18 demotion) constructs its own `TargetCompileProfileMeasurementSource`, so the required-selection type reaches it too. Its retained invocation (`spikes/target-profiles/metal-thread-execution-width`, `CompilerSelection::ProfileStrict`) passed `-std=metal4.0 -target air64-apple-macos26.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off` — no optimization flag and a different flag order — so it is **not** representable by `CompileRequest::compilation_selection_identity()` and must not pretend to be. It is carried as a caller-vouched fixture selection transcribing the retained record's exact recorded flags, in the fixture's own vocabulary, with no Metal-production authentication claim — exactly the generic caller-authored route the accepted packet preserves. It is deliberately not a variant of `MetalProfileMeasurementPopulation`, whose census is the standard declaration's four authoritative populations.
