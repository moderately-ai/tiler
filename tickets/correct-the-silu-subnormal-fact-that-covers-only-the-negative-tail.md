---
id: correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail
title: Correct the SiLU subnormal fact that covers only the negative tail
status: todo
priority: p2
dependencies: []
related: [apply-the-declared-numerical-conformance-on-every-reference-evaluation-path, derive-the-oracle-for-a-permitted-divergence-candidate]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, semantics, subnormals]
---
## User-visible outcome

`tiler::silu-f32@1`'s declared subnormal fact says what the operation does over its whole domain, so a reader deciding whether a target's flush is observable for this family gets the right answer instead of one that holds only where the fact was measured.

## Why this exists

**Fact — the declared value overstates its own measurement.** `SILU_F32_FACT_SUBNORMALS` (`crates/tiler-ir/src/semantic/silu.rs:110`) resolves to `preserved-and-unreachable-no-binary32-silu-result-or-intermediate-is-subnormal`, and its doc justifies "unreachable" from one region of the domain: `silu(-88.7228)` is `0x82b173cc`, a normal value, and `silu(-88.73)` is already `0x80000000`, so "the result drops from normal straight to `-0.0` with no subnormal band between them".

**Measurement — the claim is false near zero, where the reference is `x / 2`.** Evaluated at `nightly-2026-07-19` on the reference's own pinned formula, `silu(0x007fffff)` is `0x00400000` and `silu(0x00400000)` is `0x00200000` — both subnormal results, from subnormal operands. Reproduce in one line:

```text
python3 -c "import struct,numpy as np
f=lambda i: np.float32(struct.unpack('>f',struct.pack('>I',i))[0])
b=lambda x: struct.unpack('>I',struct.pack('>f',np.float32(x)))[0]
x=f(0x007fffff); print(format(b(np.float32(x/np.float32(np.float32(1.0)+np.float32(np.exp(np.float64(-x)))))),'08x'))"
```

**Inference — the negative tail is a real measurement and the domain claim is a generalization of it.** The tail argument is about `x / (1 + e^{-x})` for large negative `x`, where the numerator's magnitude collapses faster than the subnormal band; it says nothing about small positive `x`, where the quotient is approximately `x / 2` and a subnormal argument therefore has a subnormal image.

**Fact — the reference no longer relies on the claim.** [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md) applies both declared subnormal dimensions inside `silu_f32_under` and records the counterexample at that function's definition rather than trusting the fact. So this is a declaration defect with no live incorrect *evaluation* behind it, which is why it is filed rather than fixed in that branch: the fix is in `crates/tiler-ir`, outside its scopes.

## What this ticket must produce

- A declared value that is true over the whole binary32 domain. The obvious candidate is a `preserved` spelling with no reachability claim, but the wording is this ticket's to derive — a fact that says "unreachable" where a case exists is the failure mode being repaired, and a replacement that overstates in the other direction is the same defect.
- The doc comment carrying the counterexample bits beside the tail measurement, so the next reader sees which region each claim covers.
- A check that would have caught it: a case at the subnormal boundary of the argument domain, watched failing against the current spelling.
- The sweep for siblings. `SOFTMAX_F32_FACT_SUBNORMALS` and `RMS_NORM_F32_FACT_SUBNORMALS` both state a *reachable* divergence and are not suspect on this pattern; `BF16_FACT_SUBNORMALS` states preservation without a reachability claim. Read each in full and record the verdict rather than assuming from the shape.

## Explicit non-goals

Changing what the reference computes; changing the accuracy contract; any device measurement.

## Closes when

The declared fact is true over the domain it quantifies, the counterexample is recorded where the old claim was, and the new case has been watched failing against the old spelling.

## Graph maintenance

Filed by [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md), which found the counterexample while deciding whether the SiLU family could be documented immune to both subnormal dimensions rather than made to apply them. It could not, and the fact is why the question was asked.
