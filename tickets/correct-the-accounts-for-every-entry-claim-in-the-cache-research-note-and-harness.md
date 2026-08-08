---
id: correct-the-accounts-for-every-entry-claim-in-the-cache-research-note-and-harness
title: Correct the accounts-for-every-entry claim in the cache research note and hot-path harness
status: in-progress
priority: p2
dependencies: []
related: [replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses]
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-correct-t
lease_expires_at: 1786162862
---
## Two `research/cache` sites still make the claim the code no longer supports

Split out of `replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses`, whose scope is `implementation/cache` and could not reach `docs/research/cache/**` or `spikes/cache/**`. That ticket established the underlying fact and corrected every statement inside `crates/tiler-cache`; these two are the same claim in scopes it could not edit.

## The fact they contradict

`CollectionReport::accounts_for_every_entry` is arithmetically true by construction. The collecting step sets `selected` to the selection's length and then walks that same vector once, incrementing exactly one of the five counters per element, with no `continue` and no early return — so both sides of the equality are one loop's iteration count. **No filesystem state, race, lock contention, republication, or unreadable entry can make it false.** Demonstrated at base `aebd16c0`: perturbing `remove_if_unchanged` to unlink the entry and report `Superseded` — an entry leaving the cache entirely unnamed — leaves `accounts_for_every_entry()` returning `true`, while the disk-grounded check added under the parent ticket fails with the three departed keys listed.

Its doc comment in `collect.rs` now states what it does and does not establish, and the thirteen in-crate assertion sites were replaced with checks grounded outside the report.

## The two sites

**1. `docs/research/cache/bounded-collection.md:39`** (scope `research/cache`) says:

> `CollectionReport::accounts_for_every_entry` is the structural form of the rule: the five dispositions — removed, contended, superseded, already absent, failed — are disjoint and total over the selection, so an entry cannot leave without a line in the report that removed it. The collecting process asserts it on every round.

Two clauses are now wrong. The **"so"** does not follow — disjoint and total *over the selection* does not bound what left the *namespace*, which is the population the sentence's conclusion is about. And "the collecting process asserts it on every round" no longer describes the code: the collecting child in `expansion::harness` now checks its selection against the scan and the stated bound instead. Amend in place with a dated note, preserving the original text, matching how the same document already carries its 2026-08-04 amendment.

**2. `spikes/cache/hot-path-efficiency/harness/src/main.rs:969`** (scope `research/cache`) still asserts it:

```rust
assert!(
    report.accounts_for_every_entry(),
    "every selected entry has exactly one recorded disposition",
);
```

Its message happens to be *accurate* — it claims only the partition, not that nothing left unreported — so this is a measurement harness carrying an assertion that cannot fail rather than a false claim. The line two above it (`removed().len() == population`) is the real check and is falsifiable. Either drop the inert assertion, or ground it the way the crate's own harness now does. Decide by reading; do not assume the first.

## Not in scope here

`crates/tiler-cache` is finished and must not be reopened. Closed tickets (`design-bounded-expansion-cache-garbage-collection`, `decide-the-expansion-cache-collection-schedule`, `accept-the-expansion-cache-maintenance-boundary`) repeat the claim as historical record and are deliberately left alone — they are what the decisions were argued from.

## Checks

Touches `spikes/`, which the gate does not cover; the hot-path harness is its own workspace, run manually from the command its `README.md` documents. Run `make citations` and `tkt lint` for the documentation half.

## Per-Fact audit at base `7a7452bd`

- **"Arithmetically true by construction", and the loop shape behind it — verified by reading.** `crates/tiler-cache/src/expansion/collect.rs "selected: selected.len() as u64"` is followed by `for entry in &selected` over that same vector, a five-armed match each incrementing exactly one counter, no `continue` and no early return.
- **"The thirteen in-crate assertion sites were replaced" — verified.** `git show 0132c0c3:…/tests.rs | grep -c` gives 12 and the same on `harness.rs` gives 1; at `7a7452bd` the crate holds zero assertions of it and one doc reference.
- **"Its doc comment now states what it does and does not establish" — verified.** Landed at merge `62f46778`.
- **Site 1, the quoted sentence at `bounded-collection.md:39` — verified verbatim, and both named clauses are wrong.**
- **Site 2, the assertion in the spike harness — verified verbatim.**
- **"The line two above it (`removed().len() == population`)" — imprecise but harmless.** That check is an `assert_eq!` spanning four lines, closing two lines above the inert one. The substance — that it is the falsifiable check and the other is not — holds.
- **"Two sites" — incomplete.** Five more in-scope statements make the same claim: `bounded-collection.md`'s measurement list item 2, three sentences in `hot-path-efficiency.md`, and one in the spike's `README.md`. All are corrected here.

## Outcome

Seven sites corrected across four files, none in `crates/tiler-cache`. The inert harness assertion was **replaced rather than dropped**: the destructive collection now re-scans with `account` after the timed call and requires the namespace to be empty, which covers the one direction a report cannot speak to — removals it names but did not perform. Proven able to fail by taking the scan before the collection instead of after: `assertion left == right failed … left: 100, right: 0`. Verified with `cargo build --release && ./target/release/cache-hot-path-efficiency --quick` in `spikes/cache/hot-path-efficiency`, its own workspace, run by hand. No recorded result was produced, replaced, or withdrawn — the new scan is outside the timed call, so no retained figure moves.

`spikes/cache/hot-path-efficiency/Cargo.lock` moved as a side effect of building: it predated `tiler-digest`'s extraction, and any build there regenerates it. Kept rather than reverted, so the next person's build is clean.
