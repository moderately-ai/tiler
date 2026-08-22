---
id: decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary
title: Decide how a dynamic bounds witness enters the schedule vocabulary
status: todo
priority: p2
dependencies: [package-the-admitted-live-schedule-into-a-symbolic-kernel-program]
related: [replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges, carry-live-extent-operands-through-the-artifact-envelope]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, identity, public-boundary, schedule]
---
## User-visible outcome

A live-extent access carries a bounds proof that states its reach, instead of the zero-length `LinearRange` the verifier emits today — under an accepted public spelling and an accepted identity consequence, rather than a worker's improvisation.

## Why this exists — filed 2026-08-19 from the p0 live-bounds audit

[`replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges`](replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges.md) (p0) cannot proceed without this decision. Its worker's audit at `441f3215`, merged with that ticket, establishes the two authority steps the p0 does not carry:

**Fact — a dynamic range witness is a public-vocabulary change whose accepted record states the opposite intent.** It requires a new `BoundsProofKind` variant in `tiler_ir::schedule`. That vocabulary is accepted public surface, and the accepted `LiveRowMajorSource` record says the live extent is `crates/tiler-ir/src/schedule/model.rs "consumed in the payload"` rather than specialized into the schedule. Admitting a schedule-level witness reverses a stated design intent and is therefore Tom's, not a carrier's.

**Fact — the tag moves identity.** `BoundsProofKind` is written into the canonical scheduled-region identity encoding beside `TAG_LINEAR_RANGE`. A new tag moves every live region's schedule identity and cascades through kernel, kernel-program, and artifact identity. The neighbouring tag comments show these assignments are reconciled *across* accepted decision packets, not assigned by a worker.

**Fact — the p0 ticket's stated direction inverts the layer order.** It says the witness is "derived from the artifact's existing `AbiRoot::InputExtent` authority", but the schedule sits below the program and artifact and cannot read them. The workable direction is the reverse, which is a materially different design from the one the p0 specifies.

**Fact — the static agreement rule blocks the obvious spelling.** `KernelProgramBuilder::push_stage` requires `evaluate_static_abi(accessible_bytes)` to equal the view's window length, and `static_facts()` binds only declared *static* extents — so on a symbolic subject that check cannot evaluate at all. Publishing a live reach needs a symbolic element count, a symbolic `ByteWindow` length, and a symbolic ABI agreement rule together, not one at a time.

## Why this is blocked rather than ready

The readiness gate's first step: a local API shape is not decision-ready while its consumer or prerequisite is unresolved. No live-extent artifact is constructible or decodable at this base, so the packet cannot state what the witness must serve, and a frontier derived now would be about a population that does not yet exist. [`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`](package-the-admitted-live-schedule-into-a-symbolic-kernel-program.md) — accepted by Tom on 2026-08-19 as the complete-subject fold, and the ticket that makes a symbolic program packageable — is the release trigger.

## Release — 2026-08-19, trigger fired; precondition partly verified, and the remainder is the packet author's first task

`package-the-admitted-live-schedule-into-a-symbolic-kernel-program` is `status: done` (merged and gated), so the stated release trigger has fired. The coordinator verified the *mechanism* the block rested on, and deliberately did **not** verify the whole precondition — read this before authoring:

**Verified at `4a813d21`, by reading:**
- The refusal that made a symbolic interface unpackageable is **gone**: `grep -c SymbolicInterfaceExtent` returns **0** in both `crates/tiler-ir/src/program/error.rs` and `crates/tiler-artifact/src/program/builder.rs`.
- The live-row association has a passing **accepting arm**: `crates/tiler-artifact/src/program/tests/extent_operands.rs`, anchor `fn a_live_operand_on_the_source_bearing_symbolic_axis_associates`, which `.expect()`s success on the source-bearing symbolic axis. Seven sibling arms refuse.
- `check_extent_operand_association` refuses only when the extent `as_static()` — so a live row over a **symbolic** axis is no longer refused on that ground.

**NOT verified, and it is the load-bearing half:** that a complete live-extent artifact **constructs and decodes end to end**. The block's stated ground was "no live-extent artifact is constructible or decodable at this base, so the packet cannot state what the witness must serve". An association unit test is not that proof.

**So the packet author's first deliverable is to establish end-to-end constructibility and decodability, with the command and its output**, before enumerating any spelling. If it turns out a live-extent artifact still cannot be built or loaded, **stop and report** — the readiness gate forbids a frontier over a population that does not exist, and this ticket returns to `blocked` with the new ground recorded rather than being worked around.

## Two findings from the index-layer landing that this packet MUST NOT inherit — 2026-08-22

The index half of `admit-the-selected-data-dependent-index-representation` landed at `3e04a21c` and surfaced two facts that bear directly on the spellings this packet has to enumerate. Both were re-verified by the coordinator at source before being written here.

**1. The accepted ADR 0108 packet's tag premise is false, and its assigned value would land in the wrong tag space.** That packet states the current `LinearRange` / `ReductionDomain` bytes are `0x01` / `0x02` and assigns a gather bounds proof `0x03`. Read at `crates/tiler-ir/src/schedule/model.rs`, the real values are `TAG_LINEAR_RANGE = 0x11` and `TAG_REDUCTION_DOMAIN = 0x12`, and `git log -S` attributes them to `912bb110`, **long before that packet was written** — so this was false when written, not drift.

The file is **nibble-partitioned**: `0x01`–`0x0D` are `LogicalAccess` access-map tags (`TAG_LINEAR_IDENTITY = 0x01`, `TAG_REDUCTION_CONTRIBUTOR = 0x02`, `TAG_SCALAR_BROADCAST = 0x03`, … `TAG_PARTITIONED_COPY_SOURCE = 0x0D`), `0x11`–`0x12` are bounds proofs, `0x22`–`0x25` are scalar programs. A new bounds-proof kind at `0x03` therefore takes a byte the access-map vocabulary already writes, and the packet's injectivity argument — "the next value no accepted record claims" — rests on the false premise. **The fresh tag is `0x13`.** Do not restate `0x03` in any option this packet presents.

**2. Inlining the proof would grow `BoundsProofKind` by an order of magnitude, and the obvious remedy is a stop condition.** The packet spells `BoundsProofKind::GatherSource { …, proof: GatherIndexBoundsProof }` inline. Measured on the landed tree, `BoundsProofKind` is **72 bytes**, while `GatherIndexBoundsProof` carries three `Shape`s, two `ResolvedValueType`s, a region identity, and a domain vector — inlining it takes the enum to several hundred bytes, and **every** `BoundsProofKind` pays that, including the `LinearRange` case that is the overwhelming majority. Boxing it is the natural fix and **re-spells an accepted public field**, which is Tom's call rather than a worker's.

So this is a real frontier axis the packet must carry rather than assume away: inline-and-grow, box-and-re-spell (needs Tom), or a spelling that keeps the proof out of the enum entirely. State the measured byte cost of each — the numbers above are re-derivable and should be re-derived at the packet's own base.

## Required work when released

Author a Pareto-complete decision packet under the AGENTS.md readiness gate: enumerate every materially distinct spelling (a new `BoundsProofKind` variant; a payload-side witness that leaves the schedule vocabulary alone, consistent with the `LiveRowMajorSource` record's stated intent; a symbolic-extent widening of the existing `LinearRange`; the status quo with the population left refused); eliminate anything that can silently return a wrong reach, default a bound, or let an adapter reconstruct a second meaning of the live extent; compare survivors on correctness, fail-closed strictness, maintainability, and the exact identity cascade each implies with its tag reconciliation; state the strongest counterargument and reversal evidence for each; and present the nondominated frontier to Tom as one concrete question. Independent review before queueing.

## Closes when

Tom accepts one exact spelling with its identity consequence, or records that the population stays refused — and the p0 carrier's dependency on this ticket resolves either way.
