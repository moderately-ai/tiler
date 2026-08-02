---
id: prototype-inline-aot-integration-proof
title: Prove the complete inline AOT workflow
status: done
priority: p1
dependencies: [prototype-macro-embedding-and-cargo-behavior, prototype-metal-runtime-proof, promote-the-metal-aot-compilation-identity, make-runtime-routing-commit-authority-one-shot, admit-multi-input-elementwise-programs-at-the-compiler-boundary, generate-cfg-gated-artifact-family-delivery]
related: []
scopes: [implementation/frontend, implementation/cache, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, integration, inline-dx, milestone-0b]
---
## User-visible outcome

One ordinary inline Rust invocation — no build script, no registry, no scan, no prepare step, no runtime JIT — constructs a program, shares its external compilation through the validated cache, embeds the bytes, and runs with guarded selection and pre-commit fallback. This is the end-to-end proof of the accepted inline developer experience; every absence in that list is checkable by reading the consumer crate.

Demonstrate one ordinary inline Rust invocation constructing and optimizing a program, sharing external compilation through the validated cache, embedding manifest/metallib bytes directly, and emitting guarded runtime selection with fallback authority before commit. Require no build script, registry, scan, prepare command, or runtime source compilation.

## Closes when

- One inline Rust invocation in an ordinary crate constructs a program, optimizes it, and produces a running kernel, with **no** `build.rs`, no duplicated registry, no source scan, no Cargo subcommand, no prepare step, and no runtime source JIT — the accepted inline developer experience `AGENTS.md` names, each absence checkable by reading the consumer crate.
- The external compilation is shared through the validated expansion cache: a second build of the same subject is a hit, the hit is validated on every read rather than trusted, and a mismatch is a typed refusal rather than a silent rebuild.
- Manifest and metallib bytes are embedded directly in the produced binary, with the identity that names them derivable before the compilation it describes has run.
- Runtime selection is guarded, and the fallback authority is exercised **before** the routing commit and nowhere after it, per ADR 0051's one-way commit.
- `make full` passes, and the proof runs end to end on a qualified Apple toolchain.

## Dependency reality (verified 2026-07-31)

This ticket's four dependencies are not uniformly ready, and the gap is not the one the frontmatter suggests. `grep -m1 '^status:' tickets/<id>.md` for each:

- `prototype-metal-runtime-proof` — `done`.
- `make-runtime-routing-commit-authority-one-shot` — `done`.
- `prototype-macro-embedding-and-cargo-behavior` — `todo`, so dispatchable but unclaimed.
- `promote-the-metal-aot-compilation-identity` — `done`. **The consequence for this ticket changed: the cache-sharing half now has a reachable identity producer.** `CompilationIdentity` and its `as_bytes` are `pub` in `tiler-metal-aot`, obtained only from the public `PreparedCompilation::identity` after `Toolchain::prepare`, and `tiler-build` already consumes them at `crates/tiler-build/src/metal_assembly.rs:119`. The other facet's producer is `derive-the-pre-compilation-artifact-program-subject` (`done`), so both facets of the cache subject now have one.

Read that as: neither half is waiting on a decision any more. What remains is ordinary unclaimed work on the embedding half, plus the frontend that has to call both — `tiler` and `tiler-macros` exist as of 2026-07-31 but carry only the `tensor!` re-export and its anchor, so the grammar, expansion, and family delivery this proof needs are still open under `prototype-inline-proc-macro-frontend`, `define-inline-symbol-binding-and-runtime-value-adaptation`, and `promote-artifact-family-selection-for-the-frontend`.

**Superseded 2026-07-28 reading, kept so the correction is legible:** this section previously recorded `promote-the-metal-aot-compilation-identity` as `in-progress` on a single unmerged commit `4f8ce90`, and concluded that the cache-sharing half had no reachable identity producer because `CompilationIdentity` and `as_bytes` were `pub(crate)`. That was true when written and is false now; the promotion merged and the public boundary it gated was accepted.

## Outcome (worker, base `e6a47d9`, 2026-07-31)

**The proof was not built, and the reason is a measured gap between the approved region grammar and the compiler's admitted program set rather than unclaimed orchestration work.** The four frontmatter dependencies are all `done` and the halves they own are genuinely ready; what is missing is a region that the compiler will accept at all. Two further dependencies are added to the frontmatter above to record the real structure.

### Measurement — no region the approved grammar can express reaches the compiler

**Measurement, worktree at `e6a47d9`, 2026-07-31, `nightly-2026-07-19`, macOS 27.0 arm64.** A temporary integration test in `tiler-compiler` built the approved region as a `SemanticProgram` — three `f32[4]` inputs, `F32Multiply`, `F32Add`, one output — and called `session::compile_governed` under each of the four `NumericalContract` values. All four refused identically:

```
CompileFailure { class: UnsupportedCapability { rule: "signature" },
                 explain: "absent (refused before a target-qualified trace)" }
```

The same test built `(a * 2.0f32) * 3.0f32` — one input, two constants, four operations — and `compile_governed` returned `Ok`, so the probe distinguishes a refusing boundary from a boundary that refuses everything. This independently reproduces at `e6a47d9` the measurement `admit-multi-input-elementwise-programs-at-the-compiler-boundary` records at base `b623670`, and confirms it still holds. The probe file was removed after the run; reproduce it by building those two programs against `tiler_compiler::session::compile_governed`.

**Fact — the grammar cannot express the shape the compiler does accept.** `crates/tiler-compiler/src/request.rs:1940-1957` selects between exactly two normalizations. `normalize_pointwise` (`:1959-2079`) requires `input_count() == 1`, `output_count() == 1`, `operation_count() == 4`, and exactly one of its three leaves being the program input while the other two are constant operations. `normalize_serial_sum` (`:2092`) requires `input_count() == 1` and a `strict_serial_sum_f32_op` reduction. `crates/tiler-macros/src/grammar.rs:113-128` defines the whole region expression vocabulary as `Expression::Operand` and `Expression::Binary` over `Operator::{Multiply, Add}` — **there is no scalar-literal production and no reduction production**, and every `in` operand becomes a program input (`crates/tiler-macros/src/region.rs:578-582`). So a region has N tensor inputs and zero constants, while both recognized shapes need exactly one tensor input plus constants. The intersection is empty: no region the approved grammar admits compiles, for any N.

**Inference.** The first closing condition — "one inline Rust invocation … constructs a program, optimizes it, and **produces a running kernel**" — is unreachable through `tiler::tensor!` at this commit, and no delivery-policy, cache, or embedding work changes that, because the refusal happens at strategy selection before any target is qualified.

### What *is* ready, verified by reading

- **Fact — the AOT + cache half is complete and already exercised.** `tiler_build::accept_or_publish_metal_plan` (`crates/tiler-build/src/metal_plan.rs:192-257`) composes the two-facet cache subject from the driver's `CompilationIdentity` bytes and the *pending* artifact's canonical identity (`crates/tiler-build/src/metal_cache.rs:190-195`) — so the identity that names the bytes is derivable before the compilation it describes has run, which is this ticket's third closing condition, already satisfied. `prototypes/serial-sum-compile/src/main.rs:423-431` drives it for six proof members, and `a_checked_plan_publishes_then_hits_without_recompiling` (`metal_plan.rs:534-590`) already proves publish-then-hit across three calls with two toolchain invocations.
- **Fact — every cache hit is validated by the production validator.** `ExpansionCache::get_or_publish` (`crates/tiler-cache/src/expansion/store.rs:304-334`) routes every read through `read_entry` (`:555-616`), which re-derives the key from the carried subject, checks every section digest, and then runs the pinned `artifact_validator` (`store.rs:1075-1077`, i.e. `decode_artifact`). The validator parameter is `pub(crate)`, so no public caller can weaken it.
- **Fact — the runtime half is bytes-only, so embedding is already expressible.** `grep -rn "std::path\|&Path\|PathBuf" crates/tiler-runtime/src crates/tiler-artifact/src` returns zero matches; `DecodedProgram::decode(&[u8])` (`crates/tiler-runtime/src/load.rs:151`) is the entry, and `serial-sum-run`'s own `#[cfg(test)]` fixtures already run the full sequence over in-memory `Vec<u8>`.
- **Fact — the one-way commit is structural.** `Preflight::commit(self) -> RoutedDispatch<'a>` (`crates/tiler-runtime/src/load/route.rs:739-753`) consumes `self`, is infallible, and `Preflight` is not `Clone`; `LiveDeviceQualification` and `RoutePreparation` have no `commit` at all, pinned by `compile_fail` doc-tests at `route.rs:379-383` and `:496-500`. Fallback is expressed by dropping a pre-commit stage.
- **Fact — the byte-literal embedding pattern exists and reuses the real cache.** `spikes/embedding/self-contained/embed-macro/src/lib.rs:175-229` resolves through `ExpansionCache::get_or_publish` and emits one `Literal::byte_string`; `docs/research/embedding/self-contained-embedding.md` records that a consumer built with the whole cache root and all twelve envelopes deleted still links and runs.

### The refusal classes a mismatch actually earns, for whoever builds this next

**Fact.** A perturbed cache entry is a *miss with a reason*, not an error: `MissReason::Rejected(EntryRejection::Bundle(...))` with variants including `SectionDigest`, `TotalLength`, `Truncated`, `Magic`, `KeyMismatch`, and `KeyNotDerivedFromSubject` (`crates/tiler-cache/src/expansion/bundle.rs:465-582`), after which the bytes are quarantined and the entry rebuilt. The **typed refusal** this ticket's second closing condition asks for lives one layer up: a semantically wrong but structurally valid entry produces `MetalPlanBuildError::CacheProtocol(...)` from `validate_decoded_payload` (`crates/tiler-build/src/metal_cache.rs:257-284`), which is deliberately *not* converted into a miss. A proof demonstrating "mismatch is a typed refusal rather than a silent rebuild" must perturb semantically, not by flipping a byte — flipping a byte demonstrates the quarantine-and-rebuild path instead, which is a different (also correct) behaviour.

### Host eligibility, unchanged

**Fact.** ADR 0086 still refuses on every macOS row: `MetalHostEligibility` holds an uninhabited `NativeTranslationAuthority` (`crates/tiler-metal/src/applicability.rs:441-497`) and `native_translation_authority()` returns `None` unconditionally (`:879-881`). Any proof this ticket produces runs under the same labelled diagnostic envelope path as `serial-sum-run` — producer-declared equality via `declared_route_environment` (`prototypes/serial-sum-run/src/proof.rs:645-661`), **not** host-earned eligibility — and must print that label as `serial-sum-run` does at `proof.rs:2425-2429`.

### Dispatch findings the coordinator needs

1. **The declared scopes cannot host the proof.** A proc macro must be its own crate, and the consumer that invokes it must be a different crate. Admitting either means editing the workspace `members` list (`implementation/workspace`) and `Cargo.lock` (`implementation/cargo-lock`); adding a `tiler-cache`/`tiler-build`/`tiler-compiler` edge to the existing `tiler-macros` also rewrites `Cargo.lock` (it currently records only `tiler-ir` and `tiler-metal-aot` at `Cargo.lock:419-424`). Neither scope is on this ticket. This is the identical blocker `generate-cfg-gated-artifact-family-delivery` recorded for its own parent. A new `prototypes/<name>/**` directory additionally matches no scope glob in `ticketsplease.toml`, so it would fail `tkt guard` as a scope escape.
2. **`generate-cfg-gated-artifact-family-delivery` was `blocked` on stale facts** and is corrected to `todo` in this change; see its own dated note.
3. **`admit-multi-input-elementwise-programs-at-the-compiler-boundary` is the true critical path** and is now a declared dependency here.

### Deliberately not done

- The sibling tickets above were **not** absorbed. Widening the compiler's recognizers is that ticket's whole subject, and adding scalar-literal or reduction syntax to the region grammar is a change to `tensor!`'s observable public surface — ADR 0075 makes that Tom's, not a worker's.
- No `tensor!` behaviour changed, so **there is no boundary packet**: the delivery policy is still `FallbackOnly`, no cache is opened at expansion, and no public item was added or removed.

## Outcome — second attempt (worker, base `aa961da`, 2026-07-31)

**The proof is built and four of the five closing conditions are supported. The fifth — "produces a running kernel" — is not, and what stops it is a missing accepted public boundary rather than unwritten code.** A new dependency, `route-an-embedded-artifact-through-a-consumer-storage-seam`, carries the remainder.

An ordinary consumer crate declaring only `tiler`, containing one `tiler::tensor!` invocation with `deliver macos;`, now runs the whole expansion-time AOT flow inside `rustc`: the region is parsed, verified as a public logical program, optimized through `tiler_compiler::session`, emitted as Metal, given a complete artifact identity, looked up in the expansion cache, compiled by `xcrun metal` and `xcrun metallib` on a miss, published atomically, read back, and embedded as one byte-string literal — after which the produced binary decodes and routes those bytes and takes its fallback before any commit.

### Condition 1 — one inline invocation, and every named absence

**Supported except for the dispatch.** `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` is the fixture; `trybuild` compiles and *runs* it as a separate out-of-tree crate whose manifest names `tiler` alone.

**Measurement — macOS 27.0 arm64, Apple M4 Max, `nightly-2026-07-19`, 2026-07-31.** The same source built as a standalone crate outside the workspace, with `TILER_EXPANSION_CACHE_DIR` pointed at a private root under `$TMPDIR`:

- the crate is two files, `Cargo.toml` and `src/main.rs`; `find . -name build.rs` counts **0**;
- its whole dependency table is one line, `tiler = { path = … }`;
- the source (doc comments stripped) contains no `include_bytes!`, `include_str!`, `env!`, `option_env!`, `std::fs`, `tiler_macros`, `Command`, or any absolute path;
- the produced 3,364,808-byte binary contains the `MTLB` metallib magic exactly **once** — the embedded payload;
- with the entire cache root deleted (**2** files before removal, **0** after, directory gone — the removal is refused outright if the path held no files first), the already-built binary runs and exits 0;
- the binary contains no occurrence of the cache root path, `ai.moderately.tiler`, `.bundle`, or `TILER_EXPANSION_CACHE_DIR`.

**What is not supported: "produces a running kernel".** `crates/tiler/src/value.rs` publishes no storage access and no device object by accepted design, and `crates/tiler/tests/dependency_direction.rs` forbids any workspace package from depending on `tiler`, so no consumer of the facade can hand a kernel its operands. `tiler_runtime::load::Preflight::commit` is consequently unreachable from a consumer, and `crates/tiler/src/route.rs` contains no call to it. Adding the seam is a public boundary under ADR 0075 and is Tom's; the new dependency ticket carries it.

### Condition 2 — the validated cache, and both refusal classes

**Supported.**

**Measurement — same host and date, out-of-tree consumer, private cache root, `xcrun` shim first on `PATH` logging every invocation and returning logging wrappers for `metal` and `metallib`.** Source touched between passes so the expansion re-ran each time:

| pass | `xcrun` calls | `metal`/`metallib` runs | cache |
| --- | ---: | ---: | --- |
| cold root | 6 | 2 (`metal`, then `metallib`) | one 49,432-byte bundle published |
| warm root | 6 | **0** | validated hit |

Validation on every hit is the cache's own: `ExpansionCache::lookup` "has no fast path", re-derives the key from the carried subject, checks every section digest, and runs the pinned artifact validator. `crates/tiler-macros/src/aot/tests.rs` carries both perturbations, each watched failing before it passed:

- `a_semantically_wrong_entry_is_a_typed_refusal_rather_than_a_silent_rebuild` publishes the *other* region's envelope under the approved region's subject through the cache's own API — internally consistent in every way the cache can check — and the expansion refuses with `MetalPlanBuildError::CacheProtocol`;
- `a_damaged_entry_is_quarantined_and_rebuilt` flips one interior byte of the published bundle and the next expansion is a miss with a reason, quarantined and republished with the identical bytes.

**Measurement boundary.** A warm hit still performs six `xcrun` calls, because `Toolchain::prepare` reads the compiler fingerprint that `CompilationIdentity` folds into the key that decides hit or miss. That is narrower than the contract's "warm IDE and `cargo check` expansion must avoid `xcrun`", and `avoid-toolchain-resolution-on-a-warm-expansion-cache-hit` carries the tension rather than resolving it here.

**Corrected 2026-08-01 by `avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`, on both counts.** The six-call figure over-attributes by one: five belong to `Toolchain::resolve` and the sixth is rustc's own `--show-sdk-path` at link time, which is why an identical expansion logs five under `cargo check` and six under `cargo build`. The transcript also missed two subprocesses entirely — `resolve` executes the located `metal` and `metallib` binaries directly to read their versions, so an `xcrun` shim cannot observe them, and those two are the most expensive of the seven. The tension is resolved rather than open: the contract sentence was corrected rather than implemented, and `docs/integration/frontends.md` now carries the identity derivation and the measured cost.

### Condition 3 — identity before the compilation it describes

**Supported, and it was already true.** `accept_or_publish_metal_plan` composes the cache subject from the *pending* artifact's canonical identity and the prepared compilation's identity, both of which exist before `metal` runs. `the_second_expansion_of_one_subject_compiles_nothing` is the observable form: a design computing identity after compiling could not hit at all, and the transcript above shows the second pass compiling nothing.

### Condition 4 — guarded selection with fallback before the commit

**Partially supported, and the unsupported half is the same seam.** `crates/tiler/src/route.rs` decodes the embedded envelope, restates the producer's declared environment, matches the artifact against the identity the expansion recorded, selects the variant, reaches the first question only a device can answer, and drops the pre-commit stage — the fallback ADR 0051 permits. `grep -rn "\.commit()" crates/tiler/src` returns nothing, so "nowhere after the commit" holds by construction rather than by discipline, and `RouteOutcome` has no committed variant.

**This is producer-declared equality, not host-earned eligibility.** The environment the loader is handed is the profile `tiler-build` declared, so `ExecutionEnvironment::classify` answers whether the bytes name the profile they were built under and does *not* answer whether this machine is a host that profile applies to. ADR 0086 refuses on every macOS row and nothing in the facade can ask it: the facade holds no device. The module says so in those words, as `prototypes/serial-sum-run` does.

What is missing is the other side: a route that *can* commit, because there is something to dispatch. See condition 1.

### Condition 5 — `make full`

**Supported.** Green on the final commit: fmt, `cargo check --workspace --all-targets --locked`, clippy with warnings denied (prototypes excluded, as the target has always excluded them), `cargo nextest run --workspace --locked` — **1,750 tests, 1,750 passed, 4 skipped** — workspace doc-tests, rustdoc with warnings denied, the release-profile numerical tests (610 passed), `ticketsplease lint`, and `shellcheck`.

### Boundary packet — every observable change, none self-accepted

1. **`deliver macos;` compiles a payload instead of erroring.** The consumer-visible behaviour the accepted syntax's own documentation said was pending. Expected acceptance; stated anyway.
2. **The Metal language standard a stated policy selects moves from MSL 3.1 to MSL 4.0**, and with it every profile's governed floor: `deliver macos;` now means macOS 26.0 rather than 14.0, and `deliver ios;` 26.0 rather than 17.0. This is a *correction to match the accepted spelling's own definition* — a profile fixes each family to "that family's governed floor for the Metal language standard Tiler compiles with", and the one authoritative compile-time declaration compiles at `-std=metal4.0` for `air64-apple-macos26.0`. Leaving it at 3.1 would have delivered a consumer a payload requiring macOS 26.0 under a policy promising 14.0, gated by a `#[cfg]` that cannot see a deployment minimum. Consequence: `deliver macos 14.0;` is now the driver's own floor refusal at the version token.
3. **A `deliver` statement selecting anything but the one buildable target is refused with a new diagnostic** naming that target: `deliver ios;`, `deliver macos-and-ios;`, and any minimum or standard other than macOS 26.0 / MSL 4.0. `deliver-several-artifact-families-from-one-expansion` carries the widening.
4. **A region with a symbolic extent cannot state a selected family**, because there is no program to compile ahead of time. The refusal names `carry-symbolic-extents-into-the-semantic-program`.
5. ~~**`TILER_EXPANSION_CACHE_DIR=off` refuses for a delivering region**~~ — **retired 2026-08-01 by `expand-a-delivering-region-with-the-cache-disabled`.** The narrowing of ADR 0089 is gone rather than accepted: `ExpansionCache::disabled()` is the store-nothing mode this note said did not exist, `AotRefusal::CacheDisabled` and its diagnostic were removed in the same change, and a delivering region under `off` now compiles, embeds, and stores nothing — ADR 0089's accepted meaning, restored. No observable change survives at this number; the constructor and the two new cache report variants are that ticket's boundary packet, not this one's.
6. **The expansion states `NumericalContract::FlushSubnormalsToZeroF32`.** Derived rather than chosen: the bound declaration's measured `f32` row flushes subnormals, so `StrictF32` and `ReassociateF32` are refused by the target's own numerical contract check, and `RelaxedF32` permits contraction, which `fusion_legality` declines for a multiply adjacent to an add. `only_one_numerical_contract_is_admissible_for_the_bound_declaration` fails the day a second becomes admissible, which is when it becomes a real question for Tom. **Superseded 2026-08-02, exactly as designed:** a second became admissible, that test failed and was replaced by `the_bound_declaration_admits_the_two_flushing_contracts` (`crates/tiler-macros/src/aot/tests.rs:348`), which pins the admitted pair as `FLUSH_SUBNORMALS_TO_ZERO_F32` and `FLUSH_AND_REASSOCIATE_F32`. The old test name resolves to nothing in the tree — search for the successor instead. The region grammar now carries a mandatory `contract` statement, so the expansion no longer derives one.
7. **A toolchain failure is now a family-scoped retained diagnostic** rather than an unconditional error, per the contract's own split, so a non-Apple consumer of a `deliver macos;` region still builds. Target-neutral failures stay unconditional.
8. **New `tiler::__private` items:** `RouteFacts`, `RouteOutcome`, `bind_route_and_build`, `select_embedded_route`. `#[doc(hidden)]`, named only by generated tokens, carrying no compatibility claim — but they are a public boundary in the ADR 0075 sense and are a reviewed draft, not accepted.
9. **`crates/tiler` gains `tiler-runtime` and `tiler-artifact`**, so every consumer's build graph gains them plus `sha2`. `tiler-runtime`'s closure is `[tiler-artifact]` by ADR 0081 item 2 and it touches no device; `tiler-metal-aot` stays off the facade and `dependency_direction` still proves it.
10. **`crates/tiler-macros` gains `tiler-build`, `tiler-cache`, and `tiler-compiler`** — host-built, never in a consumer's target graph.
11. **`Cargo.lock` moves** for both edges. No new workspace member; `workspace_population.rs` is unchanged.

### Unsupported cases, stated

Symbolic extents with a selected family; several families in one selection; any family, minimum, or language standard other than macOS 26.0 / MSL 4.0; and dispatch. Each refuses explicitly with a named remedy or a named ticket. (`off` with a selected family was listed here and is no longer unsupported — see the retirement at boundary item 5.)

### Deliberately not done

- **No dispatch and no storage seam.** Public boundary, Tom's, and now a declared dependency.
- **`docs/integration/frontends.md` still says a statement selecting a family is refused, and that the cache-root resolver is uncalled.** Both are now false. The document is scope `contracts/integrations`, which this ticket does not hold; correcting it needs that scope. Flagged rather than edited.
- **The literal-extent binding gap** found while writing the facade's tests is filed as `check-a-literal-operand-extent-against-the-supplied-value` rather than fixed here: the refusal is consumer-visible and belongs with its own boundary review.
- **`admit-multi-input-elementwise-programs-at-the-compiler-boundary` was not closed.** Its stated outcome is supported at this base — `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` shows the approved three-input region compiling — but workers do not close tickets, and `tkt claim` refused this ticket for exactly that unfinished dependency, so no claim is recorded.

**Provisional boundary acceptance (2026-07-31, overnight mode).** The coordinator provisionally accepted the eleven observable changes under Tom's stated bar, headlined by the macOS 26.0 governed floor (the authoritative declaration compiles at `-std=metal4.0`/`macos26.0`; a 14.0 promise over a 26.0-requiring payload would be a lie behind a `#[cfg]` that cannot check minimums) and the four `__private` route items with the `tiler` → `tiler-runtime`/`tiler-artifact` outward edges. Recorded for Tom's morning review with one-revert isolation.
