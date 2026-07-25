---
id: widen-the-apple-numerical-probe-to-a-second-dtype
title: Widen the Apple numerical probe to a second dtype
status: in-progress
priority: p3
dependencies: []
related: [broaden-the-apple-numerical-probe-matrix, check-in-apple-numerical-behaviour-probe]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement]
claimed_from: todo
assignee: agent-dtype
lease_expires_at: 1785016235
---
`spikes/apple-targets/numerical_probe.py` measures `f32` and only `f32`. `broaden-the-apple-numerical-probe-matrix` widened the operation vocabulary to multiply, add, divide, and a source-level `fma`, and closed the `-fmetal-math-fp32-functions` and optimization-level boundaries, but it deliberately did not widen the *dtype*: `docs/research/apple-targets/numerical-behaviour.md` still records `half` and every other dtype as unmeasured.

The reason it is a separate ticket rather than another row in that matrix is that the dtype is not an axis of the existing case set. It is a change to the harness's shape in three places at once. `OPERANDS` is one `f32` vector whose eight entries are chosen for `f32`'s exponent range, and every result is rendered and compared as eight hex digits. `numerical_probe_host.m` allocates `float` buffers and dispatches one thread per `f32` element. `Kernel.source` declares `device const float *` and `as_type<float>`, and `evaluate` narrows through `struct.pack('<f', ...)`, which is `f32` by construction. A second dtype needs a second operand vector with its own subnormal boundary (`f16`'s smallest normal is `0x0400`, not `0x00800000`), a second result width in the record and in every comparison, and a second dispatch shape in the host.

The question it would answer is worth asking. `MetalSubnormalArithmetic` is declared once as a target fact and nothing in this repository establishes that the flush is dtype-independent; `air.compile.denorms_disable` is a module-level declaration, which is an argument that it should be, and an argument is not a measurement. A `half` row that flushed differently from the `f32` row would mean the declared fact has to carry the dtype.

Keep the two-layer guard intact: a `half` kernel needs its own execution witness in `half`, and the witness must not itself be subnormal in `half`. Keep the gate runtime bounded the way the `f32` matrix now does, with a covering subset and the exhaustive sweep behind `TILER_APPLE_NUMERICS_EXHAUSTIVE`.

A second machine, a second Apple GPU family, an iOS device, and a second toolchain build remain out of reach without hardware and are not in scope here.
