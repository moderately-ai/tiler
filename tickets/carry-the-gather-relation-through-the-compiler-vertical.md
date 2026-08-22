---
id: carry-the-gather-relation-through-the-compiler-vertical
title: Carry the gather relation through the compiler vertical
status: in-progress
priority: p1
dependencies: [admit-the-selected-data-dependent-index-representation]
related: [decide-the-data-dependent-index-representation-public-surface]
scopes: [implementation/compiler, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, gather, identity, normalized-output]
claimed_from: todo
assignee: worker-gathervert
lease_expires_at: 1787438923
---
## User-visible outcome

A compiler request carrying a data-dependent gather reaches the scheduled-region relation the IR now admits, instead of being recorded as absent at three separate sites while the layer beneath it is fully built.

## Why this exists

Split out 2026-08-22 by the coordinator. The gather lane landed the schedule vocabulary and the kernel wall and **stopped at a coherent boundary rather than pushing through** — the relation is admitted and verified, nothing above can reach it, and the kernel wall refuses it by name. Its stopping point is the right one; this is the vertical above it.

**Read the accepted public-surface ticket before this one.** [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md) carries the exact accepted spellings, and the delivering lane reports that the parent ticket's remainder list **names the pieces without them** — it drafted a two-way resolution at schedule level from that list alone, found it wrong against the accepted surface, and discarded it. Anyone working from the remainder list alone will re-make that error.

## Facts

**Fact — `NormalizedOutput` is `pub(crate)` with five variants and roughly twenty exhaustive matches**, each of which needs a real gather answer rather than a stub; `spell_output` would have to build a gather region. Reported by the delivering lane; **re-derive the count at your base and say which unit you report.**

**Fact — nothing named `PendingInvocation*` exists anywhere in the tree.** The invocation-validation vocabulary is to be created, not extended. Re-verify by searching the tree rather than a named path — `AGENTS.md` records that a file-path citation fails as false absence after a module split because the named file usually still exists.

**Fact — three sites record gather's absence and must flip together.** The delivering lane named them and noted the earlier remainder listed none of them: `UNPLANNED_OPERATIONS` in `crates/tiler-compiler/src/policy.rs`, `gather_is_absent_from_the_governed_fusion_roles`, and `gather_is_absent_from_the_real_request_recognition_operation_set`. **A change that flips the capability without flipping these leaves two tests asserting the opposite of the tree.**

**Fact — the tags this lane consumes were verified free by the coordinator at `754b63fb`.** Compiler access-relation tag `0x06` and the governed lowering capability row 21→22. Tag spaces here are **per-frame, not global** — `TAG_LINEAR_IDENTITY` and `TAG_COVERAGE_PADDED` are both `0x01` deliberately — so a value appearing in another frame is not a collision. A *reserved* value is: in the schedule frame, `0x09` is retired-and-never-reused and `0x36` in the reduction frame is reserved for `CooperativeContractionSplit`. Re-derive whatever you take.

## Required work

- Re-audit every Fact at your base with a per-Fact verdict before editing.
- Deliver `NormalizedOutput::Gather` / `NormalizedOutputSubject::Gather`, the `gather-f32.v1` output subtag, the access-relation tag, the invocation-validation vocabulary, and the governed capability row — to the **accepted** spellings, not the remainder list's paraphrase.
- Flip all three absence-recording sites in the same change as the capability.
- Land the ADR 0108 schedule-clause amendment with its catalog and contract sweep. AGENTS.md: applying an ADR means aligning status, catalogs, contracts, terminology, and released graph edges — read the affected documents in full before declaring the sweep complete.
- State every identity domain that steps and every one that does not, with the derivation, and **recompute pins on the merged tree**, not on your base. The layer beneath stepped nothing; do not assume that carries.

## Evidence

- Perturb the subject separately for each new refusal and quote the failure text. The lane below this one found a rule that its own read-count gate made **unreachable by pigeonhole** — it caught that by asking what it would take for each rule to say *no*, which is the check to run here too.
- Before trusting any check, state what it would take for it to say *no*, and confirm that case is reachable.
- Size enumerations from the type. `core::mem::variant_count` makes a widened vocabulary a build error at the enumeration rather than a census that silently shrinks; the lane below replaced a five-of-twelve tag sample with exactly that, because a sample could not have shown the tag it needed was free.

## Non-goals

The oracle's independent proof-identity check — blocked on a public-boundary decision and owned by [`decide-how-the-oracle-independently-checks-a-gather-proof-identity`](decide-how-the-oracle-independently-checks-a-gather-proof-identity.md). Any artifact, manifest, cache, or Metal surface. Re-opening the accepted public surface.

## Closes when

A gather request reaches the admitted relation, no site records gather as absent while the capability admits it, the ADR sweep is complete against documents read in full, every identity consequence is derived on the merged tree, each refusal has been watched firing, and the repository gate is green.
