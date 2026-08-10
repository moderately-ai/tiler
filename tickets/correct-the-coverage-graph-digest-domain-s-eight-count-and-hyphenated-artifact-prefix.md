---
id: correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix
title: Correct the coverage graph digest domain s eight-count and hyphenated artifact prefix
status: done
priority: p1
dependencies: []
related: [repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check, cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check, repair-the-artifact-abis-stale-cross-crate-no-prefix-argument]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [identity, digest, documentation]
---

A doc comment in `tiler-ir` argues that no prefix relation exists between its coverage-graph domain and `tiler-artifact`'s. **The argument is wrong twice**, and it is a cross-crate correctness argument rather than decoration — which is why this is p1 despite being a comment.

## Facts, coordinator-verified at `2a7a6f08`

**Fact.** `crates/tiler-ir/src/index/refinement.rs`, on `COVERAGE_GRAPH_DIGEST_DOMAIN`, contains the phrase `of \`tiler-artifact\`'s open \`tiler.artifact-\``, in a sentence claiming this domain opens `tiler.ir.` and that all **eight** of `tiler-artifact`'s open `tiler.artifact-`, so no prefix relation exists.

**Fact — first error, the count.** The population is **eighteen**, not eight: `crates/tiler-artifact/src/domains.rs` declares `ENVELOPE: usize = 7`, `PROOF_SIDECAR: usize = 4`, `PROGRAM_IDENTITY: usize = 7`, summing to a `variant_count` over `GovernedDomain`. The "eight" is the pre-repair figure from a check that covered 8 of 18 while reading as complete.

**Fact — second error, and the one that matters more.** The prefix is `tiler.artifact` **without** the hyphen. `domains.rs` asserts `bytes.starts_with(b"tiler.artifact") || bytes.starts_with(b"tiler.proof-sidecar.")`. The hyphen is wrong because `tiler.artifact.route-requirement.v1` separates with a `.`, so a claim quantified over `tiler.artifact-` does not cover every domain it needs to. `docs/artifact-abi.md` already records the hyphen correction; this comment is reported to be the last site carrying the narrower spelling.

**Inference — why the second error is the live one.** The count being stale weakens the argument's authority. The **spelling** being narrow means the argument does not range over the domains it claims to, so the no-prefix conclusion is asserted over the wrong set. Whether the conclusion still holds is a separate question — I verified independently that all 153 pairs across the eighteen domains have zero prefix relations, so the *conclusion* is true today — but the comment's reasoning does not establish it.

## What closes this

The comment restated so its quantifier ranges over the real population with the real prefix, and so a reader can tell which side is authoritative when the numbers disagree. The sibling repair in `tiler-digest` took the useful shape: it named `tiler_artifact::domains::GovernedDomain` as the thing that *sizes* the population, so prose disagreeing with the type is settled at the type. **Prose cannot size itself from a type — a bare "eighteen" here will rot on exactly the schedule "eight" did.** Prefer naming the enumeration over restating its cardinality.

Cite by **searchable anchor**, not line number, and note the failure mode `AGENTS.md` records: an anchor copied from rendered output can be unsearchable in source when a line break or emphasis marker splits it. Doc comments here wrap at 80 columns, so that case is live for exactly the text you are editing — **run your anchor's grep before committing to it**.

**Do not edit `crates/tiler-artifact/**`** (`implementation/artifact`, not this scope) — read it to describe it correctly.

**Check the rest of this file's cross-crate claims while you are in it.** A comment that went stale on both a count and a spelling is unlikely to be the only one, and the sibling worker found exactly this pattern by reading rather than grepping. **Name the count you checked**, so a clean result is distinguishable from an unexamined one.

## Worker's per-Fact audit, re-read at base `750b29e0`

| Ticket Fact | Verdict | Evidence at this base |
| --- | --- | --- |
| The phrase `of \`tiler-artifact\`'s open \`tiler.artifact-\`` is present on `COVERAGE_GRAPH_DIGEST_DOMAIN`, claiming **eight** | **verified** | `grep -c` returns 1 in `crates/tiler-ir/src/index/refinement.rs`; the sentence read "this domain opens `tiler.ir.` and all eight of `tiler-artifact`'s open `tiler.artifact-`" |
| Population is **eighteen**: `ENVELOPE: usize = 7`, `PROOF_SIDECAR: usize = 4`, `PROGRAM_IDENTITY: usize = 7`, summing to `variant_count` over `GovernedDomain` | **verified** | `crates/tiler-artifact/src/domains.rs`; the `const _` block asserts the three sum to `variant_count::<GovernedDomain>()`. An independent extraction of every `_DOMAIN: &[u8]` literal in that crate yields exactly 18 distinct byte strings |
| "The eight is the pre-repair figure from a check that covered **8 of 18** while reading as complete" | **imprecise** | The module header in `domains.rs` says the superseded hand-written check "covered 8 of **11**", not 8 of 18. `domains.rs` did not exist at `d48a33af`; it was added by `96dfe333`. The "8 of 18" framing is not in any source |
| Prefix is `tiler.artifact` **without** the hyphen; `domains.rs` asserts `starts_with(b"tiler.artifact") \|\| starts_with(b"tiler.proof-sidecar.")` because `tiler.artifact.route-requirement.v1` separates with a `.` | **verified** | Assertion present verbatim in `no_governed_domain_of_this_crate_prefixes_another`. `ROUTE_REQUIREMENT_DOMAIN` is the sole domain failing the hyphenated spelling and passing the unhyphenated one |
| "This comment is reported to be the last site carrying the narrower spelling" | **verified** | After this repair the only `tiler.artifact-` occurrences in the file are inside the correction note itself — the quoted retired wording and the hazard sentence about it |
| All 153 pairs across the eighteen domains have zero prefix relations | **verified independently** for the artifact half; **imprecise** on the IR pair counts | Re-derived from the source literals rather than taken on report: 153 within-`tiler-artifact` pairs (= C(18,2)), 0 relations — still correct. **Correction — 2026-08-10.** The extended counts **684** (= 18×38) and **703** (= C(38,2)) rested on a false IR population of 38. The authoritative population is `PINNED_IDENTITY_DOMAINS` in `crates/tiler-ir/src/domains.rs` (header: "sixty spellings pinned below"): **60** rows, of which **14** open `tiler.ir.` and **46** do not. Cross pairs are **1080** (18×60); within-IR pairs are **1770** (= C(60,2)). Zero-relation conclusion for the real populations is attested by the sibling ticket [`correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places`](correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places.md) and by the live artifact pairwise test; the numbers 684 and 703 are retired |

**Fact — the defect was authored, not left behind.** `git log -S` on the retired phrase returns exactly one commit touching this file, `d48a33af` (2026-08-06). At that tree `tiler-artifact` already declared eighteen governed domains and `ROUTE_REQUIREMENT_DOMAIN` already read `tiler.artifact.route-requirement.v1`. The sentence was therefore **never true at any commit**, so the repair substitutes and quotes the retired wording rather than dating a correction beside it.

**Inference — the conclusion survives but on different reasoning.** The comment's disjointness argument is sound for *this* domain by first-differing byte: `tiler.ir.…` against `tiler.artifact…`/`tiler.proof-sidecar.…` diverge at the byte after `tiler.`, and every domain is longer than that byte. It is **not** sound as a statement about the shared IR's whole set — see the out-of-scope finding below — so the repaired text is scoped to this domain and does not generalize.

## Cross-crate claim census for `crates/tiler-ir/src/index/refinement.rs` — eight claims checked

| # | Claim | Verdict |
| --- | --- | --- |
| 1 | The `COVERAGE_GRAPH_DIGEST_DOMAIN` no-prefix argument | **false twice** — repaired here |
| 2 | ADR 0104 replaced the framed `SemanticGraphIdentity` preimage, which is why both coverage tags step to `v2` | verified against ADR 0104 and both tag constants |
| 3 | `family_realizes_region_sequence` accepted by Tom 2026-08-06, as-is with no exclusion | verified verbatim in the cited ticket |
| 4 | `family_realization_law` accepted by Tom 2026-08-06, as-is with no exclusion | verified verbatim in the cited ticket |
| 5 | "`tiler.semantic-graph.v2` already writes each of them" on `IndexRefinementExecutableCoverageIdentity` | **false** — live domain is `v3`. Already owned by [`step-the-coverage-identity-comment-s-stale-semantic-graph-domain`](step-the-coverage-identity-comment-s-stale-semantic-graph-domain.md); left untouched so that ticket keeps a diff |
| 6 | "`docs/artifact-abi.md` [carries] the measured constants" | **false as of 2026-08-08** — repaired here |
| 7 | Pre-fold quadratic `134n² + 3650n + 727` | verified — retained and still live in `docs/artifact-abi.md` as the structural account of the `v1` encoding |
| 8 | ADR 0104 fold claim on `one_occurrence_of_two_graphs_is_separated_by_the_folded_graph_digest` | verified |

**Fact — claim 6 is a second live defect this ticket found and repaired.** `docs/artifact-abi.md` stopped carrying the identity-growth fit as a live value at `775d314f` on 2026-08-08, naming `spikes/program-planning/identity-growth` as the standing authority. The pointer was authored at `d48a33af` when the contract did carry it, so unlike claim 1 it was **true when written** and is dated beside rather than substituted. It is the only in-code site left pointing at that contract for the constants.

## Out-of-scope findings, for separate tickets

**Fact — `tiler-ir` admits a domain outside `tiler.ir.`, which falsifies the retired IR-opens premise.** At discovery this ticket recorded that `crates/tiler-artifact/src/domains.rs` argued "every domain the shared IR admits opens `tiler.ir.`", and that **24 of the 38** domain byte-strings `tiler-ir` declared opened something else, with **684** clean cross pairs. **Correction — 2026-08-10.** Both population numbers were false: the authoritative IR set is `PINNED_IDENTITY_DOMAINS` (**60** spellings; **14** open `tiler.ir.`, so **46** do not; cross product **1080** = 18×60). Substance retained: `EXPR_DOMAIN = b"tiler.artifact-program.abi-expr.v1\0"` still opens the *same* prefix as `tiler-artifact`'s program-identity container, no prefix relation results, and the defect is reasoning rather than a collision. Source-side status at this date: `domains.rs` no longer argues the IR-opens premise as live — it quotes the retired wording and rests separation on terminators — so the only remaining live lag of that premise is the accepted contract half in `docs/artifact-abi.md`, owned by [`repair-the-artifact-abis-stale-cross-crate-no-prefix-argument`](repair-the-artifact-abis-stale-cross-crate-no-prefix-argument.md). The artifact-source half was delivered under [`correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places`](correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places.md).

**Fact — the `tiler-digest` sibling repair restates a cardinality.** `crates/tiler-digest/src/lib.rs` reads "is the authority for that crate's whole admitted set — eighteen domains across the envelope, the proof sidecar, and the artifact program's identity and key encodings". That bare "eighteen" is the rot schedule this ticket exists to break, in the very repair it holds up as the model. Scope is `implementation/digest`, not this ticket's.

## Outcome

The owned `tiler-ir` repair landed in `fb508b95` (`Range the coverage-graph domain's no-prefix argument over the real set`, 2026-08-08) and the ticket closed in `9c1f090e`. The coverage-graph comment now delegates the artifact population to its type-sized `GovernedDomain`, uses the actual `tiler.artifact`/`tiler.proof-sidecar.` namespace runs, and proves this domain's separation by the first differing byte. The same change repointed the executable-coverage identity-growth measurement from the contract's retired live coefficients to the standing spike authority. No domain bytes, identity encoding, public surface, or runtime behaviour changed.

Both out-of-scope findings were mapped and their source repairs have landed. [`correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places`](correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places.md) delivered the artifact-source correction in `f9b0b67d`; its remaining accepted-contract half is explicitly owned by [`repair-the-artifact-abis-stale-cross-crate-no-prefix-argument`](repair-the-artifact-abis-stale-cross-crate-no-prefix-argument.md). [`repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check`](repoint-tiler-digest-s-domain-separation-note-at-the-moved-union-check.md) removed the bare cardinal and universal IR claim in `8fda6b34`. The only live remainder from this chain is therefore the named artifact-ABI contract ticket, not an unowned source defect.
