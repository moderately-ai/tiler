---
id: repair-the-records-the-sourced-semantic-shape-falsifies
title: Repair the records the sourced semantic shape falsifies
status: in-progress
priority: p1
dependencies: [carry-a-sourced-shape-on-semantic-values]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [contracts/artifacts, contracts/decisions, contracts/navigation, research/shapes, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, shapes, identity, correction]
claimed_from: todo
assignee: coord
lease_expires_at: 1786174094
---
## User-visible outcome

Every document outside `carry-a-sourced-shape-on-semantic-values`'s seven scopes that states something the landing made false says something true instead, and each repair is dated to the landing that caused it.

## Why this exists

[`carry-a-sourced-shape-on-semantic-values`](carry-a-sourced-shape-on-semantic-values.md) landed the sourced semantic shape and the fifth `SemanticIdentity` subject together on 2026-08-07. It held `implementation/{ir,compiler,reference,artifact,frontend,build}`, `contracts/foundation`, and `project/tickets`, and it repaired `docs/ir.md` inside that set. The records below are outside it and were deliberately left alone rather than edited across a scope boundary.

**Every claim below was read at the landing commit; none is inferred from the diff.**

## What the landing changed

- `tiler.semantic-graph.v2 → v3`; every extent is tagged through `SourcedShape::encode`, so a wholly static program's graph bytes move.
- `tiler.compiler.request-subject.v5 → v6` and `tiler.program-alternative.v1 → v2`, both because `SemanticIdentity` gained a fifth subject and both preimages enumerate the subject set positionally.
- `tiler.shape-env.v3`, `tiler.artifact-program.v15`, the envelope, the manifest schema, `tiler.ir.index-realization-authority.v1`, and the obligation domain all **stay**.
- A semantic *input* extent may name a declared `ShapeEnv` symbol. An *inferred result* extent may not: `ValueFact` still carries a fixed `Shape`, and a symbolic value is refused as an operation operand.

## The repairs, per document

### `docs/artifact-abi.md` — `contracts/artifacts`

1. **`:279` states a measurement that is now wrong by one.** "differing at exactly **68** byte positions" is **67** on the landed tree. Nothing structural moved — the two tag pairs and the two thirty-two-byte digests are the same 68 positions — but one more digest byte now coincides by chance, which is exactly what that paragraph's own "a digest byte can coincide" reasoning anticipates. `DIFFERING_CARRIER_POSITIONS` in `crates/tiler-artifact/src/program/codec/tests.rs` already carries 67 and a comment explaining the coincidence; this doc is the other half the test's own failure message names.
2. **`:231` names `tiler.semantic-graph.v2`** in the sentence "The semantic graph has encoded its outputs in declaration order all along — `tiler.semantic-graph.v2` writes the output list unsorted…". The claim survives; only the domain string moved to `v3`.
3. **`:414` stays true and is now true for a reason worth writing down.** "Only the three reached subjects travel" holds because `SemanticIdentity`'s *fifth* subject is also omitted, and that omission is sound only because no program whose interface names a symbol reaches `ArtifactProgramBuilder::new` — which now refuses one by name as `ArtifactBuildError::SymbolicSemanticInterface`, pinned by `a_symbolic_semantic_program_never_reaches_the_artifact_builder`. Name the fifth subject in the exclusion list and cite the refusal, so a later reader does not have to re-derive why a subject may be dropped.

### `docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md` — `contracts/decisions`

1. Two references to `tiler.semantic-graph.v2` (`:22` and `:61`) → `v3`. Both are quoted reasoning that survives.
2. `:22` names `tiler.compiler.request-subject.v5` → `v6`.
3. **`:22`'s measured identity-growth ladder moves and needs a re-run, not an edit.** It records kernel-program identity as exactly `3530n + 723` bytes over 2..=62 operations, measured at `spikes/program-planning/identity-growth`. Every semantic extent now costs one more byte and the graph identity travels twice per envelope, so both coefficients move. The record's *decision* rests on the shape of the curve rather than its coefficients, so nothing is superseded — but the numbers derived from it (the 64 MiB program bound at 19,011 operations, the embedding-ceiling crossing between 148 and 149, the 219,583 bytes at the governed budget of 62) are all stale until the spike is re-run on a tree at or after the landing.

### `docs/research/shapes/symbolic-semantic-extents.md` — `research/shapes`

This is the record the whole delivery chain derives from, and the landing consumed two of its seven rows. It is the largest repair.

1. **`:32` Fact — "the semantic layer has no symbol in it at all"** is now false. `SemanticProgramBuilder::input_sourced` and `input_resolved_sourced` take `Vec<SourcedExtent>`, the builder takes an `Arc<ShapeEnv>` at construction, and `SemanticProgram::shape` returns `&SourcedShape`. The `grep` the Fact offers as its check now returns real code. The Fact's *narrow* half survives and should replace it: `ValueFact` still takes a `Shape`, so no symbol reaches an inferred result.
2. **`:34`'s correction block** repairs the check for that Fact and inherits its staleness.
3. **`:38` Inference — "the gap is therefore a vocabulary gap"** is closed on the input side. `crates/tiler-macros/src/region.rs`'s `ProgramEvidence::DeferredSymbolicExtent` still refuses, because constructing a symbolic region as a semantic program is a *later* row of this record's own chain; say which half closed.
4. **`:81`** proposes `SemanticProgramBuilder::input_sourced::<T>(InputKey, Vec<SourcedExtent>) -> Result<Value<T>, BuildError>`. **It landed with exactly that signature** — worth recording as a proposal that survived contact.
5. **`:90` Fact — "`SemanticIdentity` owns exactly four separately typed subjects"** is now five; the fifth is `ShapeEnvIdentity`, reused rather than re-wrapped.
6. **`:92` Fact — the untagged encoding** and the domain `tiler.semantic-graph.v2\0` are both superseded; the cited `identity.rs:377` has drifted.
7. **Decision `A4`** asked whether the fifth subject is optional or total. **Resolved as total** over `ShapeEnvBuilder::new().build()`'s identity, with the elimination recorded in the landing ticket: optional would give "declares no symbols" and "has an empty environment" two spellings for one fact and would put a presence tag into every downstream enumeration. Record the answer against the decision.
8. **Decision `A5`** — the domain advance — **landed as stated**, `v2 → v3` tagged with `tiler.shape-env.v3` held.
9. **The delivery table at `:232`**: row 2 (`carry-a-sourced-shape-on-semantic-values`) and row 4 (`fold-the-shape-environment-into-semantic-identity`) are both discharged, the second by supersession into the first. Row 3 (`resolve-semantic-shape-inference-over-symbolic-extents`) is what the landing's boundary now waits on.
10. **`:109`'s deferred question and its trigger** — splitting the environment identity into graph-meaning and interface halves — is untouched and should be restated as still deferred, so a reader does not take the fifth subject's arrival for its resolution.

### `docs/research/shapes/transformer-operation-and-shape-surface.md` — `research/shapes`

`:130` states that "all seven are `todo`" for the delivery chain, and cites `SemanticProgramBuilder::input` and `input_resolved` taking a `Shape` at `crates/tiler-ir/src/semantic/program.rs:493` and `:516`. Two of the seven are now discharged, the constructors have sourced siblings, and both line references have drifted. The record's corrected claim — that the chain's *last* link is what buys one artifact across bound extents — is unaffected and should be preserved.

### `docs/research/program-planning/complete-model-ingestion-and-execution.md` — `research/program-planning`

1. **`:142`** cites `compute_graph_identity` at `identity.rs:87` lines 103 and 125 and concludes "thirteen, not three" artifact identities "under today's fixed-extent vocabulary". **The conclusion survives intact** — the C1 row's shapes are inferred results, which are still fixed — but its stated cause has narrowed to results, and property (a) it names is now half-delivered. The line references have drifted.
2. **`:220`** names `tiler.compiler.request-subject.v5` (→ `v6`) and records the pinned explain request qualifier as `689c3aefc30f48d3` → `8966151e455093ea` at `crates/tiler-compiler/src/explain.rs:4183`. **That pin was already stale before this landing** — it read `f99d1e5eb387f42f` at the landing's base commit `0132c0c3` — and it now reads `940c09e0821665a6`. Repair it once against the tree rather than replaying the chain.
3. **`:285`** carries the same "under today's fixed-extent vocabulary" parenthetical as `:142`.

### `docs/roadmap.md` — `contracts/navigation`

`:471` states, in the softmax row: "`crate::shape::Extent` is a `u64` newtype, so a semantic value cannot carry a symbolic extent at all: the L3′ rule 'an extent symbol with no proved upper bound refuses' has nothing to refuse here". **The premise is now false for an input** and true for an inferred result — which is the half the growing extent `S` actually needs, since `S` reaches the softmax as an operand rather than a program input. So the row's conclusion ("Every distinct `S` is a separate compiled artifact by construction") **survives**, and only its stated reason narrows. `:481` separately records `de9ad4cc087697d8` as the explain digest at `explain.rs:3883`; that too was already stale at `0132c0c3` and is a pre-existing drift rather than one this landing caused.

## Evidence

- Each edited sentence read in full at the landing commit before and after, not matched by grep.
- `make citations` green — it covers `docs/**`, and several of these repairs move cited line numbers.
- The `docs/artifact-abi.md` count of 67 reproduced by running `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` rather than copied from this ticket.
- The `0104` ladder re-run from `spikes/program-planning/identity-growth` on a tree at or after the landing, with the host and toolchain pin recorded, or the figures marked stale with the re-run deferred and a trigger — not silently left as they are.

## Not in scope

No behaviour change, no identity domain moves, no crate is edited. A repair that turns out to need a source change is a separate ticket.
