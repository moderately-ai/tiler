---
id: realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity
title: Realign the compiler refinement error mirror with the grouped result arity
status: todo
priority: p3
dependencies: []
related: [bind-a-partitioned-output-through-index-refinement]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler]
---
## User-visible outcome

`RefinementError::ResultArity::region_outputs` documents what it now carries, so a reader of the compiler-side error is not told the count is output roots when the IR that produces it counts distinct output tensors.

## Why this exists

**Fact — the mirror's doc comment is now a claim about superseded behaviour.** `crates/tiler-compiler/src/legality.rs:591-596` declares `RefinementError::ResultArity { region_outputs, results }` and documents `region_outputs` as "Region output-root count.". [`bind-a-partitioned-output-through-index-refinement`](bind-a-partitioned-output-through-index-refinement.md) changed the population `IndexRefinementVerificationError::ResultArity` counts to the region's *distinct output tensors*, because a partitioned output is several roots answering one semantic result. The mapping at `legality.rs:869-875` copies the field verbatim, so the value is correct and only the comment is wrong.

**Fact — no observable value changed.** A region with one root per output tensor counts the same either way, and no registered capability emits a partitioned region yet, so `a_well_formed_region_with_an_extra_output_is_rejected` (`legality.rs:1815-1832`) still observes `region_outputs: 2` for its two-distinct-output fixture.

**Inference — it is a separate ticket because the scope was held.** `implementation/compiler` carried a live claim (`region-expansion-exhaustion-loses-the-only-feasible-plan`, `agent-region-expansion`) for the whole of the binding ticket's dispatch, so the one-line edit was serialized rather than taken.

## What the work is

Restate the field's doc comment to match what IR counts, and check the surrounding `RefinementError` result-side docs (`ResultInterface`, `ResultValueType`, `IncompleteWrite` all document a `position`) against the same change: `position` is the ordered *result* position, and for a partitioned result several members report the same one.

## Closes when

The mirror's documentation states the counted population correctly, and the compiler package's targeted checks and rustdoc are green.
