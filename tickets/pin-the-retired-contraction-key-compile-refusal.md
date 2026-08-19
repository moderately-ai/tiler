---
id: pin-the-retired-contraction-key-compile-refusal
title: Pin the retired contraction key's compile refusal
status: in-progress
priority: p3
dependencies: []
related: [replace-the-standard-contraction-key-with-the-accepted-successor]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, tests, correctness]
claimed_from: todo
assignee: worker-refusal-pin
lease_expires_at: 1787148984
---
## User-visible outcome

The no-fallback half of the ADR 0112 replacement is held by a landed regression test rather than by inference: a `tiler::strict-tensor-contraction-f32@1` program, built through the public extension machinery, refuses at `compile()` with a typed rule and never falls back to the successor.

## Why this exists — filed 2026-08-19 at integration of the replacement migration

The migration's independent review (finding 3) demonstrated the refusal by a disposable probe: `SemanticRegistryBuilder::standard()` + `register_provider` with a retired-key extension provider + `SemanticProgramBuilder::apply` builds the old-key program publicly, and `compile()` refuses it with `UnsupportedCapability { rule: "semantic-authority-pairing" }` before a target-qualified explain trace. Nothing in-tree pins that refusal, so a future recognition change could silently re-admit the retired key.

## Required work

One integration test in `crates/tiler-compiler/tests/` following the review probe: build the old-key program via public machinery (mirror the extension-provider shape of `replacing_only_the_occurrence_key_moves_the_semantic_graph_identity` in `crates/tiler-ir/src/semantic/contraction/tests.rs`, using the public `OperationInferencer`/provider traits), drive it through the ordinary `compile()` entry with a governed target, and assert the exact typed refusal rule observed at the then-current base — quote the actual rule text from a run, do not assume `semantic-authority-pairing` survives verbatim. Control: the same program shape under the successor key compiles (the existing direct-path tests already hold this; cite one rather than duplicating it). Perturb the subject once — flip only the key to the successor spelling inside the new test's builder — and show the refusal disappears, proving the test reaches the key and not some other property.

## Closes when

The test lands green with the quoted refusal, its subject perturbation is recorded, and the migration ticket's artifact-cross correction note resolves its citation.

## Delivery — 2026-08-19, base `0f0100f7b2b2a3f79ec6f1b7b1e9422851a5c149`

**Landed:** `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs`, two tests, both green.

### Fact audit at this base, before editing

- **Verified.** `SemanticRegistryBuilder::standard()` + `register_provider` + `SemanticProgramBuilder::apply` does build a retired-key program publicly. Every type the route needs is re-exported from `tiler_ir::semantic`; the one internal spelling the mirror test uses, `apply_typed_single`, is `pub(super)` (`crates/tiler-ir/src/semantic/program.rs`, `pub(super) fn apply_typed_single`), so the landed test uses the public `apply` and `output_resolved` instead.
- **Verified.** `compile()` refuses that program with `UnsupportedCapability { rule: "semantic-authority-pairing" }` and no explain trace. Observed by execution, not assumed: the assertion is `assert_eq!` against the typed `CompileFailureClass`, plus `failure.explain().is_none()`.
- **Verified.** `replacing_only_the_occurrence_key_moves_the_semantic_graph_identity` exists in `crates/tiler-ir/src/semantic/contraction/tests.rs` and has the described provider shape.
- **Verified.** Nothing in-tree pinned the refusal. At this base `strict-tensor-contraction-f32` occurs in `crates/` only in two `tiler-ir` doc comments and in that one graph-identity test's `retired_key()` helper — no compiler test named it.
- **Imprecise, and load-bearing — repaired in the landed test's documentation.** The ticket reads as though the observed rule were evidence about the *key*. It is not. The check is `capabilities.lowering.semantic_snapshot() != program.semantic_registry().snapshot_identity()` in `crates/tiler-compiler/src/request.rs` (anchor: `return unsupported("capability", "semantic-authority-pairing")`), so it fires for **any** program built over a registry the installed capabilities were not paired with; `pipeline::conformance::externally_registered_operations_require_their_own_realization_authority` already pins the same rule for an unrelated external family. A single compile-refusal test would therefore have been satisfiable without the retired key existing at all.

  The retired key cannot be registered without leaving the standard authority — that is what the retirement *is* — and the public boundary offers no way to pair capabilities with an extension semantic registry (`LoweringCapabilityRegistryBuilder::new` starts empty and the governed lowering implementations are crate-private), so no public spelling reaches recognition with a retired occurrence under a coherent authority. The test file states this explicitly rather than implying a key-specific rule, and adds a second assertion — `the_standard_semantic_authority_has_no_retired_contraction_key` — that *is* key-specific: applying the retired key against `try_standard` refuses with `RegistryError::UnregisteredOperationAuthority` naming that exact key, while the successor spelling of the identical application succeeds in the same builder. The two together are the complete no-fallback statement.
- **Imprecise.** "Perturb the subject once — flip only the key to the successor spelling inside the new test's builder — and show the refusal disappears." The flip does make the refusal disappear, but not at `compile()`: the registrar refuses the registration first, because the successor is the governed contraction key and `ContractionF32ReductionDescriptor::decode` gates it. Three perturbations were run instead of one; all are recorded verbatim in the test's own documentation.

### Perturbation evidence (subject changed, assertions untouched)

| Perturbation | Observed failure |
| --- | --- |
| `RETIRED_NAME` → `SUCCESSOR_NAME` in test 1's `apply` | `the standard registry has no retired contraction key: [ValueId { owner: GraphId(1), index: ValueIndex(2) }]` |
| `RETIRED_NAME` → `"some-other-absent-key"` in test 1's `apply` | `expected a missing-authority refusal naming the retired key, got SemanticRegistry(UnregisteredOperationAuthority { key: OpKey(TypeKey(Key { namespace: "tiler", name: "some-other-absent-key", semantic_version: 1 })) })` |
| `RETIRED_NAME` → `SUCCESSOR_NAME` in the provider and in `retired_key_program` | `an extension provider may add a non-standard key: InvalidGovernedContractionDescriptor { source: MalformedFacts { actual: Bool } }` |
| test 2's program replaced by the same `td,od->to` graph spelled against `try_standard` under the successor key | `no installed capability compiles the retired contraction key: CompilationBatch { targets: [ ... ] }` — `compile()` returns a full `tiler.prototype-target-neutral-baseline.v1` compilation |

The third row is the sharpest: the registrar accepts arbitrary fact bytes under the retired name and refuses the *identical* bytes under the successor name. One string selects between two entirely different registration paths.

### Control

Cited, not duplicated: `a_contraction_compiles_through_the_ordinary_entry_point` in `crates/tiler-compiler/tests/contraction_direct_path.rs` compiles the same `td,od->to` workload under the successor key, through the same `compile()` entry and `TargetProfile::governed()` target, under every stated numerical contract. Without it both landed assertions would also pass in a build where no contraction compiled at all.

### Gates, all green in the worktree

`cargo nextest run -p tiler-compiler` (969 passed, 1 skipped), `cargo fmt --check`, `cargo clippy -p tiler-compiler --all-targets -- -D warnings`, `cargo test -p tiler-compiler --doc` (14 passed), `git diff --check`, `tkt lint`, `tkt guard`.
