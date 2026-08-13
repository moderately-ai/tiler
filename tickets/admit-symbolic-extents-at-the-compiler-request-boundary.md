---
id: admit-symbolic-extents-at-the-compiler-request-boundary
title: Admit symbolic extents at the compiler request boundary
status: done
priority: p1
dependencies: [construct-a-symbolic-region-as-a-semantic-program]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, shapes, extents]
---
## User-visible outcome

A symbolic semantic program reaches the compiler and is either planned with its extents symbolic or declined with a typed reason naming the unsupported case — never silently specialized and never refused for a reason that names the wrong authority.

## Why this exists

**Fact.** `CompilationRequest::shape_environment` is a `StaticShapeEnvironment` whose only field is a `schema_version: u32`, and `verify_request` refuses anything but `StaticShapeEnvironment::governed()`. Reproduce with `grep -n "struct StaticShapeEnvironment" -A 10 crates/tiler-compiler/src/request.rs`. It carries no symbol; it is a version gate reserving the seam.

**Fact.** The accepted specialization boundary keeps runtime extents symbolic in the logical plan by default and makes specializing an extent a physical-planning decision.

## Implementation keys

- Replace the version-only gate with a request that carries the program's own environment, rather than a second environment the caller supplies beside the program. Two environments over one program is the ambiguity `IndexRegionBuilder::new_with_shape_environment` exists to prevent.
- A normalization or capability that cannot handle a symbolic extent declines with its own typed reason. Do not let a symbolic program fall through to an existing refusal that names a different rule; the inline AOT proof already recorded how expensive a mis-attributed `UnsupportedCapability { rule: "signature" }` was to diagnose.
- Specialize nothing. A physical alternative may introduce an explicit guard that makes an extent constant within that alternative; the request boundary must not fold a value into the logical plan on the way in.
- State the measurement boundary: which normalizations admit a symbolic extent and which decline is a fact about this commit's capability set, not a claim about the compiler.

## Evidence

- A symbolic program reaches strategy selection rather than being refused before it.
- An unsupported symbolic case declines with a reason naming the symbolic extent, and the literal neighbour of the same program still compiles.
- A test asserting that no compiled plan folds a bound extent value, so the specialization refusal is checked rather than described.

## Public boundary

`CompilationRequest`'s shape-environment field is crate-internal today; if admitting the environment widens a public surface, that widening is Tom's and must be listed rather than absorbed.

## Fact audit — 2026-08-13 at base `cd1f76da2f40fa7805f543f77a2394a2ef744aae`

Re-read this session: `crates/tiler-compiler/src/request.rs` (`StaticShapeEnvironment`, `CompilationRequest`, `verify_request`, `static_shape`), `crates/tiler-ir/src/semantic/program.rs` (`try_standard_with_shape_environment`, `extent_sources`, `input_sourced`), `tickets/construct-a-symbolic-region-as-a-semantic-program.md` Outcome, and the accepted specialization boundary in `docs/research/shapes/shape-environment-contract.md`. The ticket's purpose is unchanged.

- **Verified.** `StaticShapeEnvironment` is a version-only gate. Durable anchor: `pub(crate) struct StaticShapeEnvironment` whose only field is `schema_version: u32`, and `governed()` stamps `REQUEST_SCHEMA_VERSION`. `rg -n "struct StaticShapeEnvironment" -A 12 crates/tiler-compiler/src/request.rs`.
- **Verified.** `CompilationRequest` is `pub(crate)` and carries that field. Durable anchor: `pub(crate) struct CompilationRequest<'a>` with `pub(crate) shape_environment: StaticShapeEnvironment`. No public `CompileRequest` field or setter names a shape environment.
- **Verified.** `verify_request` refuses anything but `StaticShapeEnvironment::governed()` as `UnsupportedRequestVersion`. Durable anchor: `if request.shape_environment != StaticShapeEnvironment::governed()`. The same variant is also the capability-schema refusal (`request.capabilities.schema_version != REQUEST_SCHEMA_VERSION`). The Display text `unsupported static shape environment` therefore names the leftover gate, not a program whose extents are symbolic: a symbolic program built through `CompilationRequest::governed` already passes this check, because every constructor stamps `governed()`.
- **Verified.** The accepted specialization boundary keeps runtime extents symbolic in the logical plan by default and makes specializing an extent a physical-planning decision. Durable anchor in `docs/research/shapes/shape-environment-contract.md`: `runtime extents remain symbolic in the logical plan by default. Specializing an extent to a concrete value is a physical-planning decision`.
- **Verified (coordinator-unverified construction path).** `construct-a-symbolic-region-as-a-semantic-program` is `done`. A symbolic program is built through `SemanticProgramBuilder::try_standard_with_shape_environment` on the program's one `Arc<ShapeEnv>` and `input_sourced`; the program exposes that environment through `SemanticProgram::extent_sources`. Durable anchors: `A constructor rather than a setter, and there is no setter` on `try_standard_with_shape_environment`, and `Returns the environment this program's symbolic extents resolve against` on `extent_sources`. Two environments over one program is the ambiguity `IndexRegionBuilder::new_with_shape_environment` exists to prevent. The request field is therefore the leftover version gate, not a missing environment: replace the gate with the program's own environment rather than adding a second one.
- **Imprecise (working observation, not a ticket Fact).** Recognition's `static_shape` already meets a symbolic value, but it reports the caller's handle rule (`output-handle`, `input-handle`) as `UnsupportedCapability` rather than naming the extent. Durable anchor: `A symbolic extent is refused here rather than resolved through the environment` and `.ok_or(RequestError::UnsupportedCapability { phase: "strategy", rule })`. That is the mis-attribution this ticket exists to close at the request boundary. Normalization, region formation, and program assembly already refuse a symbolic shape under `symbolic-extent` / `program-input-symbolic` / `named-output-symbolic`; they are later stages and, after this ticket, unreachable on the compile path because strategy selection declines first.

## Implementation record — 2026-08-13

`StaticShapeEnvironment` is gone. `CompilationRequest::shape_environment` is `Option<&ExtentSources>` populated from `program.extent_sources()`; `verify_request` refuses a pairing that is not that exact environment as `MismatchedShapeEnvironment`, not `UnsupportedRequestVersion`. Recognition's `static_shape` declines a symbolic value as `UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent }`, naming the extent as written. A bound symbol is still that refusal: the request boundary does not fold `ExtentSources::determined` into the logical plan.

Public `CompileRequest` still has no shape-environment field or setter. The public comment now says the environment is the program's own rather than "exactly one governed value". `CompilationRequest` and both new `RequestError` variants stay `pub(crate)`.

### Measurement boundary at this commit

This is a fact about this commit's capability set, not a claim about the compiler.

- **Request admission admits a symbolic program.** The request carries the program's environment and does not refuse for having one.
- **Strategy selection declines every symbolic extent.** Every recognized strategy is still stated over fixed extents (launch geometry, element counts, reindex/broadcast decode). The first `static_shape` on a symbolic value is the refusal, and it names the extent.
- **Normalization, region formation, and program assembly would still decline** under `symbolic-extent` / `program-input-symbolic` / `named-output-symbolic`, but they are unreachable on the compile path because strategy selection declines first. Normalization still rebuilds through `SemanticProgramBuilder` without a shape environment, so it remains fail-closed for identity reasons if a rewrite ever reached it.
- **No family at this commit is planned with extents left symbolic.** The approved elementwise region constructs and reaches strategy selection; it does not get a `NormalizedProgram`.

### Support matrix

No operation-family support-matrix row and no dtype-maturity cell moves. The compiler request now admits a symbolic program as far as strategy selection; that is a request-boundary seam, not a family-admission widening.

### Evidence

- `a_symbolic_program_reaches_strategy_selection` — the request carries the program's `ShapeEnv`, and `verify_request` returns `UnsupportedSymbolicExtent` at `phase: "strategy"`, not `UnsupportedRequestVersion`.
- `an_unsupported_symbolic_case_names_the_extent_and_the_literal_neighbour_compiles` — `(a * b) + c` over `f32[n]` refuses naming `program/0::n`; the same program over `f32[4]` compiles.
- `a_compiled_plan_does_not_fold_a_bound_extent_value` — an environment that pins `n == 4` still authors `SourcedExtent::Symbol(n)` on every value, and compilation still refuses that symbol rather than emitting a `[4]` plan.
- `dropping_the_program_environment_is_a_pairing_refusal` — `shape_environment = None` is `MismatchedShapeEnvironment`, Display `compile.request.shape-environment: request must carry the program's own environment`.

### Perturbations

Subject, not assertion, each watched failing once:

1. **Drop the program environment** (`governed_preferring` stamps `None`):
   ```
   thread 'request::tests::a_symbolic_program_reaches_strategy_selection' panicked at crates/tiler-compiler/src/request.rs:12123:22:
   a symbolic program carries its environment
   ```
2. **Mis-attribute the refusal** (restore `static_shape`'s handle-rule `UnsupportedCapability`):
   ```
   assertion `left == right` failed
     left: Err(UnsupportedCapability { phase: "strategy", rule: "output-handle" })
    right: Err(UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent: Symbol(...) name: "n" })
   ```
3. **Specialize a bound** (fold `ExtentSources::determined` inside `static_shape`):
   ```
   assertion `left == right` failed: a bound symbol must still be refused as the symbol, not compiled as 4
     left: Err(UnsupportedCapability { phase: "strategy", rule: "elementwise-shape" })
    right: Err(UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent: Symbol(...) name: "n" })
   ```

### Commands

From this worktree, after the implementation:

- `cargo test -p tiler-compiler` — 777 lib tests passed, 1 ignored; 83 integration tests passed; 13 doc-tests passed (2 rustdoc + 11 compile-fail).
- `cargo clippy -p tiler-compiler --all-targets -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler --no-deps` — clean.
- `tkt lint` — `ok: no problems found`.
- `git diff --check` — clean.
- `tkt guard --base main --format json tkt/admit-symbolic-extents-at-the-compiler-request-boundary` — `severity: warn`, `conflict: false`, `under_declared: []`. Direct `implementation/compiler` collisions with other live compiler tickets; shared `project/tickets`. Not under-declared.

Did not run `make full`. Coordinator gates at integration.
