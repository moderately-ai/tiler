---
id: broaden-the-apple-numerical-probe-matrix
title: Broaden the Apple numerical probe matrix
status: in-progress
priority: p3
dependencies: []
related: [check-in-apple-numerical-behaviour-probe]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement]
claimed_from: todo
assignee: agent-apple
lease_expires_at: 1784996387
---
`spikes/apple-targets/numerical_probe.py` measures one deliberately narrow row so it can run in the repository gate in about eight seconds. `docs/research/apple-targets/numerical-behaviour.md` records the boundaries; this ticket owns closing the ones that are cheap and could change a conclusion.

In rough order of value. `-fmetal-math-fp32-functions` is pinned to `precise`; `prototype-metal-numerical-realization` reported the signed-zero divergence also reproducing under `=fast`, which is unverified here. `-O1`, `-O3`, and `-Os` are unmeasured, and the `-O0`/`-O2` difference in how much arithmetic survives into the emitted IR shows the level is not inert. The operation vocabulary is multiply and add only: division, `half`, a source-level `fma`, and any reduction shape are unmeasured, and a reduction is the shape where reassociation would show. Reduction reassociation was probed over three fixtures in `prototype-metal-numerical-realization` and found no counterexample; that bounded negative result is not reproduced by the checked-in harness.

Every addition must keep the execution-witness guard: a kernel whose result cannot distinguish executed arithmetic from deleted arithmetic must declare `witness = None` and is inadmissible, not merely noisy. Keep the gate runtime bounded; if the matrix grows past a few seconds, split the exhaustive sweep behind an environment switch and keep a covering subset in the gate.

A second machine, a second Apple GPU family, an iOS device, and a second toolchain build remain out of reach without hardware and are not in scope here.
