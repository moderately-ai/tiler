---
id: repair-the-records-the-sourced-semantic-shape-falsifies
title: Repair the records the sourced semantic shape falsifies
status: done
priority: p1
dependencies: [carry-a-sourced-shape-on-semantic-values]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [contracts/artifacts, contracts/decisions, contracts/navigation, research/shapes, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, shapes, identity, correction]
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

## Per-Fact audit at base `cc667626`, before any edit

Every claim above was re-read at this base rather than carried. The ticket's own header says "Every claim below was read at the landing commit" — that is exactly the problem: **the landing commit is 268 commits behind this base, 41 of them touching `crates/`,** and three of the ticket's claims did not survive the interval. Line citations are retired rather than refreshed wherever the anchor is a symbol name.

| # | Claim | Verdict |
| --- | --- | --- |
| 1 | `tiler.artifact-program.v15` **stays** | **false** — `ARTIFACT_DOMAIN` reads `b"tiler.artifact-program.v16\0"` and the manifest schema is 16.0. True at the landing; stepped since by a later change, not by this one. |
| 2 | `tiler.semantic-graph.v3`, `request-subject.v6`, `program-alternative.v2`; `shape-env.v3`, envelope, `index-realization-authority.v1`, obligation domain hold | **verified**, each read from its `const` |
| 3 | An input extent may name a symbol; an inferred result may not; a symbolic value is refused as an operand | **verified** — `input_sourced`/`input_resolved_sourced` take `Vec<SourcedExtent>`; `ValueFact` holds `Shape` and `ValueFact::shape` returns `&Shape`; `BuildError::SymbolicOperandUnsupported` |
| ABI-1 | `docs/artifact-abi.md:279` reads 68 and should be 67 | **false, and doubly so.** The paragraph is at `:285`, it already reads **68**, and `DIFFERING_CARRIER_POSITIONS` also reads **68**. The constant went 68 → 67 at the landing and **back to 68** at `e4041047`; `7e5fdb0e` had already rewritten the paragraph to carry the whole history, including the 67 it read "for the span between `tiler.semantic-graph.v3` and `tiler.artifact-program.v16`". Doc and test agree. **No edit.** Had this been worked from the ticket, a correct figure would have been replaced by a stale one. |
| ABI-2 | `:231` names `tiler.semantic-graph.v2` | **verified at `:237`.** Claim survives, domain moved. Dated beside — true at that ledger step. |
| ABI-3 | `:414` "Only the three reached subjects travel" stays true; name the fifth subject | **verified at `:439`.** Sentence unchanged and still true; `a_symbolic_semantic_program_never_reaches_the_artifact_builder` exists at `crates/tiler-artifact/src/program/tests.rs`. Recorded as an addition, not a correction. |
| 0104-1 | Two `tiler.semantic-graph.v2` references, at `:22` and `:61` | **false, and never true.** There is exactly **one**, at `:61`, and there was exactly one at the landing commit too. `:22` carries `request-subject.v5`, not the graph domain. |
| 0104-1b | (consequence) rewrite the `v2` reference to `v3` | **must not be done.** The single reference sits inside a **verbatim quotation** of the doc comment on `IndexRefinementExecutableCoverageIdentity` in `crates/tiler-ir/src/index/refinement.rs`, which still reads `v2`. Rewriting it would make the ADR misquote its own source. The stale text is the source comment — **filed below, not fixed here.** |
| 0104-2 | `:22` names `request-subject.v5` → `v6` | **verified** |
| 0104-3 | The ladder needs a re-run, not an edit | **verified, and re-run.** See the Outcome section. |
| SSE-1..10 | The ten items against `docs/research/shapes/symbolic-semantic-extents.md` | **verified**, every line citation exact at this base except the delivery table pointer (`:232` is row 4; the table opens at `:227`) |
| SSE-miss | — | **Three falsifications the ticket does not enumerate**, all found by rerunning the record's own checks rather than by reading its list: (a) the `:94` Fact "`ShapeEnv` reaches neither the artifact crate nor the cache crate" is **false for `tiler-artifact`** — four hits, all refusal-side — and its positive control now returns six crates, not five; (b) Reproducible check **1** returns **63 lines** where it claims nothing, and cannot be narrowed back; (c) Reproducible check **2** greps `fn encode_shape`, which no longer exists in that file at all. |
| TOSS-1 | `:130` "all seven are `todo`"; `program.rs:493`/`:516` drifted | **verified.** Statuses read from the tickets: two `done`, one `closed` (superseded), one `in-progress`, three `todo`. The two ordinals had drifted to `:579` and `:630`. The record's *other* two citations in the same Fact — `region.rs:915` and the frontend refusal — were re-read and `:915` is **exact**. |
| L6-1 | `:142` conclusion survives, cause narrows, citations drifted | **verified.** `identity.rs:87` → `fn compute_graph_identity` at `:114`; "lines 103 and 125" → the two `shape.encode(&mut bytes);` sites at `:130` and `:152`; `shape.rs:60` → `:80`. Also **unenumerated**: the same sentence's "its seven delivery tickets are all `todo`" is false. |
| L6-2 | `:220`'s pin was already stale at `0132c0c3`, reading `f99d1e5eb387f42f`; now `940c09e0821665a6` | **verified on both halves** — `git show 0132c0c3:crates/tiler-compiler/src/explain.rs` reads `f99d1e5eb387f42f`, and base `cc667626` reads `940c09e0821665a6`. The `:4183` ordinal has drifted to 3883. |
| L6-3 | `:285` carries the same parenthetical | **verified** |
| RM-1 | `:471` is the softmax row | **false** — the softmax row is `:487`; `:471` is an unrelated paragraph about R7 scoping. The quoted sentence is verified at `:487` and the ticket's reading of it is right. |
| RM-2 | `:481` records `de9ad4cc087697d8` at `explain.rs:3883`, pre-existing drift | **imprecise.** The row is `:497`, and the sentence is an **observation scoped to its own landing's tree** ("Observed on this tree, not inherited"), so it is not a false live claim — it is dated evidence whose value has since moved. Its ordinal, uniquely, *still resolves*, which is the worse failure: line matches, value does not. |
| RM-3 | — | **Unenumerated sibling:** `:496` makes the same shape of claim with the *same* stale ordinal `:4183` and the value `689c3aefc30f48d3`. `:485` is the third and needs no repair — it already dates its reading and deliberately cites the pin without a line number, which is the convention the other two should have followed. |

**None of these repairs changes what the ticket is for.** The largest correction, ABI-1, removes one repair from the list; the three unenumerated falsifications add work inside the same scopes and the same class.

## Filed rather than fixed here

- **`crates/tiler-ir/src/index/refinement.rs` names `tiler.semantic-graph.v2` in a doc comment** on `IndexRefinementExecutableCoverageIdentity` ("`tiler.semantic-graph.v2` already writes each of them for every operation in canonical traversal order"). `GRAPH_DOMAIN` is `v3`. ADR 0104 quotes that comment verbatim, so the ADR is correct and the source is stale; the ADR now carries a note beside the quotation saying so. Fixing it is `implementation/ir`, which this ticket does not hold. One-line change, no behaviour, no identity.

## Outcome — repaired 2026-08-08 at base `cc667626`

Six documents plus the spike the ADR's measurement depends on. Every repair's treatment was selected by the ever-true test [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s context correction states: a claim true when written is **dated beside**, a claim never true at any commit is **substituted** with the retired wording quoted.

| Record | Repair | Ever-true verdict |
| --- | --- | --- |
| `docs/artifact-abi.md` `:237` | `tiler.semantic-graph.v2` → dated note naming `v3`; sentence retained | true at that ledger step → **dated beside** |
| `docs/artifact-abi.md` `:439` | three-of-four → three-of-five, with the fifth subject's omission grounded in `ArtifactBuildError::SymbolicSemanticInterface` and its pinning test | true and **still true** → recorded as an addition |
| `docs/artifact-abi.md` `:285` | none | already correct; the ticket's claim was wrong |
| ADR 0104 header | dated supersession carrying the re-run and the `v5 → v6` step | true when measured → **dated beside** |
| ADR 0104 `:61` | note beside the quotation; quotation untouched | quotation faithful; **source** is stale → filed |
| ADR 0104 Bounds | dated extension; the second displacement and the control it added | true → **dated beside** |
| `symbolic-semantic-extents.md` | seven dated blocks (the `:32` Fact, the `:38` Inference, the W4 proposal block, the four-subject Fact, the untagged-encoding Fact, the crate-reach Fact, the `v3` Inference), A4 and A5 answered, the deferred trigger restated as **not fired**, the delivery table's statuses, checks 1–3 **replaced**, `implementation_status` `not-started` → `partial` | all true when written → **dated beside**; checks replaced because each asserted an absence now filled |
| `transformer-operation-and-shape-surface.md` `:130` | dated block narrowing the premise to results and correcting the seven-`todo` count | true on 2026-08-07 → **dated beside** |
| `complete-model-ingestion-and-execution.md` `:142` | dated block; conclusion held, cause narrowed, three citations retired for anchors | true → **dated beside** |
| `complete-model-ingestion-and-execution.md` `:220` | pin sentence **substituted** (it claims to be "the latest" and was already stale when written); `request-subject.v5` **dated beside** | one clause never safe, one true when written — **both treatments in one sentence** |
| `complete-model-ingestion-and-execution.md` `:285` | dated parenthetical | true → **dated beside** |
| `docs/roadmap.md` `:487` | dated block; premise narrows, conclusion stands | true → **dated beside** |
| `docs/roadmap.md` `:496`, `:497` | ordinals dropped for the grep anchor `:485` already uses; readings dated | dated observations → **dated beside** |

### The ADR 0104 ladder, re-run rather than derived

`spikes/program-planning/identity-growth`, base `cc667626`, Apple M4 Max, macOS 27.0 build `26A5388g`, repository toolchain pin — the same host every earlier result names. Sixty-one points over 2..=62 with the class-checked wall at 63 confirmed. Retained at `results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv`; no earlier file overwritten.

| Quantity | Was | Now |
| --- | --- | --- |
| program identity | `3530n + 723` | **`3531n + 724`** |
| graph identity | `134n + 149` | **`135n + 149`** |
| 64 MiB refusal point | 19,011 operations | **19,006** |
| identity at the governed budget of 62 | 219,583 B | **219,646 B** |
| embedding-ceiling crossing | between 148 and 149 | **unmoved**, between 148 and 149 |

Compared column by column against the predecessor rather than by fit: `graph_bytes` `+n` exactly, `program_bytes` and `widest_alternative_bytes` `+(n + 1)` exactly, and **`coverage_bytes` identical at all sixty-one points**. That last is ADR 0104's fold observed directly — the graph grew and the `n` records naming it did not — and it is a sharper demonstration of the fold than any earlier run could give, because earlier displacements moved the whole identity and could not separate the two. Attribution is bounded: 268 commits separate the bases, but exactly one touches `crates/tiler-ir/src/semantic/identity.rs` or `crates/tiler-ir/src/shape/`, and it is the landing. The residual `+1` constant on `program_bytes` is **not** attributed; it is recorded as unattributed rather than guessed at.

The spike's own `README.md` carried the current reading and is updated with it: all sixty-one table rows regenerated from the new run, the fit, the refusal point, the wall control's 7,796 bytes, the P1/P2/P3 evaluation table, the whole-model figures, and `last_verified`.
