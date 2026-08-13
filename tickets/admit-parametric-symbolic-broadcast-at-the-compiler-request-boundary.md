---
id: admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary
title: Admit parametric symbolic broadcast at the compiler request boundary
status: review
priority: p1
dependencies: [carry-the-parametric-broadcast-relation-through-index-and-schedule-ir, admit-symbolic-extents-at-the-compiler-request-boundary]
related: []
scopes: [implementation/compiler, contracts/optimizer, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, shapes, broadcast]
claimed_from: todo
assignee: worker-parametric-broadcast-request
lease_expires_at: 1786645006
---
# Admit parametric symbolic broadcast at the compiler request boundary

## User-visible outcome

A semantic program containing the accepted parametric broadcast reaches physical selection without its symbolic extent being folded, split into row-specific graphs, or refused under an unrelated static-shape rule.

## Work

- Bind the compiler request to the semantic program's exact shape environment; accept no second caller-supplied environment.
- Recognize and retain the parametric broadcast access relation through normalization, region construction, physical subject binding, selection, explanation, and candidate verification.
- Specialize nothing at request admission. A provider that cannot implement the symbolic carrier declines under its own typed rule.
- Ensure any later guarded specialization is an explicitly identified physical alternative rather than a mutation of graph meaning or the baseline route.
- Update governed physical-provider revision/provenance only if the previously admitted context-to-offer function changes; document the comparison.

## Acceptance

- One symbolic program reaches selection with its environment and mapping unchanged.
- Perturbing a bound value does not change semantic, normalized-program, or request identity.
- A provider lacking parametric support declines by the named capability rule; no static-signature or generic unsupported error masks it.
- No selected plan contains a silently substituted concrete reindex/broadcast access unless it is a separately guarded and identified physical alternative.

## Stop conditions

Stop if compiler admission needs the bound extent value, if a second environment can disagree with the program, or if a provider can silently reinterpret the relation through an existing concrete access variant.

## Source-first Fact audit — 2026-08-13, exact base `4333df312a1334b24729abd24ac6a3f7140ed1ac`

- **Verified:** `encode_access_relation` matches `ReindexBijection` as `0x01`, `BroadcastReplication` as `0x02`, `LinearIdentity` as `0x03`, and every other map including `ParametricBroadcast` as `0x00`. The comment says the arm is a refusal to encode rather than a wildcard tag. Anchor: `fn encode_access_relation` and `the arm is a refusal to encode rather than a wildcard tag`.
- **Verified:** `LogicalAccess::ParametricBroadcast { operand_shape, mapping, environment }` exists in `crates/tiler-ir/src/schedule/model.rs` as a labelled draft, schedule tag `0x08`. Anchor: `Labelled draft under ADR 0075`.
- **Verified (coordinator-unverified):** `plan_elementwise` refused every structural family over a non-static domain before `recognize_structural_read` ran. Anchor: `let Some(static_domain) = shape.as_static()`. `recognize_structural_read` then required `static_shape_ref` on both operand and result and always built `BroadcastReplication`. It could not see the parametric carrier.
- **Verified (coordinator-unverified):** request-subject domain is `tiler.compiler.request-subject.v6`. Adding tag `0x05` for a previously unencodable map does not reinterpret an old payload, so the domain does not step. `0x04` remains `UNREAD_DECLARED_INPUT_TAG`. Anchor: `tiler.compiler.request-subject.v6` and `UNREAD_DECLARED_INPUT_TAG`.
- **Verified:** `CompilationRequest::shape_environment` is already the program's own `ExtentSources`. No second caller-supplied environment exists. Anchor: `The program's own environment, never a second caller-supplied one`.
- **Decision:** crate-internal request-subject tag `0x05`, no domain step, no new public compiler facade field. `implementation/ir` added because index-refinement interface comparison still required `as_static()` and would have masked the carrier as `SymbolicBoundary`.

## Implementation record — 2026-08-13

Recognition retains `LogicalAccess::ParametricBroadcast` for a sourced mapping and leaves `BroadcastReplication` / `ReindexBijection` for concrete maps. The request carries the program's own environment. Request-subject encoding writes the carrier as crate-internal tag `0x05` under `tiler.compiler.request-subject.v6`; previously encodable maps keep their bytes. `compile()` of a sourced broadcast-only program reaches physical selection; the governed physical provider declines `UnspellableRegion { rule: "parametric-broadcast" }`, reported as `UnsupportedCapability { phase: "planning", rule: "parametric-broadcast" }`. Same-shape symbolic elementwise without the carrier still declines at schedule as `symbolic-extent`. The governed physical-provider revision stays 1: previously admitted (non-parametric) context-to-offer is unchanged. Index refinement compares sourced boundary shapes so a parametric operand/result is not refused as `SymbolicBoundary`. Governed broadcast lowering emits the parametric realization against the program environment when the mapping names a symbol. The surface remains a labelled draft and is not self-accepted.

### Measurement boundary at this commit

- **Strategy admits a sourced broadcast** as `ParametricBroadcast` with the authored mapping and environment identity.
- **A fused `a * broadcast(w)` over a symbolic activation** is recognized, but its multiply still has a static index law; lowering of that neighbour remains `OperandInterface`. The compile-path program that reaches physical selection is the broadcast-only occurrence.
- **Scheduled-region construction is not invented.** The governed physical provider declines the carrier by name rather than building an `IndexRegion` with a folded launch geometry.
- **A bound symbol is still a symbol.** Variant guards pinning `n` to 4 or 10 do not move semantic, recognized-carrier, or request identity.

### Identity blast radius

None on previously encodable subjects. Static reindex/broadcast/linear maps keep tags `0x01`/`0x02`/`0x03`. Symbolic parametric subjects had no planned request identity before this commit.

### Request-subject tag/domain decision

Tag `0x05`, crate-internal. Domain stays `tiler.compiler.request-subject.v6`. No public type added.

### Perturbations

Subject, not assertion, each watched failing once:

1. **Restore the static domain gate** in `plan_elementwise`:
   ```
   thread 'request::tests::a_parametric_broadcast_program_is_recognized_with_its_carrier' panicked at crates/tiler-compiler/src/request.rs:12702:14:
   a sourced broadcast must pass strategy selection: UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent: Symbol(...) name: "n" }
   ```
2. **Leave the generic schedule refuse** in front of physical selection:
   ```
   assertion `left == right` failed: a provider without parametric support must decline that named rule, got compile.schedule.symbolic-extent: program/0::n is a symbolic extent this capability cannot plan over
     left: None
    right: Some(("planning", "parametric-broadcast"))
   ```
3. **Write the parametric tag as `0x02`**:
   ```
   assertion `left == right` failed: the parametric carrier must take tag 0x05, not the refusal 0x00
     left: Some(2)
    right: Some(5)
   ```

### Commands

From this worktree, after the implementation:

- `cargo test -p tiler-compiler` — 808 lib tests passed, 1 ignored; integration tests passed; rustdoc compile-fail tests passed.
- `cargo test -p tiler-ir` — 987 lib tests passed; integration and compile-fail tests passed.
- `cargo clippy -p tiler-compiler -p tiler-ir --all-targets -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-compiler -p tiler-ir --no-deps` — clean.
- `tkt lint` — `ok: no problems found`.
- `git diff --check` — clean.
- `tkt guard --base main --format json tkt/admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary` — `severity: warn`, `conflict: false`, `under_declared: []`. Direct collisions with other live compiler/IR tickets; shared `project/tickets`. Not under-declared.

Did not run `make full`. Coordinator gates at integration.
