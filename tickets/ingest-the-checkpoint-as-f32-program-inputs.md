---
id: ingest-the-checkpoint-as-f32-program-inputs
title: Ingest the pinned checkpoint as F32 program inputs
status: in-progress
priority: p1
dependencies: [define-the-model-weight-binding-manifest, route-an-embedded-artifact-through-a-consumer-storage-seam, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, derive-transformer-operation-and-shape-surface, spike-bf16-through-the-second-dtype-seams, drive-the-complete-forward-pass-over-three-artifacts]
scopes: [implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, ingestion, weights, dtype, consumer, language-model, class-conformance-fixture]
claimed_from: todo
assignee: worker-checkpoint-ingest
lease_expires_at: 1787004917
---
## User-visible outcome

The pinned BF16 checkpoint becomes 310 dense F32 values a program can be handed, converted once at load, so no cast appears anywhere in the executed program.

## The decision this implements, and why it is not the cheap option

[L2 recommended host-side conversion](../docs/research/shapes/transformer-operation-and-shape-surface.md) and asked L6 to refute or adopt it. [The L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) adopts it and adds the ground L2 did not have: an operation inside a program is evaluated on every execution of that program, and under the per-layer program boundary a `Cast` on the eleven layer weights runs 28 times per forward pass and **252 times over the C1 row's nine passes** — converting the 28 layers' 880,932,864 BF16 bytes into 1,761,865,728 F32 bytes on every forward pass (once per layer-program execution), against once at load. No hoisting capability could lift it out, because the boundary it would have to cross is the consumer's own loop.

**Fact.** The conversion is exact for every finite BF16 value: BF16 is a truncated F32. **Measurement, inherited from [L7](../docs/research/numerics/first-quantized-lm-profile.md):** replacing all 197 weighted projections with their BF16 round trip is bit-identical at every C1 position, maximum logit deviation `0.000000e+00`. That qualifies one row and one checkpoint.

**Correction — 2026-08-02.** The paragraph above previously added "so even a BF16 subnormal widens to an F32 normal that the qualified target's flush cannot touch". That is false: it is true of binary16 and not of BF16, which shares binary32's exponent width so that widening preserves the subnormal class — measured exhaustively at 254 of 254 in [the BF16 conversion record](../docs/research/numerics/bf16-computation-accumulator-and-conversion.md), and depended on by [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md)'s explanation of the qualified row's `bf16`-flushes/`f16`-preserves split. **This sharpens what the checks below are for rather than changing them.** A widened BF16 subnormal *is* reachable by the target's flush in general; on the pinned revision there is nothing to reach — 0 subnormal, 0 infinite, and 0 NaN stored values over all 596,049,920 elements of all 310 tensors, measured by [the corpus reachability probe](../spikes/program-planning/qwen3-corpus-reachability/README.md) — so the non-finite check this ticket owns is a counted zero on this checkpoint rather than an untested branch, and that is a property of this revision rather than of BF16. Add the subnormal count beside the non-finite one when the widened bytes are digested: it is the same pass and it is the quantity a substituted checkpoint would move. The derivation is in [the L8 corpus section](../docs/research/program-planning/model-level-qualification.md#three-rows-deliberately-absent-with-the-ground-for-each).

**Correction — 2026-08-10.** Required content and Closes when previously understated the 2026-08-02 correction's non-finite and subnormal-count obligations: both sections listed digests, manifest gates, TensorAdapter wrap, and StorageScalarMismatch refusal, but omitted the exceptional-value census that correction and [L8](../docs/research/program-planning/model-level-qualification.md) assign to this ticket. The decision paragraph also said "on every token" for the layer-byte conversion cost; that was imprecise relative to C1's nine passes (prefill is one layer walk over ten prompt tokens, not ten), and is now pass-based wording consistent with the 252 figure and L6's "once per forward pass — nine times over the conformance row". L6 I-A still carries the inherited "on every token" phrase in the research record; that is residual prose outside this ticket's scopes.

## Required content

- Acquisition under [L1](../docs/research/program-planning/first-metal-lm-workload.md)'s policy: no checkpoint bytes at any path in this repository, reconstructed on demand into a directory the consumer's own README declares and a narrow gitignore entry covers.
- Every manifest digest verified locally before the bytes carry any claim, and the weight binding manifest checked before any value is wrapped.
- 310 dense row-major F32 values wrapped through a `TensorAdapter` that offers `AdapterCapability::DenseRowMajorStorage`, reporting `StorageScalar::F32`.
- **One digest over the widened bytes**, retained on the consumer load path that wraps those `TensorAdapter` values, because the widening joins the conformance oracle's comparison surface and an assumption is not evidence. The C1 attribution fixture already retains oracle `weights.widened.sha256` (and related host.tsv rows) as ticket 9's comparison surface; that fixture surface does not close this ticket — the obligation here is to produce and gate the same class of digests where the consumer loads and wraps program inputs.
- A non-finite census over the elements on the same pass that digests the widened bytes: NaN count and infinite count, each refusing or stopping the load when non-zero (the guard this ticket owns per L8). On the pinned revision both counts are measured zeros (corpus reachability / L8), so the checks are counted zeros rather than untested branches; a substituted or corrupted checkpoint is the subject that must make them fail.
- A subnormal count recorded beside the non-finite counts on that same pass. On the pinned revision the count is the measured zero already cited above; retain it because a substituted checkpoint can move it even when non-finite stays zero.
- A BF16-storage operand offered to a program declaring F32 refuses by name as `BindError::StorageScalarMismatch`, watched failing.

## Workspace admission

This consumer is a new prototype member and nothing may depend on it. Add the member and the lockfile update in one commit so a reviewer sees both, and note that no `[scopes]` entry in `ticketsplease.toml` covers a new `prototypes/` directory — adding one is part of this ticket and is covered by the shared `project/tickets` scope. Do not add a `[scope_crates]` mapping for a prototype.

## Closes when

The 310 values exist as F32, the digests and the manifest both gate the run, the widened-byte digest is retained on the consumer load path (fixture host.tsv alone is not enough), the non-finite counts and the subnormal count are retained from the same digests pass, the non-finite guard is watched able to fail on a substituted fixture, the wrong-scalar refusal is watched failing, and no `Cast` appears in any program.
