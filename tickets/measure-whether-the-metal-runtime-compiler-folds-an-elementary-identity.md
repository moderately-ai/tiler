---
id: measure-whether-the-metal-runtime-compiler-folds-an-elementary-identity
title: Measure whether the Metal runtime compiler folds an elementary identity
status: deferred
priority: p3
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, decide-whether-to-admit-an-elementary-identity-permission, construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, metal, measurement, deferred]
---
## User-visible outcome

The half of the elementary-identity folding question the offline probe cannot reach is answered on the execution path a program actually takes, so a target profile's declaration about the dimension covers the runtime compiler rather than only the offline driver.

## Why this exists

**Measurement.** [The elementary-identity folding probe](../spikes/numerics/elementary_identity_folding/README.md) compiled sixteen kernels under six math-mode flag sets on offline `metalfe-32023.921` and found **no elementary identity folded in any of them**, with the mechanism read in the IR: `air.exp.f32` applied to a literal `0.0f` is not even constant-folded, so the AIR elementary intrinsics are opaque to LLVM's constant folder and identity combiners at that stage.

**Fact — that is the offline front end and nothing else.** A `.metallib` carries AIR; the AIR-to-GPU-ISA stage runs at pipeline creation, in a different compiler. Finding 30 of [the Apple GPU numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) is the standing reason not to extrapolate across that boundary: the runtime compiler was measured *contracting* a written multiply/add pair whatever the offline selection said, and the record's own inference is that "an offline contraction measurement is not transferable to the runtime path" and that a profile declaring honourability from an offline compilation "would be wrong about every runtime-compiled kernel".

**Inference.** So whether the runtime compiler folds `exp(a) * exp(b)` into `exp(a + b)` is `Unknown`, and a target-profile declaration resting on the offline probe alone would carry exactly the defect finding 30 names.

## Why this is deferred rather than todo

Nothing consumes a declaration about this dimension, because the dimension is not admitted: [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md) is itself deferred behind the distributivity reassessment. Measuring a runtime behaviour to support a declaration no contract asks for would produce a number with no consumer, and the measurement would need re-running against whatever runtime build is current when a declaration is actually owed — runtime compiler builds move independently of the offline driver, and the numerical-behaviour record already carries three distinct builds across two families.

## Trigger

**Fires when a target-profile declaration about the elementary-identity dimension is owed** — that is, when the dimension is admitted to the contract, since silence about a dimension is `Unknown` and would make every compilation on the profile unexecutable. **Or** when the offline probe is re-run on a newer toolchain and *does* fold, which would make the offline half a positive finding whose runtime counterpart matters immediately.

A general wish to know does not fire it, and neither does a new Xcode release on its own.

## What it would take

- A runtime-compiled counterpart to each identity kernel pair, compared on the same axis the offline probe uses. The observable is harder than the offline one: the runtime path exposes no AIR listing, so the comparison is by **returned value** rather than by emitted opcode — dispatch both spellings on arguments the counterexample survey names as disagreeing, and check whether the two spellings return the same bits when the identity says they should differ.
- That is a stronger check than the offline one rather than a weaker substitute, and it is the reason this ticket is a measurement rather than a re-run: an offline probe reads what was emitted, and a value comparison reads what was delivered.
- The same argument grid `identity_counterexample.py` establishes, so a runtime agreement where exact arithmetic says the two must differ is a fold and not a coincidence.
- Device execution on the qualified Apple row, its exact runtime `GPUCompiler.framework` build recorded, and the offline half re-run on the same toolchain so the two halves are rows of one table.

## Non-goals

Installing or mutating any toolchain component; declaring anything on a target profile; extrapolating the offline negative to the runtime path in the meantime.

## Trigger check log

- 2026-08-05 — **not fired.** No declaration is owed: no elementary-identity dimension exists in `CANONICAL_DIMENSIONS`, and the deciding ticket is itself `deferred`. The offline probe's most recent run found no fold, so the second clause has not fired either. Recheck with `grep -c 'ElementaryIdentity' crates/tiler-ir/src/numerics.rs`, which answers `0` while the dimension is unadmitted.
- 2026-08-09 — **not fired.** `CANONICAL_DIMENSIONS` still has no elementary-identity member and the admission decision remains `deferred`; no contract therefore needs a runtime realization declaration. No newer retained measurement reports an offline fold either. Recheck the complete canonical-dimension array and `tkt show decide-whether-to-admit-an-elementary-identity-permission`.
