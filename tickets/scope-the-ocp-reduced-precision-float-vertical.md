---
id: scope-the-ocp-reduced-precision-float-vertical
title: Scope the OCP reduced-precision float vertical
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, acquire-and-classify-the-two-ocp-dtype-specifications, scope-the-block-scaled-compound-value-vertical, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, fp8, ocp]
---
## User-visible outcome

The six registered OCP formats — E4M3FN, E5M2, E2M3FN, E3M2FN, E2M1FN, and E8M0FNU scale data — have one owner for the step past identity, and a reader can tell which FP8 spellings that owner does **not** cover.

## Why this exists

**Fact.** [The dtype support ledger](../docs/dtype-support.md) records all six registered with their sign, exponent, and significand widths, their exponent bias as the vendored MLIR record states it, and an explicit Boolean for every special member. Nothing beyond identity is registered: no operation signature admits one.

**Fact — their exceptional-value obligations are member-specific and a shared assumption would be wrong.** E4M3FN has NaN encodings and no infinity; the OCP FP4 and FP6 formats have neither NaN nor infinity; E8M0FNU is unsigned exponent-only scale data with NaN and no zero, sign, infinity, or subnormals. [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md) states that suffixes "are naming conventions, not a universally compositional grammar".

**Fact — five FP8 spellings the taxonomy catalogs are outside this track.** `f8E3M4`, `f8E4M3`, `f8E4M3FNUZ`, `f8E5M2FNUZ`, and `f8E4M3B11FNUZ` are classified "Recognized external owner-namespaced candidates" by [the admission policy](../docs/research/numerics/dtype-identity-admission-policy.md), so they belong to [`govern-external-dtype-namespace-registration-and-equivalence`](govern-external-dtype-namespace-registration-and-equivalence.md). The ledger's own asserted catalog size confirms none of them is registered: 27 nominal scalars is exactly `bool` plus twelve integer widths plus four IEEE binary formats plus BF16 plus these six plus three decimal formats.

**Fact — the normative bytes are metadata-only.** Both OCP specifications were acquired by hand on 2026-07-31, licence-reviewed document by document, and digested, and the bytes were discarded because neither carries a self-contained redistribution grant. Re-deriving a pinned value set requires re-acquiring through the recorded route and checking against the recorded digest. That constrains rung 2 and does not block rung 1, which [ADR 0036](../docs/decisions/0036-recognize-standard-binary-and-microscaling-formats.md) already discharged.

## Activation trigger

A selected model or kernel names the exact format, its operations, its conversion and accumulation policy, its physical representation, its runtime refusal rules, its target dispatchability, and its conformance corpus.

## Explicit non-goals

- MX and other block-scaled schemes. An element identity never implies the compound scheme; that is [`scope-the-block-scaled-compound-value-vertical`](scope-the-block-scaled-compound-value-vertical.md)'s.
- FP4 and FP6 packing, which is [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md)'s.

## Closes when

The trigger has fired and the selected format's nine obligations are stated together, **or** the formats are explicitly excluded from the intended product surface by a recorded decision.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-5 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).

## Trigger check log

- 2026-08-04 — **not fired.** Track D-5's trigger under `#### D-5 — OCP reduced-precision floats and E8M0 scale data` in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md) records `**It has not fired.**`: no selected model or kernel names an exact OCP format with its operations, conversion and accumulation policy, physical representation, refusal rules, dispatchability, and corpus.
- 2026-08-09 — **not fired.** The six OCP identities remain catalog-only. The selected quantized workload names strict-affine U8/F32 rather than one of these formats, and no kernel supplies the complete operation, conversion/accumulation, carrier, refusal, target, and corpus bundle.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. Same two halves. **Checkable half.** `rg -n 'pub fn \w+_op\(\) -> OpKey' crates/tiler-ir/src/semantic --glob '*.rs'` reports the **19** registered operation-key constructors, and `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` reports **50 unique governed keys** — unique keys through `sort -u`, not lines of output; all six OCP identities are present as catalog entries — `tiler::f4e2m1fn@1`, `tiler::f6e2m3fn@1`, `tiler::f6e3m2fn@1`, `tiler::f8e4m3fn@1`, `tiler::f8e5m2@1`, `tiler::f8e8m0fnu@1`, beside `tiler::mxint8@1` — and none of the 19 operation constructors admits one, which is the entry's "nothing beyond identity is registered" claim checked rather than restated. An operation key naming one of those formats is the changed answer, and the check is one-directional in the same way. **This condition is not mechanically checkable, and saying so is the repair.** The other half — a selected model or kernel naming the format together with its operations, conversion and accumulation policy, carrier, refusal rules, target dispatchability, and conformance corpus — is a selection recorded in a workload profile, not a code state. A human must read `docs/dtype-support.md`'s trigger row for these formats. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
