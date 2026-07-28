---
id: bind-the-artifact-variant-abi-to-the-program-abi
title: Bind the artifact variant ABI to the program ABI
status: todo
priority: p1
dependencies: [complete-program-identity-with-abi-guards-and-routing]
related: [prototype-artifact-program-model]
scopes: [implementation/artifact, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi, identity]
---
**Fact — two ABIs now describe one program, and nothing binds them.** `complete-program-identity-with-abi-guards-and-routing` gave `tiler_ir::program::VerifiedKernelProgram` its own ABI expression arena, applicability guard, per-stage launch geometry, and per-access accessible byte range, and folded all four into `tiler.kernel-program.v2` identity. `tiler_artifact::program`'s `VariantSpec` still declares its own guard, its own `LaunchSpec`, and its own per-binding `accessible_bytes`, on its own arena, under the separately versioned `guard_and_routing` schema.

**Fact — they are checked against the same third thing, not against each other.** `crates/tiler-artifact/src/program/builder.rs::check_bindings` proves a variant's accessible-byte expression evaluates to `access.view().window().length`, and `check_launch` proves its workgroup width equals `stage.kernel().requirements().threads_per_workgroup`. `crates/tiler-ir/src/program/builder.rs::check_stage_accesses` and `::check_stage_launch` prove exactly the same two equalities for the program's own expressions. So the two agree on every *value* by construction and on no *expression*: an assembler may package a variant whose accessible range is `UnsignedLiteral(24)` over a program whose own range is `rows * columns * 4`, and both verify.

**Why that matters and why it is not urgent.** Under static shapes the two forms coincide at every admissible binding, so nothing observable diverges today. Under dynamic shapes they do not: the artifact's expression is the one a runtime evaluates, and the program's is the one identity folds, so a cache hit keyed on program identity could serve an artifact whose runtime ABI computes a different number. The exact check that establishes the current state is `grep -n "accessible_bytes" crates/tiler-artifact/src/program/builder.rs crates/tiler-ir/src/program/builder.rs`.

**The one existing consumer already does the right thing by hand.** `prototypes/serial-sum-compile/src/bundle.rs::assemble` transliterates the program's arena onto the artifact's and resolves each variant use site from the replayed handle map, so its variant ABI *is* the program's. That is a producer convention, not a checked one, and this ticket is about making the artifact layer require it.

## User-visible outcome

The runtime ABI packaged for a variant is the ABI of the verified program it
carries. Program-owned applicability, launch, and accessible-range expressions
are derived from the program; callers cannot independently restate them.
Artifact-owned routing priority, launch preconditions, and deferred feasibility
remain explicit.

Whatever is chosen, `tiler.artifact-program.v2` identity changes meaning if a variant stops carrying its own expressions, so its domain tag and the `guard_and_routing` schema version are both in scope.

## Closes when

An artifact variant derives its applicability guard, launch geometry, and
per-binding accessible ranges from its bound `VerifiedKernelProgram`; callers
cannot construct a disagreement; artifact-owned launch preconditions, deferred
predicates, and routing rank are unaffected; any changed identity domain is
bumped with its reason recorded at the site; and `make full` passes.

## Ready to implement 2026-07-27 — shape confirmed, ADR 0075 status named

**Verified, so the next session does not re-derive it:**

- `VariantSpec` is `pub` with `pub applicability_guard: AbiExprId` (`crates/tiler-artifact/src/program/builder.rs:172`), and `EntrySpec`'s launch and per-binding `accessible_bytes` are likewise caller-supplied. Removing those three is a **public API removal**, so ADR 0075 applies: it may be built as a tested concrete draft, but the interface is Tom's to accept. `AGENTS.md` states that split in terms.
- The divergence is real but latent, exactly as recorded: the two ABIs are each checked against the same third value and never against each other, so under static shapes they coincide at every admissible binding and under dynamic shapes they need not.

**The implementation shape, which the ticket already settles.** `prototypes/serial-sum-compile/src/bundle.rs::assemble` transliterates the program's arena onto the artifact's and resolves each variant use site from the replayed handle map — its variant ABI *is* the program's. That is the convention to make checked: the builder performs the transliteration itself, and the three fields come off `VariantSpec`.

**One thing that got easier while this ticket waited.** `flatten-artifact-expression-identity` landed the shared arena primitives — `canonical_arena_traversal` and `compare_expr_nodes` are `pub` in `tiler_ir::program::abi`, and the artifact identity already numbers its arena from a use-site root list. A transliteration that derives the variant's expressions from the program now has one numbering to agree with rather than two key encodings, so the binding is a smaller change than when this ticket was written.

**Both version bumps are in scope and neither is optional**, per the ticket: `ARTIFACT_DOMAIN` (now `v5`, so `v6`) because a variant that stops carrying its own expressions changes what the identity means, and the `guard_and_routing` component schema for the same reason.
