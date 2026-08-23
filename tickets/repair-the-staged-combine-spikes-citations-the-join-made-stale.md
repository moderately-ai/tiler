---
id: repair-the-staged-combine-spikes-citations-the-join-made-stale
title: Repair the staged-combine spike's citations the join made stale
status: done
priority: p3
dependencies: []
related: [join-the-scheduled-region-into-the-contraction-witness, narrow-the-contraction-witness-refusal-to-staging-it-cannot-read, decide-whether-the-citation-checker-should-reach-spike-records]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, doc-drift]
---
## User-visible outcome

The staged-combine derivability spike's citations resolve against the source they name, so a reader following its record reaches the code it describes rather than text that no longer exists.

## Why this exists

Found 2026-08-22 by `worker-narrow` while working in the file the spike cites. Reported rather than folded in — `spikes/` was outside that lane's scopes.

**Fact (reported, unverified by the coordinator) — three citations return zero hits at tip.** `spikes/reference/staged-combine-derivability/README.md` cites `contraction_witness.rs` by the anchor `A kernel declaring workgroup staging combines inside the workgroup` and by `staging().len() != 0` twice. The scheduled-region join landed the same day the spike was written and moved both. Its other three anchors resolve.

**Fact repair — 2026-08-22 by `worker-spikecite`, read at `4f53343f`. The reported Fact is right about the drift and wrong about the arithmetic, and one of the three "zero hits" is not stale at all.** Retired wording preserved above. Verdicts, each `grep -oF <anchor> <file> | wc -l`, counting **occurrences** rather than lines:

- **Verified.** `A kernel declaring workgroup staging combines inside the workgroup` returns **0** against `crates/tiler-ir/src/program/contraction_witness.rs`. The source reads `staging may combine inside the workgroup` (1) — one word inserted, and moved from the refusal to the call site.
- **Imprecise — the two `staging().len() != 0` sites are not the same kind of claim, and only one is stale.** The record's *Outputs* sentence calls it "the predicate the witness actually tests", and that is false: the witness tests `staging().len() == 0` (1), at anchor `if kernel.staging().len() == 0`, with the **sense inverted** — the zero case is the early return admitting an unstaged kernel. The record's *Result* line `staging().len() != 0 = true` is instead a verbatim line of the harness's own output and still resolves, against `spikes/reference/staged-combine-derivability/harness/src/main.rs "staging().len() != 0 = {}"` (1). Repairing it would falsify a recorded run, so it is deliberately left as measured.
- **False — "its other three anchors resolve" undercounts by one; there are four, and all four resolve.** `must become identity-bearing in` (1) and `happens to equal a value tree B can produce` (1) in `contraction_witness.rs`; `A logarithmic tree is still not statable` (1) in `crates/tiler-ir/src/schedule/cooperative.rs`; `No split: a contraction's fold is the declared contributor` (1) in `crates/tiler-compiler/src/frontier.rs`. Six distinct anchors, seven occurrences in the record — the "six citations" of the brief counts distinct anchors and its "three plus three" counts occurrences, so the two halves are in different units.
- **Verified, and the coordinator's own correction stands.** One predicate, two callers: `staged_role` is defined once and called twice in `contraction_witness.rs`. The brief's line numbers were read at `ba3e9da3` and have already rotted at `4f53343f` — the predicate is at `:638` rather than `:578` and the callers at `:372` and `:395` rather than `:372` and `:387` — which is why the repair cites by anchor throughout.

**Fact — two further claims in the record were false, and in the dangerous direction.** *"no `make` target reaches `spikes/`"* and *"`spikes/` sits outside every gate"* tell a reader no check exists where one does. `make citations` reads all 68 spike records, resolves their 600 markdown links, and declines their 61 pinned citations. Demonstrated by perturbation: a false pinned citation added to this spike record raised the declined count 61→62 and the run **stayed green**; a broken markdown link failed it with `no tracked file or directory at crates/tiler-ir/src/program/no_such_file.rs`. Both perturbations reverted.

**Fact — the spike's conclusion survived the join, unchanged.** `from_program` passes `RegionJoin::Unjoined` and at anchor `cannot reach a region at all` keeps its refusals exactly what they were; `from_program_and_regions` reads the topology from a *supplied* `VerifiedScheduledRegion`. The join is a second input, not a derivation from program scope. The two dependent tickets rest on the same finding they always did.

**Fact — the gate cannot see this, by design.** `make citations` walks spike markdown **links** but explicitly **declines spike pinned citations**, on the accepted ground that a spike is evidence about the base its own record names and is repaired on demand. So this is exactly the population that decision left to human repair, arriving on schedule rather than as a surprise.

**Note the frontmatter says `last_verified: 2026-08-22`** — the same day the citations went stale. That is not dishonest: the record was verified against the base it names. It does mean a reader cannot use `last_verified` alone to judge whether a spike's citations still resolve at tip, which is worth stating where the currency convention is described.

## Required work

- Re-audit the Fact at your base and report a verdict — **run each of the six citations yourself**, and say which resolve and which do not, with counts and the unit you report.
- Repair the three stale anchors against the current source. The predicate is now `staging().len() == 0` inside `staged_role`, called from two sites — **verify that at your base rather than inheriting it**; the coordinator confirmed one predicate at `contraction_witness.rs`, not the two the earlier ticket claimed.
- **Preserve the retired wording in a dated correction**, per convention — and expect the record's own grep counts not to shrink.
- Say whether the spike's conclusion still holds. It should: the spike proved staged combine structure is not derivable from program scope, and the join added a route *from the schedule record* rather than from program scope. **If the conclusion has moved, stop and report** — that would change what two dependent tickets rest on.

## Non-goals

Changing the citation checker's declared scope, which is an accepted decision; re-running the spike; and any edit to `crates/`.

## Closes when

Every citation in the record resolves against the file it names, retired wording is preserved in a dated correction, and the record states plainly whether its conclusion survived the join.
