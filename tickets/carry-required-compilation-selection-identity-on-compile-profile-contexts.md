---
id: carry-required-compilation-selection-identity-on-compile-profile-contexts
title: Carry required compilation-selection identity on compile-profile contexts
status: blocked
priority: p1
dependencies: [record-the-compilation-selection-in-target-measurement-provenance, refuse-unknown-fact-source-provenance-schemas-in-artifact-decode, decide-the-compilation-selection-provenance-public-and-wire-surface]
related: [split-metal-profile-measurement-sources-by-compilation-selection]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, provenance, identity, numerics, public-boundary, fail-closed]
---
## User-visible outcome

Every compile-profile measurement context states exactly which backend compilation selection produced its facts. Missing, empty, malformed, or mismatched selection evidence is rejected; no profile, backend, or governed default is inferred.

## Accepted boundary

The decision is recorded in `record-the-compilation-selection-in-target-measurement-provenance`. The exact surface is a required backend-opaque canonical byte identity carried by a compile-profile-specific context. Runtime/device contexts remain separate.

For Metal, derive the identity from the same `tiler-metal-aot::CompileRequest` authority that emits the SDK selector, target/platform selection, ordered compiler flags, and ordered linker flags. Exclude source and resolved toolchain facts. Do not duplicate the selection spelling in `tiler-build`.

## Required delivery

- Audit and repair every Fact in this ticket at its implementation base before editing.
- Introduce the smallest public opaque type and required compile-profile constructor/accessor; no `Option`, `Default`, empty sentinel, implicit governed selection, or conversion from a target profile.
- Retain exact bytes, not only a digest. Reject empty input and enforce the broad complete-descriptor ceiling before proportional allocation; do not add a smaller unexplained cap.
- Prove the compiler/IR surface contains no Metal type or flag interpretation.
- Derive Metal bytes beside the invocation authority and perturb SDK selector, target, language standard, optimization, numerical flags, and linker flags independently. Each must move selection identity and profile descriptor.
- Follow the accepted linker-control rule from the decision prerequisite. The current production linker run is always empty, so do not fabricate a helper-only flag and call it a production-subject perturbation.
- Keep source and resolved toolchain perturbations in their existing provenance fields; they must not be duplicated into selection.
- Step `FACT_SOURCE_PROVENANCE_SCHEMA_VERSION` and every unframed owner domain that actually changes. Re-derive outer-domain steps rather than copying this ticket's expectation.
- Update artifact encode/decode/read views, explanation, contracts, domain ledgers, all pins, and public API docs coherently.
- Make malformed, empty, missing, and facts-versus-selection mismatch cases fail with typed errors before descriptor or artifact construction.
- Partition the authoritative `tiler-build` grid, cost, dispatchability, and numerical sources in this same compiling change. Derive each selection from the Metal authority and compare facts in backend/build ownership; generic IR/compiler code cannot interpret opaque Metal selection bytes. This atomic migration owns the production portion originally deferred to `split-metal-profile-measurement-sources-by-compilation-selection` so no intermediate revision misattributes a row.

## Performance boundary

This adds one bounded linear encode/compare at profile construction and identity hashing. It does not run in kernel execution or physical-plan search. Measure only if profiling shows this small provenance record is material.

## Closes when

Two otherwise equal compile-profile contexts with different exact backend selections have different canonical provenance and descriptors, while no caller can construct a compile-profile context without choosing one selection explicitly.
