---
id: decide-the-partitioned-copy-scheduled-region-public-surface
title: Decide the partitioned-copy scheduled-region public surface
status: in-progress
priority: p1
dependencies: [admit-the-concatenate-family-into-the-scheduled-region-vocabulary, accept-the-partitioned-concatenate-realization-law]
related: [admit-the-partitioned-copy-scheduled-region, admit-an-explicit-non-arithmetic-region-and-delivery-state, lower-the-partitioned-copy-region-through-kernel-ir, plan-concatenate-through-one-partitioned-copy-entry]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, schedule, concatenate, identity, verification]
claimed_from: todo
assignee: worker-partitioned-copy-surface
lease_expires_at: 1787060886
---
## Outcome

A Pareto-complete decision packet fixes the exact public scheduled-region representation for the already-accepted one-region partitioned-copy design, including construction, verification, diagnostics, identity, and the first supported population. It asks Tom one exact question only after an independent derivation confirms that no correctness-bearing API, proof, or identity choice remains implicit.

This ticket prepares the decision; it does not implement a draft surface or treat the accepted semantic topology as acceptance of an unstated Rust API.

## Exact-base Fact audit — 2026-08-17 at `783e9b5b743afafdf4957396dbcfdb2f4c34565c`

Re-read in full: [`admit-the-concatenate-family-into-the-scheduled-region-vocabulary`](admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md), [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md), [`repair-the-scheduled-vocabulary-census-and-concatenate-law-standing`](repair-the-scheduled-vocabulary-census-and-concatenate-law-standing.md), and [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md), plus the current schedule model, builder, diagnostics, request, physical-construction, and identity owners.

1. **Verified — the semantic topology is accepted.** Tom accepted one whole concatenate occurrence as one scheduled `PartitionedCopy` program, one verified KIR, one backend entry, and one dispatch. The accepted law already retains ordered operand members, zero-extent members, repeated occurrences such as `concat(x, x)`, distinct input bindings, and partitioned write-ownership evidence.
2. **False if read as exact public-surface authority — the accepted topology did not select the Rust representation.** The record uses `RegionProgram::Numerical { ... } | PartitionedCopy(...)` conceptually, while the downstream numerical-state decision explicitly says that exact names follow the source audit. No accepted record fixes the public copy program/member/subdomain record names, fields, visibility, exhaustiveness, constructors, accessors, or builder transition.
3. **Verified — current schedule construction is arithmetic-shaped and total.** `IndexRegion`, anchor `pub struct IndexRegion`, carries mandatory `scalar_program: ScalarProgram` and `numerical: NumericalRealization`. `ScheduledRegionBuilder`, anchor `pub struct ScheduledRegionBuilder`, stores both as required `Option` slots; `assemble` refuses either as `IncompleteRegion` and then constructs the one current `IndexRegion` shape.
4. **Verified — current diagnostics do not own partitioned-copy failures.** `ScheduledRegionDiagnostic`, anchor `One deterministic whole-region schedule-verification failure`, has only the current arithmetic/access/proof/topology vocabulary. The implementation ticket requires distinct overlap, gap, overflow, member, prefix, dtype, rank, and correspondence refusals, but no accepted source assigns their exact public variants, payloads, stable `rule()` strings, or precedence.
5. **Verified — current identity has no reserved copy spelling.** Schedule identities open `tiler.schedule.v6\0`; the present `IndexRegion` encoding is the arithmetic-shaped scalar-program-plus-numerical grammar. Neither the accepted topology nor the implementation ticket assigns a copy-program tag, member framing, proof reference encoding, or whether an append-only sum preserves v6 versus requires a coherent domain step.
6. **Verified — the request and physical consumers are arithmetic-only today.** The compiler request subject opens `tiler.compiler.request-subject.v6\0`; request normalization, physical construction, schedule verification, resource derivation, and assembly match the current scalar/numerical form. There is no `RegionProgram` or `PartitionedCopyProgram` type in `crates/` at this base.
7. **Imprecise — “partitioned copy” does not by itself bound the first public population.** The named producer is governed `tiler::concatenate-f32@1` at arities `2..=8`, with static shapes and the accepted partition law. A generic public name could also appear to admit other dtypes, slice/window copies, symbolic partitions, several outputs, or caller-authored partition records. Those populations are not authorized by the accepted concatenate decision and must be explicitly included or excluded.
8. **Verified — implementation cannot derive the exact surface mechanically.** Correct alternatives include different ownership of ordered members and proof subjects, different transactional builder APIs, and different public diagnostic/identity vocabularies. They can all preserve the accepted one-kernel semantic topology while imposing different constructible states, compatibility, and host-memory costs. Choosing among them is a consequential public/identity decision under ADR 0075.

The implementation purpose remains sound. The repair is to split this missing authority into a prerequisite, not to replace the accepted one-region outcome.

## Required decision packet

Apply the repository decision-packet readiness gate at the exact current base. Read every construction, validation, consumption, refusal, identity, schema, and dependency path rather than treating the conceptual enum spelling as settled.

The packet must fix, at minimum:

- the exact public `RegionProgram` sum and the complete copy/member/subdomain/binding records, including visibility, exhaustiveness, constructors, accessors, limits, and transactional builder states;
- the single canonical association between ordered concatenate operands, deduplicated boundary tensors, index-region roots, source/destination subdomains, bounds proofs, and partitioned ownership evidence;
- the initial supported population and every fail-closed exclusion, including dtype, arity, shape source, rank, output count, zero extents, repeated operands, symbolic partitions, slice/window copies, and generic caller-authored copies;
- exact verifier diagnostics, payloads, stable rule strings, precedence, and which malformed states are unrepresentable versus representable-and-refused;
- exact schedule and compiler-request identity tags, framing, old-byte invariants, domain/version consequences, pins, and downstream provenance movement;
- the total consumer migration through request normalization, physical construction, resource derivation, KIR handoff, explanation, and the already-accepted `FloatingPoint | BitPreservingCopy` downstream projection; and
- host runtime and memory bounds for every retained member/proof collection.

Enumerate the genuine frontier: status-quo typed refusal, the narrow governed static-F32 concatenate slice, a broader reusable partitioned-copy surface if independently justified, further bounded research, and typed deferral where applicable. Eliminate any option that lets a caller mint proof authority, infer association from ordering, fabricate numerical state, or silently admit a population the verifier cannot prove.

For every survivor, state the strongest counterargument, reversal evidence, subject perturbations, identity/public consequences, and follow-up graph. Use an independent derivation before presentation because a wrong association or tag can silently misidentify or misroute a program.

## Stop condition

Do not edit production types while this ticket is unresolved. If the source audit exposes another missing proof or downstream authority, file and order that prerequisite rather than embedding it as an implementation detail. Only Tom accepts the exact public surface.

## Closes when

Tom accepts one exact included/excluded public surface, rejects the expansion, or explicitly defers it; the accepted answer is recorded with provenance; the implementation and downstream graph encode every prerequisite; and no API, diagnostic, identity, or population choice remains for the implementer.
