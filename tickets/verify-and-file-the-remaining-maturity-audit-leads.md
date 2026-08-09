---
id: verify-and-file-the-remaining-maturity-audit-leads
title: Verify and file the remaining maturity-audit leads
status: done
priority: p3
dependencies: []
related: [correct-the-adr-gate-claims-the-python-gate-deletion-falsified, reroute-the-dtype-ledgers-cells-that-point-at-terminal-tickets, cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check, retire-the-gate-reproduction-claims-in-the-apple-numerical-record, date-the-artifact-abis-metal-golden-enumeration-to-its-step]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Six leads from the 2026-08-07 maturity audit, verified to differing degrees

The audit returned ten findings. **Four were verified by the coordinator and filed**; one more (`manifest schema`) is filed separately. These six remain, and **each must be re-verified before it becomes a repair** — the audit's base moved three times mid-run, and its own line numbers had already drifted once.

Filed as one ticket rather than six because each is small and they share a method. **Do not repair any of them on this ticket's say-so.**

1. **ADR 0074 asserts the gate runs a probe the gate cannot reach.** It says `spikes/extensions/run.py --self-test` is run by the repository gate and that the gate "compiles the workspace on every invocation". The `Makefile` header reportedly says *"Spikes deliberately have no target."* Load-bearing because convention 5b's Measurement closes by claiming the comparison "forces a fresh run at the next pin migration" — if nothing runs it, that mechanism does not exist and the measurement survives only by the toolchain pin happening not to have moved.
2. **`docs/dtype-support.md` routes BF16's remaining rungs to "the live tickets", and the audit reports that population as empty.** **The coordinator could not confirm this at the stated precision** — a scan of tickets referenced from that file found 16 non-terminal, but they are integer, quantized and sub-byte owners rather than the BF16 set under D-4. So the narrow claim may hold and the broad reading does not. **Check D-4's own eight owners specifically** before concluding anything.
3. **`docs/artifact-abi.md` contradicts itself on its own domain count** — "seven", "eight", and "four domain separators" in one document. Partly covered by `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check`, which reconciles counts against the true population; **coordinate rather than duplicating**.
4. **ADR 0076 contradicts its own dated correction inside the same bullet** — the correction says the harness "is no longer collected by anything", then the same bullet closes "reproduced by the harness the gate runs", and a second passage repeats it. A research record reportedly agrees with the correction. The conclusion survives; two surviving sentences are the stale ground.
5. **`docs/artifact-abi.md`'s "the five that carry no cooperative tile" is nine of ten.** The paragraph names five goldens; the directory holds ten and exactly one stages a barrier. The *claim* survives and is stronger (1 of 10); the enumeration no longer names its population.
6. **Two ledger cells say "until X lands" for a ticket that landed and recorded the opposite** — the named ticket is `done` and its Outcome says the thing is structural and permanent, so readers are sent to a closed ticket expecting a ruled-out fix.

## Method

For each: re-read the source in full at your base, state **verified / false / imprecise**, and separate whether the *conclusion* survives from whether its *stated ground* does. That distinction has mattered repeatedly here — a false premise does not automatically make a false conclusion, and several of these are conclusions that survive on other grounds.

Then **file each survivor as its own ticket in its own scope** rather than repairing across scopes from here. This ticket holds only `project/tickets`.

## Closes when

Each of the six carries a verified verdict; every survivor is filed in its owning scope with its evidence; and any that did not survive verification is recorded as refuted so a later audit does not re-raise it.

## Verdicts — verified 2026-08-07 at base `7c371155`

Every source was re-read in full at that base and located by symbol or quoted phrase, never by the line numbers the leads carried.

| # | Lead | Verdict | Conclusion vs stated ground | Disposition |
| --- | --- | --- | --- | --- |
| 1 | ADR 0074 asserts the gate runs a probe it cannot reach | **verified** | Conclusion and ground both hold for the two Evidence sentences. Convention 5b's Measurement is **imprecise, not false** — see below | Filed: `correct-the-adr-gate-claims-the-python-gate-deletion-falsified` |
| 2 | `docs/dtype-support.md` routes BF16's rungs to an empty live population | **verified at the narrow precision; the broad reading is false** | Conclusion holds for the BF16 set. The broad ground — that the file's referenced owners are empty of live work — is false: 16 are non-terminal, none BF16 | Filed: `reroute-the-dtype-ledgers-cells-that-point-at-terminal-tickets` |
| 3 | `docs/artifact-abi.md` contradicts itself on its domain count | **verified, and already owned** | Conclusion holds; the count sites are **five, not three** | **Not filed.** Coordinated into `cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check`, which already owns the reconciliation |
| 4 | ADR 0076 contradicts its own dated correction | **verified, and larger than reported** | Conclusion survives; the two named sentences are the stale ground. The research record cited as *agreeing* is the largest concentration of the same false claim | Filed as two: the ADR half above, plus `retire-the-gate-reproduction-claims-in-the-apple-numerical-record` |
| 5 | "the five that carry no cooperative tile" is nine of ten | **ground true, conclusion false** | The count is right about today's tree and the conclusion drawn from it is wrong. See the refutation below | Filed **with the opposite instruction**: `date-the-artifact-abis-metal-golden-enumeration-to-its-step` |
| 6 | Two ledger cells say "until X lands" for a landed ticket | **verified exactly as stated** | Conclusion and ground both hold. Refinement: the tautology claim those cells make is **still true** and must survive the repair | Filed with lead 2 (same file, same scope) |

### Lead 5 — the refutation, recorded so it is not re-raised

The lead's ground is accurate: the paragraph names five goldens, `crates/tiler-metal/goldens` holds ten, and exactly one stages. **Its conclusion — "the claim survives and is stronger, 1 of 10; the enumeration no longer names its population" — does not survive.**

The paragraph is a dated `Fact` about the `tiler.schedule.v5` step, landed at `a395852a` on 2026-08-02. `git ls-tree --name-only a395852a crates/tiler-metal/goldens/` returns **six** files: `cooperative_workgroup_reduction` plus exactly the five the paragraph names. The enumeration was complete when written. The other four goldens landed on 2026-08-05 and 2026-08-06 and did not move at a step that predates them, so restating the claim as nine of ten would assert something false. What survives is only a tense hazard — the passage reads as standing rather than historical — and the filed ticket carries an explicit instruction **not** to recount it.

### Lead 1 — the part that is imprecise rather than false

Convention 5b's Measurement closes "the record's fail-closed channel comparison forces a fresh run at the next pin migration". The comparison is real and intact in `spikes/extensions/run.py`; what is false is *forces*, because nothing invokes it. The measurement therefore survives on a weaker ground than it states. A repair that retires the measurement has overshot; only the custody claim is wrong.

### Sibling sweep — a negative result worth keeping

Every `until <ticket>` construction in `docs/` was extracted and its target's status read. Six distinct tickets, **all `done`**. Four are past-tense records of work that landed and are correct as written; the Q-SEM-015 planning gate in `docs/open-questions.md` explicitly states "All three are `done`, so the gate is open". Lead 6's two cells are the only live-tense pointers of this class in the repository, so that class is now closed rather than sampled.

## Outcome — done, 2026-08-08

Landed at merge `982a3f9e` (worker commit `58f19f2b`). `tickets/` only, carries the green gate. **Nothing was repaired**, which was the ticket's whole shape.

### Lead 5 refuted, and this is the result that justified the pass

The audit's **ground was true** — the paragraph names five goldens, the directory holds ten, exactly one stages a barrier — and its **conclusion was wrong**. The paragraph is a *dated Fact about the `tiler.schedule.v5` step at `a395852a`*, and coordinator-verified: `git ls-tree a395852a crates/tiler-metal/goldens/` returns **six** files — `cooperative_workgroup_reduction` plus precisely the five it names. The enumeration was **complete when written**. The other four landed 2026-08-06.

So the proposed "nine of ten" repair would have asserted those four moved at a step predating them — **a new false claim, produced by fixing a true observation**. The ticket was filed with the opposite instruction. The neighbouring version pair two paragraphs down was checked too and correctly left alone.

This is exactly the conclusion-versus-ground split the brief asked for, and it is the fifth time this week that reading the dated context rather than the current tree changed the answer.

### The other five, verified

- **Leads 1 + 4a** — verified. No `scripts/` directory exists; the `Makefile` header says *"Spikes deliberately have no target."* Both ADRs are stale **against their own spike README**, which already records this correctly. Convention 5b's Measurement is **imprecise rather than false**.
- **Lead 2** — the coordinator's suspicion was right **in both directions**: all eight of D-4's owners are `done`, and the 16 non-terminal tickets the dtype file references are integer, quantized and sub-byte owners with **zero BF16**. Narrow claim holds, broad reading false.
- **Lead 3** — verified and **larger**: the count sites are **five, not three**. Not filed separately; folded into the existing digest-domain ticket, whose `Closes when` phrase would otherwise have let two survive.
- **Lead 4** — **larger than reported**: 32 "gate" occurrences across 22 lines, and the record cited as *agreeing* with the correction is the biggest source of the same false claim. Two occurrences use "gate" in an unrelated sense and were named so they are not swept.
- **Lead 6** — verified exactly, and a **sibling sweep closed the class**: every `until <ticket>` construction in `docs/` resolves to six tickets, **all `done`**; four are correct past-tense records and one explicitly says the gate is open. Lead 6's two cells are the only live-tense pointers. A negative result recorded so the class is closed rather than sampled.

### Four tickets filed, with two deliberate mergers

Leads 1+4a merged (same scope, same root cause, one verification) and 2+6 merged (same file, same scope, same class) **to avoid serializing on an exclusive scope**; conversely lead 4 was **split in two** because its second half is a different scope. Scope shape drove the packaging, not lead count.

Anchor-reach demonstrated by planting drift in a new anchor and watching `make citations` fail by name at exit 2, then reverting.
