---
id: scope-first-quantized-lm-profile
title: Scope the first workload-backed quantized language-model profile
status: in-progress
priority: p2
dependencies: [define-first-metal-lm-workload, spike-first-metal-contraction-vertical, prototype-quantized-value-vertical]
related: [implement-first-quantized-backend-profile, define-initial-affine-quantization-semantics, define-quantized-value-binding-contract, implement-workload-selected-quantized-parameter-maps, own-the-dtype-support-maturity-matrix, admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode, widen-the-physical-vocabulary-for-per-axis-quantized-component-access, measure-code-domain-integer-arithmetic-on-the-qualified-apple-row, fuse-quantized-weight-decode-into-the-strict-contraction, extend-the-selected-quantized-profile-to-the-tied-embedding-matrix, group-internal-compound-materializations-by-logical-value]
scopes: [research/numerics, research/scheduling, research/apple-targets, contracts/numerics, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, quantization, language-model, matmul, metal]
---
## User-visible outcome

The first quantized LM profile is *chosen from evidence* — workload memory/accuracy needs against measured contraction behaviour — instead of a format picked by fashion. The choice arrives with its elimination record, so it does not get re-litigated per-format later.

Use the selected workload and measured contraction evidence to choose the first
quantized language-model profile. This ticket must not select a format before
the model, target, numerical behavior, and performance evidence make that
choice meaningful.

## Required analysis

- Compare candidate weight and value representations against the workload's
  memory, accuracy, packing, and Metal execution requirements.
- Define code, scale, zero-point, grouping, axis, layout, and conversion
  identity for every surviving candidate.
- Determine whether contraction consumes packed values directly or through an
  explicit dequantization boundary.
- Define the normative reference, accumulation behavior, output dtype, error
  criteria, artifact identity, weight validation, and runtime binding.
- Measure memory and performance against the non-quantized baseline on the
  selected target where feasible.
- Classify every selected logical type, compound scheme, component, storage carrier/encoding, kernel access/arithmetic type, and target-family dispatch fact in the dtype maturity ledger. A selected code width, packed layout, or native instruction cannot stand in for the other two.
- Name the exact physical-vocabulary widening required by the surviving profile. File it as a separate dependency of backend implementation with signature verification, KIR identity, ABI compatibility, target dispatchability, lowering/emission, and negative unsupported-combination tests; adding a carrier enum variant alone is not executable support.
- Separate a correctness-only execution proof from a device-optimal claim. Activate profile-specific analytical and measured cost work for packed/unpacked, explicit-dequantize, and fused candidates, keep unmeasured components `Unknown`, and make calibrated evidence a structural dependency before the selected route is described as optimal.

Eliminate any profile that cannot be validated or whose numerical realization
is unknown. A smaller artifact is not by itself evidence of a correct or faster
model.

## Ticket-producing outcome

Activate and refine `implement-first-quantized-backend-profile` for the selected
profile, or supersede it with narrower delivery tickets. File any additional
work for weight ingestion, packed contraction, conversion, conformance, and
model-level comparison with exact dependencies and scopes.

## Closes when

One bounded profile is selected from reproducible evidence or every candidate
is rejected with explicit reasons; the generic quantized-value reservation is
connected to a model-visible execution path; and all surviving work has
dependency-ordered tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L7** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L1 and L3 deliver **and** milestone 2Q supplies the quantized-value vertical proof.

**Rests on:** L1, L3, and milestone 2Q.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Quantization must be selected against that exact model and its F32 baseline rather than against a generic transformer. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **The selected backend graph must be complete before activation.** Link the exact physical-vocabulary ticket, `admit-a-dtype-dispatchability-capability-axis`, `group-internal-compound-materializations-by-logical-value`, `implement-workload-selected-quantized-parameter-maps` when the profile is non-per-tensor, `implement-first-runtime-semantic-value-precondition-enforcement` when the valid domain has runtime value predicates, and profile-specific cost calibration before any device-optimal claim.
- **Update `own-the-dtype-support-maturity-matrix` from evidence.** Advance only the cells the selected profile actually implements or tests; leave neighbouring widths, schemes, layouts, operations, targets, and runtime paths absent or reserved.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).

- **This consumes `prototype-quantized-value-vertical`'s answer** (is quantization a dtype or a compound contract) and `spike-first-metal-contraction-vertical`'s measurements — check both closed before starting, and cite their results rather than re-arguing them.

## Outcome (2026-07-31)

**One profile is selected from reproducible evidence:** per-output-channel strict-affine U8 to F32 over the pinned `Qwen/Qwen3-0.6B-Base` workload's 196 weighted projection weights, consumed through a decode fused into the contraction's weight operand access, on the qualified `apple9-f32-unified-msl4-macos26` row. The complete definition, the elimination with its ground per candidate, and the measurement boundaries are [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md); the evidence is [the Qwen3-0.6B-Base candidate quantization profile probe](../spikes/numerics/qwen3-weight-quantization-profiles/README.md). Nothing was implemented and no dtype ledger cell moved.

### The elimination, in one table

Three axes, taken in order, because an inexpressible candidate must never be compared on accuracy and an illegal one must never be compared on cost.

| Candidate | Verdict | Ground |
| --- | --- | --- |
| OCP MX (MXFP4/6/8, MXINT8) | **No** — not statable | No MX value can be constructed; the only parameter map that exists is per-tensor, which is the wrong association for a 32-element block. Registered identity is not a candidate. |
| OCP FP8 E4M3FN / E5M2 as weight storage | **No** — not statable | Registered scalar identity only; no operation signature admits one, and `MetalFloatArithmeticType` names exactly F32, F16, Bf16. It is a scalar-dtype vertical, not a quantization profile. |
| Codebook, hierarchical-scale, mask/outlier, NVFP, GGML | **No** — not spellable | Type-system reservation with no ordered role set, no producer, and no consumer. |
| BF16 storage widened at ingestion (control) | **Not a quantized candidate, and it dominates the frontier at 2×** | **Measured:** bit-identical logits at all 18 C1 positions, maximum deviation `0.000000e+00`, at 0.500 of the F32 weight bytes. It is what every quantized candidate has to beat. `spike-bf16-through-the-second-dtype-seams` owns it; nothing filed. |
| Per-tensor strict-affine U4/F32 | **No** — measured model quality | 0.4968 relative weight error; **0 of 18** greedy agreement; C1 sequence destroyed. This is the one profile with a delivered executable vertical, eliminated by evidence rather than kept because it was nearly built. |
| Per-tensor strict-affine U8/F32 | **No** — measured model quality | 0.0315 weight error; 8–9 of 18 greedy agreement; C1 sequence not reproduced in either variant. |
| Per-channel strict-affine U4/F32 | **No** — measured model quality | 0.1398 weight error; 15 of 18 without the embedding and 6 of 18 with it; sequence broken with the embedding. |
| Per-group 32/64/128 strict-affine U4/F32 | **No** — measured model quality, and inadmissible anyway | 0.0816–0.1028 weight error; 7–8 of 18; sequence broken in every variant. |
| Per-group128 strict-affine U8/F32 | **No** — contraction legality, despite the best measured accuracy | 0.0060 weight error and 18 of 18 greedy agreement, the best reading in the set. Its fused form partitions the contracted axis into contiguous intervals merged in order — L3's `contiguous_split` topology — which consumes reassociation that no contract registered for this workload grants. Hard feasibility, decided before cost. Reopens if a caller grants reassociation. |
| **Per-channel strict-affine U8/F32** | **Selected** | 0.0082 weight error, 17 of 18 greedy agreement and the exact C1 sequence in both variants, 0.251 of the F32 weight bytes with the embedding and 0.447 without. Its scale is loop-invariant over the contracted axis, so the fused decode preserves the strict ascending fold and consumes **no** numerical permission — it composes with L3's surviving `tiled` realization unchanged. Unpacked U8 also needs no packed encoding, bitstream order, tail rule, or partial-write ownership contract. |

Three results carried the decision and none of them is an accuracy preference. **A per-block scale is a reassociation of the contraction**, which is a legality fact derived from L3's own measured elimination. **An explicit dequantization boundary loses in both branches** — materializing per dispatch costs about 1.40 GB of traffic against the F32 baseline's 0.622 GB at L3's bandwidth-bound decode cell, and materializing once puts 622 MB of F32 weights back in residency — so fusion is the only route that delivers the memory win, and the fused and materialized forms are bit-identical here only because compute and expressed types are both `f32`. **The measured Metal refusal is removable by a stronger precondition, not a weaker contract:** exhaustively over the 256×256 code and zero-point domain, a normal scale makes the decode bit-identical under `FlushSubnormalsToZeroF32` and under a subnormal-preserving F32, and the smallest scale measured anywhere in the checkpoint is `1.358e-5`, about `1.2e33` times the F32 minimum normal.

### Dtype ledger cells moved: none, deliberately

The ledger classifies what Tiler has **built**, and this ticket built nothing; its own graph policy says to advance only cells a profile actually implements or tests. What changed in `docs/dtype-support.md` is evidence prose, in three places: the U4 section records that the profile was measured against a real workload and eliminated; the U8 section records that its trigger fired, names all seven fields it demanded, and states that the selected per-channel form is still not a *statable* contract because the scheme validator admits only the two per-tensor ones; the other-affine section records that the affine half of its trigger fired while the MX half did not, and that per-block was eliminated on legality rather than accuracy. No new row was added — rows 47 and 74 already cover "non-per-tensor maps" and "accepted strict-affine forms beyond the two implemented profiles", and a second row for the same cells would be the duplicated authority this corpus warns about. The five-way classification the ticket asked for (logical type, numerical interpretation, parameter map, storage carrier/encoding, kernel access type, plus target dispatchability) is a table in the research record, stated as what each delivery ticket *would earn* rather than as a claim.

One stale link was corrected along the way: the U4 trigger pointed at `admit-a-dtype-dispatchability-capability-axis`, which is `closed` as superseded by the caller-declared target profile boundary.

### Tickets filed and refined

Five filed, dependency-ordered, each scoped to surviving work — nothing filed for U4, per-block, MX, or FP8 as a realization:

1. `admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode` — deps `prototype-quantized-value-vertical`, this ticket; scopes `implementation/ir`, `implementation/reference`, `implementation/metal`, `contracts/numerics`.
2. `widen-the-physical-vocabulary-for-per-axis-quantized-component-access` — the exact physical-vocabulary ticket bullet 7 requires: a parameter component addressed by a *projection of the iteration domain* rather than as a rank-zero scalar. **No new `StorageScalar`, `StorageEncoding`, or `KernelType` is needed** — that is what choosing eight bits bought. Deps `implement-workload-selected-quantized-parameter-maps`; scopes `implementation/ir`, `implementation/compiler`, `implementation/artifact`, `implementation/metal`.
3. `measure-code-domain-integer-arithmetic-on-the-qualified-apple-row` — experiment E-1; scope `research/apple-targets`.
4. `fuse-quantized-weight-decode-into-the-strict-contraction` — deps `realize-the-strict-contraction-on-metal`, (2), `implement-workload-selected-quantized-parameter-maps`; scopes `implementation/compiler`, `implementation/ir`, `implementation/metal`.
5. `extend-the-selected-quantized-profile-to-the-tied-embedding-matrix` — `deferred`, deps `implement-first-quantized-backend-profile` and `admit-an-indirect-gather-family-for-tied-embedding-lookup`.

Three refined. `implement-first-quantized-backend-profile` moved from `deferred` to `todo` and now carries the selection table plus **six structural dependency edges** — (1), (2), (3), (4), `implement-workload-selected-quantized-parameter-maps`, and `implement-first-runtime-semantic-value-precondition-enforcement` — because its own graph-maintenance rule is that a prose list is not a dependency; its packed sub-byte requirement is recorded as not applying. `implement-workload-selected-quantized-parameter-maps` is refined with the exact selected map and, per its own activation boundary, is **actionable rather than obsolete**. `group-internal-compound-materializations-by-logical-value` records that the expected consumer did not arrive.

**Two instructed edges could not be created as written, and both corrections matter.** `admit-a-dtype-dispatchability-capability-axis` is `closed` as superseded, so it cannot be linked; `admit-a-caller-declared-target-profile` (`done`) owns canonical resolved-type dispatchability facts now, and what a selected backend owes is a measured `(target family, dtype)` row through that boundary. And `group-internal-compound-materializations-by-logical-value` got no *direct* edge, because the profile's weights arrive as role-addressed compound **interface inputs** — the executed program contains no `Quantize` and no `Assemble`, so it materializes no compound value internally, which is the path `prototype-quantized-value-vertical` already proved end to end.

**That second one needed checking rather than asserting, and checking changed it.** `tkt path` puts the grouping ticket on the critical path anyway, transitively: backend profile → `implement-first-runtime-semantic-value-precondition-enforcement` → `carry-semantic-enforcement-plans-through-program-and-artifact` → `admit-strict-affine-quantize-physical-candidate` → grouping. The chain is real because the runtime-enforcement vertical is scoped to strict-affine **`Quantize`**, which does produce a compound value internally, while the selected profile needs the value domain of a **`Dequantize` input** enforced instead — positive normal scale, in-range zero point, parameter extents matching the codes' axis-0 extent, and no packed tail at all. So the selected profile currently blocks on an internal-grouping capability it never exercises. That is a scope mismatch inside an existing authority, not a requirement of this profile: a dated note on the enforcement ticket states it and leaves widening-or-splitting to its owner, and no competing ticket was filed for an authority that already has one.

### Measurement gaps, as bounded experiments

E-1 code-domain integer arithmetic on the qualified Apple row — **no integer arithmetic has ever been measured on any Apple GPU in this repository**, so this blocks every executable claim; ticket filed. E-2 the fused decode's achieved bandwidth against the F32 baseline, blocked because the contraction probe hard-codes `float` in all three of its producers including an exact binary32 oracle with no integer analogue; it blocks every device-optimal claim and belongs with `calibrate-device-cost-models`. E-3 a second accuracy row beyond C1, owned by L8. E-4 error-compensating calibration, which could revisit the U4 rows on accuracy and could not revisit per-block on legality. Each states inputs, outputs, and a stop condition in the record.

### Verification

`tkt lint`; `git diff --check`; `tkt guard --base 724ac4e`; `make full`. Every fail-closed check in the new spike was fault-proved: the checkpoint digest check fired against a substituted file, and the restore-exactness check fired when one tensor was dropped from every restore. The conversion itself has a positive control — an affine round trip's error is bounded by half a step and reaches it, measured at `0.06666672` against a half-scale of `0.06666667`. The Stage B F32 baseline reproduces the retained C1 fixture's exact 18-token sequence, which anchors the differential readings; it does **not** run the pinned `transformers` 4.51.0 reference, and the record says so rather than implying otherwise.

### Deliberately not done

No ticket for weight ingestion — turning checkpoint bytes into the profile's components is the same question L1 left open for BF16-to-F32, and `design-model-ingestion-and-complete-execution` owns it. No model-level accuracy budget, which L8 owns and which this record supplies measured inputs for rather than competing with. No implementation of any kind, and no device measurement.
