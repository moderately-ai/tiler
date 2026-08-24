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

This is the retained exact-base source inventory behind [The closed-world conformance claim universe by owner](../../../docs/research/verification/conformance-claim-universe-by-owner.md). It is a reading and census record, not a harness and not a repository gate. The machine-readable entry point is [`inventory.tsv`](inventory.tsv).

## Base and digest

All source verdicts were made at:

```text
37a8107e9999b29b51a5c7458b5fd0bc0a408e3a
```

From the repository root:

```sh
git rev-parse HEAD
shasum -a 256 spikes/verification/conformance-claim-universe-by-owner/inventory.tsv
wc -l spikes/verification/conformance-claim-universe-by-owner/inventory.tsv
cut -f1 spikes/verification/conformance-claim-universe-by-owner/inventory.tsv | tail -n +2 | LC_ALL=C sort | uniq -d
```

Recorded result: digest `192aafe506547f4ec9d730cf9ff2c5b1a869dfa23cfa038e29d9759c10f4487a`, 26 lines including the header, and no duplicate family id. The SHA-256 binds this retained snapshot only; it does not establish normative authority.

## Reading population

The inventory followed every row from construction through consumption and refusal, rather than treating a definition grep as ownership. The principal owner paths were:

- semantic registry, standard operation/type construction, algebraic declarations, index realization laws, numerical policy, target profiles, schedules, kernels, and programs under [`crates/tiler-ir/src`](../../../crates/tiler-ir/src/);
- reference registry, standard registrations, evaluation, and validation under [`crates/tiler-reference/src`](../../../crates/tiler-reference/src/);
- rewrite, lowering, request, feasibility, budget, explain, and physical-provider paths under [`crates/tiler-compiler/src`](../../../crates/tiler-compiler/src/);
- artifact program/proof construction, codecs, verification, and identity under [`crates/tiler-artifact/src`](../../../crates/tiler-artifact/src/);
- AOT stages under [`crates/tiler-metal-aot/src`](../../../crates/tiler-metal-aot/src/), runtime routes under [`crates/tiler-runtime/src`](../../../crates/tiler-runtime/src/), expansion cache under [`crates/tiler-build/src`](../../../crates/tiler-build/src/), and the cross-layer evidence boundary under [`crates/tiler-conformance/src`](../../../crates/tiler-conformance/src/).

The governing contracts were [Correctness and testing](../../../docs/correctness-and-testing.md), [ADR 0106](../../../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md), the accepted decisions catalog, and the owner module contracts linked above.

## Reproducing the evidence envelopes

The root spike's test-count vocabulary was rerun exactly. It counts `#[test]` attributes by source bucket; it does not count features, test functions hidden behind other attributes, generated cases, or doc-tests.

```sh
rg -n '#\[test\]' crates/tiler-ir/src | wc -l
rg -n '#\[test\]' crates/tiler-reference/src | wc -l
rg -n '#\[test\]' crates/tiler-reference/tests | wc -l
rg -n '#\[test\]' crates/tiler-compiler/src | wc -l
rg -n '#\[test\]' crates/tiler-compiler/tests | wc -l
rg -n '#\[test\]' crates/tiler-ir/src/schedule | wc -l
rg -n '#\[test\]' crates/tiler-ir/src/kernel | wc -l
rg -n '#\[test\]' crates/tiler-artifact crates/tiler-build crates/tiler-runtime crates/tiler-metal-aot | wc -l
rg -n '#\[cfg\((target_os = "macos"|not\(target_os = "macos"\))\)\]' crates/tiler-metal/src | wc -l
rg -n '#\[cfg\((target_os = "macos"|not\(target_os = "macos"\))\)\]' crates/tiler-metal-aot/src | wc -l
```

Recorded outputs, in order: `484`, `182`, `152`, `115`, `94`, `265`, `137`, `954`, `10`, `5`.

The search vocabulary is complete only for this stated syntactic question: literal `#[test]` and the two literal macOS `cfg` spellings in the named trees. It is intentionally not used as the claim universe.

## Reproducing bounded owner censuses

These commands locate the complete owner mechanisms and the bounded source populations. Counts requiring loop expansion are stated explicitly.

```sh
rg -n 'operation_definitions|value_type_definitions|index_realization_laws' crates/tiler-ir/src/semantic/registry.rs
rg -n 'BUILT_IN_SCALARS|COMPLEX_COMPONENTS|MICROSCALING_SCHEMES|register_value_type' crates/tiler-ir/src/semantic/standard.rs crates/tiler-ir/src/semantic/types.rs
rg -n 'registrar\.register\(|register_validator\(' crates/tiler-reference/src --glob '*.rs'
rg -n 'GOVERNED_CAPABILITY_COUNT|LoweringFamily::' crates/tiler-compiler/src/lowering.rs crates/tiler-compiler/src/lowering --glob '*.rs'
rg -n 'tiler\.(normalize|algebraic)/[^" ]+' crates/tiler-compiler/src --glob '*.rs'
rg -n 'STRATEGY_[A-Z0-9_]+' crates/tiler-compiler/src --glob '*.rs'
rg -n 'DIMENSION_COUNT|CANONICAL_DIMENSIONS' crates/tiler-ir/src/numerical --glob '*.rs'
rg -n 'variant_count::<(BudgetResource|ExplainDisposition|CompileStage)>' crates --glob '*.rs'
rg -l '^evidence_classes:.*bounded-measurement' spikes -g README.md | wc -l
```

Vocabulary justification:

- semantic and reference populations are derived from the frozen owner maps; searches locate registration construction, then loops are expanded from their full arrays;
- reference operation registration has 19 literal call sites plus six additional concatenate variants and three additional quantization variants, yielding 28; validator registration yields seven after its two loop expansions;
- the three rewrite identities and six strategy constants are explicitly labelled **source-known**, because construction remains fragmented or extensible;
- `variant_count` and owner-sized arrays are accepted only where the type owns the population; and
- `bounded-measurement` is exact for record metadata, while the individual performance claims inside those 46 records remain unknown.

A failed search for any expected owner was followed by opening the module tree, because module splits and re-exports make file-local zeroes unsafe.

## Subject perturbations

No production source was changed. These are negative-control designs for the future checks; every complete claim states what source mutation must make it fail.

| complete census | subject mutation | required failure |
| --- | --- | --- |
| semantic operations/types/algebra | add an owner registration, add a type-family member, or change a previously absent algebraic property to positive | owner iterator population and semantic-registry identity change; a pinned manifest must reject the old snapshot |
| index laws | register an eighteenth law | fixed owner population and registry identity fail |
| reference capabilities/validators | add a registration through the standard owner | owner-derived manifest population and reference-registry identity change; no source-call count may stay authoritative |
| lowering family/capabilities | add a `LoweringFamily` variant or a governed capability | exhaustive matches or the fixed 22-row assertion fail |
| budgets | add a `BudgetResource` variant | `variant_count`-sized `ALL` fails to compile until disposed |
| explain dispositions | add a variant | `variant_count`-sized owner census and exhaustive mappings fail |
| numerical dimensions | add a dimension | `DIMENSION_COUNT`-sized canonical arrays fail until extended |
| local compilation stages | add an AOT or call-failure variant | type-sized `ALL` or exhaustive stage/fallback mapping fails |
| runtime route failures | add a route-failure variant | exhaustive `fallback_permitted` fails to compile |
| top-level cache contract | add a sixth accepted property | the owner obligation manifest proposed by the follow-up must reject the undisposed property |

For every unknown family, the negative control is also explicit in [`inventory.tsv`](inventory.tsv): introduce a new subject through a construction site the current bounded census does not own. Any global suite that remains green has demonstrated the exact fail-open defect this inventory refuses.

## Unsupported and unobservable cases

This record cannot support:

- a claim that the optimizer, planner, schedule/KIR/program verifiers, artifact/proof codecs, runtime completions, detailed cache protocol, or performance assertions are completely enumerated;
- promotion of owner-private iterators to public API;
- a normative candidate-universe domain, schema, digest algorithm, or revision rule;
- selection of a goal profile or any green/yellow/red support disposition;
- current performance outside a retained record's named host, toolchain, inputs, and base; or
- inference that a family has zero members because a search returned zero.

Those absences are represented by positive unknown rows and linked follow-up tickets, so they cannot be mistaken for successful empty enumerations.
