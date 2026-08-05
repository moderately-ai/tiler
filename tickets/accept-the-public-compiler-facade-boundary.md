---
id: accept-the-public-compiler-facade-boundary
title: Accept or revise the public compiler facade boundary
status: awaiting-decision
priority: p1
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary, admit-ordered-multi-output-programs-at-the-compiler-request-boundary]
related: [prototype-public-compiler-api, prototype-optimizer-conformance-gate]
scopes: [contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom receives one packet for the `tiler_compiler::session` facade, and the conformance gate's stated precondition — "before the public compiler facade is accepted" — becomes an event that can actually happen instead of a condition with no node.

## Why this node exists

**Fact — a contract states a precondition on an acceptance nothing owns.** `docs/correctness-and-testing.md:106-111`: "The optimizer conformance owner must exercise an external operation through the ordinary capability/refinement path, non-isomorphic and fan-out or multi-output graphs, deterministic typed explain records, and identity/provenance assertions at every implemented layer **before the public compiler facade is accepted**." No acceptance ticket exists among the `accept-*` tickets in `tickets/` — reproduce with `ls tickets/accept-*.md` and read each. The precondition is therefore unevaluable: nothing can be before an event with no owner.

> **The paragraph above is a statement about the board *before* this node was filed, and it reads in the present tense, corrected 2026-08-04 by the stale-claim sweep.** Run as written, `ls tickets/accept-*.md` now lists 17 files including this one, so a reader following the check finds an owner and cannot tell whether the Fact is refuted or merely self-referential. **It is self-referential: this node is the owner, and filing it is what discharged the paragraph.** Read it as the reason the node exists rather than as a current absence. The precondition at `docs/correctness-and-testing.md:106-111` is now evaluable — its consequent is this node — and both cited spans were re-read at base `c4b4bdb9` and still resolve: `:106-111` carries the precondition, `:113` the produced evidence, and `:117` the gap. Nothing about the decision moves and only Tom closes this node.

**Fact — the facade is explicitly a reviewed draft, not a stabilized boundary.** `docs/correctness-and-testing.md:117`: "the session facade is a reviewed experimental draft rather than a stabilized or published API; reviewed visibility is not stabilization." That sentence is correct and should stay correct until this node closes — it is the disclosure that keeps the draft from being read as accepted.

**Fact — the gate's evidence is real and partial, and the packet is the difference.** `docs/correctness-and-testing.md:113` records what is produced: out-of-crate capability installation with a companion negative test that fails closed, external-operation compilation through the ordinary path, and the fail-closed siblings. `:117` records what is not: ordered multi-output programs rejected rather than compiled, a bounded admitted program subject, and no installation evidence for the scalar-lowering family. The packet is that gap, stated exactly.

**Inference — without this node, two other tickets have no acceptance event to land against.** [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) flips the gate's multi-output row, and [`resolve-or-retire-the-scalar-lowering-provider-seam`](resolve-or-retire-the-scalar-lowering-provider-seam.md) settles its third gap; both close into a precondition whose consequent does not exist.

## Ripens when

Set with Tom, 2026-08-01: this node ripens **after the general program-shape recognizer and ordered multi-output admission land and stop reshaping the request surface** — its two dependencies. The reason is exact rather than procedural: both change the shape of what `session::compile` accepts and what it refuses by name, so accepting the facade first would accept a boundary about to move, and re-accepting it afterwards would spend Tom's time twice on one surface.

Until then the dependencies keep this node structurally unripe as well as stated as such, and `rollup` names them as the cause.

## Decision boundary

Not research or implementation work. When it ripens, the packet states the exact public surface — `session::compile`, `session::compile_governed`, `CompileRequest` and its installation methods, `InstalledCapabilities`, `Compilation` and its accessors, and the refusal vocabulary `CompileFailureClass` exposes, which is already quoted verbatim by a frontend at `crates/tiler-macros/src/region.rs:57` and is therefore observable behaviour rather than an internal detail — with ownership, naming, validation and identity obligations, and the conformance evidence for and against each. One atomic question at a time.

**Citation corrected 2026-08-04 by the stale-claim sweep; the claim it supports is stronger than the citation was.** `crates/tiler-macros/src/region.rs:57` is inside that module's header prose about symbolic extents and quotes no refusal class — `grep -n 'CompileFailureClass' crates/tiler-macros/src/region.rs` returns nothing. The frontend that consumes the vocabulary is `crates/tiler-macros/src/aot.rs`: it imports `CompileFailureClass` at `:207` and `rendered_refusal` at `:438` matches it **exhaustively** — `UnsupportedCapability { rule }` at `:440`, `NoFeasiblePlan` at `:457`, `BudgetExhausted` at `:465`, `InvalidRequest { rule }` at `:471`, `InvalidCompilerOutput` at `:476` — with a comment at `:480` recording that the enum is `#[non_exhaustive]` so a class added later reaches the wildcard. Its own doc comment at `:417` says the rendering is "Derived from [`CompileFailureClass`] rather than from the refusal's `Debug`", which is the point this paragraph was making: the class set and its `rule` payloads are a consumer-visible surface, and the frontend renders each variant into a distinct compile error rather than one opaque message. Cite `aot.rs:438-480` in the packet.

## Closes when

Tom accepts or revises the facade; `docs/correctness-and-testing.md:106-111`'s precondition is discharged or restated against what he accepted; `:117`'s reviewed-draft disclosure is corrected in the same change, because it becomes wrong the moment acceptance lands; and any surface he declines is named as declined rather than left in the tree unmarked.
