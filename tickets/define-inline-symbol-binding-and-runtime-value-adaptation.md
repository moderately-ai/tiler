---
id: define-inline-symbol-binding-and-runtime-value-adaptation
title: Define inline symbol binding and runtime value adaptation
status: in-progress
priority: p1
dependencies: [promote-the-symbolic-index-profile-to-a-public-boundary, admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [prototype-inline-proc-macro-frontend, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/frontend, implementation/ir, implementation/runtime, contracts/integrations]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: []
claimed_from: todo
assignee: loop-define-inlin
lease_expires_at: 1785532186
---
## User-visible outcome

The approved `sym n; in a: f32[n], ...; out ...` region binds every symbolic extent from actual operand metadata through one checked ShapeEnv environment, validates repeated uses consistently, and returns a consumer-neutral result through an explicit adapter rather than assuming a concrete tensor library.

## Implementation keys

`sym n;` declares one logical extent variable. Its runtime value is unified from every operand dimension that names `n`; at least one occurrence must source it, and every additional occurrence must equal the first checked value. The macro does not inspect values outside its invocation, infer dtype/shape at expansion, or choose one operand occurrence as a semantic identity authority. Generated binding facts name exact input keys and axes through the promoted ShapeEnv vocabulary, so declaration order does not change graph identity.

**Ratified by Tom on 2026-07-30.** Operand unification is the default meaning of the approved `sym n;` form. Expansion selects a canonical input-dimension source independent of declaration order, emits equality obligations for every other occurrence, and reports unbound or inconsistent symbols with typed span-local errors. Future explicit source syntax remains reserved for interface parameters or target properties; it does not replace the ergonomic operand-derived form.

The frontend lowers operations through the public logical operation registry. It emits one adapter-neutral invocation over traits owned by the `tiler` facade: read-only dtype/shape/storage metadata, checked runtime input binding, and construction of the result value.

**Ratified by Tom on 2026-07-30.** The public runtime-value boundary is a facade-owned opaque wrapper parameterized by a consumer-supplied adapter. The facade contract describes only the capabilities Tiler needs and exposes no Candle, Metal, or other consumer-specific type, lifetime, storage layout, allocation policy, or device object. An integration owns its adapter and the conversion into and out of the wrapper; the wrapper may carry the adapter's value and context without making either part of graph semantics or artifact identity. Raw foreign values plus an adapter argument at every invocation remain unnecessary surface area, and a global adapter registry is forbidden. The bounded proof uses an independent test adapter. Candle is neither an implementation target nor a design authority for this ticket.

## Required evidence

Compile-pass fixtures bind one symbol from one and multiple operands and return the declared output. Typed span errors cover unbound symbols, inconsistent repeated extents, rank/dtype mismatch, unsupported adapter capability, and multiple outputs beyond the bounded profile. Generated tokens contain no source scan, runtime JIT, external file reference, or dependency on a consumer's undeclared internal crate. Each negative check is perturbed once and observed failing.

## Closes when

The exact ShapeEnv-to-runtime binding and minimal opaque wrapper and adapter traits are compile-checked, the public facade boundary is reviewed by Tom, the proof demonstrates that an arbitrary external consumer can supply the adapter without a facade change or global registration, and `prototype-inline-proc-macro-frontend` can consume the boundary without inventing what `sym n` or `let d` means.

## Implementation outcome (2026-07-31)

**Scope correction, made rather than absorbed silently.** The ticket declared no `implementation/cargo-lock`, and that under-declares the work it specifies. Composing with the promoted `ShapeEnv` profile ("compose, never duplicate") requires a `tiler-macros` → `tiler-ir` edge, and re-exporting the one storage-scalar authority instead of minting a second requires a `tiler` → `tiler-ir` edge; both change `Cargo.lock`. The shared scope is declared above rather than escaped. No other scope moved: the branch touches `crates/tiler/**`, `crates/tiler-macros/**`, `docs/integration/**`, `tickets/**`, and `Cargo.lock`, and nothing under `crates/tiler-ir/**` or `crates/tiler-runtime/**` despite both being declared.

**Where each half landed, and why.**

- **The unification is expansion-time and lives in `tiler-macros`** (`crates/tiler-macros/src/binding.rs`, crate-private under ADR 0074 convention 7, following the `cache_root`/`delivery` precedent). It builds a real `ShapeEnv` and restates none of it: a symbol is a `ShapeSymbol` in a fixed region scope, its value is a `RootBinding` over `BindingSource::InputDimension { input, axis }` at `LiveDevicePreflight`/`RuntimeValidated`. That phase is forced rather than chosen — `InputDimension` floors at `LiveDevicePreflight` and `EXTENT_PHASE_CEILING` is the same phase.
- **An additional occurrence is an obligation, not a binding.** ADR 0008 gives each symbol one root binding and `ShapeEnv` rejects a second, so "`b` axis 1 is also `n`" is unrepresentable as a binding. It is carried beside the environment as a runtime equality. This is the reason a companion type exists at all rather than being duplication.
- **The canonical source is the least occurrence by interface key then axis**, not the first written. Reordering the `in` list therefore leaves `ShapeEnvIdentity` unchanged, which `declaration_order_does_not_change_the_environment` asserts.
- **The runtime boundary is facade-owned** (`crates/tiler/src/value.rs`, `pub`, reviewed draft) with the emitted facts and the checks that read them in `crates/tiler/src/expansion.rs`, re-exported through `__private`. The adapter is a type-level marker with associated functions, carried in `Tensor<A>`'s type parameter: no global registry, no adapter argument per invocation, no consumer type in the contract. A result is `A::Value`, so the integration gets its own tensor back.

**Eliminations, stated so they can be refuted.** A facade-local element-type enum was eliminated on correctness, not taste: `tiler-macros` cannot depend back on `tiler`, so the correspondence between what an expansion decides and what the facade means would be held by the emitted token text alone; sharing `tiler_ir::program::StorageScalar` makes the emitter an exhaustive match over the real vocabulary, so widening it is a build error. Placing the boundary in `tiler-runtime` was eliminated because the ratified decision says facade-owned and because the dependency closure is identical either way. Storage *access* was eliminated from this ticket because nothing dispatches yet; the dense row-major property is reserved as an adapter capability instead.

**Not attempted.** No grammar, no token parsing, no semantic translation, no dispatch, no artifact embedding. `RegionDeclarations` is populated only by tests and stands ready for `prototype-inline-proc-macro-frontend` to fill from real tokens.

**Public boundary packet — nothing here is self-accepted.** Before acceptance Tom reviews: the new `pub mod tiler::value` namespace and the `tiler` → `tiler-ir` edge it requires; `TensorAdapter` and its three associated types and three associated functions; `Tensor<A>` and its five accessors; `AdapterCapability` and its two variants and its deliberate exhaustiveness under ADR 0074 clause 5c; `ValueMetadata` and `ResultRequest`; `OperandAxis`; `BindError` and its seven variants and their exact rendered texts; the `StorageScalar` re-export; and the `__private` items generated tokens name — `RegionFacts`, `OperandFacts`, `SymbolFacts`, `AxisRef`, `ResultFacts`, `ResultAxis`, `BoundExtents`, `bind_region`, `build_result`. The representative call sites are `crates/tiler/tests/runtime_value_adapter.rs` and the two compile-pass fixtures under `crates/tiler/tests/facade/pass/`, each of which supplies its own adapter using nothing but the public surface.

**Evidence.** Sixteen perturbations, each applied alone and each observed failing the test that guards it: the six runtime refusals, the malformed-facts refusal, the six expansion-time refusals, the canonical-order rule, the emitted-token golden, and the facade-only path scan. Targeted `cargo nextest run -p tiler -p tiler-macros -p tiler-ir`, per-package Clippy with warnings denied, and `make full` all pass.

**Unsupported cases, rejecting explicitly rather than approximating.** More than one result; a region with no operands; operands on different adapter contexts (documented, not checked — `build_result` takes the caller's context, and generated code passes the first operand's); per-value storage properties such as a strided view, which an adapter declines wholesale through `DenseRowMajorStorage` rather than per value; and storage access of any kind.

## Graph maintenance

- Follow facade admission explicitly because the accepted wrapper and adapter traits are facade-owned; shared frontend scope is not a substitute for that dependency.
- Keep Candle and every other concrete consumer adapter outside this ticket and relate a later integration only after the neutral test adapter proves the boundary.
- Release `prototype-inline-proc-macro-frontend` only after the exact public value and symbol-binding draft is reviewed and accepted.
