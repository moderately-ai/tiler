# Contraction-pragma runtime probe

This bounded experiment asks one question the [numerical-behaviour probe](../README.md#numerical-behaviour-probe) is structurally unable to ask: **does `#pragma METAL fp contract(off)` survive `newLibraryWithSource` to the Metal runtime compiler?**

[Finding 30](../../../docs/research/apple-targets/numerical-behaviour.md#30-the-runtime-compiler-contracts-under-relaxed-and-fast-at-every-width-whatever-the-offline-selection-says) measures the runtime compiler contracting a written multiply/add pair under `mathMode = Relaxed` and `Fast` at all three widths, and [finding 10](../../../docs/research/apple-targets/numerical-behaviour.md#10-mtlcompileoptions-exposes-a-different-surface-from-the-offline-flag-set) records that `MTLCompileOptions` exposes no contraction property to turn that off with. Finding 10's last paragraph records the pragma as an available mechanism — accepted offline without diagnostic, and removing the `contract` fast-math flag from the emitted IR — and records equally deliberately that the numerical probe did **not** use it, because changing the source bytes would have destroyed the byte-identical offline/runtime pairing that whole comparison rests on. So the pragma's effect on the runtime path was unmeasured rather than known to be absent.

It matters which. If the pragma is a defence, a BF16 program can be given an unfused guarantee on this row by *emitting different source*, which is [ADR 0076](../../../docs/decisions/0076-declare-target-honourable-numerical-realizations.md)'s `SupportedWithExactEmulation` shape — honoured by emitting different operations rather than by setting a compile option. If it is not, [`docs/numerical-semantics.md`](../../../docs/numerical-semantics.md)'s refusal stands as written.

## Why it is a sibling rather than an axis on the numerical probe

The numerical probe's contraction comparison *is* byte-identical source through two compilers. A pragma variant is by construction not byte-identical source — it is the one perturbation that comparison forbids. Folding it in would either break the pairing or force a second source per case that the pairing then has to except, and it would move `probe.harness_sha256` in all four retained numerical records for a question none of them asks. [`aot-runtime-compiler-observer`](../aot-runtime-compiler-observer/README.md) and [`code-domain-integer-decode`](../code-domain-integer-decode/README.md) are the precedents for a sibling that shares this host row, the dispatch host, and nothing else.

Sharing the dispatch host is deliberate and is the reason a difference here is a difference between two sources: `numerical_probe_host.m` is reused unmodified, so the pragma and control libraries take the identical path to the GPU — same pipeline creation, same shared buffers, same `MTLCommandBufferStatusCompleted` check before readback, same sentinel-seeded output buffer. Modifying it would have moved `probe.host_source_sha256` in every retained numerical record, which is exactly the cost this directory exists to avoid paying.

## The changed bytes, stated exactly

One line, **31 bytes including its newline**, inserted immediately after the single `using namespace metal;` line. Every other byte of the control source is unchanged, at all three widths:

```diff
 #include <metal_stdlib>
 using namespace metal;
+#pragma METAL fp contract(off)
```

The line after the insertion point is the source's existing blank line, unchanged; it is elided above only because a diff's leading space on a blank context line is trailing whitespace `git diff --check` rejects.

The control is not a re-rendering of the kernel — it is `numerical_probe.Kernel.source()` for the very kernels the retained records measure, so `sources/contraction_pair.control.metal` is byte-identical to the retained numerical-probe `sources/contraction_pair.metal` finding 30 was taken from (both `5b4a39ca…`; the check is one `shasum -a 256` over the two paths). The pragma source is that string with one `str.replace` on the anchor, refused unless the anchor appears exactly once.

**File scope is a choice with a reason.** `#pragma METAL fp` and `#pragma clang fp` are accepted at file scope or at the start of a compound statement. The generated kernels open a nested `if` block, so a block-scope placement would sit inside the guarded region and would be a second variable between the two sources rather than one. The offline companion compilation below is what establishes that this placement is live rather than silently inert.

## The control is the failure proof

A run in which the *unperturbed* neighbour does not return the fused value establishes nothing about the pragma: whatever suppressed the fusion there is not the pragma, because the pragma is not in those bytes. Every cell therefore dispatches control and pragma **in one host invocation**, on one device and one queue, control first; `require_controls` refuses the whole run and publishes nothing unless all twelve controls return `fused` with an `executed` witness.

That refusal was watched firing rather than assumed. `--perturb-control` applies the pragma to the control too:

```sh
uv run python pragma_probe.py --result-dir /tmp/perturbed --perturb-control
```

reports `12 of 12 controls did not fuse, so this run establishes nothing about the pragma`, exits 1, and creates no result directory. The check names and counts its population, so a run in which the loop silently did not execute is distinguishable from a run in which every cell passed.

## The guard layers, and the one that is missing here

`newLibraryWithSource:options:` returns an opaque `MTLLibrary`, so the numerical probe's first guard layer — reading the emitted module — is unavailable on this path exactly as it is there. Two things replace it.

**The execution witness, on every case.** Finding 28's witness operand is `1.0` at each width, and its result is contraction-independent by construction (`401f` at `bf16`, `4101` at `f16`, `40200000` at `f32`), so it reports that the arithmetic ran without reporting anything about fusion. A case whose witness does not report `executed` is recorded `inadmissible-<status>` and can support no conclusion, which is the same rule `numerical_probe.subnormal_verdict` applies.

**An offline companion compilation, which is not evidence about the runtime compiler.** Both sources are also compiled offline at `-O2 -ffp-contract=fast -Wall -Werror -S -emit-llvm`, under both math modes, and the emitted floating-point flags are recorded. This says whether the pragma is live *at all* in this exact spelling and placement on this row; without it, a negative runtime result would be indistinguishable from a misplaced pragma. It is recorded under `offline.*` and never under `case.*`, and `probe.offline_companion_scope` says so in the record.

**One channel this probe cannot read.** The dispatch host consults `newLibraryWithSource`'s `NSError` only when the returned library is nil, so a runtime compilation that succeeded *with* a warning is indistinguishable here from one that succeeded silently. What is established on the runtime path is that compilation succeeded and what the kernel then computed — not that the pragma was accepted without diagnostic. `probe.runtime_diagnostic_channel` carries that boundary in the record.

## The discriminating constants are reused, not re-derived

Finding 28 records that the obvious `x * 1.5 + 1.0` spelling discriminates on **no** operand of either narrow vector: such a kernel returns byte-identical results under every contraction setting while proving nothing, and its execution witness still reports `executed`. The scales are therefore `0x3FBE` at `bf16` and `0x3E02` at `f16`, each one ulp from 1.5 and each the nearest value that discriminates at its vector's ordinary normal. This probe reads them out of `numerical_probe.BY_NAME` rather than restating them.

The two candidate results per operand are **derived** by `numerical_probe.evaluate` — the unfused candidate by rounding each statement, the fused one by evaluating `x*a + b` as an exact rational and rounding once — and the discriminating operand is *found* as the one lane where they disagree rather than assumed to be a particular index. A kernel whose candidates agreed everywhere would refuse the run rather than produce eight lanes of agreement and a confident wrong conclusion. The derivation reproduces finding 28's and finding 30's published values exactly:

| kernel | scale | discriminating operand | unfused | fused | witness |
| --- | --- | --- | --- | --- | --- |
| `contraction_pair_bf16` | `3fbe` | `3eab` | `3fc0` | `3fbf` | `3f80 → 401f` |
| `contraction_pair` | `3fc00000` | `3eb97ef9` | `3fc58f9e` | `3fc58f9d` | `3f800000 → 40200000` |
| `contraction_pair_f16` | `3e02` | `3555` | `3e00` | `3e01` | `3c00 → 4101` |

## Running it

On a macOS host with the Apple Metal toolchain, from this directory:

```sh
uv run python pragma_probe.py \
  --result-dir results/<yyyy-mm-dd>-contraction-pragma-macos-msl31-<toolchain>
```

`--work-dir` keeps the generated sources, emitted IR, and per-cell manifests for inspection; without it they live in a temporary directory the harness removes. `--perturb-control` is the failure demonstration above. A missing toolchain or SDK exits 1 with the reason on stderr; so does an absent Metal device, a refused offline compilation, a control that did not fuse, and a record row carrying a tab or newline. The producer stages the record, its retained sources, and their manifest into a sibling directory and renames it into place only after every check passes, so a refusal publishes nothing.

Nothing runs this for you. No `make` target reaches `spikes/`, the dispatch is hand-run, and a toolchain change that moved a measured value would not fail any gate — only re-running this probe detects that drift.

## Result on 2026-08-02: the pragma **is** a defence on this row

The retained record is [`results/2026-08-02-contraction-pragma-macos-msl31-xcode26.6-metal32023.883/record.tsv`](results/2026-08-02-contraction-pragma-macos-msl31-xcode26.6-metal32023.883/record.tsv), schema `tiler.apple-contraction-pragma-runtime/v1`, produced from commit `29a9680` on an Apple M4 Max reporting `supportsFamily:MTLGPUFamilyApple9`, registry ID `4294968452`, arm64 macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, offline `Apple metal version 32023.883 (metalfe-32023.883)`, runtime `GPUCompiler.framework` build **`metalfe-32023.921`** — the same runtime compiler build finding 30 was measured against.

**Twelve cells, and the two columns disagree in every one.** Three widths × `mathMode` ∈ {`Relaxed`, `Fast`} × `MTLLibraryOptimizationLevel` ∈ {`Default`, `Size`}. `summary.control_fused` is 12, `summary.pragma_unfused` is 12, `summary.pragma_fused` is 0, `summary.pragma_other` is 0, and every one of the 24 cases carries an `executed` witness.

| width | `Relaxed` / `Default` | `Relaxed` / `Size` | `Fast` / `Default` | `Fast` / `Size` |
| --- | --- | --- | --- | --- |
| `bf16` control | `3fbf` fused | `3fbf` fused | `3fbf` fused | `3fbf` fused |
| `bf16` **pragma** | `3fc0` unfused | `3fc0` unfused | `3fc0` unfused | `3fc0` unfused |
| `f32` control | `3fc58f9d` fused | `3fc58f9d` fused | `3fc58f9d` fused | `3fc58f9d` fused |
| `f32` **pragma** | `3fc58f9e` unfused | `3fc58f9e` unfused | `3fc58f9e` unfused | `3fc58f9e` unfused |
| `f16` control | `3e01` fused | `3e01` fused | `3e01` fused | `3e01` fused |
| `f16` **pragma** | `3e00` unfused | `3e00` unfused | `3e00` unfused | `3e00` unfused |

Every control value reproduces finding 30 exactly, which is the check that this probe measured the same thing that record did before perturbing it.

**The offline companion confirms the pragma is live in this spelling and placement, and shows how it interacts with the `fast` umbrella.** Under `-fmetal-math-mode=relaxed -ffp-contract=fast` the control emits `fmul reassoc nsz arcp contract afn` and the pragma source emits `fmul reassoc nsz arcp afn` — `contract` removed and nothing else changed. Under `-fmetal-math-mode=fast` the control emits the single umbrella flag `fast` and the pragma source emits `reassoc nnan ninf nsz arcp afn` — the umbrella decomposed into exactly its constituents minus `contract`. Both compiled clean under `-Wall -Werror`, so `offline.*.diagnostics` is `none` in all twelve offline rows. The declared `air.compile_options` set is identical between control and pragma in every row, which is finding 22's point again in a new place: the module-level declaration is not a summary of the licences applied.

**What produced it is recoverable, which the recorded revision alone would not establish.** `probe.repository_base_revision` names `29a9680`, the base this branch was cut from, but the run was taken from a working tree that also carried the then-uncommitted harness — the same gap the [compatibility probe's retained-producer note](../README.md#artifact-family-and-reproducibility-probe) describes. The record and `input-manifest.tsv` close it by binding the bytes rather than the revision: `probe.harness_sha256` `c125c671…`, `probe.numerical_harness_sha256` `e7b831d6…`, and `probe.host_source_sha256` `31a3e02f…` each equal the `shasum -a 256` of the checked-in `pragma_probe.py`, `../numerical_probe.py`, and `../numerical_probe_host.m` at the commit that lands this directory. Reproduce all three in one line:

```sh
shasum -a 256 pragma_probe.py ../numerical_probe.py ../numerical_probe_host.m
```

**Determinism.** Two consecutive runs from the same tree differ in exactly one row, `environment.date_utc`, across 214 others — every `case.*`, `offline.*`, `source.*`, `summary.*`, and remaining `environment.*` row identical, and both `input-manifest.tsv` and all six retained sources byte-identical. The retained record is the second of those two runs. Reproduce with:

```sh
uv run python pragma_probe.py --result-dir /tmp/pragma-run-a
diff /tmp/pragma-run-a/record.tsv results/2026-08-02-contraction-pragma-macos-msl31-xcode26.6-metal32023.883/record.tsv
```

## Measurement boundary

This is one host, one Mac GPU, one offline toolchain build, one runtime compiler build, one pragma spelling, one placement, one kernel shape, and one MSL version. Specifically **not** established:

- **The other spelling.** `#pragma clang fp contract(off)` is recorded by finding 10 as equally accepted offline and is **not** swept here. One spelling answers the question; a second would double every cell to compare two mechanisms rather than measure one.
- **The iOS families.** `IOsSimulator` refuses to create a `bfloat` pipeline at all on this row (finding 26), so the width the contract question is about cannot be dispatched there, and no iOS device is attached. The pragma's runtime effect is `Unknown` for both.
- **Any other kernel shape.** This measures a two-statement multiply/add. Finding 16 records that `-ffp-contract=off` is not a defence against a source-level `fma`, and nothing here suggests the pragma would be either; a program that wrote `fma` would still get a fused result.
- **Durability across compiler builds.** Finding 8's reason to keep re-measuring applies with full force: the runtime compiler ships with the OS and can move without the offline one moving. A source-level control the runtime compiler honours today is a measured property of `metalfe-32023.921`, not a guarantee of the language.
- **A warning-free runtime acceptance**, for the reason `probe.runtime_diagnostic_channel` states.
- **Any consequence for the contracts.** What a target profile may now declare, and how an emitted-pragma realization is recorded so that a consumer can tell it from a natively unfused one, is contract work outside this spike's scope. It is carried by [`declare-the-metal-emitted-pragma-unfused-realization`](../../../tickets/declare-the-metal-emitted-pragma-unfused-realization.md).
