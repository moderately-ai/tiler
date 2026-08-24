---
schema: "tiler-doc/v1"
id: "tiler.spike.verification.conformance-claim-universe-by-owner"
kind: "experiment"
title: "The closed-world conformance claim universe by owner"
topics: ["verification", "conformance", "identity", "registries"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis"]
supports: ["tiler.research.verification.conformance-claim-universe-by-owner"]
entrypoints: ["spikes/verification/conformance-claim-universe-by-owner/README.md", "spikes/verification/conformance-claim-universe-by-owner/inventory.tsv"]
last_verified: "2026-08-24"
ticket: "inventory-the-closed-world-conformance-claim-universe-by-owner"
---

# The closed-world conformance claim universe by owner

This is the exact-base reading record behind [the owner inventory](../../../docs/research/verification/conformance-claim-universe-by-owner.md). [`inventory.tsv`](inventory.tsv) is a review matrix, not an authoritative manifest. Its raw SHA-256 is only file-integrity provenance: there is no canonical universe projection or stable system-universe identity at this base.

## Base and audit boundary

Every row was re-read at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a` through its owner, construction, consumption, refusal, identity, and relevant tests. Important corrections from the rejected first draft are retained rather than hidden:

- the cache owner is `tiler-cache` under [ADR 0082](../../../docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md), not `tiler-build`;
- `TargetProfile` and both target encodings are owned by `tiler-compiler`; producer declaration `tiler.target-profile.declaration.v11` and checked feasibility descriptor `tiler.target-profile.descriptor.v11` are distinct identities;
- the independently serialized artifact stage domain is `tiler.artifact-program.stage.v4`, while the kernel-program domain is `tiler.kernel-program.v13`;
- semantic type definitions are keyed by `ValueTypeDefinitionKey`; the catalog owns 34 standard definitions, while quantization separately owns strict-affine;
- four production rewrite identities are source-known, not three, but no compiler-wide rewrite owner exists; and
- six governed physical strategies use constants ending in `_STRATEGY`; extensible providers keep the system-wide population unknown.

## Executed evidence-envelope commands

These commands were executed exactly as printed. Each output is a count of source attribute occurrences, except the last two, which count matched early `return;` occurrences. They are evidence envelopes, never feature denominators.

```sh
rg -o '^\s*#\[test\]' crates/tiler-ir/src/semantic -g '*.rs' | wc -l
rg -o '^\s*#\[test\]' crates/tiler-reference/src -g '*.rs' | wc -l
rg -o '^\s*#\[test\]' crates/tiler-reference/tests -g '*.rs' | wc -l
rg -o '#\[test\]' crates/tiler-compiler/src/pipeline/tests -g '*.rs' | wc -l
rg -o '#\[(tokio::)?test\]' crates/tiler-compiler/tests -g '*.rs' | wc -l
rg -o '#\[test\]' crates/tiler-ir/src/schedule -g '*.rs' | wc -l
rg -o '#\[test\]' crates/tiler-ir/src/kernel -g '*.rs' | wc -l
rg -o '#\[(tokio::)?test\]' crates/tiler-metal/src crates/tiler-metal-aot/src crates/tiler-artifact/src crates/tiler-runtime crates/tiler-build/src crates/tiler-conformance/src prototypes/serial-sum-compile/src prototypes/serial-sum-run/src prototypes/candle-metal-adapter/src -g '*.rs' | wc -l
rg -n -U 'let (Some|Ok)\([^\n]*\) = (resolved_toolchain|resolved_system_toolchain)\([^\n]*\) else \{\n\s*return;' crates/tiler-metal/src/golden_compilation.rs | rg -c 'return;'
rg -n -U 'let (Some|Ok)\([^\n]*\) = (resolved_toolchain|resolved_system_toolchain)\([^\n]*\) else \{\n\s*return;' crates/tiler-metal-aot/src/driver.rs | rg -c 'return;'
```

Recorded outputs in order: `484`, `182`, `152`, `115`, `94`, `265`, `137`, `954`, `10`, `5`.

The vocabulary is intentionally narrow and complete only for the stated syntax: literal test attributes in the named buckets and the exact multiline early-return form. It does not see doc tests, generated cases, or alternate spellings.

Two bounded source searches were also executed:

```sh
rg -o 'const [A-Z0-9_]+_STRATEGY\b' crates/tiler-compiler/src/physical.rs | wc -l
rg -o 'RewriteRuleIdentity::new\("tiler.pipeline"|pub\(crate\) const (COMMON_SUBEXPRESSION_RULE|ORDERED_REASSOCIATE_ADD_RULE|ORDERED_REASSOCIATE_MULTIPLY_RULE)' crates/tiler-compiler/src/pipeline.rs crates/tiler-compiler/src/rewrite.rs | wc -l
```

They returned six strategy-constant occurrences and four production identity declarations. Neither count is authoritative beyond the named files and spellings; the matrix therefore labels both system populations `unknown`.

## Executed subject perturbations

Each experiment inserted a real `AuditProbe` variant immediately after the named enum declaration with `apply_patch`, ran the command shown, captured the diagnostic below, and then removed the variant with a reverse `apply_patch`. The final `git status --short` showed no source changes. These are independent mutations; no single red build is reused for another family.

| exact census | command actually run | observed rejection |
| --- | --- | --- |
| `LoweringFamily` (1) | `cargo check -p tiler-compiler` | `E0004: non-exhaustive patterns: LoweringFamily::AuditProbe not covered` at `key_token` and consumers. |
| `CapabilityAxis` (7) | `cargo test -p tiler-compiler --lib capability_axis_descriptor_tags_ascend_with_the_derived_order --no-run` | `E0080: CANONICAL_AXES has stopped naming every CapabilityAxis`, plus `E0004`. |
| `BudgetResource` (15) | `cargo test -p tiler-compiler --lib budget_resource_keys_are_distinct --no-run` | `E0308: expected an array with a size of 16, found one with a size of 15`, plus `E0004`. |
| `ExplainDisposition` (16) | `cargo test -p tiler-compiler --lib every_disposition_variant_is_reached_by_a_legal_event --no-run` | `E0004: ExplainDisposition::AuditProbe not covered` in independent exhaustive mappings. |
| `AvailabilityPhase` (5) | `cargo check -p tiler-ir` | `E0004: AvailabilityPhase::AuditProbe not covered` in ABI encoders and consumers. |
| `NumericalDimension` (11) | `cargo check -p tiler-ir` | `E0004: NumericalDimension::AuditProbe not covered` in key/tag/behaviour mappings. |
| AOT `CompileStage` (2) | `cargo check -p tiler-metal-aot` | `E0308: expected an array with a size of 3, found one with a size of 2`, plus `E0004`. |
| `CallFailureStage` (7) | `cargo test -p tiler-compiler --lib failure_stage --no-run` | `E0004: CallFailureStage::AuditProbe not covered` in key/fallback/ordinary mappings. |
| `AdapterRouteFailure` (7) | `cargo check -p tiler-runtime` | `E0004: &AdapterRouteFailure::AuditProbe not covered` in fallback, display, and source mappings. |
| cache publication `Phase` (9) | `cargo test -p tiler-cache --lib every_phase_name_round_trips --no-run` | `E0308: expected an array with a size of 10, found one with a size of 9`, plus `E0004`. |

One negative control stayed green and therefore caused a downgrade: adding `AuditProbe` to `BudgetRefusal` and running `cargo check -p tiler-compiler` completed successfully with only `warning: missing documentation for a variant`. The four-row source snapshot is not a complete census until an owner manifest or exhaustive consumer rejects that mutation.

All registry, structural-record, and contract-derived counts that were not independently perturbed are likewise labelled **bounded snapshot**, not complete. This includes semantic operations/types/algebra, index laws, reference capabilities/validators, lowering capabilities, deterministic budget fields, feasibility outcomes, and the cache contract's five prose clauses. The linked owner-manifest descendants are the missing checks.

## Matrix integrity, not universe identity

After editing, run:

```sh
wc -l spikes/verification/conformance-claim-universe-by-owner/inventory.tsv
cut -f1 spikes/verification/conformance-claim-universe-by-owner/inventory.tsv | tail -n +2 | LC_ALL=C sort | uniq -d
shasum -a 256 spikes/verification/conformance-claim-universe-by-owner/inventory.tsv
```

Executed results: `37` lines including the header; no duplicate family-id output; raw-file SHA-256 `e443e5803b654275ab930f6ae10c1009284011dcc26740a325e2762994cf6e5e`. The digest is a raw-file checksum only. A canonical projection cannot be supplied without deciding which owner fields are normative, how unknowns are encoded, and what revision rule applies. [The authority/change-policy ticket](../../../tickets/decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md) owns that decision.

## Unsupported cases

The record cannot support a complete system-universe verdict, a public owner API, a normative schema/digest, a goal profile, or current performance outside a retained measurement's bounds. Unknown rows are positive blockers, not empty sets. In particular, complete rewrite, physical-provider/strategy, feasibility-obligation, schedule/KIR/program, artifact/proof, runtime-completion, target-instance, cache-obligation, and performance-claim populations remain unobservable without their named descendants.
