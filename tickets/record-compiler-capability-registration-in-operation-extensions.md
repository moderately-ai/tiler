---
id: record-compiler-capability-registration-in-operation-extensions
title: Record compiler capability registration in the operation-extension contract
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contracts, extensions]
claimed_from: todo
assignee: agent-record-compiler-capability-registration-in-operation-extensions
lease_expires_at: 1784834740
---
docs/operation-extensions.md (around line 104-105) states "Compiler capability registration is not [implemented]: its owner is prototype-operation-capability-registry." That ticket is now complete and merged (`crates/tiler-compiler/src/capability.rs`), so the sentence is stale. The capability-registry agent correctly left this untouched because the file is contracts/foundation, outside its implementation scope.

Update the passage to record that compiler capability registration for the index/access and scalar-lowering families is now implemented in tiler-compiler, while keeping the accurate distinction the same paragraph already draws: the registry resolves available lowering knowledge and provenance but does NOT prove an occurrence was lowered correctly — that checked refinement remains owned by prototype-semantic-index-refinement. Verify against the merged code before restating: the module is `tiler_compiler::capability`, registration is transactional with deterministic collision/ambiguity diagnostics, provenance excludes TypeId and addresses, and providers emit only through the canonical IndexRegionBuilder via narrow contexts. Do not overstate — this is a reviewed prototype public surface, not a stabilized compiler-session API, and scheduled-kernel and opaque physical providers remain owned by their own later tickets. Run the full documentation gate before completion.
