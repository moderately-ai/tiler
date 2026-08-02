---
id: probe-the-bf16-contraction-pragma-on-the-metal-runtime-path
title: Probe the BF16 contraction pragma on the Metal runtime path
status: done
priority: p3
dependencies: []
related: [design-the-bf16-computation-and-accumulator-contract, probe-metal-runtime-compilation-numerics, declare-metal-numerical-honourability]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, apple-targets, numerics, bf16, contraction, measurement]
---
## User-visible outcome

A measured answer to whether a BF16 program can be given an *unfused* guarantee on the Apple runtime compilation path. Today it cannot: the retained record measures the runtime compiler contracting a written multiply/add pair at every width under `relaxed` and `fast`, with no `MTLCompileOptions` counterpart to `-ffp-contract`, so a BF16 contract resolving ADR 0015 contraction to `Forbidden` is a disproved predicate on every measured row rather than an honourable one.

## Why the question is open rather than answered

**Measurement — finding 30 of [the Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md).** Runtime-compiled `contraction_pair_bf16` returns the fused `3fbf` under `relaxed` and `fast` and the separately rounded `3fc0` under `safe`, at both runtime optimization levels, on `MacOs` and `IOsSimulator` alike. Boundary: runtime compilers `metalfe-32023.921` and `metalfe-32023.830.1` on Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6.

**Measurement — finding 10.** `MTLCompileOptions` exposes no contraction property, so nothing was substituted for `-ffp-contract` and each runtime case is compared against every offline contraction setting instead.

**Measurement — finding 10's last paragraph, which is the lead this ticket follows.** `#pragma METAL fp contract(off)` and `#pragma clang fp contract(off)` are both accepted without diagnostic by `xcrun metal -Wall -Werror -std=metal3.1` on that row, and both remove the `contract` fast-math flag from the emitted IR under `-ffp-contract=fast`. The record deliberately did **not** use it in the runtime probe, because doing so would have changed the source bytes and destroyed the byte-identical offline/runtime pairing the whole comparison rests on. It is recorded as an available mechanism and not adopted as a substitute.

**Inference.** So the pragma's effect on the *runtime* path is unmeasured rather than known to be absent, and the difference matters: if it is a defence, a BF16 unfused contract becomes honourable through emission rather than through a compile option, which is ADR 0076's `SupportedWithExactEmulation` shape — honoured by emitting different source. If it is not, the refusal stands and is correct.

## What must be measured

- The `contraction_pair_bf16` source, unchanged except for the pragma, compiled through `[device newLibraryWithSource:options:]` with `mathMode = Relaxed` and with `Fast`, at both `MTLLibraryOptimizationLevel` values, dispatched through the same terminal-status-checked readback the retained harness uses.
- The same at `f32` and `f16`, because finding 30's result is width-independent and a pragma that worked at only one width would be a finding rather than a convenience.
- The unperturbed neighbour in the same run: the identical source **without** the pragma must still return the fused value under the same options. A run in which neither returns the fused value establishes nothing about the pragma.
- The discriminating constants are already chosen and must be reused, not re-derived: finding 28 records scale `0x3FBE` at `bf16` and `0x3E02` at `f16`, each one ulp from 1.5 and each the nearest value that discriminates at its vector's ordinary normal, with the witness operand `1.0` contraction-independent at both. Finding 28's own warning applies — the obvious `x * 1.5 + 1.0` kernel discriminates on no operand in either narrow vector and would report the opposite conclusion while its execution witness reports `executed`.

## Required evidence

- The pragma probe lives beside the existing harness under `spikes/apple-targets/`, runs by hand from its own directory, and retains its result rows with the exact environment row.
- The changed source bytes are stated explicitly, since the whole reason the retained record excluded this is that the pairing depends on them. This probe is a *separate* measurement and must not be folded into the byte-identical comparison.
- A negative result is a result: if the pragma does not survive to the runtime compiler, record it with the exact diagnostic or the exact returned pattern, and the refusal in `docs/numerical-semantics.md` stands unchanged.

## Closes when

Both directions are recorded with their controls, the retained record gains the rows, and `docs/numerical-semantics.md`'s statement that the pragma's runtime effect is unmeasured is either corrected or confirmed with its measurement boundary.

## Outcome — the pragma is a defence, measured 2026-08-02

**Measurement.** `#pragma METAL fp contract(off)`, inserted at file scope as one 31-byte line with every other byte of the source unchanged, survives `newLibraryWithSource:options:` to runtime compiler `metalfe-32023.921` and unfuses the written multiply/add pair in **all twelve** cells of `{bf16, f32, f16} × {Relaxed, Fast} × {Default, Size}`. The unperturbed neighbour returns finding 30's fused value in every one of those cells and every one of the 24 cases carries an `executed` witness: `bf16` `3fbf → 3fc0`, `f32` `3fc58f9d → 3fc58f9e`, `f16` `3e01 → 3e00`. Boundary: Apple M4 Max reporting Apple9, registry ID `4294968452`, arm64 macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, SDK 26.5 build 25F70, offline `metalfe-32023.883`, `-std=metal3.1`, `MacOs` only.

**Where the evidence lives.** Harness and README at [`spikes/apple-targets/contraction-pragma-runtime-probe/`](../spikes/apple-targets/contraction-pragma-runtime-probe/README.md); retained record at `results/2026-08-02-contraction-pragma-macos-msl31-xcode26.6-metal32023.883/record.tsv`, schema `tiler.apple-contraction-pragma-runtime/v1`, with the six sources and their manifest beside it. Finding 33 of [the Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) states it, and findings 10 and 30 gained the cross-references that keep them from reading as still-open.

**How the requirements this ticket set were met.** The control is the unperturbed source and is byte-identical to the retained numerical-probe `sources/contraction_pair.metal` finding 30 was taken from (`5b4a39ca…`). The discriminating scales are read out of `numerical_probe.BY_NAME` — `0x3FBE` at `bf16`, `0x3E02` at `f16` — rather than re-derived, and both candidate results per operand are derived by `numerical_probe.evaluate`, reproducing findings 28 and 30 exactly. The probe lives beside the numerical harness rather than inside it, reuses the dispatch host unmodified so no retained numerical record's digests move, and runs by hand from its own directory. The control requirement is the failure proof and was watched firing: `--perturb-control` reports `12 of 12 controls did not fuse`, exits nonzero, and publishes nothing. Two consecutive runs differ in one row, `environment.date_utc`, across 214 others.

**What it does not establish**, each recorded in the probe README and finding 33: the `#pragma clang fp contract(off)` spelling, block-scope placement, a translation unit mixing operations with different contraction contracts, a source-level `fma` (finding 16 says the flag is no defence there and nothing suggests the pragma is), any iOS family (finding 26 refuses `bf16` on the simulator), any other runtime compiler build, and a warning-free runtime acceptance — the dispatch host reads `newLibraryWithSource`'s error only when the library is nil.

**Consequence filed rather than absorbed.** An unfused BF16 contract is now honourable on this row by *emitting different source*, which is ADR 0076's `SupportedWithExactEmulation` shape and not a compile option. Four documents outside this ticket's scope still say the pragma's runtime effect is unmeasured — `docs/numerical-semantics.md`, `docs/research/numerics/bf16-computation-accumulator-and-conversion.md` twice, and ADR 0091 — and whether Tiler's emitter may write the pragma at all is a contract decision with a translation-unit-scope problem inside it. [`declare-the-metal-emitted-pragma-unfused-realization`](declare-the-metal-emitted-pragma-unfused-realization.md) carries all of it.

## Graph maintenance

- Triggered by `design-the-bf16-computation-and-accumulator-contract`, which recorded this as an explicit deferral with this exact closing evidence.
- Nothing depends on it today, and that is why it is `p3`: no contract currently rests on the answer, because a contract is written against what a target delivers. It becomes blocking the moment a BF16 contract declaring contraction forbidden is offered to a Metal profile.
- Not a widening of the numerical probe's gate. The retained gate must keep its byte-identical pairing; this probe is a sibling with its own source and its own rows. Confirmed on landing by naming and counting the population rather than by inspection: `grep -l "probe.harness_sha256" spikes/apple-targets/results/*/record.tsv` finds **13** retained records, and `git status --porcelain -uall spikes/apple-targets/results/` is empty on this branch — none of them moved, because the sibling adds its own producer and reuses `numerical_probe_host.m` byte for byte.
- The trigger this ticket named is now inherited by [`declare-the-metal-emitted-pragma-unfused-realization`](declare-the-metal-emitted-pragma-unfused-realization.md), filed `todo` at `p2`: the measurement exists, so what is left is a stale-fact sweep across four documents plus one emitter-authorization decision that is Tom's. It is a higher priority than this ticket was, because four documents now carry a sentence the evidence refutes.
