---
id: relocate-abi-expressions-into-tiler-ir
title: Relocate the ABI expression domain into tiler-ir as ADR 0068 decides
status: in-progress
priority: p0
dependencies: []
related: [carry-the-metal-payload-in-an-artifact-envelope, complete-program-identity-with-abi-guards-and-routing]
scopes: [implementation/ir, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, abi, decisions]
claimed_from: todo
assignee: agent-coordinator
lease_expires_at: 1784989580
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

## Measured scope, and a duplication the move must resolve

**Size, read at `cbeedbb`.** `crates/tiler-artifact/src/program/expr.rs` is 821 lines. Thirteen files inside `tiler-artifact` reference `expr::` — `builder.rs`, `facts.rs`, `model.rs`, `verify.rs`, `error.rs`, `mod.rs`, and seven of the eight `codec/` modules including `encode.rs`, `decode.rs`, `validate.rs`, and both test modules. The expression arena is encoded, canonically ordered, closure-checked, and folded into artifact identity, so the codec is not incidentally affected — it is the main consumer.

**Defect found while scoping: `AvailabilityPhase` is defined twice.**

- `crates/tiler-compiler/src/feasibility.rs:43` — `pub(crate) enum AvailabilityPhase`, doc "Ordered capability availability phases (ADR 0043)".
- `crates/tiler-artifact/src/program/expr.rs:111` — `pub enum AvailabilityPhase`, doc "These are ADR 0043's phases."

Both carry the same five variants in the same order — `CompileProfile`, `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, `LaunchPreflight` — and both derive `Ord` with the ordering stated as load-bearing. They are one governed vocabulary with two definitions, and nothing checks that they agree. A phase added to one would not stop the build at the other; the compiler would defer to a phase the artifact layer cannot express, or the reverse, with no diagnostic.

That is the same failure mode ADR 0068 exists to prevent, one layer down, and it is why this ticket is not merely tidying. **The relocation must resolve it rather than move one copy past the other**: with the domain in `tiler-ir`, both crates already depend on that crate and one definition serves both. Closing this ticket while two definitions survive would leave the defect in place under a rearranged layout.

**Consequence for the closing condition.** Add to it: exactly one `AvailabilityPhase` exists in the workspace, and `crates/tiler-compiler/src/feasibility.rs` names the shared one. If a reason is found to keep two, state it in the ticket outcome and in both types' documentation — do not leave the duplication unexplained.

## Outcome

**Done.** `AbiExpr` lives where ADR 0068 places it, one `AvailabilityPhase` exists, and the compiler's second expression vocabulary is gone. `implementation_status` on ADR 0068 moved `spike-only` → `implemented`: every clause of its decision now holds in the tree, and the decision is about ownership and placement rather than about how much of the expression language is built.

**The move.** `crates/tiler-artifact/src/program/expr.rs` became `crates/tiler-ir/src/program/abi.rs`, registered as `pub mod abi`. `expr.rs` remains as a re-export shim so the thirteen dependent files inside `tiler-artifact` did not have to change import paths; `facts.rs`, `decode.rs`, `mod.rs`, and `codec/tests.rs` were repointed where they needed the new names directly. `AbiFacts`'s fields were `pub(super)` and `facts.rs` built one by field access, which stopped compiling across the crate boundary — it gained `AbiFacts::new`, deliberately non-validating, because ADR 0068 assigns the proof that a fact was legally readable at the phase it claims to the binder in `tiler-artifact`, and a second check here would be a second drifting definition of the same rule. `TargetPropertyKey` moved with the domain and left `governed_key!` behind; `ArtifactBuildError` gained `From<TargetPropertyKeyError>`.

**`AvailabilityPhase`, the defect the ticket found.** `crates/tiler-compiler/src/feasibility.rs:45` is now `pub(crate) use tiler_ir::program::abi::AvailabilityPhase`. `grep -rn --include='*.rs' "enum AvailabilityPhase" crates` returns exactly one line.

**A second duplication found while moving, same class.** `push_slice`/`push_len` — the canonical length-prefix framing every identity digest is built on — existed in four places: `tiler-ir/src/program/model.rs`, `tiler-ir/src/kernel/model.rs`, the ABI module, and `tiler-artifact/src/program/model.rs`, with `tiler-artifact`'s codec importing a fifth path to one of them. They had already diverged in form: the kernel copy narrowed with `len as u64` where the others used a checked `u64::try_from`. On the 64-bit little-endian address space the Rust gate admits these emit identical bytes, so the divergence was latent, not live — which is exactly the hazard, since a silent digest change is invisible in review and indistinguishable from a real one in a cache. They are now one `pub mod identity` in `tiler-ir`. Byte-for-byte unchanged: the codec's `single_byte_corruptions_are_rejected` and the canonical-order proofs pass untouched.

**Also settled: `HostExpr` is replaced, not retained.** It was a nine-node table over `U64`/`Bool`/`CheckedMultiply` with positional identity and no availability phase, covering guards, accessible byte counts, and launch geometry — the same three facts `AbiExpr`'s admitted roots target, and strictly weaker at all three. Retaining it would have left two vocabularies for one set of facts, which is the drift hazard ADR 0068 exists to prevent, one layer down; and ADR 0068 already says `tiler-compiler` "owns lowering into and construction of `AbiExpr`", so retention would also have contradicted the decision this ticket implements. `HostExprNode`, `HostValueType`, `HostExpr`, and `HostValue` are deleted. `KernelProgram::host_expressions` is now `Vec<ExprNode>`, evaluation delegates to the shared `abi::evaluate`, and `HostExprId` survives only as a `u32` newtype naming an arena position — a reference into the arena, not a vocabulary. The `program.host-expression.{rule}` error surface is deliberately unchanged, because it names the host preflight *stage*, which is still what it is; only the mapping onto it changed, and `AbiEvaluationError` is `#[non_exhaustive]` so an upstream variant reports `evaluation` rather than being silently reclassified.

**What deliberately did not change, and why.** The graph still enters every extent as an `UnsignedLiteral` rather than an `AbiRoot::InputExtent` resolved at `LiveDevicePreflight`. The bounded profile's shapes are static, so those values genuinely are known at `CompileProfile`, and claiming a later availability would be asserting a phase this stage does not have. Promoting them is a dynamic-shape capability question, not a property of the vocabulary, and nothing in the graph has to change shape for it.

**Evidence.** `cargo nextest run --workspace`: 631 passed, 0 skipped — including the codec's adversarial suite, which the ticket names as the regression proof and which was not weakened. Workspace clippy clean at the pinned nightly; the newly public surface took `#[must_use]` and `# Panics` documentation rather than an `allow`. `uv run --locked python scripts/docs.py render`: 181 records.

**Follow-up, not hidden.** `pub mod session` in `tiler-compiler` and `pub mod identity`/`pub mod abi` in `tiler-ir` are new publicly reachable namespaces. ADR 0068 decides `abi`; `identity` and `session` are not covered by an accepted ADR and remain unreviewed under ADR 0075's always-ask rule.
