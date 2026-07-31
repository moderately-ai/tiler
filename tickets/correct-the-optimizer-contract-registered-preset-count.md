---
id: correct-the-optimizer-contract-registered-preset-count
title: Correct the optimizer contract's registered preset count
status: todo
priority: p2
dependencies: []
related: [admit-a-reassociating-contract-without-contraction]
scopes: [contracts/optimizer]
shared_scopes: []
paths: []
tags: [docs, numerics, compiler]
---
## User-visible outcome

`docs/compiler/optimizer.md` names every numerical contract this build registers, so a reader who follows it to `StrictF32NumericalContract::governed_profile` finds the same set.

## Why this is its own ticket

**Fact.** `admit-a-reassociating-contract-without-contraction` registered a fourth preset — `NumericalPolicyPreset::PermitReassociation`, key `tiler.reassociate-f32.v1` — permitting reassociation and forbidding contraction. Its allowed scopes were `implementation/compiler` and `contracts/numerics`; `docs/compiler/**` is `contracts/optimizer` and was outside them, so the sentence was left stale rather than edited off-scope.

**Fact — the exact stale text**, `docs/compiler/optimizer.md` line 306: "`StrictF32NumericalContract::governed_profile` in `crates/tiler-compiler/src/request.rs` returns the exact set of numerical contracts this build registers: strict, flush-to-zero, and relaxed. The relaxed contract permits ordered reassociation for operation families that declare ordered associativity, while the other two forbid it". Both halves are now wrong: the set has four members, and two of them permit reassociation.

**Fact — nothing else in that file needs to move.** The following sentences about distributivity, `normalize_serial_sum` rejecting multi-input programs, and ADR 0015 contraction being an independent permission are all still true; the last is strengthened by the new preset rather than contradicted.

## Implementation keys

Correct the enumeration and the "while the other two forbid it" clause. `docs/numerical-semantics.md` already carries the derivation of why the fourth preset is a different meaning rather than a relaxation — link it rather than restating it, since two copies of one derivation is the drift this sweep exists to remove.

## Closes when

`docs/compiler/optimizer.md` names the four registered contracts and says which permit reassociation; no other sentence in the file disagrees with `governed_profile`; `tkt lint` and the batch gate pass.

## Graph maintenance

- Companion to the corpus sweep, not a blocker: nothing depends on this text.
- If a fifth contract is ever registered, this file and `docs/numerical-semantics.md` both name the set and both must move in that change.
