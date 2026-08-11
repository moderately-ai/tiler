---
id: scope-precision-schedule-co-search-under-accuracy-obligations
title: Scope precision-schedule co-search under accuracy obligations
status: in-progress
priority: p3
dependencies: [define-the-model-level-conformance-corpus, implement-analytical-component-cost-model, emit-analytical-costs-through-the-typed-cost-vocabulary]
related: [implement-workload-selected-quantized-parameter-maps, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels, derive-the-oracle-for-a-permitted-divergence-candidate, prototype-quantized-value-vertical]
scopes: [research/numerics, research/cost-model, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, cost-model, trigger-fired]
claimed_from: todo
assignee: sol-precision-cosearch
lease_expires_at: 1786418733
---
## User-visible outcome

Precision becomes a search dimension jointly with the schedule — the SageAttention-class question — under the existing accuracy-obligation machinery, so a quantized-attention-shaped plan is something the optimizer can *propose and price* rather than something a caller must fully specify.

## Why this exists, and what the fired trigger now permits

**Fact.** The delivered quantized vertical is the per-tensor strict-affine machinery from `prototype-quantized-value-vertical` (caller-stated encoded values and a governed dequantization path), not optimizer-selected profiles as compile-request inputs. The L7 selected per-output-channel U8 profile in `docs/research/numerics/first-quantized-lm-profile.md` is a selection record with `implementation_status: "not-started"`; it does not register a scheme, admit a parameter map, or install a lowering capability. `implement-workload-selected-quantized-parameter-maps` remains `awaiting-decision` and is not yet an input path. Separately, `require_elementary_accuracy` assesses per-target obligations before contract resolution. **Inference.** Joint search over (dtype assignment × schedule) is a different capability: the candidate space multiplies, the accuracy obligation becomes a search constraint rather than a precondition, and identity must distinguish plans differing only in precision assignment. The literature to mine when fired: mixed-precision tuning (**Precimonious**, **HiFPTuner**), quantization-search (**HAQ**), and the Sage/SmoothQuant papers as primary sources on what the attention-specific wins actually were. Deferred behind both the conformance corpus (nothing prices an accuracy trade without an oracle to check it) and the cost authority.

## Trigger

`define-the-model-level-conformance-corpus` reaching named model-level corpus rows with fixed required outcomes (pass / refused / failed / disagreed) for model-level hazards, plus any cost authority landing — both, not either. The numerical-contract permitted-divergence oracle is a separate object owned by `derive-the-oracle-for-a-permitted-divergence-candidate`, not a family of corpus rows.

## Trigger check log

- 2026-08-05 — **not fired.** The corpus ticket is not `done` and no cost authority exists in `crates/`. Recheck: `grep -m1 '^status:' tickets/define-the-model-level-conformance-corpus.md` and the cost-type grep from the tuning-loop ticket's log.
  - **Correction — 2026-08-10.** The cost-absence clause was false under the same `^(pub )?` grep failure recorded on `design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature`: `pub(crate)` cost types and `ANALYTICAL_MODEL_KEY` already existed in `crates/tiler-compiler`, and structural cost also predated that log. The cost reading on this day is superseded by the 2026-08-09 fire; whether the corpus frontmatter was still non-`done` on 2026-08-05 is not re-derived here.
- 2026-08-09 — **fired.** `define-the-model-level-conformance-corpus`, `implement-analytical-component-cost-model`, and `emit-analytical-costs-through-the-typed-cost-vocabulary` are all `done`. The corpus supplies named rows with fixed required outcomes (pass / refused / failed / disagreed) for model-level hazards, while the compiler now carries structural and analytical cost authorities with typed cost assessments. Calibration is still future work, but the trigger asked for *any* cost authority, not calibrated device truth; this ticket is therefore `todo` and must decide how uncalibrated versus measured costs constrain the first co-search scope.
  - **Correction — 2026-08-10.** Earlier fire-log wording said "named permitted-divergence/refusal rows"; those are model-level corpus required-outcome rows, not the numerical-contract permitted-divergence oracle. That derivation is independently `done` on `derive-the-oracle-for-a-permitted-divergence-candidate`.

## Required work

Bounded research scope only — produce a first co-search envelope the board can brief or refuse, not an implementation:

1. **Precision-assignment space.** Decide what enters the first co-search: caller-stated encoded maps only (the delivered per-tensor strict-affine vertical) versus optimizer-minted dtype / precision assignment, and whether selected per-axis maps (`implement-workload-selected-quantized-parameter-maps`) are in or out of that first envelope.
2. **Accuracy-obligation stage.** State how accuracy obligations stay hard feasibility (today: `require_elementary_accuracy` before contract resolution; no trade against estimate) versus become search constraints, and at which stage each check runs for precision×schedule candidates.
3. **Identity.** State how plan identity encodes precision-only differences so two admitted plans that differ only in precision assignment are distinguishable for cache, explain, and selection — without inventing a domain here beyond the decision.
4. **Cost ranking under no semantic cost-pruning.** State how uncalibrated analytical, structural, and measured costs may rank admitted precision×schedule plans without pruning semantic alternatives on estimate; decide whether first co-search may use uncalibrated analytical authority or must wait for measured / calibrated costs.
5. **Primary sources and eliminations.** Name which primary sources (Precimonious, HiFPTuner, HAQ, SageAttention / SmoothQuant class, and any others admitted) are read and what each eliminates for the first envelope.
6. **Landing home.** Name where the scoping record lands (`docs/research/numerics/`, `docs/research/cost-model/`, or `docs/research/program-planning/`) once the envelope is fixed.

## Closes when

A scoping record or ticket Outcome decides the first co-search envelope with each of the Required-work decisions stated so a reader can refute them: precision space, accuracy stage, identity encoding, cost ranking under the no-semantic-cost-pruning invariant, primary-source eliminations, and landing home. Status moves only after that record exists; fire alone does not close this ticket.

## Explicit non-goals

- Implementing joint precision×schedule search, optimizer-minted precision assignment, or any production co-search path.
- Device cost calibration (`calibrate-device-cost-models`) or treating uncalibrated estimates as measured truth.
- Opening a new public crate, module, trait, or call-site boundary without Tom's acceptance of the exact surface.
- Treating the L7 first-quantized LM profile or `implement-workload-selected-quantized-parameter-maps` as already-delivered compile inputs.
- Splitting a free-standing literature survey ticket until this ticket's scope decides literature is a separate deliverable.
- Equating model-level corpus required-outcome rows with the numerical-contract permitted-divergence oracle.
