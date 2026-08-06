---
id: recompute-the-explain-request-qualifier-for-the-silu-subnormal-fact
title: Recompute the explain request qualifier for the SiLU subnormal fact
status: todo
priority: p1
dependencies: []
related: [correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, numerics, identity]
---
## User-visible outcome

The workspace gate is green again on the tree that corrected `tiler::silu-f32@1`'s declared subnormal fact. Exactly one pinned identity moved, it is known, its replacement value is measured, and this ticket applies it.

## The moved pin

**Fact, measured on `tkt/correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail`.** `crates/tiler-compiler/src/explain.rs:4134` pins the request qualifier of `deterministic_trace_is_sealed_and_rendered_separately`:

- old: `f3244b2242ebcb5c`
- new: `6dd42be71c6745fe`

**Measurement.** Two independent runs on that branch reported the identical `left` value, and it was the only failure in a full-workspace run: `2727 tests run: 2726 passed, 1 failed, 7 skipped`. Reproduce with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and read the `left` value the assertion reports. **Recompute it on the merged tree rather than copying this value** — if another branch carrying a registry-folding change lands first, both are green separately and neither value is the merged one.

## Why it moved, and which half

**Fact.** The request subject folds the frozen semantic registry snapshot, whose definition projection writes `definition.canonical_facts().value()` for every registered operation (`encode_operation_definition`, `crates/tiler-ir/src/semantic/registry.rs:2828-2846`). The producing branch changed one field of `tiler::silu-f32@1`'s fact record, so those bytes land in that one field of the qualifier. This is the semantic half moving alone; no compiler-side registry changed.

**Inference — no encoding version is owed, and the producing branch established that by reading each owning site.** `tiler.semantic-definition-projection.v5` (`registry.rs:1784`) and `tiler.semantic-registry.v7` (`:2673`) count rendering revisions and nothing about the rendering moved: `CanonicalValueData::Utf8` encodes as tag `7` followed by a length-prefixed `push_slice` (`crates/tiler-ir/src/semantic/types.rs:996-999`), so a payload of any length stays injective under the unchanged rendering. The standard semantic provider stays at revision 7 on its own documented rule (`registry.rs:2240-2255`). Only the resulting digest moves.

## What to do

Replace the literal at `crates/tiler-compiler/src/explain.rs:4134` with the value observed failing on the merged tree, and append a paragraph to that test's rebaseline ledger stating the change, which half of the subject moved, and that no encoding version stepped — the convention every prior entry in that ledger follows.

## Why it is a separate ticket

The producing branch holds `implementation/ir` only. `crates/tiler-compiler/**` is `implementation/compiler`, a distinct exclusive scope held at the time by the live claim `reach-a-verified-kernel-through-the-structural-families`, whose branch had zero commits — so file-level disjointness against it was **vacuous rather than verified**, and an edit there could not be justified. The worker stopped at that boundary rather than editing outside it, exactly as `recompute-the-explain-request-qualifier-for-the-bf16-realization-rows` did.

## Closes when

The literal is updated with its ledger paragraph and a full-workspace run is green.
