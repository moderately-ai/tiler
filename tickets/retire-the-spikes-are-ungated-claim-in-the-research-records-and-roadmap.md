---
id: retire-the-spikes-are-ungated-claim-in-the-research-records-and-roadmap
title: Retire the spikes-are-ungated claim in the research records and roadmap
status: done
priority: p2
dependencies: []
related: []
scopes: [research/apple-targets, research/shapes, research/verification, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, spikes, falsified-evidence]
---
## User-visible outcome

No research record or roadmap entry still tells a reader that no `make` target reaches `spikes/`, and none attributes a spikes-gating rule to `AGENTS.md` that `AGENTS.md` does not state.

## Why this exists

Filed 2026-08-22 by the coordinator from the residue [`retire-the-spikes-are-ungated-claim-across-the-repository`](retire-the-spikes-are-ungated-claim-across-the-repository.md) reported and correctly declined as outside its scopes. That lane landed as `d144e1df` and repaired 16 files under `contracts/decisions` and `tickets/**`; these sites are under `research/*` and `contracts/navigation` and need their own owners.

**Fact — the claim is false since `04d5eae9`.** `make full` depends on `check`, which depends on `citations`, so `make citations` resolves every local markdown link in every retained spike record. Verified by the coordinator at `516d42c5`: the run reports `spikes  601 link(s) from the live spike record files above, which is the one property checked in that corpus`. What remains true is that no target **builds or runs** a spike, and that spike **pinned citations** are declined by decision — so a repair states the narrower true claim rather than deleting the sentence.

**Fact — `docs/research/shapes/stable-rust-shape-evidence.md:64` carries both defects at once**, and is the worst of the set. Read by the coordinator at `516d42c5` rather than relayed. It states that "`AGENTS.md` compiles a spike workspace only when its retained `.stderr` files were captured on the toolchain `rust-toolchain.toml` pins", attributing to `AGENTS.md` a rule about `scripts/check_rust.py` — **which `e197176` deleted along with all of `scripts/`**. It then closes "no `make` target reaches `spikes/`, so this is now a by-hand check and an unnoticed pass is possible again". The *custody conclusion* survives and should be re-grounded, not withdrawn: nothing compiles that workspace or runs `verify_evidence.py`, so an unnoticed pass really is possible. What is false is the gate-reach clause and the `AGENTS.md` attribution.

**Fact — `docs/roadmap.md:80` states "they are run by hand, no `make` target reaches them"** of the two inline-dispatch spikes. The first clause is true; the second is false in the same narrow way. Verified at `516d42c5`.

**Inference — the remaining five research sites are a census, not a list.** The delivering lane reported 6 live sites under `docs/research/**` across owners `research/{apple-targets, artifacts, cache, scheduling, shapes, verification}`, of which the shapes record above is one. **That count is the delivering lane's and is unverified by the coordinator.** Re-derive it rather than trusting it; that lane's own report records a first census silently corrupted by a backtick inside a double-quoted shell regex, so build the search from a file, state the unit, and say whether you counted lines or occurrences.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict, running each command yourself.
- Re-derive the census rather than inheriting the six. State the unit — `grep -c` counts **lines**, not occurrences — and anchor the pattern.
- **Repair to the narrower true claim, never by deletion.** No target builds or runs a spike; `make citations` reaches every retained spike record for its markdown links, and declines its pinned citations by decision. Where a record's conclusion rests on the false clause, re-ground it on what survives — the custody gap in the shapes record is real and must not be lost in the repair.
- Repair the `AGENTS.md` misattribution first where one exists. A claim borrowing an authority that does not state it is worse than a claim standing alone.
- **Preserve retired wording in dated corrections**; grep counts cannot shrink across a successful repair, so a shrinking count is a false progress signal.
- Add the scopes the sites you touch require and explain them in the ticket as scheduling metadata.

## Non-goals

`spikes/**` itself, which [`correct-the-spike-records-that-still-say-spikes-is-outside-every-gate`](correct-the-spike-records-that-still-say-spikes-is-outside-every-gate.md) owns; editing `Makefile`, `AGENTS.md`, or `check-citations.sh`, none of which is wrong; the six dated audit transcripts under `research/documentation`, which are historical records and should be left; and re-deciding whether the checker should reach spike pins, which is settled.

## Closes when

No live research record or roadmap entry claims no `make` target reaches `spikes/`, no record attributes a spikes-gating rule to `AGENTS.md`, every surviving conclusion is re-grounded on what remains true, the census is re-derived with its unit stated, and retired wording is preserved.

## Coordinator census at `516d42c5`, 2026-08-22 — the six is wrong in both directions, and one site collides with a live ticket

Run by the coordinator from a Python file rather than a shell regex, for the reason the delivering lane's own report gives. **Unit: matching lines**, one count per line even where a line carries the claim twice. Vocabulary, stated so a reader can judge what it would miss: `no \`make\` target reaches/reach`, `outside every gate`, `nothing gates`, `not reached by the/any gate`, `no gate reaches`, case-insensitive. It is **one vocabulary and not the subject's boundary** — a record phrasing the claim as "runs only by hand" or "the gate does not see" lands outside it, so build the census from the spike-mentioning side as the delivering lane did and treat this list as a floor, not an enumeration.

**Seven live sites, not six**, plus five dated transcripts to leave alone:

- `docs/research/README.md:20` — **the research entry point**, which the reported six omits and which is the highest-traffic site in the set.
- `docs/research/apple-targets/numerical-behaviour.md:19` and `:660` — two sites in one file, so an owner-count of one understates the work.
- `docs/research/artifacts/manifest-fixed-content-growth.md:221`
- `docs/research/shapes/stable-rust-shape-evidence.md:64` — the AGENTS.md misattribution named above.
- `docs/research/verification/kani-bounded-encoder-verification.md:111`
- `docs/research/program-planning/flash-class-capability-set.md:102` — **see the collision below; this one is not yours.**

Left alone as historical records, matching the delivering lane's recommendation: five files under `docs/research/documentation/ticket-audit-2026-08-10/reports/`.

**Scope collision, resolved here rather than at merge.** `flash-class-capability-set.md` is owned right now by [`repair-the-flash-class-records-falsified-supplied-greps`](repair-the-flash-class-records-falsified-supplied-greps.md), which holds `research/program-planning` exclusively and is reading that file line by line for a different defect. **Do not take `research/program-planning`, and do not edit that file.** Its spikes-ungated site at `:102` belongs to that lane; this ticket's Non-goals now exclude it. If that lane closes without repairing `:102`, file the remainder rather than reaching across a live claim.

**So the scopes this ticket needs are** `research/apple-targets`, `research/artifacts`, `research/shapes`, `research/verification`, whatever owns `docs/research/README.md`, and `contracts/navigation` for `docs/roadmap.md`. Add them and explain them as scheduling metadata; confirm each against `ticketsplease.toml` rather than trusting this list.


## Worker-research repair record at base `2c312826`, 2026-08-22

**Scopes added, as scheduling metadata:** `research/apple-targets`, `research/shapes`, `research/verification`, `contracts/navigation` — one per file actually edited. `research/artifacts` was **not** added: `docs/research/artifacts/manifest-fixed-content-growth.md:221` was re-audited and found not to carry the defect this ticket names (see verdict below), so no edit landed there and the scope was left unclaimed.

**Per-Fact verdicts, re-run at this base:**

1. **The core `make citations`-reaches-`spikes/` fact — VERIFIED.** `make citations` at this base reports `spikes  601 link(s) from the live spike record files above, which is the one property checked in that corpus` and `spikes  61 pinned citation(s) DECLINED … from 68 live file(s) of 68 read`, matching the ticket's citation exactly.
2. **`docs/research/shapes/stable-rust-shape-evidence.md:64` carries both defects — VERIFIED**, read in full before editing. Repaired: the `AGENTS.md` misattribution first, then the gate-reach clause, preserving both retired phrases verbatim in a dated `(Corrected 2026-08-22. …)` note. The custody conclusion (nothing reproduces `verify_evidence.py`) was re-grounded, not withdrawn.
3. **`docs/roadmap.md:80` ("they are run by hand, no `make` target reaches them") — VERIFIED**, anchor still at line 80 at this base. Repaired in place inside the existing 2026-08-07 correction paragraph, with the retired clause quoted and dated 2026-08-22.
4. **The "six is wrong, seven live sites" census — VERIFIED as a floor, and one further site found on full read.** `docs/research/README.md:20` was already repaired by commit `c924a2e7` ("Correct the navigation portals that call spikes wholly ungated"), an ancestor of this ticket's base — so that site needed no further work. `docs/research/apple-targets/numerical-behaviour.md` carried **three** live sites, not the two the coordinator's vocabulary caught (`:19` and the `Inference` paragraph, now at a shifted line after edits) — a third, at the tail of the file's own 2026-08-08 "Correction" section, paraphrased the same defect as "the `make` targets cover `crates/` and `prototypes/` only" without using any of the coordinator's floor vocabulary. All three repaired.
5. **`docs/research/artifacts/manifest-fixed-content-growth.md:221` — Fact is FALSE/imprecise, not repaired.** Read in full. Line 221 (and line 175, which uses near-identical wording) says "nothing gates a number written in prose" / "each pinned by no test and read by no gate — `make citations` resolves links and never reads a number" — a claim that no check verifies a *numeric value* stays accurate, unrelated to whether any `make` target reaches `spikes/`. The word "spike" on that line comes only from an adjacent evidence link (`[the identity-growth spike](...)`), which is what made the coordinator's automated census match it; reading the whole paragraph shows no "spikes are ungated" claim anywhere in this file. No edit made; no scope claimed.

**Re-derived census, own vocabulary stated.** Built from a Python script (avoiding the shell-backtick corruption the delivering lane's report warned about) scanning every `.md` under `docs/` plus top-level `README.md` (excluding `spikes/**`) for a line containing both a case-insensitive `spike` match and a case-insensitive match on `no \`make\` target|nothing gates|outside every gate|not reached by (the|any) gate|no gate reaches|gate(s)? does not (see|reach)|run(s)? only by hand|by hand[^.]*no|ungated|no target (builds|runs)|isn.?t (reached|covered)|not covered by (either|any|the) target`. Unit: **matching lines** (one per line, same as the coordinator's unit). Found 24 lines. Stated so a reader can judge what it misses: it is anchored to explicit gate/target vocabulary and would miss a claim phrased only as e.g. "the checker never looks at spikes" with none of those words — same floor property the coordinator's own census names.

Of the 24: 3 already repaired by `c924a2e7` before this ticket's base (`README.md:50`, `docs/README.md:78`, `docs/research/README.md:20`); 4 are historical audit transcripts under `docs/research/documentation/ticket-audit-2026-08-10/reports/` (non-goal, left alone); 9 are under `docs/decisions/**` (`contracts/decisions`, out of this ticket's scope by definition — three of the nine, in `0090`, `0075`, and `0074`, explicitly name the sibling `retire-the-spikes-are-ungated-claim-across-the-repository` ticket as their repair, landed `d144e1df`; the other six were not re-audited here since `contracts/decisions` is not an authorized scope for this ticket); 1 is `docs/research/cache/build-tool-exercise.md:159` (see remainder below); 1 (`manifest-fixed-content-growth.md:221`) is a false positive, verdict above; the remaining 6 (`docs/roadmap.md:80` and the three sites in `numerical-behaviour.md`, plus `stable-rust-shape-evidence.md:64` and `kani-bounded-encoder-verification.md:111`) are this ticket's live repairs, all now landed.

**Remainder found outside this ticket's authorized scope — reported, not edited.** `docs/research/cache/build-tool-exercise.md:159`: *"Collecting this driver into the repository gate. Not a gap under the current contract. No `make` target touches `spikes/` — a spike is a recorded measurement …"* This is the same false claim, under `research/cache`, which this ticket was not authorized to touch and which the coordinator's seven-site census did not name. Filing a follow-up ticket for `research/cache` is recommended rather than reaching for it here.

**Checks run at this base, all green:** `make citations` (spikes link count unchanged at 601, all citations/links still resolve), `git diff --check` (no whitespace errors), `tkt lint`, `tkt guard --base 2c312826b60e275002f7332f167dd3c795861b9`.
