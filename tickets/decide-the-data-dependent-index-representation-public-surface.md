---
id: decide-the-data-dependent-index-representation-public-surface
title: Decide the data-dependent index representation public surface
status: in-progress
priority: p1
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, name-the-fact-source-on-retained-write-ownership-evidence]
related: [admit-the-selected-data-dependent-index-representation, revise-adr-0108-with-a-complete-data-dependent-index-vertical, admit-an-invocation-scoped-gather-index-validation-receipt]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, indexing, identity, correctness]
claimed_from: todo
assignee: worker-gather-surface
lease_expires_at: 1786969799
---
## User-visible outcome

Before data-dependent gather enters the verified index/schedule vocabulary, Tiler has one accepted exact public representation, checked address-only read association, proof authority, diagnostic surface, and identity migration. No implementation guesses a Rust spelling or treats a dynamic invocation obligation as a timeless proof.

## Discovery — 2026-08-16, exact main `f46ac65cc6050c6804f9376f2fb86e44430c8c31`

A source-first pre-dispatch audit of `admit-the-selected-data-dependent-index-representation` found that accepted ADR 0108 chooses the append-only tagged-access semantic design but explicitly says `This decision accepts no public Rust spelling`. The implementation necessarily changes public index-law, builder, lowering-context, access-view, error, schedule-relation, and proof vocabulary. ADR 0075 reserves that exact surface for Tom.

The audit also found unresolved correctness authority:

- a gather has an F32 value source plus an address-only U32 index read, while current pointwise verification pairs every read with one scalar leaf;
- no exact multiplicity/order/alias rule associates that extra read without either type-confusing it as a value input or silently dropping it;
- no current proof type can establish dynamic gather bounds, and no closed producer is authorized to mint `StaticallyProved`;
- a dynamic invocation-validation obligation must stop before executable refinement and must not become `Unknown` or a timeless receipt; and
- exact law/schedule/proof/request tags and the frozen-registry request/cache cascade are unselected.

Current source also still contains stale proposed/returned ADR 0108 maturity and 5/3/3 census prose. The implementation ticket has a hard live scope conflict with `name-the-fact-source-on-retained-write-ownership-evidence`, whose worktree changes the same index proof/model paths.

## Required decision packet

After the write-ownership fact-source dependency lands, re-audit exact current source and produce a Pareto-complete Tom packet that fixes:

1. the exact public tagged gather access, index-law, builder, lowering-context, read-view, error, schedule-relation, and proof type names, fields, constructors, and accessors;
2. a checked-sum access model that distinguishes scalar value reads from the address-only U32 index read, including order, multiplicity, aliasing, reachability, rank, axis, shared-domain, and source-coordinate validation;
3. the closed authority that derives `StaticallyProved`, the exact cases it may prove, occurrence/access binding, proof identity, and the rule that every other accepted gather retains `InvocationValidationRequired` and stops before executable refinement;
4. exact append-only tags and identity consequences for index access, index law, schedule relation, bounds/proof evidence, compiler request/explain identity, frozen capability/registry rows, and cache subjects;
5. preservation of every old direct-access byte and verifier guarantee, with fresh gather injectivity pins and independent subject perturbations;
6. exact public diagnostic ownership and precedence;
7. the complete unsupported population; and
8. ADR 0108 application across decision catalogs, IR/open-question contracts, source maturity prose, census tests, and downstream graph state.

Compare the strongest complete replacements, narrower fail-closed slices, further bounded research, and deferral. Eliminate any option that lets a caller assert proof authority, loses the address-only read association, defaults a dynamic obligation, aliases request identity, or silently reaches dispatch.

## Known semantic boundary

ADR 0108 currently supports only the selected tagged design: F32 source, U32 index, one gathered axis, shared domain, complete index coordinates and direct source coordinates, initially program-input sources, static semantic result shape, rank-zero index allowed and nonzero source rank, duplicates allowed. Keep signed/other unsigned/float indices, clamp/wrap/truncation, inferred axis, recursive or multiple indirect reads, scatter, data-dependent result shape, mutable-device zero-copy, caller assertions, inline-kernel validation, dynamic dispatch receipts, artifact/runtime carriage, and Metal emission outside this decision unless a separately accepted dependency owns them.

## Graph and stop

This ticket depends on the accepted ADR and the active write-ownership proof migration so its source/identity audit occurs on the real next base. Only after Tom accepts the exact surface may `admit-the-selected-data-dependent-index-representation` return to implementation readiness.
