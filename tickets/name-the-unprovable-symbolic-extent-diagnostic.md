---
id: name-the-unprovable-symbolic-extent-diagnostic
title: Give an unprovable symbolic extent its own region diagnostic
status: done
priority: p2
dependencies: []
related: [implement-shapeenv-index-bindings]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, diagnostics]
---
**Fact — what landed.** `implement-shapeenv-index-bindings` makes the region verifier refuse an access over a domain whose symbolic extent the `ShapeEnv` neither bounds nor determines. It reports that refusal as the existing `IndexRegionDiagnostic::BoundsNotProven` or `WriteOwnershipNotProven`, chosen by access mode.

**Fact — that is sound but imprecise.** Both are refusals in the taxonomy `docs/ir.md` establishes, so nothing is misclassified: "a result carrying only proof-resource diagnostics leaves its predicates open, while one carrying any other diagnostic is a rejection whatever else accompanies it." The landed code is deliberately *not* `ProofResourceLimit`, because no enumeration stopped. It has the precedent of `verify_access_exhaustively`, which reports the same pair when a tensor's element count is unrepresentable.

**What is missing.** The diagnostic does not say *why*. A consumer reading `BoundsNotProven` cannot distinguish "interval propagation overlapped a boundary and the finite fallback disproved it" from "the extent is symbolic and the environment bounds it nowhere". Only the second is fixable by adding a constraint, and that is the action a frontend would need to be told to take.

`IndexRegionDiagnostic` is already `#[non_exhaustive]`, so adding this
diagnostic follows the repository's additive-growth convention. The public
meaning still needs to be documented and tested rather than hidden under the
older generic refusal.

## Closes when

A missing symbolic bound produces a distinct diagnostic naming the affected
access or dimension and extent symbol. Genuine interval failure continues to
produce `BoundsNotProven` or `WriteOwnershipNotProven`; positive and negative
neighbors prove the distinction, and `make full` passes.

## Outcome (2026-07-27)

`IndexRegionDiagnostic::ExtentBoundNotStated { access, symbol }` names the access and the extent symbol whose bound the environment never states. `BoundsNotProven` and `WriteOwnershipNotProven` continue to report a bounded extent whose proof simply did not close.

### The discriminator is not what the ticket implied, and reading the solver is what found it

The obvious implementation — report the named diagnostic when the environment yields **no interval** for an extent — **would never have fired**, and the suite would have stayed green while proving nothing.

**Fact:** `constraint::solve` seeds every symbol at `0..=MAX_EXTENT` and narrows from there (`constraint.rs:1000-1001`). An unconstrained symbol therefore has an interval reaching the domain ceiling, not a missing one. `Solution::interval` returns `None` only when a class bound *exceeds* the domain, which is a different and rarer condition. **Fact:** `MAX_EXTENT` is `IMPOSSIBLE - 1` with `IMPOSSIBLE = 1 << 64`, so the ceiling is exactly `u64::MAX`.

This was caught by the two tests that *should* have moved and did not: `an_unbounded_symbolic_extent_is_refused_rather_than_enumerated` builds an environment with no constraints at all and kept passing its `BoundsNotProven` assertion under the first implementation.

The condition is therefore "the environment states no upper bound", and it lives on `ExtentInterval::states_no_upper_bound` in `constraint.rs`, beside the constant that defines the ceiling, with a `const` assertion that `MAX_EXTENT == u64::MAX` so widening the domain fails there rather than silently disabling the diagnostic.

### Both halves of the access are consulted

`unbounded_extent_symbol` walks the boundary axes and then the iterated domain's extents. Either can be the unbounded one, and a caller told about the wrong half would constrain the wrong symbol.

### The symbol is a rendering, not the type

`ShapeSymbol` and the whole `shape::env` authority are `pub(crate)`, and `sourced.rs` records that promoting the symbolic index profile is a separate reviewed step owned by `implement-shapeenv-index-bindings`. The variant carries `scope::name` as a `String` — what a frontend needs in order to name the symbol it must constrain, and what `ExtentSourceError`'s own messages already print. Putting `ShapeSymbol` in a public enum would have performed that promotion by implication.

### Neighbours prove the distinction in both directions

- **Positive:** an environment with no constraints yields `ExtentBoundNotStated` naming `…::n`, and still not `ProofResourceLimit` — nothing enumerated, so nothing ran out of budget.
- **Positive:** the undetermined dynamic copy, where both accesses rest on an equality over two unbounded extents.
- **Negative:** `n` bounded to `1..=5` against a four-element axis still reports `BoundsNotProven`, with an explicit assertion that it is **not** the new diagnostic. Without that assertion the new variant could have swallowed the generic case and every test would still pass; with it, reporting the named diagnostic where the frontend has already stated a constraint fails.
