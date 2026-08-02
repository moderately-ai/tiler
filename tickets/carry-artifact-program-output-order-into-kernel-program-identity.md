---
id: carry-artifact-program-output-order-into-kernel-program-identity
title: Carry artifact program output order into kernel-program identity
status: todo
priority: p1
dependencies: []
related: [admit-ordered-multi-output-programs-at-the-compiler-request-boundary, implement-general-dag-partitioning]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity, artifacts]
---
## User-visible outcome

Two verified kernel programs that publish the same named outputs in different orders carry different canonical identities, so an artifact's ordered output interface is part of what its identity means rather than something a consumer must re-derive.

## Why this exists

**Fact — the artifact identity encoder sorts its output records.** `crates/tiler-ir/src/program/model.rs:1778-1792` builds one `Vec<u8>` per output holding the output key and the canonical id of the published value, calls `outputs.sort_unstable()`, and folds the sorted list. Two `KernelProgram`s differing only in the order of their `push_output` calls hold the same key set bound to the same values, so they produce the same sorted list and the same `CanonicalKernelProgramIdentity` bytes — while `KernelProgram::outputs()` (`:816-822`) still yields them in the two different declared orders.

**Fact — nothing else recovers the order.** `verify_outputs` (`crates/tiler-ir/src/program/verify.rs:584-608`) checks coverage as a *set* in both directions: every semantic interface output must be published by some record, and every `ValueRole::Output` value must be named by some record. It never relates the published order to `SemanticSubject::outputs`, which `interface_output_shapes` (`crates/tiler-ir/src/program/builder.rs:1315-1332`) reads from the semantic program *in order*. So the declared order is observable through `outputs()`, unconstrained by verification, and absent from identity.

**Fact — the semantic layer treats output order as identity, which is the asymmetry.** `crates/tiler-ir/src/semantic/identity.rs:128-132` encodes the output list in declaration order with no sort, and `canonical_traversal` (`:143-170`) *seeds* the canonical value numbering by visiting outputs in declaration order, so order reaches the graph identity twice over. `crates/tiler-compiler/tests/multi_output_boundary.rs::two_programs_differing_only_in_output_order_have_distinct_identities` pins that behaviour. The artifact layer discarding what the semantic layer preserves is the defect: a lowering that permuted an artifact's output interface would be invisible to the identity that is supposed to name it.

**Inference — latent today, live the moment multi-output lands.** `crates/tiler-compiler/src/program.rs:1254-1261` refuses any program whose `core.outputs().len() != 1`, and a one-element list sorts to itself, so no program the compiler currently produces can exhibit the collision. `tiler-ir` is a library and does not carry that restriction: `program::tests::storage_reuse_is_admitted_only_with_an_explicit_handoff` already builds and verifies a two-output program through the public builder. The collision is therefore reachable through `tiler-ir`'s own surface now, and becomes reachable through the compiler as soon as [`implement-general-dag-partitioning`](implement-general-dag-partitioning.md) can plan ordered multi-output covers.

## Boundaries

- Decide and state which of the two readings is the contract, because the current code is neither: either the declared publication order is meaningful, and identity plus `outputs()` must both carry it, or it is incidental, and `verify_outputs` must pin the published order to `SemanticSubject::outputs` so `outputs()`'s "ordered" claim is true. Do not leave order observable, unconstrained, and unencoded.
- The sort is not obviously wrong in isolation — the sibling `edges` and `splits` lists above it are sorted deliberately, each with a comment saying the entity "names entities rather than being named by one, so its declaration position carries no meaning identity should preserve". Establish why an output record is different (it *is* named, positionally, by the caller's interface) rather than deleting a sort that its neighbours justify.

## Identity domain consequence

This is an identity-domain step and must be executed completely or not at all. The subject is `PROGRAM_DOMAIN` at `crates/tiler-ir/src/program/model.rs:1374`, currently `tiler.kernel-program.v6`; `crates/tiler-ir/src/program/mod.rs:44` is the ledger paragraph recording each prior step and must move in the same commit, as must `CanonicalKernelProgramIdentity`'s own step list.

Note the appends-only claim is **false** here and should not be asserted: a one-output program's bytes do not move, but `tiler-ir`'s existing two-output fixtures do, so the version steps. Recompute every pinned identity on the tree the step lands into and enumerate each moved pin in the report.

## Required failure-path evidence

Each observed failing against an accepted neighbour: two programs differing only in `push_output` order yield distinct identities, and re-declaring one of them in its original order reproduces its identity byte for byte; a one-output program's identity is unchanged by everything except the domain step itself; and whichever ordering rule is chosen, a program violating it is rejected by `verify_outputs` with a named diagnostic rather than silently accepted.

## Closes when

The chosen contract is stated in the program module's own documentation, identity distinguishes two publication orders (or verification forbids the second order from existing), the domain version and both ledger sites moved together in one commit with every moved pin enumerated, and `make full` passes.
