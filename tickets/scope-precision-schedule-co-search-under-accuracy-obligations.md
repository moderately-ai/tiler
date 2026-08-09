---
id: scope-precision-schedule-co-search-under-accuracy-obligations
title: Scope precision-schedule co-search under accuracy obligations
status: todo
priority: p3
dependencies: [define-the-model-level-conformance-corpus, implement-analytical-component-cost-model, emit-analytical-costs-through-the-typed-cost-vocabulary]
related: [implement-workload-selected-quantized-parameter-maps]
scopes: [research/numerics, research/cost-model, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, cost-model, trigger-fired]
---
## User-visible outcome

Precision becomes a search dimension jointly with the schedule — the SageAttention-class question — under the existing accuracy-obligation machinery, so a quantized-attention-shaped plan is something the optimizer can *propose and price* rather than something a caller must fully specify.

## Why this exists, and what the fired trigger now permits

**Fact.** Quantized profiles exist as compilation *inputs* (`implement-workload-selected-quantized-parameter-maps` and the strict-affine machinery), and `require_elementary_accuracy` assesses per-target obligations before contract resolution. **Inference.** Joint search over (dtype assignment × schedule) is a different capability: the candidate space multiplies, the accuracy obligation becomes a search constraint rather than a precondition, and identity must distinguish plans differing only in precision assignment. The literature to mine when fired: mixed-precision tuning (**Precimonious**, **HiFPTuner**), quantization-search (**HAQ**), and the Sage/SmoothQuant papers as primary sources on what the attention-specific wins actually were. Deferred behind both the conformance corpus (nothing prices an accuracy trade without an oracle to check it) and the cost authority.

## Trigger

`define-the-model-level-conformance-corpus` reaching a usable oracle for permitted-divergence comparison, plus any cost authority landing — both, not either.

## Trigger check log

- 2026-08-05 — **not fired.** The corpus ticket is not `done` and no cost authority exists in `crates/`. Recheck: `grep -m1 '^status:' tickets/define-the-model-level-conformance-corpus.md` and the cost-type grep from the tuning-loop ticket's log.
- 2026-08-09 — **fired.** `define-the-model-level-conformance-corpus`, `implement-analytical-component-cost-model`, and `emit-analytical-costs-through-the-typed-cost-vocabulary` are all `done`. The corpus supplies named permitted-divergence/refusal rows, while the compiler now carries structural and analytical cost authorities with typed cost assessments. Calibration is still future work, but the trigger asked for *any* cost authority, not calibrated device truth; this ticket is therefore `todo` and must decide how uncalibrated versus measured costs constrain the first co-search scope.
