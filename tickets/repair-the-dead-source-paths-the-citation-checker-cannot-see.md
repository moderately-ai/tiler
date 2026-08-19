---
id: repair-the-dead-source-paths-the-citation-checker-cannot-see
title: Repair the dead source paths the citation checker cannot see
status: in-progress
priority: p2
dependencies: []
related: [repoint-the-optimizer-contract-s-request-module-citations, repair-the-ticket-population-facts-the-splits-and-retirements-falsified]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, citations, audit]
claimed_from: todo
assignee: worker-deadpath
lease_expires_at: 1787166711
---
## User-visible outcome

No live ticket or document names a `crates/**.rs` path that does not exist, except where the mention is deliberate history or a deliberate negative example. The four dead-path families nobody has repaired stop routing readers and workers at files that were moved months ago.

## Why this exists

Filed 2026-08-19 by the coordinator, from a finding the `repoint-the-optimizer-contract-s-request-module-citations` lane surfaced and a census the coordinator then ran at `f7a356de`.

**Fact — `make citations` is blind to this entire class, by design.** `check-citations.sh` checks a code span carrying a path **plus a pin** (a line or an anchor); a bare path with no pin is deliberately not checked, and the script says so at its own anchor `A bare path with no pin is deliberately NOT checked`. The census prints the exclusion rather than hiding it: the current run reports `not checked  10595 bare path mention(s) carrying no line or anchor`. So a bare mention of a **deleted** file is green forever, and every module split this repository has done has left some behind. This is the case AGENTS.md's "a mechanical check does not discharge a reading obligation" exists for — the checker is working as documented; the coverage gap is real anyway.

**Fact — the optimizer-contract repair demonstrated the mechanism.** In `docs/compiler/optimizer.md`, nine of ten `crates/tiler-compiler/src/request.rs` mentions were bare paths sitting in the unchecked bucket; only one was a markdown link, and it resolved **green against the surviving module spine** while naming symbols that had all moved into `request/` submodules. Repointing them into the pinned `path "anchor"` form moved them into the checked population — the doc-citation count rose 1165 → 1173 on that single document edit.

**Fact — fourteen distinct non-existent `crates/**.rs` paths are cited across live `docs/` and `tickets/`.** Census run at `f7a356de` over `docs/` and `tickets/`, excluding `docs/research/documentation/ticket-audit-2026-08-10/**` (dated history by convention). Counts are **files containing the mention**, not occurrences:

| dead path | files | disposition |
| --- | --- | --- |
| `crates/tiler-ir/src/schedule/builder.rs` | 73 | already owned — repair lanes landed 2026-08-19 |
| `crates/tiler-ir/src/index/refinement.rs` | 36 | already owned — same |
| `crates/tiler-artifact/src/program/codec/tests.rs` | 32 | already owned — same |
| `crates/tiler-artifact/src/program/tests.rs` | 31 | already owned — same |
| **`crates/tiler-compiler/src/feasibility.rs`** | **14** | **unowned** — moved to `crates/tiler-compiler/src/target/feasibility.rs` |
| **`crates/tiler-compiler/src/honourability.rs`** | **7** | **unowned** — moved to `crates/tiler-compiler/src/target/honourability.rs` |
| **`crates/tiler-artifact/src/program/codec/digest.rs`** | **6** | **unowned** — ~~no successor located; determine whether it was renamed, folded, or deleted~~ **deleted deliberately, capability re-homed in `crates/tiler-digest/src/lib.rs`** (audit below) |
| **`crates/tiler-reference/tests/governed_scalar_reference.rs`** | **3** | **unowned** — ~~no successor located~~ **never existed at any commit** (audit below) |
| **`crates/tiler-build/src/metal_profile.rs`** | **3** | **unowned** — ~~no successor located~~ **folded into `crates/tiler-build/src/metal_declaration.rs`** (audit below) |
| **`crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs`** | **2** | **unowned** — ~~no successor located~~ **renamed at `1eef65fe` to `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs`** (audit below) |
| **`crates/tiler-artifact/tests/proof_sidecar_facade.rs`** | **2** | **unowned** — ~~no successor located~~ **never existed at any commit** (audit below) |
| `crates/burn-ir/src/operation.rs` | 1 | **not a defect** — third-party Burn, correctly external |
| `crates/burn-fusion/src/backend.rs` | 1 | **not a defect** — same |
| `crates/tiler-compiler/src/no-such-file.rs` | 1 | **not a defect** — a deliberate perturbation example in `pin-ticket-source-citations-against-the-tree-they-name` |

The two `target/` relocations were confirmed with `find crates -name feasibility.rs` and `-name honourability.rs` at this base. The five "no successor located" rows were **not** investigated further and must not be treated as deletions on this ticket's word.

## Source-first Fact audit — 2026-08-19 by `worker-deadpath` at base `c8403a8facd35261476d1091c30fe8436aa9916b`

**The table above is verified exactly, and re-running it at this base returns every row +1.** The census was re-derived over `git ls-files docs tickets` filtered to `*.md`, excluding `docs/research/documentation/ticket-audit-2026-08-10/`, matching `crates/…\.rs` and testing each distinct path against the filesystem: 2948 files scanned, 409 distinct `crates/**.rs` paths mentioned, **14 distinct dead paths** — the same fourteen, in the same order. Every row reads exactly one file higher than the table states, and the extra file is this ticket itself, which now carries all fourteen paths in the table. Excluding this file reproduces 73 / 36 / 32 / 31 / 14 / 7 / 6 / 3 / 3 / 2 / 2 / 1 / 1 / 1, which is the table byte for byte. **The counts are files containing the mention**, as the table says; the occurrence counts are higher and are reported beside them: 129 / 67 / 52 / 47 / 19 / 9 / 8 / 6 / 5 / 3 / 5 / 2 / 2 / 2.

**Imprecise — the bare-path figure moved.** The Why-this-exists section quotes `not checked  10595 bare path mention(s) carrying no line or anchor` from the run at `f7a356de`. `make citations` at this base reports **10592**. The claim the figure supports — that the class is uncounted by design and green forever — is unaffected.

**False — "the four owned families should have shrunk".** The Required-work section states this as the expected signal. It cannot happen, and the reason is structural rather than a lane failing to land. The four repair lanes **did** land: `re-anchor-the-schedule-builder-line-citations`, `point-the-bare-builder-path-mentions-at-the-split-modules`, `repair-the-accepted-decision-records-the-splits-and-retirements-falsified`, and `repair-the-research-records-the-key-replacement-and-splits-falsified` are all present in the tree as dated 2026-08-19 notes. But the repository's repair convention *retains* the retired path so a later grep lands inside the correction rather than on a live claim, which is the same property `check-citations.sh` protects when it declines to demand that a correction's retired extent resolve. That file is deliberately cited here by name only and carries no pinned anchor from this ticket, because [`fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it`](fail-a-partial-path-whose-leading-component-has-vanished-instead-of-skipping-it.md) holds `implementation/workspace` and is editing it in parallel; an anchor planted into a file being rewritten is a citation that breaks on somebody else's merge. A file-containing-the-mention census therefore cannot fall for a repaired file. Checked on three: `docs/decisions/0012-physical-reduction-topology.md`, `docs/decisions/0022-reduction-identities-and-initial-values.md`, and `docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md` each hold exactly **one** `crates/tiler-ir/src/schedule/builder.rs` occurrence at this base, and in all three it is the 2026-08-19 repair note itself. The count is not the coverage signal; the ticket-state and note-coverage partition below is.

**Successors, located by symbol and by history rather than by path guess.** `find` and `grep` over `crates/`, plus `git log --diff-filter=ADR --name-status --all` per path:

| dead path | verdict | evidence |
| --- | --- | --- |
| `crates/tiler-compiler/src/feasibility.rs` | moved | `crates/tiler-compiler/src/target/feasibility.rs "pub(crate) struct CheckedTargetProfile {"` and `crates/tiler-compiler/src/target/feasibility.rs "const GOVERNED_FEASIBILITY_RULE_SET_KEY:"` |
| `crates/tiler-compiler/src/honourability.rs` | moved | `crates/tiler-compiler/src/target/honourability.rs` exists; note that `NumericalDimension` and `CANONICAL_DIMENSIONS` are no longer in it at all — both are `crates/tiler-ir/src/numerics.rs "pub enum NumericalDimension {"` |
| `crates/tiler-artifact/src/program/codec/digest.rs` | **deleted deliberately with the capability re-homed** | `crates/tiler-artifact/src/program/codec/` holds no `digest.rs`; the three names are `crates/tiler-digest/src/lib.rs "pub enum DigestAlgorithm {"`, `"pub struct Digest([u8; DIGEST_BYTES]);"`, `"pub const DIGEST_BYTES: usize = 32;"`, and the invariant the records quote survives verbatim at `crates/tiler-digest/src/lib.rs "place that maps the governed tag to an implementation is the"` |
| `crates/tiler-build/src/metal_profile.rs` | folded, and the symbol was demoted with it | `crates/tiler-build/src/metal_declaration.rs "fn declare_metal_f32_subnormal_behaviour("`, now private rather than `pub` — the retirement `decide-the-compilation-selection-provenance-public-and-wire-surface` records |
| `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs` | renamed | `git show --name-status 1eef65fe` shows `D` on it beside `A` on `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs`, whose module doc opens `It began as` and names the old file |
| `crates/tiler-reference/tests/governed_scalar_reference.rs` | **never existed at any commit** | `git log --all -- <path>` is empty. `register-governed-scalar-reference-evaluation` says in terms that a new target "fails the workspace contract", so the tests went into the admitted one: `4bb20040` modified `crates/tiler-reference/tests/index_region_oracle.rs` and created no new file, and the cited test is there now as `a_lone_non_canonical_nan_contributor_canonicalizes_in_both_oracles` |
| `crates/tiler-artifact/tests/proof_sidecar_facade.rs` | **never existed at any commit** | `git log --all -- <path>` is empty and so is `git log --all -- 'crates/tiler-artifact/tests/'`; that directory has never existed. The proof facade's tests are `crates/tiler-artifact/src/proof/tests.rs`. `decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights` already recorded this at `6d1bd6e8` |

Two of the five were therefore never renames or deletions at all: they are paths a ticket outcome asserted for work that landed somewhere else. Repointing either would have invented a file that never was, which is the failure mode this ticket's Required-work section names.

**The repairable population is two sites, not fourteen paths.** Partitioning every site by ticket state — `done` and `closed` are terminal in `ticketsplease.toml` and are history by the repository convention this ticket's Required-work section states — leaves the seven unowned paths with these live/history splits, counted in files (occurrences in brackets), excluding this ticket:

| dead path | live files | history files |
| --- | --- | --- |
| `crates/tiler-compiler/src/feasibility.rs` | 1 (1) | 13 (17) |
| `crates/tiler-compiler/src/honourability.rs` | 1 (1) | 6 (7) |
| `crates/tiler-artifact/src/program/codec/digest.rs` | 2 (2) | 4 (5) |
| `crates/tiler-build/src/metal_profile.rs` | 0 | 3 (5) |
| `crates/tiler-reference/tests/governed_scalar_reference.rs` | 0 | 3 (4) |
| `crates/tiler-artifact/tests/proof_sidecar_facade.rs` | 0 | 2 (2) |
| `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs` | 0 | 2 (4) |

Four of the seven have **no live site at all**. Of the four live sites that remain, two were already covered by a dated note before this ticket opened and are deliberate history: `docs/decisions/0076-declare-target-honourable-numerical-realizations.md` names `honourability.rs` inside an "As landed" paragraph whose very next paragraph is the 2026-07-31 `Correction` repointing it at `target/honourability.rs`, and `docs/dtype-support.md` names `feasibility.rs` inside a fenced reproduction command whose surrounding comment exists to say the path stopped resolving. Both are left. That leaves **two** genuine live defects, both citing `crates/tiler-artifact/src/program/codec/digest.rs`, both in `contracts/decisions`, and both repaired by this ticket.

## Required work

- Re-run the census at your actual base and report it; ~~the four owned families should have shrunk and the unowned ones should not have moved~~ **— corrected by the audit above: no family shrinks, because a repaired file keeps one mention inside its own dated correction. Read the live/history partition, not the totals.** **Report both counts** so coverage is distinguishable from sampling.
- For each unowned path, locate the successor by symbol rather than by path guess, and repoint. Where no successor exists, say so and repair the sentence to state what is true rather than repointing at a plausible-looking neighbour — inventing a target is worse than leaving a dead path, because it reads as verified.
- **Prefer the pinned `path "anchor"` form over a bare path at every site you touch**, so the repair moves the citation into the checked population and the next split fails loudly instead of silently. Report the doc-citation count before and after as evidence that it did.
- Leave deliberate history alone: a `git show <sha>:path` style reference, a dated correction quoting a retired path, and a `done`/`closed` ticket's citation are all history by repository convention. Partition the census by ticket state and report both halves.
- Leave the three non-defects alone, and say in your report that you did, so a later census does not rediscover them.

## Non-goals

Changing `check-citations.sh`'s bare-path policy — that exclusion is deliberate, documented, and counted, and widening it is a separate decision with its own cost. Any source change. The four already-owned families beyond re-running their counts.

## Closes when

Every unowned dead path is repointed to a located successor or its sentence repaired to state what is true, the census is quoted before and after with its ticket-state partition, the doc-citation count movement is reported, `make citations` is green, and the three non-defects are named as deliberately untouched.

## Outcome — 2026-08-19 by `worker-deadpath`

**Two documents repaired, both in `contracts/decisions`, both for `crates/tiler-artifact/src/program/codec/digest.rs`.** Each carries a dated navigation note in the established convention — the historical sentence is retained, the retired path stays greppable inside it, and the note repoints at the located successor in the pinned `path "anchor"` form. [ADR 0082](../docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md)'s Context Inference gains a note pinning `crates/tiler-digest/src/lib.rs "pub enum DigestAlgorithm {"` and `crates/tiler-digest/src/lib.rs "pub const DIGEST_BYTES: usize = 32;"`. [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)'s refused option (a) gains a note pinning `crates/tiler-digest/src/lib.rs "place that maps the governed tag to an implementation is the"` — the invariant option (a) quotes, which survives verbatim in the crate option (b) created, wrapped across three `//!` lines, so the shortest break-free clause is what is pinned rather than the rendered sentence.

**Nothing else was repointed, and each abstention is a reason rather than an omission.** Four of the seven unowned paths have no live site at all; every site is a `done` or `closed` ticket, which is terminal in `ticketsplease.toml` and history by the convention this ticket states. The two remaining live sites were already covered by a dated correction before this ticket opened and are left as history. Two of the five "no successor located" rows turned out never to have existed at any commit, so there was nothing to repoint to and nothing was invented; both are recorded in the audit above and both already carry a correction in their own `done` tickets.

**Non-defects, deliberately untouched and named here so a later census does not rediscover them:** `crates/burn-ir/src/operation.rs` and `crates/burn-fusion/src/backend.rs` are third-party Burn paths inside `https://github.com/tracel-ai/burn` permalinks at revision `e5467f02` and are correctly external; `crates/tiler-compiler/src/no-such-file.rs` is the deliberate perturbation example in [`pin-ticket-source-citations-against-the-tree-they-name`](pin-ticket-source-citations-against-the-tree-they-name.md), which is `done`.

**Mapped remainder, out of this ticket's stated `crates/**.rs` population.** A second scan over the 485 bare `.rs` code spans in the 125 files of `contracts/decisions` and `contracts/navigation`, resolving each the way `check-citations.sh` would, found one further unresolvable path that is neither in the fourteen nor a non-defect: `no_physical_provider_installation_seam.rs`, a spike `trybuild` fixture named at two sites in [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) and tracked nowhere. It already carries a 2026-08-08 dated correction naming [`refresh-the-forkless-physical-provider-spike-against-the-landed-seam`](refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md) as its owner, so it is left to that owner. The other three unresolvable spans in that scan are not defects: `build.rs` names the Cargo concept, `file.rs` is a placeholder inside ADR 0113's description of the citation form itself, and `metal-0.33.0/src/buffer.rs` is a version-pinned external crate source the checker excludes by name.

**Counts.** `make citations` before, at base `c8403a8f`: 1308 citations, 1181 from `docs/`, 122 from tickets, 310 anchor-only, 10592 bare paths. After: **1320 / 1184 / 131 / 322 / 10642**, green. Every one of the twelve new citations is anchor-form — the `path "anchor"` form this ticket asked for, and the line-only count is unchanged at 1361 — split three into the two ADRs and nine into this ticket's audit and outcome. The bare-path count rises by 50 because the audit above and the two notes name retired paths that cannot carry an anchor, which is the legitimate case `check-citations.sh` documents when it declines to demand that a file whose deletion a record is reporting must resolve. That rise is the honest cost of writing the history down; the three `docs/` citations are the part that moved into the checked population, where the next split of `tiler-digest` will redden them.

**Both checked properties were perturbed at the subject and both failed by name.** Renaming `crates/tiler-digest/src/lib.rs` to `digest_impl.rs`, which is what a future split of that crate would do, reddened exactly the five new anchor citations: `FAIL docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md / citation: crates/tiler-digest/src/lib.rs "place that maps the governed tag to an implementation is the" / anchor occurs nowhere in crates/tiler-digest/src/lib.rs`, with `check-citations: 5 citation(s) do not resolve against this tree.` Separately removing `site-the-governed-digest-so-layered-identity-encoders-can-reach-it.md` from the index *and* the disk reddened ADR 0082's new link: `FAIL docs/decisions/0082-… / link: [...](../../tickets/site-the-governed-digest-so-layered-identity-encoders-can-reach-it.md) / no tracked file or directory at …`, with `check-citations: 3 markdown link(s) do not resolve against this tree.` Both perturbations were reverted and `git status --porcelain` confirmed only the three intended files modified. Worth recording for the next worker: moving the file **without** touching the index changed nothing at all, because link targets resolve against `git ls-files` rather than the filesystem — a perturbation that only `mv`s a link target cannot show that the link check works.
