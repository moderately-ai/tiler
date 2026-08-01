---
id: admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary
title: Admit a general program-shape recognizer at the compiler request boundary
status: in-progress
priority: p1
dependencies: []
related: [reach-a-verified-kernel-through-the-structural-families, carry-the-elementary-numerical-dimensions-in-the-region-realization]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
claimed_from: todo
assignee: worker-recognizer
lease_expires_at: 1785610282
---
## User-visible outcome

A semantic program whose exact shape the compiler has never been taught reaches the optimizer on the strength of its operations and values, instead of being refused because it matches none of three hardcoded whole-program templates.

## Why this exists, and why nothing owns it

**Fact — the boundary is a fixed three-way template match.** `select_supported_strategy` (`crates/tiler-compiler/src/request.rs:2138`) tries `normalize_serial_sum` (`:2573`), `normalize_contraction` (`:2180`), and `normalize_pointwise` (`:2373`) in that order and, when all three refuse, returns the collected refusal. Each demands an exact operation cover — a four- or five-operation scale-bias-then-strict-serial `Sum`, a well-formed binary contraction, or a four-operation pointwise expression over one input plus constants — so a program composing two of them, or containing an admitted family none of them spells, is refused before any target-qualified explain trace exists.

**Fact — the roadmap already names an owner, and that owner disclaims it.** `docs/roadmap.md:409` and `:410` each read "R6 needs the whole-program recognizer, which [`reach-a-verified-kernel-through-the-structural-families`] owns". That ticket's Non-goals (`tickets/reach-a-verified-kernel-through-the-structural-families.md:37`) say verbatim: "A `ScalarProgram` copy variant, a standalone materializing reindex kernel, **a general program-shape recognizer**, and anything about the contraction family — which is blocked by the same recognizer and has its own tickets." The job is attributed to a ticket that explicitly excludes it, so no node owns it.

**Inference — this is the highest-leverage compilation-infrastructure gap on the board.** Three registered families are held at R5 by this recognizer alone, each row saying so in its own words at `docs/roadmap.md:409-411` ("the ceiling is the recognizer, not the family"); the contraction reached R6 only because `normalize_contraction` was added as a *fourth template* rather than because the boundary generalized; and `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` depends on this directly.

## Boundaries

- Generalizing recognition is not licence to admit an unrecognized program silently. An unsupported program must still refuse with `UnsupportedCapability` and a rule naming the property that was not recognized, in the split-rule idiom (`input-arity`, `output-arity`, `operation-set`, `dtype-f32`) that `admit-multi-input-elementwise-programs-at-the-compiler-boundary` established and Tom accepted.
- A recognizer may only admit what the physical layer can express. Admitting a program at the boundary that then fails mid-pipeline is worse than the refusal — the failure mode `admit-multi-input-elementwise-programs-at-the-compiler-boundary` located and declined to ship. Where the wall is below the boundary, file that widening as its own ticket and depend on it, as that precedent did.
- This is pre-production software with no external consumers: a normalization the general path subsumes is removed, not preserved beside its replacement.

## Closes when

A semantic program that no current normalization matches — minimally, one composing two admitted families in a single program — compiles through `tiler_compiler::session` to an emitted region; every refusal path names the unrecognized property under its own rule and each was observed failing against an accepted neighbour; and the roadmap's attribution of the whole-program recognizer is repointed at what this ticket lands. The roadmap edit itself belongs to [`correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings`](correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings.md) under `contracts/navigation`, which this ticket does not hold.
