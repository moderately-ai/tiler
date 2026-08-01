---
id: prototype-inline-aot-integration-proof
title: Prove the complete inline AOT workflow
status: in-progress
priority: p1
dependencies: [prototype-macro-embedding-and-cargo-behavior, prototype-metal-runtime-proof, promote-the-metal-aot-compilation-identity, make-runtime-routing-commit-authority-one-shot, admit-multi-input-elementwise-programs-at-the-compiler-boundary, generate-cfg-gated-artifact-family-delivery]
related: []
scopes: [implementation/frontend, implementation/cache, implementation/compiler, implementation/artifact, implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, integration, inline-dx, milestone-0b]
claimed_from: todo
assignee: worker-prototype-inli
lease_expires_at: 1785550316
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
