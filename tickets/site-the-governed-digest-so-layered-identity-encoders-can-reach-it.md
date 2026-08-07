---
id: site-the-governed-digest-so-layered-identity-encoders-can-reach-it
title: Site the governed digest so layered identity encoders can reach it
status: review
priority: p2
dependencies: []
related: [decide-whether-executable-coverage-evidence-folds-as-a-digest, decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest]
scopes: [contracts/decisions, implementation/ir, implementation/artifact, contracts/artifacts, implementation/digest, implementation/workspace, implementation/cargo-lock, implementation/build, implementation/frontend, contracts/foundation, research/artifacts, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [identity, decision, public-boundary, architecture]
claimed_from: todo
assignee: agent-digest-crate
lease_expires_at: 1786066635
---
## User-visible outcome

The one question blocking [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) is answered: whether the governed digest is reachable from the crate that mints layered IR identities, or whether those identities stay restricted to canonical-bytes-only folds and the coverage encoding keeps its quadratic.

**This is Tom's. It is a consequential public crate, module, and type boundary, and it moves an ownership two accepted ADRs currently place elsewhere.**

## Why this exists

**Fact — the crate that needs a digest cannot reach one, and the dependency cannot be reversed.** `IndexRefinementExecutableCoverageIdentity` is minted in `crates/tiler-ir/src/index/refinement.rs`. `tiler-ir` is the workspace's bottom crate: `crates/tiler-ir/Cargo.toml` lists exactly `num-bigint`, `num-integer`, and `num-traits`, and no workspace member sits below it. The one governed digest — `DigestAlgorithm`, `Digest`, `DIGEST_BYTES` — lives in `crates/tiler-artifact/src/program/codec/digest.rs`, and `tiler-artifact` depends **on** `tiler-ir`.

**Fact — duplicating it is refused by the module that owns it.** `digest.rs`'s own module doc: being "the *only* place that maps the governed tag to an implementation is the whole point, and that property is what a second component reaching for a hash function would destroy."

**Fact — this exact constraint has been decided once already, in the direction of moving the consumer.** ADR 0082 records that the expansion cache's previously assigned owner was replaced because it "proved unable to reach the governed digest this decision requires it to validate against". That precedent resolved a reachability problem by relocating the *consumer*; here the consumer is the bottom crate and cannot be relocated.

**What it is worth.** ADR 0104 measures the consequence: folding the per-coverage-record graph identity turns kernel-program identity from `134n² + 3650n + 719` into `3525n + 719`, moving the per-invocation embedding-ceiling crossing from between 50 and 51 semantic operations to between 148 and 149 — and, more than the number, turning a quadratic into a linear. The current crossing sits **at** the roadmap's own ≥ 51-operation decoder layer, below the governed `semantic_operations` budget of 62, with no typed refusal at the artifact layer.

## The candidates, with what each enables and prevents

- **Move `DigestAlgorithm` and `Digest` into `tiler-ir`.** Cheapest edit and no new crate. Cost: the shared IR gains a `sha2` dependency and a hashing responsibility, and `docs/artifact-abi.md` plus ADR 0050 currently site the governed digest with the artifact envelope, where `tiler-cache` reaches it. The ownership statement in both would move.
- **Introduce a crate below both** owning the governed algorithm, its tag table, and its domains. Keeps the shared IR free of hashing and gives the "one authority for a governed constant" rule its own home. Cost: a new workspace member and a new public surface, against ADR 0056's four-library framing and the consumer-closure statements `tiler`, `tiler-runtime`, and `tiler-artifact` carry in their manifests.
- **Answer no, and keep layered identities canonical-bytes-only.** Preserves every current statement. Cost: ADR 0104 cannot execute, so the coverage encoding stays quadratic and the embedding ceiling keeps binding at decoder-layer size. That is a decision to accept the ceiling, and it should be recorded as one rather than reached by default — with a named trigger for reopening.

## Explicit non-goals

Not re-deciding ADR 0104's *choice* among the three coverage encodings — that derivation is complete and recorded on its own ticket and record. Not adding a second hashing implementation anywhere, under any of the three answers. Not touching the manifest's identity digest, which is [ADR 0103](../docs/decisions/0103-declare-the-manifests-artifact-identity-by-digest.md)'s and already lands inside the crate that owns the digest.

## Closes when

Tom has answered, and the answer is either an accepted boundary with ADR 0104 unblocked and its identity-domain step scheduled, or a recorded decision to keep layered identities canonical-bytes-only with the ceiling accepted and a reopening trigger stated.

## Decision — a new bottom crate

**Decided by Tom on 2026-08-06 at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket): the governed digest moves to a new workspace crate below `tiler-ir`**, owning `DigestAlgorithm`, `Digest`, `DIGEST_BYTES`, the tag table, and the domain-separation discipline, with `tiler-artifact` re-exporting so every current consumer keeps its path. Grounds accepted: hashing is a separate responsibility from tensor IR; the one-authority property gets a structural home rather than riding in whichever crate needed it first; future layered-identity consumers reach it without reopening the boundary. ADR 0104 is accepted with this answer and its identity-domain step is scheduled; this ticket becomes the implementation vehicle and closes when the crate exists, the relocation lands with the ownership statements in `docs/artifact-abi.md` and the ADR 0050/0056 framing moved coherently, and ADR 0104's coverage-fold step lands whole with every pin recomputed.

## Outcome — 2026-08-06

Delivered whole at **`d48a33af`** on branch `tkt/site-the-governed-digest-so-layered-identity-encoders-can-reach-it`, over base `b54138b1`. `make full` green on the completed branch, `tkt lint` clean, `git diff --check` clean, `tkt guard` reporting no scope escape. (This paragraph's own hash is recorded in the follow-up commit that adds it; `d48a33af` is the commit carrying every source, contract, and pin change.)

### The crate

**`tiler-digest`**, at `crates/tiler-digest`. The name follows the workspace's `tiler-<noun>` idiom and names the thing the crate owns rather than the responsibility it serves — `tiler-hash` would name the mechanism, and the governed value is a digest whose algorithm is one admitted choice behind a wire tag. It is the workspace's bottom crate: one dependency, `sha2`, and nothing below it.

It owns `DigestAlgorithm`, the opaque `Digest`, `DIGEST_BYTES`, the wire tag table, and the domain-separation discipline as documentation. It deliberately owns **no domain**: a domain names a subject, and the authority that decides what a subject is owes the no-prefix check over its own admitted set, so siting the domains here would gather that obligation into a crate that knows none of the subjects it is discharged for.

**One surface change, and it is a narrowing.** `digest_parts` — the general "digest this sequence of parts" call, `pub(crate)` while `tiler-artifact` owned the module — does not exist in any crate now. Promoting it across the new boundary would have handed an outside caller exactly the ambiguous concatenation its own documentation names as the caller's obligation, which is the property the module's charter says is the whole point. It is replaced by `digest_qualified(domain, qualifiers, body)`: one domain, fixed-width qualifiers, and exactly one trailing variable-length run. That is the shape both non-test callers already had — `section_digest` (purpose tag, schema major, schema minor, then section bytes) and the sidecar's `payload_digest` (canonical ordinal, then bytes) — so the obligation moved from prose into the signature. `digest(domain, bytes)` is now defined through it, and a test pins that the two agree with no qualifiers, because a future implementation that framed the qualifier run would move every unqualified digest ever taken.

One constant was dropped rather than moved: `BLOCK_BYTES`, unused and surviving only under `codec/mod.rs`'s crate-level `#![allow(dead_code)]`. Nothing referenced it.

### The relocation sweep

- `crates/tiler-artifact/src/program/codec/digest.rs` deleted; `codec/mod.rs` re-exports `tiler_digest::{DIGEST_BYTES, Digest, DigestAlgorithm}` at the same path, and `program/mod.rs`'s public re-export is unchanged, so `tiler_artifact::program::{…}` resolves for every consumer (`tiler-cache`, `tiler-build`, `crate::proof`, two spike harnesses).
- `sha2` moved off `tiler-artifact`'s manifest onto `tiler-digest`'s. `tiler-ir` gains `tiler-digest`. No other manifest changed; `crates/tiler/Cargo.toml`'s consumer-closure comment restated (`tiler-artifact` is `tiler-digest` and `tiler-ir`, `tiler-digest` is `sha2` — redistributed, not widened).
- The four-domain no-prefix test moved from the digest module to `codec/tests.rs`, where the domains it checks live. Its doc now carries the cross-crate argument as well.
- `docs/artifact-abi.md`: the governed-digest section states the new home, that the contract governs the digest's *use* rather than its home, and the two admitted pre-image shapes; the no-prefix obligation gains a paragraph stating that it now spans crates and is discharged there by construction — `tiler.artifact-` against `tiler.ir.` — because neither crate can hold the check and `tiler-digest` knows no domains.
- `docs/architecture.md`: a `tiler-digest` component-ownership row; a paragraph deriving the twelfth-library admission beside the ADR 0082 one it parallels; the `tiler-cache` single-edge paragraph notes the digest half is now a re-export with the edge unchanged; the library-ordinal sentence names ADR 0104 as the twelfth and cites `workspace_population.rs` as what makes a stale count fail.
- `docs/ir.md`: the accepted 2026-08-04 coverage-projection paragraph gains a dated correction — `v2`, digest not preimage, subject unchanged.
- ADR 0050: a dated correction in Traceability naming the new home, with the decision, the tag `0x01`, the key, and every governed byte explicitly unchanged.
- ADR 0056: a dated note under its own "later component splits may preserve source compatibility through re-exports when evidence justifies them" consequence — this is that clause's first exercise, and the package count is superseded again at twelve.
- ADR 0082 item 3: a dated correction that the promoted surface is now a re-export, that `digest_parts` was deleted rather than promoted, and that the ambiguity refusal became structural.
- ADR 0104: `implementation_status` `not-started` → `implemented`; `applies_to` gains `tiler.contract.architecture`; the status paragraph records the execution; the boundary-question section records Tom's answer and where each moved ownership statement went; the "what the tree still does" clause in Alternatives is corrected in tense.

### The coverage fold, and the program-domain derivation

`encode_executable_coverage_identity` writes `DigestAlgorithm::GOVERNED.digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())` in place of `push_slice(subject.graph.as_bytes())`. New domain: **`tiler.ir.index-refinement-coverage-graph.v1`**, in the discipline's style and under `tiler.ir.` so the cross-crate no-prefix argument holds. Both coverage tags step: `tiler.ir.index-refinement-executable-coverage.v1` → `v2`, and the staged sibling with it.

The digest is written **unframed**. A length prefix exists to make a variable-length run self-delimiting and thirty-two fixed bytes already are; the arithmetic confirms it, since framing would have cost eight bytes per record and put the curve at `3533n` rather than the `3525n` ADR 0104 derives and the harness measures.

**`PROGRAM_DOMAIN` does not step, and the derivation is recorded in its own ledger.** Every prior step moved every program's bytes, so "the bytes moved" is not what decides a step; what decides it is whether a reader of the previous version, handed the new bytes, recovers a *different program* rather than failing. Here it cannot: the coverage record's grammar in `encode_identity` is unchanged — four-byte occurrence, then `push_slice` over the coverage identity — and the framed run opens with the coverage identity's own separator, which stepped to `v2` for exactly this change. So no `v11` identity over a `v1` fold can equal one over a `v2` fold, injectivity across the step rests on a separator that did step, and every holder of the older bytes misses. This is the `tiler.schedule.v4`/`v5` and `tiler.contract.f32.v2` shape — content moved below a fold that re-derives no subset of it — and not the `v9` shape, which changed the coverage record's own grammar in this encoder and had to step. `STAGE_KEY_DOMAIN`, `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v15`, and manifest schema 15.0 hold by the same framing argument, each verified by reading its encoder. **Neither choice was found defensible; one was.** The stop condition did not fire.

### Pins, enumerated old → new

Two, both in `crates/tiler-build/src/metal_plan.rs::the_standard_metal_path_publishes_its_recorded_identities`, recomputed on this tree in the documented order (artifact first, because the subject assertion is unreachable until it passes):

| pin | old | new |
| --- | --- | --- |
| `ARTIFACT_IDENTITY` | `e57b8852b4a9172057dba08f4758574b96fe140a0f2d974390e890dc7425c59d` | `2b0162eb461edeaa8069a022e54057572bf7992970205a5a33f1efee2df896ca` |
| `CACHE_SUBJECT` | `f107cd81f779decff8c2bb15fd61881a2e79ad004457b042fcbfdea25ad97c88` | `8e48d6fbfca8c490c883a557be2c7c5dfcb8264a751c84e585c574d4cd12f186` |

The superseded pair is recorded in the test's ledger under `v11-without-the-fold`, and the ledger gains a paragraph naming this as the first step in it that moved these values without stepping a domain above the one it changed.

**Two more pinned populations moved and are *not* digest goldens**, so they are enumerated here rather than in a table:

- `crates/tiler/tests/workspace_population.rs`: `EXPECTED_MEMBERS` 14 → 15, `tiler-digest` added, and the doc's "Eleven production crates" → "Twelve".
- `crates/tiler-ir/src/program/tests.rs::published_output_interface_order_reaches_program_identity`: the expected occurrence count of an output key inside program identity was derived as `coverage_records + 2` and is now the literal `2` — the semantic fold and the output section, with no per-record graph restatement. The coverage population is asserted non-empty beside it, so a program covering nothing cannot satisfy the literal while proving nothing. This test *is* the per-record restatement measured from the outside, and its move is the change's most direct evidence.

Nothing else in the tree pins an identity byte or an identity length; the checked-in goldens (`tiler-metal/goldens/*.metal`, the `trybuild` `.stderr` set, the schedule/registry/profile hex fixtures, the FIPS vectors) fold no executable coverage and were verified unaffected by the green gate.

The two `compile_fail` doctests on `IndexRefinementExecutableCoverageIdentity` and the `ForeignCoverageGraph` check are meaning-intact: neither touches the encoding — the doctests hold that no byte constructor exists, and `ForeignCoverageGraph` reads a separate unencoded in-memory field on `CoveredOccurrence`. Both pass (`cargo test --workspace --doc` is what reaches the doctests).

### Tests added

- `tiler-digest`: the moved unit coverage in full — FIPS vectors, every padding branch, the exhaustive `0..=192` residue sweep against `hashlib`, domain separation, tag round-trip, the two `#[ignore]`-adjacent measurement tests — plus two new cases for the new entry point: `a_qualified_digest_with_no_qualifiers_is_the_plain_digest` and `a_qualifier_separates_two_subjects_sharing_a_domain_and_a_body`.
- `tiler-ir`: `one_occurrence_of_two_graphs_is_separated_by_the_folded_graph_digest`. It pins all three halves the fold has to preserve — the graph preimage no longer occurs anywhere in the record, the governed digest sits at exactly the position it left, and two graphs at one occurrence ordinal still mint different bytes with the digest field the only one that moved. The neighbouring replay-and-substitution test already perturbs the graph and watches the bytes move, and would keep passing if the encoder had written the identity whole; the position assertion is what says which encoding produced the difference.

Watched failing throughout: the three gate failures the change was expected to produce (workspace population, the two metal pins, the occurrence count) each fired before being addressed, and the new IR test was confirmed to fail against the `v1` encoding by construction — it asserts the absence of a preimage that `v1` writes.

### Measurements

Run manually from `spikes/program-planning/identity-growth`, before and after, on this host (Apple M4 Max, macOS 27.0, repository toolchain pin), ordinary compilation path, 2..=8 operations plus the nine-operation probe:

| | fitted curve | n = 8 | n = 9 probe |
| --- | --- | --- | --- |
| before | `134n² + 3650n + 727` | 38,503 B | 44,431 B |
| after | `3525n + 727` (quadratic coefficient **0**) | 28,927 B | 32,452 B |

**Agreement with the prediction, and the one delta.** ADR 0104 predicted `3525n + 719`. The linear coefficient agrees to the unit. The constant is **727, eight bytes higher**, because the `tiler.kernel-program.v11` staged-realization step landed between the record's arithmetic and this execution and adds an unconditional eight-byte zero count to every program — the pre-fold curve on this same tree is `134n² + 3650n + 727`, not the record's 719, so the delta is in the base rather than in the fold. `3525·9 + 727 = 32,452` reproduces the out-of-domain probe to the byte. No conclusion moves: recomputed on 727, the embedding-ceiling crossing at the post-0103 multiplicity of two is still between **148 and 149** operations (`2 × 522,427 = 1,044,854` against `2 × 525,952 = 1,051,904`, ceiling 1,048,576), the 64 MiB program bound is **19,038** operations against the record's "roughly 19,000", and at the governed budget of 62 identity is **219,277 bytes**, whose doubling is **41.8%** of the ceiling where the quadratic encoding stood at 283%.

`docs/research/artifacts/manifest-fixed-content-growth.md` §6b is updated from prediction to measurement, including the explicit statement that every `+ 719` in its Sections 5 and 6 is eight bytes low and why no conclusion moves.

**What the ladder harness's re-run must account for**, owned by `widen-the-identity-growth-ladder-to-the-governed-operation-budget` and deliberately not fixed here: it still exits non-zero on its own wall probe, because the governed `semantic_operations` budget moved from 8 to 62 and its ninth point compiles instead of refusing — a finding it is built to report. Its re-run needs (1) `OPERATIONS` and `BEYOND_THE_WALL` widened to the real budget, (2) a **linear** fit rather than the quadratic one `exact_quadratic` reports — the second difference is now zero at every step, so the quadratic path still fits but reports a degenerate `0n²`, and its "the quadratic coefficient *is* the graph slope" mechanism sentence is now false and prints anyway, (3) the constant 727 rather than 710 or 719, and (4) the whole of its retained `results/2026-08-05-…/growth.tsv` and its README's ladder, fit block, refusal point, and P1/P2/P3 margin table treated as superseded, since every row moved. Its `--perturb=fit` self-refutation still works against a linear curve.

### Scopes added, each with its reason

`implementation/digest` (the new crate; mapped to `tiler-digest` in `[scope_crates]` as `ticketsplease.toml` requires of a crate-admission ticket), `implementation/workspace` (workspace members and the `sha2` comment), `implementation/cargo-lock`, `implementation/build` (the two pins), `implementation/frontend` (`workspace_population.rs` and `crates/tiler/Cargo.toml`'s closure comment), `contracts/foundation` (`architecture.md`'s ownership row and `ir.md`'s coverage-projection correction), `research/artifacts` (the growth record's §6b), `research/program-planning` (only `spikes/program-planning/identity-growth/Cargo.lock`, a two-package mechanical consequence of the member change — no source, README, or retained result in that spike was touched).

### Owed coordinator sentences

Two, both in scopes deliberately not held:

1. **`docs/status.md`** (`contracts/navigation`), artifact-and-cache-infrastructure bullet, line 30: append — "**Fact — 2026-08-06: the governed content digest is its own crate.** `tiler-digest` is the workspace's twelfth library and its bottom one, owning `DigestAlgorithm`, the opaque `Digest`, `DIGEST_BYTES`, and the wire tag table under [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md); `tiler-artifact` re-exports all three from `tiler_artifact::program`, so `tiler-cache`'s single decided edge and every consumer path are unchanged. It moved because `tiler-ir` needed the one governed algorithm to fold each executable-coverage record's bound graph identity as a digest, which turned kernel-program identity from `134n² + 3650n + 727` bytes into a measured `3525n + 727` — quadratic in operation count into linear, moving the per-invocation embedding-ceiling crossing from between 50 and 51 operations to between 148 and 149."
2. **`docs/decisions/README.md`** (`contracts/navigation`), the ADR 0104 catalog row inside the generated block: its contracts list should gain `[System architecture](../architecture.md)`, matching the `tiler.contract.architecture` entry added to the record's `applies_to`.

### Observation filed for the coordinator, not acted on

`crates/tiler-compiler/src/governed/contraction_conformance.rs` and `crates/tiler-reference/tests/contraction_profile_cells.rs` each carry a hand-written SHA-256, justified in-file by "`sha2` is a workspace dependency, but adding it to this crate would edit `Cargo.lock`, which this work does not own" — a scope statement about a past ticket, not about the digest's home. Both are test-local, both check themselves against the published FIPS vectors, and both digest conformance facts rather than identities, so neither is a defect. But they are now two second hashing implementations that could reach a dependency-light bottom crate, which is the one-authority concern this ticket's own crate exists to make structural. Out of scope here (`implementation/compiler`, `implementation/reference`); worth a narrow ticket.
