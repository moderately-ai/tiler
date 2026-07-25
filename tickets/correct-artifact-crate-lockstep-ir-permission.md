---
id: correct-artifact-crate-lockstep-ir-permission
title: Correct the artifact crate's retired lockstep IR permission
status: done
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

## Outcome

The opening sentence of `crates/tiler-artifact/src/lib.rs` now states the boundary ADR 0070 and ADR 0071 leave, in the crate's own voice: it depends on `tiler-ir` for the shared target-neutral representation, retains programs that crate has already verified, rebuilds decoded values through its checked constructors, owns no second editable program model, and never invokes compiler passes. The retired lockstep-IR permission is gone; the retained prohibition on invoking compiler passes is kept.

Each clause was verified by reading before it was written, because the ticket's stated risk was asserting something the code does not do:

- **Fact — the only dependency is `tiler-ir`.** `crates/tiler-artifact/Cargo.toml`'s `[dependencies]` table contains exactly `tiler-ir.workspace = true`. There is no `tiler-compiler` edge to invoke a pass through, so the prohibition is structural rather than merely documented.
- **Fact — programs are retained, not rebuilt.** `crates/tiler-artifact/src/program/model.rs:387` stores `pub(super) program: VerifiedKernelProgram` — the shared IR's own verified product, held verbatim. There is no artifact-owned duplicate program model.
- **Fact — no IR builder is invoked.** `grep -rn "KernelProgramBuilder\|ScheduledRegionBuilder\|SemanticProgramBuilder\|lower_scheduled_region"` over `program/builder.rs`, `program/model.rs`, `program/verify.rs`, and `program/codec/*.rs` returns one hit, and it is prose: `program/codec/mod.rs:64` explaining why a decoder *cannot* call `KernelProgramBuilder::new`. Non-test code calls none of them.
- **Fact — decoded leaves go through checked constructors.** `program/codec/decode.rs` reconstructs interface and provenance values via `InputKey::from_owned` (lines 349, 1007), `OutputKey::from_owned` (line 367), `ProviderIdentity::from_owned` (line 923), and `Shape::try_from_dims` (line 905), each mapping the shared IR's rejection into a typed codec error.

**Wording decided against the ticket's own phrasing.** The ticket proposed "reconstructs values through that crate's checked builders and verifiers". The landed sentence says *checked constructors* instead, because the preceding two facts show the decode path calls no `tiler-ir` builder at all — it calls checked leaf constructors and retains an already-verified program. Writing "builders" would have reintroduced exactly the class of defect the ticket exists to remove: a crate doc asserting a capability the crate does not exercise.

Nothing else in the module doc changed; the paragraph beneath the opening sentence was already written to the post-ADR-0070 world and now agrees with the sentence above it. No contract, ADR, or public item changed — this is a doc comment on an existing crate. Validated by the full repository gate.
