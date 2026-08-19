---
id: repair-the-dead-source-paths-the-citation-checker-cannot-see
title: Repair the dead source paths the citation checker cannot see
status: todo
priority: p2
dependencies: []
related: [repoint-the-optimizer-contract-s-request-module-citations, repair-the-ticket-population-facts-the-splits-and-retirements-falsified]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, citations, audit]
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
| **`crates/tiler-artifact/src/program/codec/digest.rs`** | **6** | **unowned** — no successor located; determine whether it was renamed, folded, or deleted |
| **`crates/tiler-reference/tests/governed_scalar_reference.rs`** | **3** | **unowned** — no successor located |
| **`crates/tiler-build/src/metal_profile.rs`** | **3** | **unowned** — no successor located |
| **`crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs`** | **2** | **unowned** — no successor located |
| **`crates/tiler-artifact/tests/proof_sidecar_facade.rs`** | **2** | **unowned** — no successor located |
| `crates/burn-ir/src/operation.rs` | 1 | **not a defect** — third-party Burn, correctly external |
| `crates/burn-fusion/src/backend.rs` | 1 | **not a defect** — same |
| `crates/tiler-compiler/src/no-such-file.rs` | 1 | **not a defect** — a deliberate perturbation example in `pin-ticket-source-citations-against-the-tree-they-name` |

The two `target/` relocations were confirmed with `find crates -name feasibility.rs` and `-name honourability.rs` at this base. The five "no successor located" rows were **not** investigated further and must not be treated as deletions on this ticket's word.

## Required work

- Re-run the census at your actual base and report it; the four owned families should have shrunk and the unowned ones should not have moved. **Report both counts** so coverage is distinguishable from sampling.
- For each unowned path, locate the successor by symbol rather than by path guess, and repoint. Where no successor exists, say so and repair the sentence to state what is true rather than repointing at a plausible-looking neighbour — inventing a target is worse than leaving a dead path, because it reads as verified.
- **Prefer the pinned `path "anchor"` form over a bare path at every site you touch**, so the repair moves the citation into the checked population and the next split fails loudly instead of silently. Report the doc-citation count before and after as evidence that it did.
- Leave deliberate history alone: a `git show <sha>:path` style reference, a dated correction quoting a retired path, and a `done`/`closed` ticket's citation are all history by repository convention. Partition the census by ticket state and report both halves.
- Leave the three non-defects alone, and say in your report that you did, so a later census does not rediscover them.

## Non-goals

Changing `check-citations.sh`'s bare-path policy — that exclusion is deliberate, documented, and counted, and widening it is a separate decision with its own cost. Any source change. The four already-owned families beyond re-running their counts.

## Closes when

Every unowned dead path is repointed to a located successor or its sentence repaired to state what is true, the census is quoted before and after with its ticket-state partition, the doc-citation count movement is reported, `make citations` is green, and the three non-defects are named as deliberately untouched.
