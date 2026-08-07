---
id: date-adr-0079-s-one-crate-claims-for-the-second-diverging-member
title: Date ADR 0079's one-crate claims for the second diverging member
status: done
priority: p3
dependencies: []
related: [record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [docs, doc-drift]
---
## User-visible outcome

[ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md)'s Consequences state the extent of the unsafe exception as of *today* rather than as of a superseded 2026-07-25, so a reader auditing the workspace's unsafe posture finds two diverging members where the record names one.

## Why this exists

Found on 2026-08-07 by [`refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew`](refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew.md), which was repairing the same defect class in ADR 0106 and read ADR 0079 in full to check whether it carried it. It does, and it is out of that ticket's scope: ADR 0079 is a different record and its own decision is untouched.

**Fact — there are now two members that drop `[lints] workspace = true`, verified at `3e0074d5`.** `for f in crates/*/Cargo.toml prototypes/*/Cargo.toml; do grep -q '^\[lints\]' "$f" || echo "$f"; done` returns `crates/tiler-conformance/Cargo.toml` and `prototypes/serial-sum-run/Cargo.toml`. Both declare `[lints.rust] unsafe_code = "deny"`; every other member inherits the workspace `forbid`. Tom authorized the second on 2026-08-07 on [`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md), which is exactly the acceptance ADR 0079 item 4 reserves to him, so **the decision is not violated — only the record's arithmetic is stale.**

The stale statements, each in unpinned present tense:

- **Consequences bullet 2** — "spent only in the one layer that must speak to an Objective-C API". Now two.
- **Consequences bullet 4 (the per-site gap)** — "Nothing counts, locates, or constrains `#[allow(unsafe_code)]` attributes inside **the one crate permitted to have them**". Two things changed. The population is two crates, and the gap is now asymmetric: `tiler-conformance` carries `bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones`, which walks `src/`, counts the `unsafe` blocks and the reasoned allows, and fails on a third site or a new file carrying one — precisely the check this bullet says nothing performs, implemented in-crate rather than in the deleted Python gate. It does **not** cover `prototypes/serial-sum-run`, where the gap is unchanged.
- **Consequences bullet 5** — "A production runtime crate will face this boundary again. Nothing here pre-approves it: `tiler-prototype-run` is a non-published proof executable". A `crates/` member now has the divergence. It is still not a *reusable library* — `tiler-conformance`'s reverse-dependent set is empty and stays empty under ADR 0106 item 2 — so the bullet's actual reservation survives and its stated ground does not.

`docs/decisions/0079…md:31`'s complete-extent grep is pinned to `43f685f` and is correctly left alone.

## How to repair it

**Date rather than overwrite**, for the same reason [ADR 0106's refresh](refresh-adr-0106-and-the-architecture-rows-the-conformance-crate-outgrew.md) did: every statement above was true when accepted, which is the ADR 0077/0088 shape. Append a dated `**Superseded — 2026-08-07**` note to each affected bullet naming what changed, with the verifying command and the commit it was run at. Do not substitute — that shape is reserved for a clause that was never true at any commit, as `correct-the-scope-set-claim-in-adr-0106-s-missing-component-evidence` used it.

Also worth stating in the same pass, because it is the most consequential half: the in-crate site-population test is real enforcement of item 3 that ADR 0079's Implementation boundary lists as review-only, and it is a *pattern* the prototype could adopt without re-implementing the deleted Python gate. Whether it should is [`pin-the-admitted-unsafe-sites-in-the-workspace-gate`](pin-the-admitted-unsafe-sites-in-the-workspace-gate.md)'s question, not this ticket's.

## Explicit non-goals

Do not change what ADR 0079 decides — items 1 through 4 stand, including that a second member dropping inheritance needs Tom, which is exactly what happened. Do not re-open the unsafe rule, the site count, or the workspace `forbid`. Do not edit `crates/` or `prototypes/`; this is a documentation repair.

## Closes when

Each stale Consequences bullet carries a dated note with its verifying command and commit, the asymmetry between the two diverging members is stated rather than averaged, and no repaired site carries a bare count without its command and commit.

## Repaired before dispatch, 2026-08-07 — as written this ticket would have put a FALSE claim into an accepted ADR

Verified by the coordinator reading `docs/decisions/0079-…md`, `crates/tiler-conformance/src/bf16_vertical/tests.rs` and `AGENTS.md` in full.

### The one false Fact, and it is the dangerous one

Struck: "the in-crate site-population test is **real enforcement of item 3** that ADR 0079's Implementation boundary lists as review-only."

`the_unsafe_site_population_is_the_two_named_ones` (`crates/tiler-conformance/src/bf16_vertical/tests.rs:497-548`) asserts a file-count floor (`files.len() >= 12`, `:505`), that `unsafe {` and the literal `unsafe_code,` appear in no file but `device_buffer.rs` (`:521-531`), and that `blocks == 2` and `allows == 2` (`:536`, `:544`). It checks **none of item 3's four conditions** (`ADR:57-60`): not structural unavoidability, not the `reason` *text* — it counts the token `unsafe_code,` and never reads the string — not the bounding assertion, not the `SAFETY` comment.

So **`ADR:85` and `ADR:87` are still true and must not be dated.** Dispatched as written, a worker would write a false enforcement claim into an accepted ADR. What the test actually closes is Consequences bullet 4's counting/locating gap, and only for `tiler-conformance`.

### The site population is four — and the obvious command finds none of them

State it explicitly, with a command that works. `grep -rn 'allow(unsafe_code' --include='*.rs' crates prototypes` returns **exactly one hit, and it is a doc comment** (`crates/tiler-conformance/src/lib.rs:110`) — zero of the four real attributes, because all four wrap across lines. Use:

```sh
python3 -c "import re,glob; print(sum(len(re.findall(r'#\[allow\(\s*unsafe_code', open(f).read())) for f in glob.glob('crates/**/*.rs',recursive=True)+glob.glob('prototypes/**/*.rs',recursive=True)))"
```

Four attributes, four `unsafe` blocks, two crates: `crates/tiler-conformance/src/device_buffer.rs` (`write_bytes` attr `:61-64` block `:80`; `read_bytes` attr `:91-94` block `:110`) and `prototypes/serial-sum-run/src/buffer.rs` (`write_f32` attr `:35-37` block `:52`; `read_f32` attr `:67-69` block `:85`). Both crates declare `[lints.rust] unsafe_code = "deny"`; every other member inherits `forbid`.

### `Closes when` under-covered its own repair list — seven more sites

It said "each stale **Consequences** bullet", but four of the stale claims are in the Decision section and two more are elsewhere. A worker could satisfy it and leave the ADR still saying "the diverging crate". Extend to:

- **`ADR:47`** — "**Both admitted sites** carry `#[allow(unsafe_code, reason = "…")]`…" Four now. Its second half ("neither the crate root nor any module carries one") **remains true** — no `#![allow` exists in either `src/` tree — so date the count without disturbing it.
- **`ADR:49`** — "**The diverging crate** replaces `forbid` with `deny`… throughout **that crate**." Two diverging crates.
- **`ADR:59`** — "**Both landed sites** assert the byte length they are about to touch against `buffer.length()`." Four, and all four do assert.
- **`ADR:64`** — "**A third site is a new decision.**" Reads prospective; the third and fourth landed 2026-08-07 under Tom's conformance rule.
- **`ADR:75`** — carries **two independently false clauses**, and this one the ticket missed entirely: "so **nothing enforces the pins now**" (something now enforces a count-and-location pin inside `tiler-conformance`) and "**`AGENTS.md` correctly states that no check keeps an inventory of admitted sites**" (`grep -in inventor AGENTS.md` is **empty**; the surviving text is `AGENTS.md:220`, and commit `7b1e3a7e` removed the clause).
- **`ADR:71` and `ADR:73`** — "the **one layer**" and "the **one crate** permitted to have them", already named by this ticket.
- **`ADR:83`** — the Implementation boundary's status paragraph, which the repair to `:35` above changes the reading of.

Also check **`ADR:78`** ("`tiler-prototype-run` is a non-published proof executable") — classify it; the argument it supports may now need `tiler-conformance` named beside it.

### Non-goals: add `AGENTS.md`

`ADR:75`'s false AGENTS.md clause is repairable **inside the ADR**, by restating what AGENTS.md says now. `AGENTS.md` itself is `implementation/workspace`, which this ticket does not declare — and the existing non-goals forbid only `crates/` and `prototypes/`, so a worker who decided to "restore" the sentence would escape scope silently. **Do not edit `AGENTS.md` from this branch.**

### Live scope collision — sequence these two

`pin-the-admitted-unsafe-sites-in-the-workspace-gate` is `todo` with `scopes: [implementation/workspace, contracts/navigation, contracts/decisions]`, and its settled work rewrites exactly `ADR:73-77` — the bullet this ticket dates. **Run `tkt why` on the pair before batching, and do not dispatch them together.** This ticket should land first: it corrects what the ADR *says*, and the other changes what the gate *does*, so dating first avoids the pin ticket rewriting text that is still wrong.

### Facts that verify unchanged

Two members (`:21`); the walk not covering `prototypes/serial-sum-run` because it roots at `CARGO_MANIFEST_DIR/src` (`tests.rs:500`); the empty reverse-dependent set (no manifest declares `tiler-conformance` as a dependency); and the `43f685f` pin (`ADR:31`).


> **The command above was itself wrong and is corrected, 2026-08-07.** It returned **5**, not 4 — it matched the `crates/tiler-conformance/src/lib.rs` doc comment, which is precisely the false positive it was written to exclude. Found by the worker on [`date-adr-0079-s-one-crate-claims-for-the-second-diverging-member`](date-adr-0079-s-one-crate-claims-for-the-second-diverging-member.md) rather than by the coordinator who wrote it. The fix is the `^\s*` line anchor, and printing **per-file locations rather than a bare total**, so a miscount is visible instead of merely wrong:
>
> ```sh
> python3 -c "
> import re, glob
> pat = re.compile(r'^\s*#\[allow\(\s*unsafe_code', re.M)
> for f in sorted(glob.glob('crates/**/*.rs', recursive=True) + glob.glob('prototypes/**/*.rs', recursive=True)):
>     n = len(pat.findall(open(f).read()))
>     if n:
>         print(n, f)
> "
> ```
>
> Correct output is two files at two each: `crates/tiler-conformance/src/device_buffer.rs` and `prototypes/serial-sum-run/src/buffer.rs`. Substituting `r'^\s*unsafe\s*\{'` gives the identical two files at two each, and **that pairing is the evidence that no block escaped its attribute** — a count alone cannot show it. This is the same defect class the ticket exists to prevent, committed in the ticket's own repair text.

## Outcome — done, 2026-08-07

Landed at **`d4863d6d`**. **41 insertions, 0 deletions** — every original line survives byte-for-byte, following the dated-correction convention. `ADR:85` and `ADR:87` were **left standing undated**, which was the trap: the in-crate test enforces none of item 3's four conditions, so both paragraphs remain true.

**The worker found two errors in the coordinator's brief**, both of the class this ticket exists to prevent:

1. **The enumeration command returned 5, not 4** — it matched the `crates/tiler-conformance/src/lib.rs` doc comment, precisely the false positive it was written to exclude. Fixed with a `^\s*` line anchor and per-file output rather than a bare total. Corrected here and in [`pin-the-admitted-unsafe-sites-in-the-workspace-gate`](pin-the-admitted-unsafe-sites-in-the-workspace-gate.md).
2. **The AGENTS.md inventory clause was removed by `6a2360f9`** (2026-08-06), not `7b1e3a7e` — which rewrote the sentence but kept the clause. `git log -S'inventor' -- AGENTS.md` returns `6a2360f9` and `99bc4c77`, not `7b1e3a7e`.

It also added a nuance the brief missed and the coordinator verified: **`spikes/` is outside the workspace member set**, inherits no lint table, and carries 8 unsafe blocks of its own across 5 files that this ADR does not govern. And it dated `ADR:55`, which the brief's site list had omitted.

Verified independently: zero deletions; no crate-root `#![allow]` in either diverging member; exactly two members drop `[lints] workspace = true`; `Cargo.toml:33` sets `publish = false`. Delta is `docs/` only, so it carries the green gate; `tkt lint` rerun.
