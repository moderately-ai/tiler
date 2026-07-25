---
id: broaden-the-apple-numerical-probe-matrix
title: Broaden the Apple numerical probe matrix
status: done
priority: p3
dependencies: []
related: [check-in-apple-numerical-behaviour-probe]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement]
---
`spikes/apple-targets/numerical_probe.py` measures one deliberately narrow row so it can run in the repository gate in about eight seconds. `docs/research/apple-targets/numerical-behaviour.md` records the boundaries; this ticket owns closing the ones that are cheap and could change a conclusion.

In rough order of value. `-fmetal-math-fp32-functions` is pinned to `precise`; `prototype-metal-numerical-realization` reported the signed-zero divergence also reproducing under `=fast`, which is unverified here. `-O1`, `-O3`, and `-Os` are unmeasured, and the `-O0`/`-O2` difference in how much arithmetic survives into the emitted IR shows the level is not inert. The operation vocabulary is multiply and add only: division, `half`, a source-level `fma`, and any reduction shape are unmeasured, and a reduction is the shape where reassociation would show. Reduction reassociation was probed over three fixtures in `prototype-metal-numerical-realization` and found no counterexample; that bounded negative result is not reproduced by the checked-in harness.

Every addition must keep the execution-witness guard: a kernel whose result cannot distinguish executed arithmetic from deleted arithmetic must declare `witness = None` and is inadmissible, not merely noisy. Keep the gate runtime bounded; if the matrix grows past a few seconds, split the exhaustive sweep behind an environment switch and keep a covering subset in the gate.

A second machine, a second Apple GPU family, an iOS device, and a second toolchain build remain out of reach without hardware and are not in scope here.

## Outcome

**What landed.** `spikes/apple-targets/numerical_probe.py` moved to record schema `tiler.apple-numerical-behaviour/v4`. `-fmetal-math-fp32-functions` stopped being a fixed flag and became an axis swept on both compilation paths; `-O1`, `-O3`, and `-Os` joined `-O0` and `-O2`; and four kernels joined the vocabulary — two power-of-two divisions, two divisions the driver keeps as `fdiv`, a source-level `fma` over the contraction pair's own constants, and a two-add chain whose value says where the parentheses went. `Kernel` now carries an ordered `steps` tuple instead of a `scale_bits`/`bias_bits` pair, which is what lets a kernel be two adds or one division. A case key names its fp32-functions value only when it departs from the pinned `precise`, so every key the schema `v3` record and the research memo cite keeps its exact meaning.

**What was measured** — Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, offline `metalfe-32023.883`, runtime `metalfe-32023.921` (macOS) and `metalfe-32023.830.1` (iOS Simulator 26.0 build 23A8464), all three families compiled, macOS and the simulator dispatched. Five new findings, recorded as 15 to 19 in `docs/research/apple-targets/numerical-behaviour.md`.

- **15.** `x / 2.0f` and `x / 0.5f` are emitted as a single `fmul` even under `-fmetal-math-mode=safe -ffp-contract=off`, so a power-of-two divisor measures the multiplier and not division. On divisors the rewrite cannot absorb, a surviving `fdiv` flushes its subnormal input (`x / 0.375f` returns `00000000` for `00400000`, whose exact quotient is the normal `00aaaaab`) and its subnormal result (`x / 3.0f` returns `00000000` for the normal `00800000`), sign-preservingly. Under `relaxed` and `fast` `arcp` substitutes a reciprocal multiply and, for these two divisors, changed no returned value.
- **16.** A source-level `fma` returns the fused `3fc58f9d` at every `-ffp-contract` setting including `off`, where the identical constants written as two statements return `3fc58f9e` at `off` and `on`. `-ffp-contract=off` is a defence against contraction, not against a fusion the source asked for.
- **17.** `(x + 2**-24) + 2**-24` returns `3f800000` for `x = 1.0` under `safe`, keeping two `fadd`s, and `3f800001` under `relaxed` and `fast`, keeping one carrying `reassoc`, on both compilation paths. **Reassociation is observable on this row**, which bounds the three-fixture negative result `prototype-metal-numerical-realization` recorded rather than contradicting it.
- **18.** `-fmetal-math-fp32-functions=fast` changes no emitted module and no returned bit pattern for multiply, add, or the `MultiplyThenAdd` shape, and the signed-zero divergence reproduces under it identically — which re-establishes the one claim `prototype-metal-numerical-realization` made that the pinned flag had left unverified. It is no evidence about the functions the flag actually governs, none of which this matrix calls.
- **19.** `-O1`, `-O3`, and `-Os` are identical to `-O2` in emitted operation count for four kernels across three math modes, and no returned value differs at any of the five levels. `-O0` is the sole outlier, which confines finding 7's refinement to one level instead of to "low optimization".

**A harness defect the widening exposed, and its correction.** This front end lowers `fma(x, a, b)` to `@air.fma.f32`; `FUSED_INTRINSIC` named only the `llvm.` spellings, so a kernel whose entire body is one fused multiply-add was reported as emitting **zero** floating-point operations. The verdict still failed closed, but the count was wrong in the direction a reader acts on, since a surviving operation reported as zero is indistinguishable from a deleted one — the reading finding 7 rests on. Both spellings are matched now and a gate assertion pins the fused kernel's operation list.

**What the guard gained.** `inadmissible` is now the shared two-layer guard and `subnormal_verdict` and the new `order_verdict` are thin classifiers over it, so an evaluation-order claim is refused for exactly the reasons a subnormal claim is. `evaluate` derives a kernel's exact result under both flush hypotheses, and a portable guard test holds every declared `SubnormalProbe`, `OrderProbe`, and `Witness` to that derivation — including that a witness must give the same value whether or not subnormals flush, which the previous "no witness value may be subnormal" check did not catch for a subnormal *intermediate*.

**How the gate runtime was bounded.** `cases` assembles a `covering` set and an `exhaustive` one, selected by `TILER_APPLE_NUMERICS_EXHAUSTIVE`. The covering set keeps at least one case of every kernel, math mode, optimization level, contraction setting, and fp32-functions value, and a portable test fails if it stops doing so. Two records are retained, `probe.matrix` names which produced each, and `matrix_mismatch` refuses to compare one against a run of the other. `pytest spikes/apple-targets` went from about 20 s to about 47 s on a host concurrently running several other gates; the exhaustive sweep adds 99 offline cases and about 5 s to a probe run.

**What was decided, and what was left.** The `half` axis is deliberately not folded in: it is a change to the operand vector, the result width, and the dispatch host at once rather than another row in the matrix, and it is now `widen-the-apple-numerical-probe-to-a-second-dtype`. That ticket also carries the question findings 1 to 19 cannot answer — whether the subnormal flush is dtype-independent, which `air.compile.denorms_disable` argues for and nothing measures. A multi-element reduction over a buffer, threadgroup, or subgroup also remains unmeasured; finding 17 exposes the reassociation licence within one thread, which is the licence a reduction would be subject to, and is not itself a reduction.
