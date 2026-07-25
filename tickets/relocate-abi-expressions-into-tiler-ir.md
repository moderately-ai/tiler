---
id: relocate-abi-expressions-into-tiler-ir
title: Relocate the ABI expression domain into tiler-ir as ADR 0068 decides
status: todo
priority: p0
dependencies: []
related: [carry-the-metal-payload-in-an-artifact-envelope, complete-program-identity-with-abi-guards-and-routing]
scopes: [implementation/ir, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi, decisions]
---
Accepted ADR 0068 decides where the ABI expression domain lives, and the tree does not match it. That divergence is currently blocking artifact assembly, so this is a live obstacle rather than tidying.

**Fact — the decision.** `docs/decisions/0068-co-locate-abi-expressions-with-executable-program-ir.md` (`decision_status: accepted`, `implementation_status: spike-only`): "Place the public `AbiExpr` domain type, admitted roots, validation, canonical identity, and authoritative pure checked evaluation semantics with the experimental executable-program representations in `tiler-ir`." And, separately: "`tiler-compiler` owns lowering into and construction of `AbiExpr`; it is not the runtime expression authority."

**Fact — the tree.** `AbiRoot` is defined at `crates/tiler-artifact/src/program/expr.rs:159`. `crates/tiler-ir/src/program/` contains `builder.rs`, `error.rs`, `handles.rs`, `mod.rs`, `model.rs`, `tests.rs`, `verify.rs` and no ABI expression module. `crates/tiler-compiler/Cargo.toml` declares `tiler-ir` as its only dependency, and the compiler carries its own separate `HostExpr`/`HostExprId` vocabulary (`crates/tiler-compiler/src/program.rs`) rather than constructing `AbiExpr` at all.

**Inference — the divergence is what makes artifact assembly look impossible.** `carry-the-metal-payload-in-an-artifact-envelope` needs someone to construct ABI expressions for accessible byte ranges and launch geometry. ADR 0068 says that someone is `tiler-compiler`. With `AbiExpr` in `tiler-ir` the compiler constructs them over a crate it already depends on and no new dependency edge is required. With `AbiExpr` in `tiler-artifact` the only options are adding `tiler-compiler -> tiler-artifact` or exposing the plan as a record for an orchestrator to assemble — and both were considered and rejected on that ticket precisely because each is a durable commitment that this relocation makes unnecessary.

**The rationale the ADR gives, which is the thing to preserve.** Owning `KernelProgram` in `tiler-ir` while owning its expression type in `tiler-artifact` "creates a dependency cycle or leaves the program dependent on an external side table for verification and identity". Its stated consequence is that "a `KernelProgram` remains self-contained and independently verifiable before artifact construction" — which today it is not, because its guards, sizes, and launch geometry cannot be expressed in a type it can reach.

## Scope

Move the `AbiExpr` domain — the node vocabulary, admitted roots, type and availability-phase rules, validation, canonical identity, and checked evaluation — into `tiler-ir`. Leave in `tiler-artifact` exactly what ADR 0068 assigns it: the versioned wire encoding, compatibility policy, runtime fact binding and phase enforcement, failure classification, and backend-payload mappings.

The envelope codec encodes the expression arena and re-derives identity over it, so the encoding, the canonical-order proof, and the arena-closure checks must keep working unchanged; the codec's existing adversarial cases are the regression suite for that. Do not weaken any of them to make the move fit.

## Also settle, because the move forces the question

The compiler's `HostExpr`/`HostExprId` is a second expression vocabulary covering the same ground — guards, byte counts, launch geometry — built as a fixed nine-node table for the serial-sum subject (`crates/tiler-compiler/src/program.rs:903`). Once `AbiExpr` is reachable from `tiler-compiler`, decide explicitly whether `HostExpr` is replaced by it or retained with a stated reason. Two expression vocabularies for one set of facts is the drift hazard ADR 0068 exists to prevent, and leaving both unexamined would reintroduce it one layer down.

## Closes when

`AbiExpr` lives where ADR 0068 places it, `tiler-compiler` can construct one without a new dependency edge, the codec's encoding and adversarial cases pass unchanged, ADR 0068's `implementation_status` reflects reality, and `uv run --locked python scripts/check_repository.py` passes.
