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
