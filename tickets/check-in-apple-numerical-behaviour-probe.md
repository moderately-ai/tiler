---
id: check-in-apple-numerical-behaviour-probe
title: Check in the Apple numerical behaviour probe
status: in-progress
priority: p2
dependencies: []
related: [draft-target-honourable-numerical-contract-adr, prototype-metal-numerical-realization, compile-golden-msl-through-the-aot-driver-in-the-gate]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, measurement]
claimed_from: todo
assignee: agent-check-in-apple-numerical-behaviour-probe
lease_expires_at: 1784917695
---
ADR 0076's fifth open question, made owned. The measurements the whole record rests on exist only in two ticket outcomes and in the ADR's own re-verification. **No research record owns them, no harness is checked in, and nothing in the repository gate re-establishes them.** `AGENTS.md` requires a reproducible experiment to live under `spikes/` and requires research documents to link to the checked-in harness supporting a claim; neither holds here.

The ADR's `evidence` frontmatter is currently forced to cite the design records plus the Apple compatibility probe *for its own disclaimer* — that probe explicitly states it "did not observe the numerical behavior these flags request" — rather than any record about the numerical behaviour itself. That is honest and it is a gap.

## What was measured, by a hand-built harness that is not checked in

Environment: Apple M4 Max, macOS 27.0 build 26A5388g, `Apple metal version 32023.883 (metalfe-32023.883)`, macOS SDK 26.5 build 25F70, `-target air64-apple-macos13.0 -std=metal3.1`. Offline compilation through `xcrun metal` then `xcrun metallib`; dispatch through a hand-written Objective-C host using `MTLCreateSystemDefaultDevice`, `newLibraryWithURL:`, one thread per output, `MTLResourceStorageModeShared`, bit patterns in and out.

The findings a harness must reproduce:

- **`safe` still disables denormals.** `air.compile.denorms_disable` is emitted under `safe`, `relaxed`, and `fast` alike; under `safe` it appears alongside `air.compile.fast_math_disable` and no emitted `fmul`/`fadd` carries a fast-math flag.
- **Input and result flushing are separable, and both occur.** `x * 2.0f` on operand `0x00400000` returns `0x00000000` where preserving the operand gives the *normal* `0x00800000` — isolating input flush. `x * 0.5f` on the normal operand `0x00800000` returns `0x00000000` where the correct result is subnormal — isolating result flush. Both hold at `-O0` and `-O2`, under `safe` and `fast`.
- **The flush preserves the sign of zero.** `0x80400000 * 2.0f` returns `0x80000000`. ADR 0076 item 1 makes this load-bearing: a flush behaviour that does not state which zero it produces is under-specified against measured hardware.
- **Materialization is unaffected.** A load-then-store kernel with no arithmetic returns `00000001`, `00400000`, `007fffff`, `80400000` unchanged under every mode.
- **Math mode changes a conforming result.** For `MultiplyThenAdd { scale 1.0, bias +0.0 }`, operand `0x80000000` returns `0x00000000` under `safe` and `0x80000000` under `relaxed` and `fast`. IEEE-754 round-to-nearest requires the former.
- **Contraction changes a conforming result.** A multiply and an add as two statements over `scale = 1.5`, `bias = 1.0` return `0x3fc58f9e` for operand `0x3eb97ef9` under `-ffp-contract=off` and `=on`, and the fused `0x3fc58f9d` under `=fast`.
- **The trap.** A relaxed mode can *appear* to honour a strict contract by deleting the arithmetic. `x * 1.0` folds to a copy under every mode; the `scale 1.0, bias +0.0` kernel retains exactly one floating-point operation under `safe` — the `+0.0` fadd, unremovable without `nsz` — and zero under `relaxed`. The surviving `fadd` is what flushes. A probe that concluded "subnormals preserved" from that kernel under `relaxed` would infer the wrong fact, and this is the single most important behaviour for a checked-in harness to encode as a guard rather than merely reproduce.

## Two routes; pick one and say why

Either a `spikes/apple-targets/` (or `spikes/numerics/`) harness owns a checked-in subnormal and math-mode probe with a research record under `docs/research/apple-targets/`, or these measurements are re-run through the harness `compile-golden-msl-through-the-aot-driver-in-the-gate` established in `crates/tiler-metal/src/golden_compilation.rs`. The second is cheaper for the compile-side facts and does not reach the on-device ones — `golden_compilation` compiles and links but never dispatches, so it cannot observe a returned bit pattern. That asymmetry is probably decisive, but check it rather than inheriting the judgement.

Whichever route, the harness must self-skip where no qualified Apple toolchain or device resolves, following the existing `DriverError::{ToolchainUnavailable, SdkUnavailable}` classification, so it cannot make the gate red on a host without Xcode. Keep the exact environment row with the results; these are host-qualified measurements, not portable guarantees about Metal.

Once a record exists, update ADR 0076's `evidence` frontmatter to cite it and close that open question in the record.
