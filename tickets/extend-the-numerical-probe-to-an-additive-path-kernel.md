---
id: extend-the-numerical-probe-to-an-additive-path-kernel
title: Extend the numerical probe to an additive-path kernel
status: todo
priority: p3
dependencies: []
related: [broaden-the-apple-numerical-probe-matrix, check-in-apple-numerical-behaviour-probe]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement]
---
ADR 0076's re-verification records one measurement the checked-in harness does not reproduce: "An emitted `x + 0x00800000` returns `0x00800000` for the operand `0x80400000`, where preserving the operand would produce the subnormal `0x00400000`, confirming input flushing on the additive path." Every kernel in `spikes/apple-targets/numerical_probe.py` that adds does so after a multiply — `Kernel` builds `x * scale` then `+ bias`, and no kernel sets `scale_bits=None` with a `bias_bits` — so nothing isolates an add whose subnormal operand comes straight from the buffer. `docs/decisions/0076-declare-target-honourable-numerical-realizations.md` now names this as the one re-verified observation its evidence record does not re-establish.

This is narrower than `broaden-the-apple-numerical-probe-matrix`, which treats multiply and add as the measured vocabulary and owns widening past it. The gap here is inside the vocabulary already claimed as measured: the add is only ever measured downstream of a multiply, so an additive-path input flush is asserted by the ADR and re-established by nothing.

Add a bias-only kernel (`scale_bits=None`, `bias_bits=0x00800000`) with an execution witness, so the ADR sentence is either reproduced and owned by `docs/research/apple-targets/numerical-behaviour.md` or contradicted and corrected there. `Kernel.source` already emits the add alone when `scale_bits is None`, because its constant loop skips a `None`. The witness needs care: the obvious `3f800000` does **not** work, since `1.0 + 2**-126` rounds back to `3f800000` and an executed add would then be indistinguishable from a deleted one. `00800000` does — `2**-126 + 2**-126` is exactly `01000000` — and it is already in the operand vector, so `Witness(operand=0x00800000, executed=0x01000000, deleted=0x00800000)` is the admissible form. Keep the gate runtime bounded and keep the two-layer guard; an observation without an execution witness stays inadmissible.

If the observation does not reproduce, treat it the way `supersede-the-multiply-by-one-subnormal-claim` treats the `x * 1.0f` row: correct the record and state what the surrounding claim still rests on. The claim that Apple GPU `f32` arithmetic flushes subnormal inputs does not depend on the additive path — findings 2 and 3 carry it on the multiplicative path with witnesses — so a negative result is a correction to one cited row, not to a conclusion.
