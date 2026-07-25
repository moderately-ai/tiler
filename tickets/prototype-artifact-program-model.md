---
id: prototype-artifact-program-model
title: Implement the artifact-facing program model
status: done
priority: p0
dependencies: [prototype-kernel-program-ir]
related: [prototype-neutral-artifact-codec]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, manifest]
---
Project verified KernelProgram content into a bounded versioned artifact model:
entry points, ABI and launch expressions, portfolios/routing predicates,
target requirements, reached admission and selected-provider provenance, and
backend payload descriptors. Runtime and codecs consume this model without
optimizer internals; unused compilation-environment providers do not become
packaged artifact identity.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

`tiler-artifact` is built out from its four-line stub into one new public module, `tiler_artifact::program`, holding the bounded versioned artifact projection of a verified `tiler_ir::program::VerifiedKernelProgram`. **Awaiting Tom's review under ADR 0075** (new publicly reachable namespace); the module documents itself as a reviewed draft boundary per ADR 0074 §7 and the ticket stays `in-progress`.

### What the layer models

A `VerifiedArtifactProgram` carries: the governed component schema versions (`ArtifactSchema` over program, ABI expression, guard-and-routing, and target-requirement schemas); the ordered named interface a runtime binds against, projected in semantic-interface order with the storage element type read from the plan; a portfolio of complete plan variants in routing-priority order, each with its verified kernel program, its applicability guard, its declared `TargetProfileRef` and `FeasibilityRuleSetRef`, and its deferred feasibility predicates with phase and query authority; one executable entry per program stage, each with the neutral ABI bindings in kernel buffer-parameter order and a launch contract (thread counts, zero-work policy, launch preconditions); backend payload descriptors named only by governed keys, schema, content digest, and execution policy; and the provenance the packaged plan actually reached.

The ABI's facts are **derived, not declared**. A producer supplies only expressions; a binding's element type, address space, access mode, alignment, and program role are read from the kernel signature and the materialized value the stage access addresses, and each expression is proven against that program: an accessible-byte formula must evaluate to the byte view's exact length under the program's declared shapes, and the threads-per-workgroup formula must evaluate to the bound kernel's proven requirement. That removes a whole class of ABI/plan disagreement rather than diagnosing it.

### Consumability without optimizer internals

The crate depends on `tiler-ir` only, and the module's read views hand back shared-IR views (`StageRef`, `MaterializedValueRef`, `ByteWindow`, `ResourceRequirements`, `NumericalRealization`, `CanonicalKernelIdentity`) rather than re-modelled copies. A runtime binds inputs, evaluates a guard, reads bindings and launch geometry, resolves a backend entry, and walks stages/values/views/allocations/dependencies through `tiler_ir::program`'s own views. No compiler-owned object — region cover, fusion alternative, cost, search state, explain record — is reachable or required. `crates/tiler-compiler` is untouched by this branch.

### Reached versus unused provenance (ADR 0072)

Two independent provenance levels, both held to the line, with the unused half excluded in the strongest available way — it is never retained:

- **Semantic admission.** Identity folds `SemanticIdentity`'s graph, reached-definitions, and admission-provenance subjects and deliberately **omits the registry-snapshot subject**, which is the one that moves when an unused provider changes.
- **Selected capability providers.** `CompilationEnvironment` is a construction-time authority: `select_provider` rejects a provider the environment never offered, and only selected providers are retained and folded into identity. The environment's remainder is dropped at construction, so it cannot reach identity even in principle.

Tests proving both directions (`crates/tiler-artifact/src/program/tests.rs`):

- `a_reached_capability_provider_revision_changes_identity` — same environment, selected revision 1 vs 2, identity differs.
- `an_unused_environment_provider_does_not_change_identity` — same selection, environment gains an unselected provider and then bumps its revision; identity unchanged across all three.
- `a_reached_semantic_provider_revision_changes_identity` — asserts graph and reached-definitions equal, admission provenance different, and artifact identity different.
- `an_unused_semantic_provider_revision_does_not_change_identity` — asserts the registry snapshots genuinely differ and admission provenance is equal, then that artifact identity is equal. This is the test that fails if the whole `SemanticIdentity` bundle is folded in.

### Which ABI/launch types would move if ADR 0068/0070 win

The divergence recorded in `complete-program-identity-with-abi-guards-and-routing` is not resolved here and no dependency on its resolution was added. The expression domain was placed in one self-contained file written to move, so the resolution ticket has a concrete answer:

**Would move to `tiler_ir::program`** — all of `crates/tiler-artifact/src/program/expr.rs`: `AbiType`, `AbiValue`, `AvailabilityPhase`, `AbiRoot`, `AbiUnaryOp`, `AbiBinaryOp`, the private `ExprNode` arena representation, `AbiEvaluationError`, `AbiFacts`, the canonical `expr_key` encoder, the type/phase/interface-root predicates, and the pure lazy checked evaluator. Its only dependencies are `tiler_ir::semantic::InputKey`, `tiler_ir::shape::Axis`, and the governed `TargetPropertyKey` newtype, which would move with it. `AbiExprView` and `AbiExprRef` (in `model.rs`) are the read projection of that arena and would move alongside whatever product owns the arena.

**Would stay in `tiler-artifact`** — `facts.rs` in full (`AbiFactBinder`, `AbiBindingError`): binding live facts and enforcing that a fact could legally be queried at the phase it claims is the half ADR 0068 explicitly assigns to this crate. Also staying: everything artifact-shaped in `model.rs`, `keys.rs`, `builder.rs`, and `verify.rs` — schema versions, interface projection, payload descriptors, backend entry mappings, target-profile and feasibility references, and the artifact identity encoder.

**Consequence for the resolution ticket.** If the domain moves, `tiler-artifact` keeps the same public shape minus the re-exported expression vocabulary, and this crate's builder switches from owning an arena to referencing the program's. The one design point that must be decided there rather than mechanically: whether the arena becomes per-`KernelProgram` (which would let program identity fold guards and ABI as ADR 0072 says) or stays per-portfolio and shared across variants, which is what this layer does today because several variants legitimately reuse one formula.

**Resolved 2026-07-25, and this section's prediction was half right.** `relocate-abi-expressions-into-tiler-ir` moved the domain, leaving `expr.rs` as a re-export; `complete-program-identity-with-abi-guards-and-routing` then gave each `VerifiedKernelProgram` a per-program arena, guard, and entry ABI folded into `tiler.kernel-program.v2`. The design point above was decided **both ways rather than either**: the program arena is per-program, and this crate's portfolio arena stayed per-portfolio and shared across variants, because several variants do legitimately reuse one formula and because the artifact arena additionally carries launch preconditions and deferred predicates that no single program owns. The builder does *not* yet reference the program's arena, so the two are related only through the program facts both are checked against; `bind-the-artifact-variant-abi-to-the-program-abi` owns closing that.

### A convention gap this work found, with concrete evidence

`tiler_ir::kernel`'s `KernelType`, `AddressSpace`, and `BufferAccess` are `#[non_exhaustive]`. This crate must encode all three into identity, and **a cross-crate encoder cannot obtain the compile break ADR 0074 §3 relies on** — a wildcard arm is mandatory. Mapping an unknown variant to a sentinel tag would let two structurally different subjects share identity bytes, exactly the hazard §3 names. The encoder therefore rejects: `ArtifactDiagnostic::UnrecognizedForeignVariant { subject }` with `ForeignEnumSubject::{KernelType, AddressSpace, BufferAccess}`. It is unreachable today and has no test, and it is documented as such.

This is a direct counterexample to the same-crate assumption `resolve-non-exhaustive-recognizer-hole` asks to verify ("a same-crate assumption that later moves crates would break silently"). The assumption does not hold for an encoder in a different crate from its enums, and it will not hold for `extend-canonical-identity-encodings-for-reserved-variants` either once an encoder crosses a crate boundary. Rejecting is sound but strictly weaker than a compile error: a widened `KernelType` silently makes previously packageable artifacts unpackageable rather than failing to build.

### Bounded profile and deferrals

Implemented and rejected-by-absence rather than approximated: expression roots are literals, input extents, and governed target properties — element strides, view start elements, remainder, and narrowing widths other than 16 and 32 bits are absent. `BindingKind::Buffer` is the only transport (metadata blocks, inline scalars, error records are reserved variants). `RoutingPolicy::StablePriority` is the only policy and has no setter. All variants of one artifact must share the semantic graph, the named interface, the numerical realization, and the declared target profile, so a fat multi-profile envelope is out of profile. Accessible-range and launch expressions are restricted to interface-only roots so they are computable before any device query; guards and preconditions may name device and prepared-kernel facts subject to a static phase check. Section framing, digests, wire encoding, and compatibility policy are `prototype-neutral-artifact-codec`'s. Validation obligations, enforcement plans, and publication modes from `docs/artifact-abi.md` are not modelled and are not reserved by a type here.

### Verification

39 unit tests plus one module doctest. Coverage: verified-product construction and read-through-shared-IR consumability; identity determinism; identity independence of payload, provider, and expression declaration order; arena deduplication; the four provenance tests above; cross-program rejection (a variant realizing another semantic graph) and forged-handle rejection (expression and payload handles minted by another builder); one negative test per insertion-time rule (unavailable provider, accessible-range disagreement, launch disagreement, non-predicate guard, device property in a size expression, root phase escape, non-deferred predicate phase, unselected deferred authority, entry and binding cardinality, duplicate variant/payload/deferred predicate/launch precondition, mistyped operand and select branches); one per whole-artifact rule (empty portfolio, missing selected provider, unreachable expression, unreferenced payload, colliding backend entries) including recovery of the intact builder; and expression semantics (lazy select, phase-refusing binder, unbound root, narrowing overflow).

Gate results at the final commit: `uv run --locked python scripts/check_rust.py` passes; `uv run --locked python scripts/docs.py validate` passes; `ticketsplease lint` passes; `git diff --check` clean; `ticketsplease guard tkt/prototype-artifact-program-model` reports no scope escape. **The complete `scripts/check_repository.py` gate could not be run on this host**: it fails its first check with `ticketsplease must be 0.11.0` because the host binary auto-updated to 0.12.0 while `tool-versions.toml` pins 0.11.0. That is environmental, is unrelated to this branch, and `tool-versions.toml` is outside this ticket's scopes; it is with Tom.
