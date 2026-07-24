---
id: correct-artifact-crate-lockstep-ir-permission
title: Correct the artifact crate's retired lockstep IR permission
status: todo
priority: p3
dependencies: []
related: [record-an-adr-for-the-metal-aot-crate-admission]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, documentation, artifact]
---
`crates/tiler-artifact/src/lib.rs`'s module documentation grants the crate a permission an accepted ADR retired, in the same sentence that states the boundary that was retained.

**Fact — the crate says it.** Line 3 of `crates/tiler-artifact/src/lib.rs` reads: "This crate may depend on lockstep prototype IR types, but it must never call compiler passes."

**Fact — half of that is retired.** ADR 0056's fourth Consequences bullet originally read "`tiler-artifact` may use lockstep internal IR types during the prototype but may not invoke compiler passes", and now carries an in-body marker: "**Retired:** ADR 0070 gives `tiler-ir` sole ownership of the shared target-neutral representation and makes `tiler-artifact` depend on it directly, and ADR 0071 requires artifact decoding to reconstruct values through those checked builders and verifiers. There is therefore no lockstep compiler-internal type to borrow and no artifact-owned duplicate program model to introduce later. The retained boundary is the unchanged prohibition on invoking compiler passes." The crate doc preserves exactly the clause that was retired and exactly the clause that was retained, without distinguishing them.

**Why it matters more than a stale comment usually would.** The rest of the same module doc is already written to the post-ADR-0070 world — it describes projecting a verified `tiler_ir::program::VerifiedKernelProgram` into the artifact model and says "Nothing here requires a consumer to link `tiler-compiler`, to reconstruct a semantic graph, a region cover, a cost model, or a search state". The opening sentence contradicts the paragraph beneath it. A reader who takes the opening line at face value would conclude that borrowing a compiler-internal type is still sanctioned during the prototype, which ADR 0070 removed the possibility of and ADR 0071 forbids the artifact side of.

**What closes this.** Rewrite the sentence so it states the boundary ADR 0070 and ADR 0071 actually leave: `tiler-artifact` depends on `tiler-ir` for the shared target-neutral representation, reconstructs values through that crate's checked builders and verifiers, owns no second editable program model, and never invokes compiler passes. Keep it to the crate's own voice — this is a doc comment, not a place to restate either ADR. Verify by reading `crates/tiler-artifact/src/lib.rs` and `crates/tiler-artifact/src/program/` in full before editing, since the fix is one sentence and the risk is asserting something about the crate that its code does not do.

Scope is `implementation/artifact` alone. Found while auditing the ADR corpus for `record-an-adr-for-the-metal-aot-crate-admission`; it is a code-documentation correction, not a contract or decision change, and needs no ADR.
