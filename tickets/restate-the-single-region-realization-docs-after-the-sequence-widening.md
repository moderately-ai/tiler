---
id: restate-the-single-region-realization-docs-after-the-sequence-widening
title: Restate the single-region realization docs after the sequence widening
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-restate-t
lease_expires_at: 1786159357
---
## User-visible outcome

The index-law and refinement docs describe the staged-realization model as landed, so a reader stops treating single-region accessors and claims as total.

## Why this exists (drift audit 2026-08-06 — one coherent story from the staged-law landing, wanting one reader who holds the whole sequence model)

The cluster, each verified by the audit at source: refinement.rs:1186/1324 claims "the one-stage realization every registered law produces" (the staged law registers in this file's own tests); law.rs:1933 and refinement.rs:4655 claim the single-region `verify` path is "the one the compiler drives today" (the compiler drives `verify_sequence`; the only `.verify(` sites in tiler-compiler are cfg(test)); legality.rs:38-41's module header states unconditional oracle-evaluability that `single_region()`'s own doc refutes for chains; oracle.rs:426 counts two uninstalled scalar capabilities where the registry has five; law.rs's module header describes region-identity comparison where the authority compares sequence identities; law.rs:270 promises a pre-interface refusal only `verify` delivers; refinement.rs:72-79's MAX_OPERAND_BINDINGS rationale predates per-stage bindings and no longer explains the constant's safety; refinement.rs:1478's # Errors omits the first refusal its body returns.

## Closes when

Every listed claim states sequence-model truth, the # Errors lists match their bodies, and the bindings-constant rationale derives the per-stage population.

## Per-Fact audit at base `6d1bd6e80564e66e5936bccd71611abf8a9da3a6` (2026-08-07)

Every Fact above was re-read at this base. **Every line citation in the section above is stale**; the substance survives in seven of eight cases, and one premise is false in the direction of understatement. Anchors below replace the line numbers.

1. **"the one-stage realization every registered law produces"** — substance **verified**, citation false, and its stated *reason* **false**. Both sites exist, at `crates/tiler-ir/src/index/refinement.rs "The accessors are named for what they return"` and `crates/tiler-ir/src/index/refinement.rs "For a one-stage realization this is the only region"` — drift of roughly +170 and +180 lines from `1186/1324`. The parenthetical "the staged law registers in this file's own tests" is **false**: the staged laws register in the *standard* semantic authority. `the_family_region_sequence_query_agrees_with_the_resolved_law` builds `FrozenSemanticRegistry::standard()` and asserts `family_realizes_region_sequence` is true for both `tiler::rms-norm-f32@1` and `tiler::softmax-f32@1`. Run at this base: `1 test run: 1 passed`.
2. **"the one the compiler drives today"** — substance **verified**, citations false, parenthetical **imprecise**. The phrase does not occur verbatim anywhere; the claim lives at `crates/tiler-ir/src/index/law.rs "a_staged_law_refuses_the_single_region_realization"` (was `law.rs:1933`, actually ~3130) and as a comment at `crates/tiler-ir/src/index/refinement.rs "And through the single-region entry point"` (was `refinement.rs:4655`, actually ~5024). The claim is false as stated: `refine_index_region` drives `verify_sequence` at `crates/tiler-compiler/src/legality.rs "verify_sequence(capability.authority().refinement()"`, which is non-test — `legality.rs`'s `#[cfg(test)]` opens at line 1201 and both `.verify(` sites there are 1558 and 1677. The parenthetical "the only `.verify(` sites in tiler-compiler are cfg(test)" is **imprecise**: `crates/tiler-compiler/src/explain.rs` has non-test `.verify()` calls, but on the explain trace, not on `ResolvedIndexRealization`. True as restricted to this method.
3. **`legality.rs:38-41` module header** — **not audited, out of scope.** `crates/tiler-compiler/**` is `implementation/compiler`, not this ticket's `implementation/ir`, and the crate has live branches. Needs its own ticket.
4. **`oracle.rs:426` scalar-capability count** — **not audited, out of scope.** `crates/tiler-reference/**` is `implementation/reference`. Needs its own ticket.
5. **law.rs module header describes region-identity comparison** — **verified.** `crates/tiler-ir/src/index/law.rs "builds the expected canonical region"` said region where `verify_sequence` compares `expected.identity() != realization.identity()` over the sequence.
6. **"a pre-interface refusal only `verify` delivers"** — substance **verified**, citation false (`law.rs:270`; actually ~505, at `crates/tiler-ir/src/index/law.rs "realizes_region_sequence"`). The doc's "Asked *before* any interface checking" describes one of three callers: `verify` orders it first, while the two public queries `family_realizes_region_sequence` and `realizes_region_sequence` check no interfaces at all, and `verify_sequence` never asks it.
7. **`MAX_OPERAND_BINDINGS` rationale** — substance **verified**, citation **false in both name and line**. The constant is `MAX_INDEX_REFINEMENT_OPERAND_BINDINGS` at ~101-108, not `MAX_OPERAND_BINDINGS` at 72-79. The retired rationale claimed the value "deliberately matches the region boundary population ceiling *so* an alias-expanded binding inventory cannot exceed the boundary inventory the region itself may retain" — a derivation that was already wrong before the widening (`operand_binding_population_is_bounded_before_collection` fixes a binding inventory of 16,384 against a distinct expanded population of 1,024) and that the widening adds a third multiplier to: `bind_operands` pushes one binding per (operand use, expanded component, **reading stage**) triple.
8. **`# Errors` omits the first refusal its body returns** — substance **verified**, citation false (`refinement.rs:1478`; actually `verify_sequence` at ~1744). Its `# Errors` named "scalar authority, effect, ordered tensor interfaces, or the realized sequence", while the body's first statement is `check_lowering_authority`, whose six refusals — operation, attribute, numerical-contract, occurrence, capability-signature, and semantic-authority mismatch — are none of those. `verify`'s own `# Errors` inherited the same omission through its "Otherwise" clause.

**Additional in-scope defect of the same kind, found while auditing Fact 1 and fixed here.** `crates/tiler-ir/src/index/law.rs "of this law's twelve variants whose realization is a region"` read "one of the two of this law's ten variants ... the other eight are single-region". The enum carries **twelve** variants and `realizes_region_sequence` matches **three** of them, so the correct reading is three of twelve, other nine. This is exhaustive finite evidence: the predicate is a total match over a closed enum.

**Out-of-scope sibling of that defect, not edited.** `crates/tiler-compiler/src/region.rs "nine of the ten registered laws are"` carries the same stale count and is `implementation/compiler`. Reported, not touched.

## Outcome

Restated in `crates/tiler-ir/src/index/law.rs` and `crates/tiler-ir/src/index/refinement.rs` only; no `docs/` file was touched, so this delta is `crates/`-only and does **not** carry the green gate. Neither file carries any dated correction, so the house convention followed here is plain restatement with the retired reasoning recorded in this ticket and the commit — the dated-strike convention lives in `crates/tiler-conformance` and `crates/tiler-cache`, not in `tiler-ir`.

Maturity and evidence tiers of the restated claims: three of twelve law variants realize sequences, and the two public query surfaces project that — **exhaustive finite evidence** over a closed enum. `tiler::rms-norm-f32@1` and `tiler::softmax-f32@1` carry staged laws in the standard authority — **tested guarantee**. The compiler driving `verify_sequence` rather than `verify` — **implemented support**, one non-test call site. The binding population being an independent ceiling — **tested guarantee** at its two boundary cases.

**Support-matrix navigation note.** This advances no support-matrix or maturity row. It is a correction to doc comments describing an already-landed capability; the staged-realization row was advanced by the landing this ticket restates, not by this ticket. No ledger update is owed.
