---
id: resolve-or-retire-the-scalar-lowering-provider-seam
title: Resolve or retire the scalar-lowering provider seam
status: todo
priority: p1
dependencies: []
related: [own-or-close-the-adr-internal-open-questions, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [implementation/compiler, contracts/optimizer, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, extension-seams]
---
## User-visible outcome

`ScalarLoweringProvider` is either an installation seam a compiled program actually reaches — with the same out-of-crate installation evidence the index-access family already carries — or it is gone. It stops being a registered public seam that nothing on the compile path exercises.

## Why this exists

**Fact — the family registers and resolves, and no compile stage resolves it.** `docs/compiler/optimizer.md:235` states it outright: "`ScalarLoweringProvider` remains only implemented-and-resolvable support because no compile stage resolves that family." `docs/correctness-and-testing.md:117` repeats it as a gap in the conformance gate's own evidence — "Scalar-lowering providers register and resolve but no compile stage resolves that family, so no installation evidence exists for it."

**Fact — every caller is a test.** `resolve_scalar_lowering` is declared at `crates/tiler-compiler/src/capability.rs:1150`. Its call sites are `crates/tiler-compiler/src/capability.rs:1939`, `:2165`, `:2213`, `:2229`, `:2339`, `:2369` and `crates/tiler-compiler/src/legality.rs:2143`. The `#[cfg(test)] mod tests` boundaries are `capability.rs:1688-1689` and `legality.rs:1432-1433`, so every one of those sites is inside a test module. Reproduce with `grep -rn "resolve_scalar_lowering" crates/` and compare each line against those two boundaries.

> **Every number above has drifted, corrected 2026-08-04 by the stale-claim sweep at base `c4b4bdb9`. The Fact is unchanged and was re-derived rather than assumed — this is the one citation in the ticket whose *argument* is a line-number comparison, so following the stale numbers would have made the comparison meaningless rather than merely inconvenient.** Current: `resolve_scalar_lowering` is declared at `crates/tiler-compiler/src/capability.rs:1096`. Its call sites are `capability.rs:1897`, `:2123`, `:2171`, `:2187`, `:2297`, `:2328` and `crates/tiler-compiler/src/legality.rs:1683` — seven, the same count. The `#[cfg(test)]` attributes are at `capability.rs:1637` and `legality.rs:1000`. **Every one of the seven sites is below its file's boundary**, so the conclusion holds: the declaration at `:1096` is production, and no production caller exists. Reproduce with `grep -n 'resolve_scalar_lowering' crates/tiler-compiler/src/capability.rs crates/tiler-compiler/src/legality.rs` and `grep -n '^#\[cfg(test)\]' crates/tiler-compiler/src/capability.rs crates/tiler-compiler/src/legality.rs`, then compare — the second command prints exactly one line per file, which is what makes the comparison total rather than a sample.

**Fact — an accepted ADR names the question and assigns nobody.** [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md):144 asks "Whether `ScalarLoweringProvider` should reach the compile path at all", observes that `lowering.rs` resolves only `IndexAccess` and that "an index-access provider emits its own per-point scalar work through the same context", and closes with "No owner is assigned." This ticket is that owner.

**Inference — a registered public seam nothing exercises is unvalidated extension surface.** AGENTS.md's contract requires extension mechanisms to preserve validation, feasibility, and versioned identity, and states that "extensible" does not mean unknown behaviour is optimizable. A seam whose only evidence is in-crate unit tests makes no such guarantee, and the asymmetry with the index-access family — which has out-of-crate installation evidence plus a negative test that fails closed, per `docs/compiler/optimizer.md:235` — is what makes the gap visible.

## Run the elimination and state which candidate survived

This ticket must not land a shrug. Test both candidates against correctness, performance, and long-term maintainability, and state the derivation so a reader can refute the elimination rather than only the conclusion.

- **Wire it.** The scalar family becomes a second seam a program reaches, with out-of-crate installation evidence and a companion negative test omitting one family and failing closed — the exact pair `docs/compiler/optimizer.md:235` describes for index-access, so a passing installation test cannot be explained by an installer ignoring its argument. What this must answer is what a scalar provider decides that an index-access provider delegating per-point work does not.
- **Retire it.** ADR 0078's own reading — that a scalar decomposition is an affordance an index-access provider may delegate to rather than a seam a program reaches — is accepted, and the registration, resolution, and their tests are removed. This supersedes an accepted ADR 0078 seam and is therefore **Tom's**: draft the superseding record, do not self-accept, and do not delete the seam before he accepts it.

## Closes when

One candidate is landed with its elimination written out; `docs/compiler/optimizer.md:235`, `docs/correctness-and-testing.md:117`, and ADR 0078:144's unowned question are each corrected or closed in the same change; and, if wiring, an out-of-crate caller installs a scalar-lowering provider, compiles through `session::compile`, and observes the artifact plan recording that provider as the lowering authority, with the companion omission observed failing closed.
