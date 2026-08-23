---
id: record-a-dated-run-for-the-qwen3-checkpoint-input-spike
title: Record a dated run for the Qwen3 checkpoint input spike
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [research/program-planning, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, documentation]
claimed_from: todo
assignee: worker-qwen
lease_expires_at: 1787459513
---
## User-visible outcome

The Qwen3 checkpoint input spike carries a dated recorded run, so it can be governed and catalogued honestly rather than left uncatalogued because nothing establishes when it last worked.

## Why this exists

Filed 2026-08-22 by the coordinator as the bounded remainder of [`reach-every-spike-record-from-the-experiment-catalog`](reach-every-spike-record-from-the-experiment-catalog.md), landed as `6cc903e4`. That lane brought spike-record reachability from 61 to 63 of 65 and **correctly refused this one**, hitting the stop condition the ticket set rather than inventing a date.

**Fact — the record carries no dated evidence.** Reported by `worker-catalog` and consistent with the coordinator's independent reachability check. `spikes/program-planning/qwen3-checkpoint-f32-inputs/` contains only `.gitignore`, `Cargo.lock`, `Cargo.toml`, `README.md`, and `src` — no `results/` directory, and the body carries no `## Result on <date>` section. Its sole git history is one commit, `911c24c2` on 2026-08-17, which is a **landing** date rather than a recorded run.

**Why the refusal was right, and must not be undone by stamping a date.** `spikes/README.md`'s currency convention now tells readers to trust `last_verified` as the date the evidence was taken. An invented or landing-derived `last_verified` is **worse than an absent one**, because the convention converts it into a claim a reader will rely on. The repository has already recorded that `last_verified` says *when a human last looked*, not what tree they looked at — which is precisely why it cannot be back-filled from a commit date.

**Inference — this is a real spike, not an abandoned directory.** It is a consumer-owned Cargo workspace ingesting the pinned Qwen3 checkpoint, structurally analogous to its already-catalogued siblings `qwen3-conformance-fixture` and `qwen3-corpus-reachability`, and it is cited by four `done` tickets. So the outcome is to run and record it, not to delete it — but confirm that framing by reading before acting.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict, listing the directory yourself and checking the git history.
- **Run the harness from its own directory** following its README, and record a dated result in the record's own convention — matching how its catalogued siblings record theirs. Note that nothing under `spikes/` is built or run by any `make` target, so this is a deliberate hand run.
- Derive `last_verified` from **that run**, and record `verified_at_commit` for the tree you ran against if that field's definition has by then been settled — see the escalation below.
- Add the governed frontmatter and the catalog row, so the record becomes reachable from the experiment catalog.
- **If the harness cannot be run** — a missing checkpoint, an unavailable input, an API that has moved — **stop and report that instead**. Recording that it cannot currently run, with the reason, is a truthful outcome; a stamped date is not.

## Non-goals

Changing the currency convention. Adding a gate, target, or census over `spikes/`, all eliminated on recorded evidence. Editing the record's substance beyond adding its result. Deciding the `verified_at_commit` vocabulary question, which is escalated to Tom on the parent ticket and is not this ticket's to settle — if it is still open when you run, record `last_verified` alone and say so.

## Closes when

The record carries a dated run derived from an actual execution, its governed frontmatter is present and derived rather than stamped, it is reachable from the experiment catalog, **or** the record states with evidence that the harness cannot currently be run and why.

## Outcome — 2026-08-22

**Fact verdicts, re-audited at this base (`41a018fb`).** Both verified true, still.

- **The directory listing.** `ls -la spikes/program-planning/qwen3-checkpoint-f32-inputs/` shows exactly `.gitignore`, `Cargo.lock`, `Cargo.toml`, `README.md`, `src/` — no `results/` directory, confirmed myself.
- **The git history.** `git log --all --format="%H %ad %s" --date=iso -- spikes/program-planning/qwen3-checkpoint-f32-inputs/` returns two commits, both timestamped `2026-08-17 18:03:57 -0400 spike: ingest pinned checkpoint as F32 inputs` (`911c24c2` and its predecessor `23e64aee`, same content, evidently a rebase) — a landing date, not a recorded run, confirmed myself. Before this change the body carried no `## Result on <date>` section (siblings `qwen3-conformance-fixture` and `qwen3-corpus-reachability` don't use that literal heading either — they date findings inline as `**Measurement — <date>...**` paragraphs under `## Findings`, which is the convention this run now follows).

**The harness ran; no download was needed.** The pinned checkpoint was already present in the local Hugging Face cache at the exact required revision and digest: `shasum -a 256` on `~/.cache/huggingface/hub/models--Qwen--Qwen3-0.6B-Base/snapshots/da87bfb608c14b7cf20ba1ce41287e8de496c0cd/model.safetensors` independently returned `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba`, matching the README's pin, so this run acquired nothing and installed nothing.

From `spikes/program-planning/qwen3-checkpoint-f32-inputs/`: `cargo build --release` (clean), `cargo run --release -- <the snapshot path above>` (widened digest `d2abe344f7a4e4c0ea79c4a3c524ca851b095d930064e086d980972fe95c8437`, census `nan=0 infinite=0 subnormal=0`, matching this ticket's own 2026-08-17 landing measurement exactly), `cargo nextest run --no-capture` (8/8 pass, six negative controls printed their refusal text), `cargo clippy --all-targets -- -D warnings` (clean), `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` (clean). Full commands and output are recorded in the record's own new `## Findings` section rather than duplicated here.

**Frontmatter and catalog.** Added governed `tiler-doc/v1` experiment frontmatter to the record with `last_verified: "2026-08-22"` derived from this run (not stamped, not from the commit date), and a `## Findings` section carrying the dated measurements. Added the record's row to `spikes/README.md`'s experiment catalog under "Physical planning and lowering", beside its two siblings. Also appended the new record to the `experiments:` clauses of the two research rows it `supports` in `docs/research/README.md` (`complete-model-ingestion-and-execution` and `first-metal-lm-workload`), per [the metadata contract](../docs/document-metadata.md#validation-and-catalog-updates)'s "edit the affected catalog entry in the same change that edits the metadata behind it."

**`verified_at_commit` — left unset.** Still not defined in `docs/document-metadata.md` at this base (`grep -c verified_at_commit docs/document-metadata.md` → 0) and the parent ticket's escalation is still open, so per this ticket's own instructions only `last_verified` was recorded, and the record says so.

**Scopes.** Added `research/program-planning` (covers `spikes/program-planning/**` and `docs/research/program-planning/**`, per `ticketsplease.toml`) and `contracts/navigation` (covers `spikes/README.md` and `docs/research/README.md`) — scheduling metadata for the paths this ticket's required work touches, added via `tkt set --add-scope`.

**Checks.** `cargo nextest run --no-capture`, `cargo clippy --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` all passed in the spike's own isolated workspace (see above). `tkt lint`, `make citations`, `git diff --check`, and `tkt guard --base 41a018fb tkt/record-a-dated-run-for-the-qwen3-checkpoint-input-spike` were run against this branch; see the commit for their output. No file under `crates/`, `Makefile`, `Cargo.toml`/`Cargo.lock` (root), or `check-citations.sh` was touched.
