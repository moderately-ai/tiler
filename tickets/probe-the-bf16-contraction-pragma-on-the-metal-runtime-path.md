---
id: probe-the-bf16-contraction-pragma-on-the-metal-runtime-path
title: Probe the BF16 contraction pragma on the Metal runtime path
status: in-progress
priority: p3
dependencies: []
related: [design-the-bf16-computation-and-accumulator-contract, probe-metal-runtime-compilation-numerics, declare-metal-numerical-honourability]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, apple-targets, numerics, bf16, contraction, measurement]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785634941
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

## Graph maintenance

- Triggered by `design-the-bf16-computation-and-accumulator-contract`, which recorded this as an explicit deferral with this exact closing evidence.
- Nothing depends on it today, and that is why it is `p3`: no contract currently rests on the answer, because a contract is written against what a target delivers. It becomes blocking the moment a BF16 contract declaring contraction forbidden is offered to a Metal profile.
- Not a widening of the numerical probe's gate. The retained gate must keep its byte-identical pairing; this probe is a sibling with its own source and its own rows.
