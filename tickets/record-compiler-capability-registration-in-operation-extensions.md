---
id: record-compiler-capability-registration-in-operation-extensions
title: Record compiler capability registration in the operation-extension contract
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions]
---
docs/operation-extensions.md (around line 104-105) states "Compiler capability registration is not [implemented]: its owner is prototype-operation-capability-registry." That ticket is now complete and merged (`crates/tiler-compiler/src/capability.rs`), so the sentence is stale. The capability-registry agent correctly left this untouched because the file is contracts/foundation, outside its implementation scope.

Update the passage to record that compiler capability registration for the index/access and scalar-lowering families is now implemented in tiler-compiler, while keeping the accurate distinction the same paragraph already draws: the registry resolves available lowering knowledge and provenance but does NOT prove an occurrence was lowered correctly — that checked refinement remains owned by prototype-semantic-index-refinement. Verify against the merged code before restating: the module is `tiler_compiler::capability`, registration is transactional with deterministic collision/ambiguity diagnostics, provenance excludes TypeId and addresses, and providers emit only through the canonical IndexRegionBuilder via narrow contexts. Do not overstate — this is a reviewed prototype public surface, not a stabilized compiler-session API, and scheduled-kernel and opaque physical providers remain owned by their own later tickets. Run the full documentation gate before completion.

## Outcome

Rewrote the closing paragraph of the "Registry lifecycle and coherence" section in `docs/operation-extensions.md` (previously lines 104-113) to record that compiler capability registration for the index/access and scalar-lowering families is now implemented in `tiler_compiler::capability`, merged from `prototype-operation-capability-registry`. Each restated property was verified against the merged `crates/tiler-compiler/src/capability.rs`:

- module path is `tiler_compiler::capability` (`crates/tiler-compiler/src/lib.rs:6` `pub mod capability;`; crate `tiler-compiler`);
- registration is transactional per call, rejecting a duplicate as a deterministic `DuplicateCapability` collision, and resolution reports deterministic `AmbiguousCapability` / `MissingCapability` diagnostics with candidates in canonical provider order (`capability.rs:745-822`, `913-951`, `1156-1208`);
- canonical provenance is built from durable semantic/provider identities and canonical projections, never `TypeId`, vtable, function, registration, or allocation addresses (`capability.rs:205-213`, `1222-1289`);
- providers emit only through the canonical `IndexRegionBuilder` wrapped by the narrow `ScalarLoweringContext` / `IndexAccessLoweringContext`, which never expose the raw builder, region finalization, or provider-owned IR construction (`capability.rs:350-641`); and
- the two covered families are `LoweringFamily::IndexAccess` and `LoweringFamily::ScalarLowering` (`capability.rs:66-73`).

Preserved the distinction that the registry resolves available lowering knowledge and provenance but does not prove an occurrence was lowered correctly; that checked refinement remains owned by the in-progress `prototype-semantic-index-refinement`. Avoided overstating: labelled the surface a reviewed prototype boundary rather than a stabilized compiler-session API, and noted that scheduled-kernel and opaque physical providers remain owned by their own later tickets. The header status line ("compiler capabilities proposed") and frontmatter were left unchanged, since the contract remains a proposed prototype interface pending Tom's acceptance. Documentation gate, `git diff --check`, and scope guard all passed.
