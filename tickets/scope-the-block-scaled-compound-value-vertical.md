---
id: scope-the-block-scaled-compound-value-vertical
title: Scope the block-scaled compound value vertical
status: deferred
priority: p3
dependencies: [scope-the-ocp-reduced-precision-float-vertical, implement-workload-selected-quantized-parameter-maps]
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, own-the-dtype-support-maturity-matrix, widen-the-physical-vocabulary-for-per-axis-quantized-component-access]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, microscaling, quantization]
---
## User-visible outcome

The six OCP MX schemes, NVFP4, and the project block codecs have one owner, and a reader can tell that reaching them is not a scalar-dtype widening and not reached by implementing a per-axis map.

## Why this exists

**Fact.** [ADR 0038](../docs/decisions/0038-recognize-ocp-mx-schemes.md) recognizes six OCP MX 1.0 32-element compound scheme identities. [The dtype support ledger](../docs/dtype-support.md) records all six registered with their constituent element and E8M0 scale types, block size 32, and scale-selection contract, every static contract offered to one refused with `microscaling.unsupported-contract` — and **no MX value constructible**, because "the only parameter-index map that exists is per-tensor, which is the wrong association for a 32-element block, so admitting a contract would put a false numerical claim into durable identity".

**Fact — NVFP4 is not MXFP4.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md) records that NVFP4 uses a conceptual FP8 E4M3 local scale per 16 values plus an FP32 tensor-global scale, that supported weight layouts may add two-dimensional scaling, and that a backend may encode the local scale through a specialized unsigned format such as PTX `.ue4m3`. Group size and scale format both differ from OCP MXFP4.

**Fact — this is where the ledger's thirteen-rung recipe breaks, and it names the breakages.** Rung 9 "**fails: one logical value maps to two physical buffers**, so a one-buffer-per-value assumption breaks here"; rung 10's transport mapping is not one-to-one; rung 3's ULP metric is "not applicable today, and the refusal is the right answer until a block-aware metric exists"; and rung 4's oracle must model the shared scale because "a value is a block, not an element".

## Activation trigger, in three parts

1. A selected model format names its exact constituent scheme; **and**
2. a non-per-tensor parameter-index map exists — a prerequisite rather than a substitute, and the per-axis map now being implemented is not a 32-element block map.
3. Separately and sufficiently for the eliminated per-block and per-group affine maps: a caller grants reassociation. The ledger records that those maps were eliminated on **legality** — a scale varying along the contracted axis makes a fused contraction partition that axis into contiguous intervals merged in order — and not on accuracy, where they measured best of any candidate.

## Closes when

The trigger has fired and the vertical is stated including the block oracle, the two-buffer physical consequence, and the block-aware accuracy metric — or block scaling is explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-9 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- Depends on [`scope-the-ocp-reduced-precision-float-vertical`](scope-the-ocp-reduced-precision-float-vertical.md) for its constituent element semantics and on [`implement-workload-selected-quantized-parameter-maps`](implement-workload-selected-quantized-parameter-maps.md) for the non-per-tensor map that is its stated prerequisite. Numerical and reference authority precedes any optimizer or backend claim here, as the ledger's graph policy requires.

## Trigger check log

- 2026-08-04 — **not fired.** Track D-9's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md):201. Verified independently against the tree: `ParameterIndexMapKind` still has exactly one variant, `PerTensor` (`crates/tiler-ir/src/semantic/types.rs:1247-1249`), so the stated prerequisite — a non-per-tensor parameter-index map — does not exist, and [`implement-workload-selected-quantized-parameter-maps`](implement-workload-selected-quantized-parameter-maps.md) is `todo`. Recheck: `grep -n 'enum ParameterIndexMapKind' -A 4 crates/tiler-ir/src/semantic/types.rs`.
- 2026-08-09 — **not fired.** `ParameterIndexMapKind` still contains only `PerTensor`, and `implement-workload-selected-quantized-parameter-maps` remains `todo`; the selected quantized profile uses unpacked U8 rather than an OCP MX, NVFP4, or project block codec. No exact constituent scheme plus block map has arrived.
