---
id: record-the-closure-of-the-quantized-profile-e-1-measurement-gap
title: Record the closure of the quantized profile's E-1 measurement gap
status: done
priority: p2
dependencies: []
related: [measure-code-domain-integer-arithmetic-on-the-qualified-apple-row, implement-first-quantized-backend-profile]
scopes: [research/numerics]
shared_scopes: []
paths: []
tags: [research, numerics, quantization, measurement]
---
## User-visible outcome

[The first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) stops carrying an open measurement gap that has been closed. Its `Unknown` row about the decode's integer machinery, its E-1 experiment entry, and its delivery table's fourth row are updated to name the retained measurement and the exact boundary it left, so a reader planning the backend work is not told an answered question is open.

## Why this is a separate ticket

[`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) ran E-1 and holds `research/apple-targets`. The profile record lives in `research/numerics`, which that ticket does not hold, so the closure was recorded in [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md) as finding 32 and this remainder was filed rather than edited off-scope or left implicit.

## What to change, and what not to

- The **Unknown** paragraph in "Elimination axis 3" states "Nothing in this repository has measured integer arithmetic on any Apple GPU" and cites `numerical_probe.py:792` for the dtype axis. The first sentence is now false and must be corrected rather than deleted; the second is still true and should stay, because the sibling harness is what measured the decode chain and the numerical probe's dtype axis is unchanged.
- The **E-1** entry under "Bounded experiments this record could not run" should become a run experiment with its retained record path, its verdict, and its stop condition's outcome.
- The **delivery table's** row 4 should record the outcome rather than the intent.
- The **maturity ledger table's** "Target-family dispatchability" row says `absent; no dtype dispatchability axis exists`. Finding 32 is a measured observation of one decode chain and is *not* a dispatchability axis; do not promote the row on the strength of it. Say exactly what was measured.
- Do **not** widen the finding. One family, one GPU, one toolchain row, one flag row, `u8` codes only, one non-overflowing subtraction, no timing, and no packed sub-byte extraction. E-2 is untouched.

## Closes when

Every sentence in the profile record whose truth depended on E-1 being unrun is corrected, the retained record is cited by path, and no claim in it exceeds finding 32's stated boundary.

## Graph maintenance

- Filed by [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) on landing E-1.
- [`implement-first-quantized-backend-profile`](implement-first-quantized-backend-profile.md) reads the profile record as its authority, so this correction should land before it claims device executability.

## Outcome (2026-07-31)

**Fact.** All four named sites in the profile record moved, and only those: the elimination-axis-3 `Unknown` became a dated `Measurement` citing finding 32 and the retained record path, keeping the two still-true halves (the numerical probe's dtype axis is unchanged; the U4 extraction remains undispatched); the E-1 entry became a run experiment with the verdict, the operand-side flush observation, the retained record path, and the boundary; delivery-table row 4 records the outcome; and the dispatchability ledger row states exactly what finding 32 is and is not — a measured decode chain, deliberately not a dispatchability row.

**Fact — the sweep.** `grep -c "has measured integer arithmetic on any Apple GPU" docs/research/numerics/first-quantized-lm-profile.md` returns 0 after the edit; no other sentence in the record conditioned on E-1 being unrun (`grep -n "E-1" docs/research/numerics/first-quantized-lm-profile.md` shows only the corrected sites). No claim exceeds finding 32's boundary; E-2 stays open and still blocks every device-optimal claim.
