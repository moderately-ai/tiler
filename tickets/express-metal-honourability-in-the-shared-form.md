---
id: express-metal-honourability-in-the-shared-form
title: Express the Metal subnormal fact as a per-dimension honourability declaration
status: todo
priority: p0
dependencies: [compose-numerical-honourability-and-retire-the-strict-boolean, prototype-public-compiler-api, admit-a-caller-declared-target-profile]
related: [declare-metal-numerical-honourability, draft-target-honourable-numerical-contract-adr]
scopes: [implementation/metal, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, feasibility]
---
The remaining half of `declare-metal-numerical-honourability`, split out when its two settled questions landed. ADR 0076 item 3.

`declare-metal-numerical-honourability` settled the two questions that did not depend on the shared honourability form: the backend-local conformance step survives alongside the profile declaration with a stated reason, and the four golden fixtures stay governed under the strict declared realization. What it could not do is the piece that gives the ticket its name — express `MetalSubnormalArithmetic` as a per-dimension honourability declaration in the shared form, so `feasibility` can assess it as a peer of `CheckedTargetProfile` *before* emission rather than discovering it during.

## What is true today

**Fact — `crates/tiler-metal/src/target.rs`.** `MetalSubnormalArithmetic::{FlushesToZero { zero_sign }, PreservesSubnormals}` is a required caller-stated field of `MetalTargetFacts`, with its measurement recorded on the type. It is consulted in exactly one place, `emit::subnormal_gap`, and only during emission.

**Fact — the old strict-f32 boolean has been retired.** The compiler now has a
private per-dimension honourability form, but no measured Metal fact reaches it.

**Fact — the two crates cannot see each other.** `tiler-metal` depends on `tiler-ir` and `tiler-artifact`; `tiler-compiler` depends on `tiler-ir`. Neither depends on the other, and `AGENTS.md` requires the compiler core to stay independent of Metal types. Verified with `grep -n 'tiler-' crates/tiler-metal/Cargo.toml crates/tiler-compiler/Cargo.toml` at `94fb26e`.

## User-visible outcome

Make measured Metal numerical behavior available to compiler feasibility before
emission, keyed by both numerical dimension and arithmetic dtype, while
retaining one authoritative declaration and preserving backend
re-verification.

## The ownership decision

**Inference — no existing crate can hold both the shared honourability form and the Metal fact.** The form is a compiler authority and the fact is a Metal target property, so a declaration expressed in the shared form has three candidate sitings and they are not equivalent:

- **`tiler-ir` owns the vocabulary.** The honourability declaration becomes a target-neutral IR type both crates already depend on, and `tiler-metal` states its value. This keeps the compiler free of Metal types and gives one declaration both sides read. It widens `tiler-ir`'s remit from program vocabulary toward target vocabulary, which needs an argument rather than an assumption. `FlushedZeroSign` already lives there, which is evidence the boundary is not obviously wrong.
- **A checked adapter owned by an orchestrator.** A component depending on both
  reads `MetalTargetFacts` and constructs the compiler profile, but only if the
  adapter is total, versioned, and tested against every vocabulary member. An
  unchecked consumer-written translation is eliminated because it cannot prove
  completeness or faithfulness.
- **A third crate owns the declaration.** Clean dependency-wise and the most machinery for the least immediate return.

Do not pick by convenience. Whichever siting is chosen must keep the fact declared exactly once: `declare-metal-numerical-honourability` recorded that a second declaration of the same target property is the failure mode to avoid, because two checkpoints reading one declaration cannot diverge and two declarations can.

**This also decides ADR 0076's open question.** That ADR leaves the siting of the profile declaration mechanism explicitly open, and `declare-metal-numerical-honourability` deliberately did not answer it by omission. Whatever this ticket decides must be recorded there as an accepted decision rather than left implicit in the code.

## Constraints inherited, not up for renegotiation

- **Honourability is a stated target fact, never a probed one.** Under `-fmetal-math-mode=relaxed` a `scale 1.0, bias +0.0` kernel returns subnormal operands unchanged, which looks like preservation and is not: `x * 1.0` folds to a copy under every math mode, the surviving `+0.0` fadd is the operation that flushes, and `relaxed` deletes it. Do not close any part of this by observing a compiled kernel. `docs/backends/metal.md` records the trap.
- **Do not close a gap by widening a rule.** A mismatched zero sign must stay a rejection. Letting a program that asked for positive-zero flushing run on a sign-preserving target returns `0x80000000` where it asked for `0x00000000` — a wrong answer, not a relaxed one.
- **Keep the measurements on the declaring types.** `MetalTargetFacts` documents its measured basis on the type itself; preserve that wherever the declaration moves.
- **The backend-local conformance step stays.** `MetalNumericalGap` and `require_declared_realization` are not retired by this ticket; `declare-metal-numerical-honourability` recorded why, and `crates/tiler-metal/src/record.rs` carries the reasoning. This ticket adds the earlier checkpoint, it does not remove the later one.

## Closes when

The Metal subnormal fact is expressed as a per-dimension honourability declaration in the shared form; `feasibility` assesses it before emission and rejects with the shape `compose-numerical-honourability-and-retire-the-strict-boolean` defines — naming the dimension, the required behaviour, the declared target behaviour, and the declaring profile's versioned identity; the siting is recorded in ADR 0076; and `make full` passes.
