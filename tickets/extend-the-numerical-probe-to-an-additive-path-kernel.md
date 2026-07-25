---
id: extend-the-numerical-probe-to-an-additive-path-kernel
title: Extend the numerical probe to an additive-path kernel
status: done
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

## Outcome

**The observation reproduces.** It is now finding 20 in `docs/research/apple-targets/numerical-behaviour.md`, reproduced rather than corrected, so no retraction was needed.

**What landed.** The `add_smallest_normal` kernel — a single `x + as_type<float>(0x00800000u)` with the emitter's NaN canonicalization after it and nothing before it, so the operand reaching the `fadd` is the one the buffer supplied. `Witness(operand=0x00800000, executed=0x01000000, deleted=0x00800000)` exactly as the ticket specified, and `ADDITIVE_INPUT_FLUSH = SubnormalProbe(operand=0x80400000, preserving=0x00400000, flushing=0x00800000)`. Six offline cases per family — three math modes at `-O0` and `-O2` — plus six runtime cases per dispatched family.

One detail differs from the ticket's text and does not change what was built. `broaden-the-apple-numerical-probe-matrix` landed first and replaced `Kernel`'s `scale_bits`/`bias_bits` pair with an ordered `steps` tuple, so the kernel is spelled `steps=(Step(0x00800000, "+"),)` rather than `scale_bits=None, bias_bits=0x00800000`. The property the ticket relied on — that the source emits the add alone — is the same property, now structural rather than a consequence of a loop skipping a `None`.

**What was measured** — Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, offline `metalfe-32023.883`, runtime `metalfe-32023.921` (macOS) and `metalfe-32023.830.1` (iOS Simulator 26.0 build 23A8464). The operand `80400000` returns `00800000` at `-O0` and `-O2`, under `safe`, `relaxed`, and `fast`, on the offline and runtime paths, for macOS and the iOS Simulator; the compile side asserts the `fadd` for all three families including `IOsDevice`. Preserving the operand would give the subnormal `00400000`. Every subnormal operand in the vector returns `00800000` for the same reason.

**Three things the kernel turned out to be good for beyond the ADR sentence.**

- **A flush does not have to show up as a returned zero.** The flushed subnormal here is an addend, not the whole result, so the returned value is normal. Until this probe, every declared `flushing` candidate was a zero and a well-formedness test asserted it; that assertion is gone and each candidate is now derived from its kernel by exact arithmetic under the substitution hypothesis.
- **A witnessed additive observation under the relaxed modes.** Adding a nonzero constant is an identity on no operand, so the `fadd` survives under `relaxed` and `fast` — measured present in all six configurations — where `scale 1.0, bias +0.0` loses its arithmetic entirely (finding 7). The trap kernel can supply no admissible relaxed-mode observation at all; this one can.
- **A third outcome kept distinct.** `00000000` would mean the operand survived the add and the subnormal *result* was flushed instead. That is a different mechanism and classifies as `unexpected-result` rather than being folded into either candidate.

**What is left, and for whom.** ADR 0076 states in two places that this observation is not re-established by the harness. Both are now stale in the direction that matters. The file is `contracts/decisions` and not this ticket's scope, so `retire-adr-0076-additive-path-caveat` carries the repair, with the instruction to record that the gap closed rather than to delete the Fact that it existed.
