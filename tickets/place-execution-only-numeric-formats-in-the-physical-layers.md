---
id: place-execution-only-numeric-formats-in-the-physical-layers
title: Place execution-only numeric formats in the physical layers
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, scope-the-block-scaled-compound-value-vertical, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, target-profiles]
---
## User-visible outcome

TF32, the PTX `.ue4m3` and `.ue8m0` scale encodings, `x86_fp80`, and `ppc_fp128` are statable as physical facts where a backend needs them, and remain unstatable as logical element identities.

## Why this exists

**Fact.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md)'s `### Target ABI and execution-only floating formats` classifies TF32 as a compute or operand precision contract, `.ue4m3` as a target scale-data format distinct from the signed `f8E4M3*` logical elements, `x86_fp80` as a target ABI extended format that is not binary128, and `ppc_fp128` as a double-double that is not binary128 either. Its conclusion 7 keeps them in operation and physical compute contracts "unless an explicit tensor interchange use case proves otherwise".

**Fact.** [The dtype support ledger](../docs/dtype-support.md)'s `### Execution-only formats` records an architectural seam at the numerical contract and type-system reservations at the physical carrier and kernel-vocabulary layers, with recognized identity deliberately `absent/unsupported`.

**Inference.** This track's whole content is *where* such a fact lives, not what the element semantics are, because these formats have no element semantics to give. That is why it is one track over four unrelated formats: they share exactly one obligation, and it is a placement obligation.

## Activation trigger

A selected backend operation needs one of them **and** can state its conversion boundaries, its delivered numerical behaviour, its target detection, its artifact identity, and its refusal. Promoting any of them into logical identity requires a separate semantic decision that this ticket does not pre-authorize.

## Closes when

The trigger has fired and the physical fact is stated at the layer that owns it with its conversion boundary and refusal, and no logical identity was minted.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-11 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- A target-profile or Metal change this eventually needs is that owner's, not this ticket's; declare the scope when the trigger fires rather than reserving it now.

## Trigger check log

- 2026-08-04 — **not fired.** Track D-11's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md): no selected backend operation needs TF32, `.ue4m3`, `.ue8m0`, `x86_fp80`, or `ppc_fp128`, and none could yet state its conversion boundaries, delivered behaviour, target detection, artifact identity, and refusal. **Correction — 2026-08-10.** The original `:217` line locator is historical; at audit D-11 is found by heading `#### D-11 — Execution-only and target ABI formats` (near line 224). The **not fired** verdict is unchanged.
- 2026-08-09 — **not fired.** No selected backend operation, target-profile fact, `KernelType`, `StorageScalar`, or physical carrier names TF32, `.ue4m3`, `.ue8m0`, `x86_fp80`, or `ppc_fp128`. The trigger remains a real selected physical consumer, not the existence of a taxonomy or ledger row. **Correction — 2026-08-10.** The earlier claim that a fresh read found these names *only* in the research taxonomy, the support ledger, and the semantic-catalog negative census was false: ADRs and other docs also name them as deliberate exclusions (e.g. [ADR 0036](../docs/decisions/0036-recognize-standard-binary-and-microscaling-formats.md), [numerical-semantics](../docs/numerical-semantics.md), [dtype-identity-admission-policy](../docs/research/numerics/dtype-identity-admission-policy.md), [dtype-family-research-tracks](../docs/research/numerics/dtype-family-research-tracks.md)), and the catalog negative census covers only `"tf32"`, `"x86_fp80"`, and `"ppc_fp128"` — not `ue4m3` or `ue8m0`. Under `crates/`, those five spellings still appear only in that three-name census (`alias_spellings_and_lookalikes_have_no_authority`).
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **Checkable half, and the anchoring is the whole difficulty.** `rg -n -w -i -e 'tf32' -e 'ue4m3' -e 'ue8m0' -e 'x86_fp80' -e 'ppc_fp128' crates/ --glob '*.rs'` returns **3** lines, all of them the string literals `"tf32"`, `"x86_fp80"`, and `"ppc_fp128"` inside `crates/tiler-ir/src/semantic/catalog/tests.rs`, where a refusal test asserts these names are *not* registered. That is the baseline to subtract; a fourth line elsewhere is the changed answer. **Two traps were live here and both are recorded so the pattern is not casually relaxed.** Without `-w` the same search returns 35 files, because case-insensitive `tf32` matches `StrictF32NumericalContract`. And a `--glob '!**/tests*'` exclusion does **not** make the result empty on this repository, because `#[cfg(test)]` modules are inline — the identical mistake a sibling deferral made with `grep -v tests`. **This condition is not mechanically checkable, and saying so is the repair.** The trigger is a *selected backend operation* that needs one of these formats and can state its conversion boundaries, delivered numerical behaviour, target detection, artifact identity, and refusal. A human must read `docs/research/numerics/mature-dtype-taxonomy.md`'s `### Target ABI and execution-only floating formats` and the selected backend profile. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
