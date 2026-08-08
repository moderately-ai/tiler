---
id: replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses
title: Replace four assertions that cannot fail in the cache and spike harnesses
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-replace-f
lease_expires_at: 1786161224
---
## Four assertions with no falsifying input

From the 2026-08-07 read-only audit. Each was read in full by the auditor; **re-verify each yourself before changing it** — the audit is a claim, and one of its own findings needed correcting on a nuance it missed.

**1. `CollectionReport::accounts_for_every_entry` is arithmetically true by construction.** `crates/tiler-cache/src/expansion/collect.rs`. Its only constructor is `collect_at`, which sets `selected: selected.len() as u64` and then loops over **that same vec** with a match over `Result<Disposition, _>` — four `Ok` arms plus `Err`, each incrementing exactly one counter, no `continue`, no early return. Golden and computed come from one loop over one vec, so **no filesystem state, race, lock contention, or corrupted entry can break the equality**. The docstring claims "A collection that dropped an entry it did not report would break this equality." Asserted at **13 sites** (`harness.rs` once, `tests.rs` twelve times); at two of them the population is additionally pinned at zero two lines earlier, so it reads `0 == 0`.

**2. `a_parsed_key_round_trips_to_its_exact_text` — the injectivity loop body executes zero times.** `crates/tiler-cache/src/expansion/fuzz.rs`. `parse_label` requires exactly 64 lowercase-hex bytes; the generator draws length from `below(80)` and characters from a 26-symbol alphabet of which 16 are hex, so P(accept) ≈ (1/80)·(16/26)^64 ≈ 4e-16 per iteration, ~3e-12 expected over 8192. `Gen` is `splitmix64` on a **fixed seed**, so this is **deterministically zero, not merely unlikely** — the `assert_eq!` inside `if let Ok(parsed)` never evaluates. The regression it targets (uppercase hex accepted) would still be zero under that seed. The test's *first* loop does real work; only the injectivity half is inert.

**3. `every_phase_name_round_trips` — `assert!(listed)` is unconditionally true.** `crates/tiler-cache/src/expansion/harness.rs`. The loop iterates `Phase::KILL_POINTS` and every arm of the exhaustive match evaluates to `Phase::KILL_POINTS.contains(&phase)`. `KILL_POINTS` is a fixed `[Self; 9]` whose length is compared to nothing, so a tenth `Phase` added to the enum, to `as_str`, and to the or-pattern compiles cleanly — the loop still yields nine, the assert still passes, and the new phase is **unmeasured and unreachable via `Phase::parse`**, which searches `KILL_POINTS`. The two `assert_eq!`s on `parse` around it are real; only this one is inert.

**4. `fused_operations_are_unexpressible` — a verdict that is a literal tautology.** `spikes/numerics/bf16-second-dtype/src/perturb.rs`. It declares `let admitted = [Operation::Multiply, Operation::Add];` then reports `detected: admitted.len() == 2` — decided at compile time from an array literal two lines above, with **no link of any kind** to `Operation`: no `variant_count`, no exhaustive match. Adding `Operation::FusedMultiplyAdd` forces arms in `apply`/`as_str` but leaves this reporting `[DETECTED]` beside nine perturbations that genuinely can fail — in a module whose own header reads *"Every check this spike reports would still pass if it could not fail."*

## What each needs

**Name the input that would make it fail, and if you cannot, the assertion is not worth keeping in that form.** For 1 and 3 the honest repair is to compare against something the subject does not also produce — an independently obtained population, or a length pinned to the enum. For 2 it is a generator that actually reaches the accepting language, or an explicit corpus. For 4 it is an exhaustive match or `variant_count`, so a widened `Operation` is a build error at the claim.

**Watch each fail, separately, perturbing the subject rather than the assertion**, and quote the message. Four repairs means four demonstrations; one perturbation that reddens several proves none of them individually.

## Scope

`scopes: [implementation/cache]` covers findings 1–3. **Finding 4 is under `spikes/`**, which is a different scope — confirm the mapping in `ticketsplease.toml` and, if it is not yours, **file it separately rather than reaching**. Report which you could and could not reach.

## Outcome — findings 1–3 repaired, finding 4 filed out

Worked at base `aebd16c0`. **All four findings re-verified by reading each subject in full; all four are correct, and for each the falsifying input is `none`** — no runtime input reaches any of them. Details per finding below.

**Scope confirmed.** `implementation/cache` = `crates/tiler-cache/**`, which covers findings 1–3. Finding 4's file is under `spikes/numerics/**` = `research/numerics`, so it was **not** touched; it is filed verbatim-verified as `pin-the-bf16-spike-admitted-operation-verdict-to-its-own-enum`. A second out-of-scope site found while working — the same false claim in `docs/research/cache/bounded-collection.md:39` and a surviving inert assertion at `spikes/cache/hot-path-efficiency/harness/src/main.rs:969`, both `research/cache` — is filed as `correct-the-accounts-for-every-entry-claim-in-the-cache-research-note-and-harness`.

### 1. `CollectionReport::accounts_for_every_entry` — verified

`selected` is the selection's length and the loop walks that same vector, incrementing exactly one counter per element. No filesystem state, race, contention, republication, or unreadable entry can break it. **The method is retained** — it is named in the accepted maintenance boundary (`accept-the-expansion-cache-maintenance-boundary`), so removing it is Tom's — but its doc comment and the module header now state what it does and does not establish. All **13** assertion sites are replaced with checks grounded outside the report:

- **12 in `tests.rs`**, via new `collect_checked` / `collect_at_checked` / `entry_keys_on_disk` fixtures that walk the entries tree with the production path parser on both sides of every collection and require the departed set to equal the named set exactly, the named keys to be distinct, and nothing to appear.
- `concurrent_collectors_do_not_double_count` cannot use a before/after pair (eight collectors mutate at once), so its independent population is the 16 keys the test published: the removals named across every report must be exactly that set, each once. This strictly strengthens the old `removed == 16` count, which two opposite errors could still satisfy.
- **1 in `harness.rs`**, the collecting child, which races real publishers and so cannot use a before/after pair either. It now checks `selected()` against `accounting().entry_count()` and the stated ceiling — the scan and the bound, neither produced by the disposition loop.

### 2. `a_parsed_key_round_trips_to_its_exact_text` — verified, and measured

Confirmed by replaying the exact generator: of 8,192 draws, **87** land at the 64-byte width and **0** of those are all-lowercase-hex; the longest leading hex run among the 87 is **13** of the 64 required. The `assert_eq!` inside `if let Ok(parsed)` never evaluated. The second loop now draws half its cases from the accepting alphabet at widths straddling `KEY_LABEL_BYTES`, keeps the near-miss alphabet for the refusal half, decides the verdict from an oracle computed off the text, checks the exact rejection variant, and re-spells every accepted text with one uppercase letter. Measured populations: **accepted 862, refused 7,330, uppercase re-spellings refused 862**; each is asserted non-empty so the corpus cannot silently stop reaching a half.

### 3. `every_phase_name_round_trips` — verified

`listed` was `KILL_POINTS.contains(&phase)` over a phase yielded by `KILL_POINTS`. Replaced by pinning the list to its own enum: `KILL_POINTS: [Self; mem::variant_count::<Self>()]`, so a tenth phase is an array-length **build error**, plus a runtime pairwise-distinctness assertion so the length cannot be satisfied by a repeated phase. `#![cfg_attr(test, feature(variant_count))]` at the crate root — gated on `test` because `fault` is `cfg(test)`, so ordinary builds of `tiler-cache` still need no unstable feature. Follows the `tiler-metal` idiom. The two `assert_eq!`s on `parse` around it were real and are untouched.

### Watched failing, separately

Each perturbs the subject, not the assertion; each run is filtered to the single test so no perturbation is credited to more than one repair.

1. `remove_if_unchanged` unlinks and reports `Superseded` → `a_collection_names_every_entry_it_removed` fails: *"the entries that left the namespace are not the ones the report named: [three keys] left unnamed, [] named but still present"*. **A probe `assert!(report.accounts_for_every_entry())` placed immediately before it passed on the same perturbed subject** — direct evidence the old assertion cannot fail on an unreported departure.
2. `decode_nibble` folds `A-F` → `a_parsed_key_round_trips_to_its_exact_text` fails: *"text/1: \"Cc7e41…\" is a second spelling of an accepted key and was not refused for its case — left: Ok(CacheKey(Digest(\"cc7e41…\")))"*. **The pre-repair test was then run against the same folding parser and passed green**, which is the regression the ticket named.
3. `AfterRename` replaced by a repeated `AfterFileSync` → `every_phase_name_round_trips` fails: *"a phase is listed twice, so the kill points are one short of the enum and a phase is measured by nothing: [… AfterFileSync, AfterFileSync, AfterDirectorySync] — left: 8, right: 9"*. A tenth `Phase` variant additionally fails the build: *"expected an array with a size of 10, found one with a size of 9"*.
4. `select`'s aggregate pass stops one candidate short → the collecting child fails: *"a collection under an entry ceiling of 1 must select every entry over it, and the scan saw 4 — left: 2, right: 3"*. Driven directly (`harness_collector_child` armed by `TILER_CACHE_HARNESS_COLLECT_ROOT`) because the parent reports only `Terminated` for a dead child.

Every perturbation was reverted and the tree returned to green.
