---
id: reproduce-the-identical-output-chain-stage-key-collision
title: Reproduce the identical-output-chain stage-key collision at the current compiler boundary
status: done
priority: p2
dependencies: []
related: [refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, multi-output, identity, research]
---
## Why this evidence is separate

**Historical Measurement — 2026-08-06.** At base `afdac9c9`, two independent same-shaped epilogue chains over different declared inputs reached `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))`; changing one chain's extent avoided the collision. The current compiler test module preserves that finding only as prose under the source-verifiable anchor `identical shape assemble two stages carrying one canonical key`. It deliberately uses different extents in its executable fixture, so it does not prove the old failure still occurs.

**Fact — current identity seam.** `stage_key` derives a stage key from the bound kernel identity and proof-bound coverage. `verify_unambiguous` rejects two equal stage keys. Program identity then orders stages and names value definitions through those keys. Changing the key's subject or merging stage instances is therefore an identity question; reproducing the failure is not.

## Work

1. Read the complete multi-output fixture, program assembly, stage-key derivation, unambiguous-key verification, and compiler failure mapping.
2. Add a current compiler regression using two independently declared inputs and outputs whose producer chains have the same shape. Keep the different-extent neighbor and a one-chain control.
3. Record the exact current public failure. If the pair now compiles, record the program/stage population and why the historical collision disappeared.
4. If it still collides, inspect the two assembled stages and state exactly which fields agree and which distinct bindings, values, outputs, launches, or coverage facts the current stage key omits.
5. Perturb the subject—shape, binding, or occurrence ownership rather than the expected error—and quote the changed result.

Do not change stage keys, program identity, IR verification, assembly, or the public failure vocabulary here.

## Outcome — current evidence at `b3b1652faa6c0060e4958782c2d5d37b563b9f8b`

**Measurement — public boundary, 2026-08-10.** `same_shaped_epilogue_chains_reach_invalid_compiler_output` now constructs `sum(x * x, axis 1) * 2.0` as `sx` and `sum(y * y, axis 1) * 3.0` as `sy`, with two independently declared `[1, 4]` inputs. Under `NumericalContract::REASSOCIATE_F32`, public `session::compile` returns request-wide `Err(CompileFailure)`, `class() == CompileFailureClass::InvalidCompilerOutput`, with `explain().is_some()`. The one-chain `[1, 4]` control compiles. Changing only `y` from `[1, 4]` to `[1, 2]` also compiles while retaining two inputs, two named outputs, the operation families, constants, and contract. The targeted command is `cargo nextest run -p tiler-compiler --test multi_output_boundary same_shaped_epilogue_chains_reach_invalid_compiler_output`.

**Measurement — exact internal cause and subject perturbation, 2026-08-10.** Temporarily changing the existing `two_chain_program` call anchored by `"y", "halved", 3.0_f32.to_bits(), 2` to end in `4`, without changing its assertion, and running `cargo nextest run -p tiler-compiler --lib the_two_chain_program_is_refused_by_regions_until_the_budget_admits_both` failed with the prefix:

```
the chain compiles: Explained { source: InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage }))), ... }
```

Restoring the extent to `2` restores the successful neighbor; the probe left no diff in `crates/tiler-compiler/src/pipeline/tests.rs`. The permanent public regression carries both directions without parsing private failure detail.

**Fact — source-backed colliding-stage comparison.** The equal keys belong to the two split combiners. At the anchor `fn final_reduction_region`, equal `[1, 4]` subjects under one request derive the same scheduled final-pass region and therefore the same bound kernel identity and launch geometry. Each physical combiner nevertheless owns a different fold occurrence's second `SemanticStage`, derived at the anchor `map(|atom| atom.next_stage())`. Assembly retains those distinct stage claims in `AssemblyStage::coverage`, but `build_cover_core` calls the anchor `fn covered(`, whose `filter(|atom| atom.is_first())` projects both later-stage claims out; both shared-program stages therefore reach `stage_key` with the same kernel identity and an empty proof-bound coverage list.

The assembled stages remain distinct elsewhere. Their `AssemblyBinding::Internal` entries name different partial and result positions; the pushed accesses consequently name different views, materialized values, and allocations; their `AssemblySplit` records name different producers, combiners, partials, and results; and distinct dependencies carry those results into epilogues ultimately published as `sx` and `sy`. Their launches agree rather than distinguish them. The anchor `fn stage_key(stage: &StageData)` folds only the bound kernel identity and the projected `CoveredOccurrence` records: it does not fold launches, accesses or bindings, views, values, allocations, dependencies, split membership, downstream output publication, or the later-stage occurrence ownership removed by `covered`. `verify_unambiguous` therefore sees two equal stage keys before those distinct facts can participate in program identity.

**Unsupported cases.** This evidence is bounded to the governed static-`f32` target, the reassociation contract that admits the split, the square-prologue reduction/epilogue family, and the four-contributor split against the two-contributor neighbor. It does not establish behavior for other targets, dtypes, contracts, producer families, extents, or any remedy. No identity domain, stage key, verifier, assembly rule, schema, or public failure vocabulary changed.

## Closes when

The historical collision is either reproduced at the current public compile boundary with a source-backed stage comparison and controls, or proved obsolete with the landing that removed it. The dependent decision ticket carries the resulting evidence rather than the 2026-08-06 measurement alone.
