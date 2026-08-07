---
id: realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity
title: Realign the compiler refinement error mirror with the grouped result arity
status: done
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

## Outcome — 2026-08-07

**All four result-side docs on `RefinementError` (`crates/tiler-compiler/src/legality.rs`) now state what the grouped `bind_results` actually reports.** Doc comments only: no field, variant, signature, `Display` string, or mapping arm changed, so no value a consumer observes moved and `a_well_formed_region_with_an_extra_output_is_rejected` still asserts `region_outputs: 2, results: 1` unmodified. The population each doc describes was read from `bind_results` (`crates/tiler-ir/src/index/refinement.rs:3344-3409`) rather than from the ticket's summary.

### Restated docs, old → new

**`ResultArity::region_outputs` — the ticket's named defect.**

- old: `/// Region output-root count.`
- new: `/// Count of the distinct output tensors the region's roots write.` plus `/// Tensors rather than roots, because a partitioned output is several roots over one tensor answering one semantic result.`
- why: `bind_results` groups roots by `region.access(root.access())?.tensor()` in first-encounter order and compares `outputs.len()` — the distinct-tensor count — against `occurrence.results.len()`. Roots are only the population before grouping.

**`ResultInterface::position` — restated, was ambiguous.**

- old: `/// Ordered result position.`
- new: `/// Ordered result position. The compared boundary is the output tensor a partitioned result's roots share, so one such result disagrees at one position rather than once per root.`
- why: the role/value-type/shape comparison runs once per group, against `region.tensor(*tensor)` — the shared boundary — not once per member. A reader told only "ordered result position" would expect one root behind it.

**`ResultValueType::position` — restated, was ambiguous.**

- old: `/// Ordered result position.`
- new: `/// Ordered result position, not a root ordinal: any one of a partitioned result's roots writing the wrong type reports the position of the result they jointly answer.`
- why: this check loops `for ordinal in &members[position]` over every member's `written_value`, and every member reports the same `position`. The error does not identify which root was mistyped.

**`IncompleteWrite` — variant line and `position`, both restated.**

- old variant line: `/// A region output is not backed by a complete unique write.`
- new variant line: `/// A root writing the result carries no write-ownership evidence.`
- old field: `/// Ordered result position.`
- new field: `/// Ordered result position, not a root ordinal: a partitioned result reports this one position whichever of its roots lacked evidence.`
- why the variant line moved too, beyond the ticket's named `position` sweep: the guard is `access.write_ownership_proof().is_none()`, and `WriteOwnershipProof::PartitionMember` is a proof. A partition member is *not* a complete unique write of the output and is nonetheless admitted, so the old line named a condition that no longer refuses. Left unchanged it would tell a reader the opposite of what the site does.

`ResultArity`'s variant line (`/// The region produces a different number of outputs than results.`) was read and left: "outputs" is resolved by the field doc immediately below it, and rewording it would restate the same fact twice.

### Observation for the coordinator — `Display` drift, not taken here

The four result-side `Display` strings still say `region output {position} …` (`legality.rs:690-705`), which now names a result position as a region output. They are **verbatim mirrors** of the IR's own strings (`refinement.rs:3020-3034`), so changing one side alone desynchronizes the mirror this ticket exists to keep aligned, and it is an observable-output change rather than documentation. `implementation/ir` was held by a concurrent worker for this dispatch. Worth a narrow ticket covering both crates' strings together; not filed, left to the coordinator.

### Commands — all on the branch tip

```sh
cargo fmt --all --check
cargo check -p tiler-compiler --all-targets
cargo clippy -p tiler-compiler --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler
cargo nextest run -p tiler-compiler   # 726 passed, 1 skipped
git diff --check
tkt lint
tkt guard tkt/realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity
make full
```
