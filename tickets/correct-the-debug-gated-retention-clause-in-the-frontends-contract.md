---
id: correct-the-debug-gated-retention-clause-in-the-frontends-contract
title: Correct the debug-gated retention clause in the frontends contract
status: done
priority: p2
dependencies: []
related: [repair-the-dangling-ticket-link-in-the-frontends-contract, accept-the-retention-read-back-s-caller-visible-boundary]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [retention, contracts, documentation]
---

`docs/integration/frontends.md` tells a reader that retention is conditional on a debug configuration. It is not. Retention is unconditional and caller-independent, so the contract promises a gate that does not exist.

## Facts, verified 2026-08-08 by the coordinator

**Fact.** The stale clause is anchored by the phrase *"Debug configuration may retain canonical MSL and tool diagnostics under the cache entry."*

**Fact.** `crates/tiler-build/src/metal_cache.rs` states `retained: stage_retention(&outputs)` unconditionally, and its own doc comment names the property in terms this clause contradicts: *"**Always stated, never discovered.**"* There is no configuration, environment variable, `cfg`, or profile in that path.

**Fact, tightened by the worker on 2026-08-08.** `Toolchain::run_stage` in `crates/tiler-metal-aot/src/driver.rs` captures stderr at **two** sites — `stderr: ToolOutput::capture(&output.stderr)` in the `!status.success()` arm and `Ok(ToolOutput::capture(&output.stderr))` as the success value — so the output survives on success, which is the case the clause implies is dropped. The original wording attributed the two sites to the *file*, which has four reads of `output.stderr`; the other two are in `capture_tool` and `capture`, where they format the detail of a failed `xcrun` or `--version` probe and retain nothing. Anchor on `run_stage`'s own doc comment, "**A success returns the same capture a failure carries.**"

## Why this was split out rather than fixed in place

The link repair that found it was scoped to citations and was explicitly told not to change what the contract *states*. That was the right boundary: this is a contract clause, and correcting it changes a promise to a consumer rather than a reference. It needs its own read of what the delivered behaviour actually guarantees.

**Second site, same sentence.** `tickets/retain-canonical-msl-under-a-debug-expansion-cache-entry.md` carries the identical stale sentence and the same dead link. That ticket is `done`, so `check-citations.sh` skips it as terminal and **no check will ever flag it**. Repair it here or state deliberately that a terminal ticket is a historical record left as written — either is defensible, but the choice should be explicit rather than a consequence of the checker's skip rule.

## What closes this

The clause restated to match delivered behaviour, with the caller-visible boundary that is still Tom's cited rather than pre-empted — `accept-the-retention-read-back-s-caller-visible-boundary` is `awaiting-decision`, which is `parked` and **not** terminal, so a citation to it stays live and keeps being checked. Do not describe the note's ungated-versus-gated question as settled; that is the open decision, not this ticket's to answer.

Before closing, grep the contract for other conditional language about retention. A clause that survived because it reads plausibly is likely to have siblings; name the count either way.

## Worker verification — 2026-08-08, at base `0f319ec8`

**Per-Fact verdicts.** Fact 1 **verified**: the anchor phrase resolves, as the only match in `docs/integration/frontends.md`. Fact 2 **verified** by reading `crates/tiler-build/src/metal_cache.rs` in full — `retained: stage_retention(&outputs)` sits in the compile closure with no predicate, and `stage_retention`'s doc comment opens "**Always stated, never discovered.**" Fact 3 **verified but imprecise**, corrected in place above: the two capture sites are `run_stage`'s, not the file's.

**A third defect the ticket did not name.** The clause was wrong about its *subject* as well as its gating. The canonical MSL is not what the retention carries and never needed retaining: `tiler_build::metal_compile_request` puts the emitted translation unit's source into `PayloadMetadata::source`, inside the payload identity preimage, so it travels in the artifact envelope under the digest that names it — `crates/tiler-cache/src/expansion/retention.rs`'s "What belongs here, and what belongs in the envelope" states the split and gives the second-unkeyed-authority reason for refusing a copy. The replacement clause therefore separates the two rather than restating one gate correctly over both.

**Sibling census — one clause, no siblings.** Scope `contracts/integrations` maps to `docs/integration/**`, which is `frontends.md` and `candle.md`. `retain*`/`retention` occurs 18 times across 14 lines of `frontends.md` and 4 times in `candle.md`; `debug` occurs 3 times, all in `frontends.md`. Every occurrence was read. `candle.md`'s four are storage lifetime and wrapper information, unrelated to cache retention. In `frontends.md`, two are the unrelated senses "retain diagnostic spans" and "retain their accepted stable proc-macro contracts", one is a measurement about linker copies, two describe the *failure* path's family-scoped diagnostic and are unconditional and accurate, and the rest are the **Landed** bullets, which were corrected on 2026-08-05 and 2026-08-08 respectively. **The corrected clause was the only conditional-gating statement about retention in the scope.** Command: `grep -nEi 'retain|retention|debug' docs/integration/*.md`.

**Terminal twin — the stale sentence is left verbatim, and the reason is not the checker's skip rule.** `tickets/retain-canonical-msl-under-a-debug-expansion-cache-entry.md` does not *assert* the clause; it **quotes** it, in quotation marks, attributed to `docs/integration/frontends.md`, as the dated Fact "the permission exists and nothing delivers it" that is the ticket's whole reason for existing. Repairing a quotation would make that record false — the ticket was filed *because* the contract said that. The same reading applies to its Outcome's "The Metal producer retains nothing today", which was true on 2026-08-05, is dated as such, and names the follow-up that changed it. What *was* repaired there is a dead pointer, which is not a claim: its link to `state-a-debug-retention-from-the-inline-frontend.md` resolved to nothing, the ticket having been retitled to `emit-from-a-populated-retention-in-the-inline-expansion` on 2026-08-07. The historical name is kept as written and the link now resolves. No check would have flagged it — `check-citations.sh` reads terminal states from `ticketsplease.toml` and `done` is `category = "terminal"`, so the file is abandoned at its frontmatter.

**The parked decision is cited, not pre-empted.** `accept-the-retention-read-back-s-caller-visible-boundary` is `awaiting-decision`, which is `category = "parked"`, so the citation stays in the checked population. The clause states the delivered note as ungated — which is what `crates/tiler-macros/src/retention.rs` and that ticket's own Excluded surface record — and states that whether it should stay ungated is Tom's.
