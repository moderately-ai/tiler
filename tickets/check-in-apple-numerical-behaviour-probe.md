---
id: check-in-apple-numerical-behaviour-probe
title: Check in the Apple numerical behaviour probe
status: in-progress
priority: p2
dependencies: []
related: [draft-target-honourable-numerical-contract-adr, prototype-metal-numerical-realization, compile-golden-msl-through-the-aot-driver-in-the-gate]
scopes: [research/apple-targets]
shared_scopes: [project/tickets, contracts/navigation]
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

## Outcome

### Route

**Fact — the spike route, and the reason is not the one the ticket suggested first.** A `spikes/apple-targets/` harness with a research record under `docs/research/apple-targets/`. The ticket suspected the asymmetry was decisive and it is, but check it rather than inherit it: four of the seven findings are returned bit patterns, and `crates/tiler-metal/src/golden_compilation.rs` compiles and links and never creates a device, a command queue, or a command buffer. Reaching them from there would mean adding a Metal runtime dependency to a crate whose whole point is that it emits source and stays out of a consumer's device graph. Two further reasons compound it: `golden_compilation.rs` is `implementation/metal`, which `choose-one-owner-for-apple-target-vocabulary` currently holds, and the seven findings need a math-mode and contraction matrix that would have to be bolted onto a module whose subject is the four golden fixtures under one governed realization. `implementation/metal` was therefore not added. `contracts/navigation` was added as a **shared** scope, because a new research record regenerates the catalog blocks in `docs/research/README.md` and `spikes/README.md`; that follows the precedent of every other record-adding ticket here and the declared-overlap warning against `own-operation-family-support-matrix` is the expected non-failing kind.

### Where it lives

`spikes/apple-targets/numerical_probe.py` generates the probe kernels in the emitter's output shape, compiles each to LLVM IR, AIR, and a linked metallib across the matrix, dispatches through `spikes/apple-targets/numerical_probe_host.m`, and classifies. `spikes/apple-targets/test_numerical_probe.py` holds the assertions and is collected by the gate through `pytest`'s existing `spikes/apple-targets` test path, so no change to `pyproject.toml` or `scripts/check_repository.py` was needed. `spikes/apple-targets/results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv` is the retained measurement with its environment row. `docs/research/apple-targets/numerical-behaviour.md` is the research record. `spikes/apple-targets/.gitignore` ignores `/local-work/`, the optional `--work-dir` output; the retained record is not ignored.

**Measurement — the gate ran it here, it did not skip.** `uv run --locked python scripts/check_repository.py` collected 170 Python tests, 23 of them from `test_numerical_probe.py`, all passed, none skipped, in 15.9 s total; the probe itself takes about 8 s. Environment: Apple M4 Max, macOS 27.0 build 26A5388g, arm64, Xcode 26.6 build 17F113, SDK `macosx` 26.5 build 25F70, `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883`.

### The seven findings, all reproduced

1. **`safe` still disables denormals.** Reproduced. `air.compile.denorms_disable` under `safe`, `relaxed`, and `fast`, at all three `-ffp-contract` settings; under `safe` alongside `air.compile.fast_math_disable` with no fast-math flag on any `fmul`/`fadd` at `off`/`on`.
2. **Input and result flushing are separable and both occur.** Reproduced, and widened. `x * 2.0f` on `00400000` returns `00000000` where the exact result is the normal `00800000`; `x * 0.5f` on the normal `00800000` returns `00000000` where the exact result is the subnormal `00400000`. Both at `-O0` and `-O2` and under all three modes, not only `safe` and `fast`; `relaxed` was added because both kernels carry execution witnesses and could close the boundary rather than record it.
3. **The flush preserves the sign of zero.** Reproduced. `0x80400000 * 2.0f` returns `80000000` in all six configurations.
4. **Materialization is unaffected.** Reproduced. `00000001 00400000 007fffff 00800000 80400000 80000000 3eb97ef9 3f800000` returned unchanged under every mode, with zero floating-point operations emitted.
5. **Math mode changes a conforming result.** Reproduced. `scale 1.0, bias +0.0` on `80000000` returns `00000000` under `safe`, `80000000` under `relaxed` and `fast`, at both `-O0` and `-O2`.
6. **Contraction changes a conforming result.** Reproduced exactly. `3eb97ef9` returns `3fc58f9e` under `-ffp-contract=off` and `=on`, `3fc58f9d` under `=fast`. The canonicalized control also reproduces the caveat: no fusion is observed through the canonicalization at any setting, and the same source without it does fuse, so the canonicalization is not a barrier.
7. **The trap.** Reproduced and sharpened; see below.

### Three disagreements with the recorded values

**Measurement — `x * 1.0f` does not flush, and `prototype-metal-numerical-realization` says it does.** That ticket records "an emitted `x * 1.0f` returns `0x00000000` for the operand `0x00000001`". Measured here it returns `00000001` unchanged, under `safe`, `relaxed`, and `fast`, at `-O0` and `-O2`. ADR 0076's re-verification already said the opposite of the ticket ("`x * 1.0` is folded to a copy under every mode"), so this confirms the ADR against the ticket it cites. The surrounding claim survives on the other two rows, which reproduce with execution witnesses; the `x * 1.0f` row is the one that proves nothing, and it is precisely the shape the ADR later names as the trap. Filed `supersede-the-multiply-by-one-subnormal-claim`.

**Measurement — the fast-math flag spellings ADR 0076 records hold only at `-ffp-contract=fast`.** The ADR states "under `relaxed` each carries `reassoc nsz arcp contract afn`; under `fast` each carries `fast`" without naming a contraction setting. At `off` and `on` they are `reassoc nsz arcp afn` and `reassoc nnan ninf nsz arcp afn`. No conclusion depends on the spelling; the full nine-cell table is in the research record so a future reader does not see a spurious difference.

**Measurement — counting operations in the emitted LLVM IR is necessary and not sufficient.** ADR 0076 says "counting floating-point operations in the emitted LLVM IR explains it". At `-O2` it does. At `-O0` the `scale 1.0, bias +0.0` kernel under `relaxed` and under `fast` retains **two** floating-point operations in the front end's IR and the GPU still returns every operand unchanged, negative zero included: a stage below the emitted IR — the AIR-to-ISA compilation at pipeline-state creation — removed them. This is the most consequential thing measured here, and it strengthens rather than weakens the ADR: it makes the case that honourability must be a stated target fact harder to argue against, because even reading the compiler's own IR is not enough.

### How the trap is encoded as a guard

**Fact — two layers, and neither is optional.** `subnormal_verdict` returns `preserved` or `flushed-to-zero` only when the emitted module retains at least one floating-point arithmetic instruction (`fcmp` deliberately excluded, so a NaN test cannot stand in for a surviving multiply) **and** the same kernel in the same configuration returns its declared **execution witness**: a non-subnormal operand whose result differs from the operand exactly when the arithmetic ran. Every other outcome is a named refusal — `no-emitted-arithmetic`, `arithmetic-not-executed`, `no-execution-witness`, `witness-disagrees`, `unexpected-result` — so an inadmissible observation cannot degrade into a boolean. A kernel that is an identity on every operand has no possible witness and declares `witness = None`; `multiply_one` is such a kernel and nothing it returns is ever evidence.

**Fact — the guard is tested where a GPU is not needed.** Six of the twenty-three tests are pure functions over synthetic observations and run on a host with no Apple toolchain: a witness must separate execution from deletion and none of its values may be subnormal; zero emitted operations is never evidence; a witness showing deletion is never evidence *while the unguarded reading still says "preserved"*, which is asserted so the test cannot stop exercising the trap; a witnessless kernel is never evidence; and a witnessed surviving operation **is** admitted, so the guard cannot pass by refusing everything. On a host with a GPU, `test_a_deleted_operation_never_reads_as_preservation…` asserts the naive and guarded readings actually disagree on this row, in all four relaxed configurations, and that `safe` is still admitted.

**Fact — the retained record is compared, and the comparison has teeth.** A live run's case rows must equal the retained record's whenever the environment row matches; when it differs the test announces the difference and skips, because a different toolchain build legitimately produces different values. The same test then corrupts one retained row and asserts the comparison rejects it, so the mechanism cannot rot into a no-op. This is the direct answer to the `metallib` byte-count precedent.

### Self-skip

**Fact.** `Reason::{TOOLCHAIN, SDK, DEVICE}` mirrors `DriverError::{ToolchainUnavailable, SdkUnavailable}` and adds the one axis that classification has no name for, because `golden_compilation` never dispatches: a host with a Metal compiler and no usable GPU. `ProbeUnavailable` is a skip; `ProbeFailure` is a defect and is raised for everything that goes wrong after the tools resolve. The classification is itself tested — an empty `PATH` classifies as `TOOLCHAIN`, a host exiting 3 as `DEVICE`, a host exiting 4 as a `ProbeFailure`, a truncated result list as a `ProbeFailure`. Skips print on standard error and appear in `pytest -ra`. `TILER_REQUIRE_METAL_TOOLCHAIN`, the same variable `golden_compilation.rs` reads, turns a skip into a failure. **Measurement — both verified by hand:** with `PATH` pointing at an empty directory, `12 skipped, 7 passed, exit 0` and the reason `toolchain-unavailable: xcrun is not on PATH` in the `-ra` summary; with `TILER_REQUIRE_METAL_TOOLCHAIN=1` additionally set, `12 failed, exit 1`.

### What ADR 0076's evidence should become

**Proposal.** `evidence: ["tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.operation-conformance-matrix", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.apple-targets.compatibility"]`. The compatibility probe stays, because the ADR still cites it for the flag-acceptance row and for its own disclaimer and the Traceability prose already scopes that correctly. The Traceability "Measured evidence" line should name the research record and the harness rather than a ticket outcome, and the fifth open question is answered and should be removed. That file is `contracts/decisions` and was not touched; `repoint-adr-0076-evidence-at-the-numerical-record` carries the exact edit.

### Follow-ups filed

`repoint-adr-0076-evidence-at-the-numerical-record` (p1, `contracts/decisions`); `probe-metal-runtime-compilation-numerics` (p2, `research/apple-targets`) for the `MTLCompileOptions.mathMode` path the harness does not exercise; `supersede-the-multiply-by-one-subnormal-claim` (p3) for the disagreement above; `broaden-the-apple-numerical-probe-matrix` (p3) for `-fmetal-math-fp32-functions=fast`, the unmeasured optimization levels, and the operation vocabulary beyond multiply and add.

### Measurement boundaries

One machine, one GPU, one macOS build, one toolchain build, one SDK, one target triple, one MSL version. `-fmetal-math-fp32-functions` is pinned to `precise`. `-O1`, `-O3`, and `-Os` are unmeasured. Every compilation is offline; the runtime `newLibraryWithSource:options:` path is not re-established. The operation vocabulary is multiply and add; division, `half`, a source-level `fma`, and every reduction shape are unmeasured, and the three-fixture reduction-reassociation negative result from `prototype-metal-numerical-realization` is not reproduced. Nothing here is evidence about another Apple GPU family, an iOS artifact, Catalyst, or any non-Apple Metal implementation.
