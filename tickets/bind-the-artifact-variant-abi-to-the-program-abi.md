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

## Scope

Decide and implement how a variant's ABI is bound to its program's. The candidates, with what each preserves:

- **Derive.** `push_variant` reads the program's arena, guard, launch and accessible ranges and replays them itself; `VariantSpec` stops carrying them. Preserves one authority and removes the transliteration every assembler would otherwise repeat. It must not remove what a program cannot carry: launch preconditions, deferred feasibility predicates, and a portfolio's variant priority are artifact-owned and stay in `VariantSpec`.
- **Check.** `VariantSpec` keeps declaring them and `push_variant` proves each declared expression is content-equal to the program's, by `tiler_ir::program::abi::expr_key`. Preserves the artifact's freedom to re-spell an expression and costs a rejection surface for a re-spelling that means the same thing — which is exactly the freedom that has no use case.

Recommendation: derive. The check branch preserves a freedom nothing wants and leaves two arenas that a reader must diff to trust.

Whatever is chosen, `tiler.artifact-program.v2` identity changes meaning if a variant stops carrying its own expressions, so its domain tag and the `guard_and_routing` schema version are both in scope.

## Closes when

An artifact variant's applicability guard, launch geometry, and per-binding accessible ranges are provably the ones its bound `VerifiedKernelProgram` states; a variant that disagrees is rejected with a typed diagnostic naming the use site; the artifact-owned launch preconditions, deferred predicates, and routing rank are unaffected; any changed identity domain is bumped with its reason recorded at the site; and `uv run --locked python scripts/check_repository.py` passes.
