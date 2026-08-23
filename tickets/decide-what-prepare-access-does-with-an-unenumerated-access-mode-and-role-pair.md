---
id: decide-what-prepare-access-does-with-an-unenumerated-access-mode-and-role-pair
title: Decide what prepare_access does with an unenumerated access-mode and role pair
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing]
claimed_from: todo
assignee: worker-prepaccess
lease_expires_at: 1787486699
---
## User-visible outcome

`prepare_access` states what it does with every access-mode and tensor-role pair, so a widened `TensorRole` is a decision someone makes rather than a case silently admitted.

## Why this exists

Filed 2026-08-23 by the coordinator from `worker-drafttrav`'s stop condition on [`make-the-draft-time-index-traversals-outside-compact-rs-exhaustive`](make-the-draft-time-index-traversals-outside-compact-rs-exhaustive.md), which landed as `192153a2` and took every non-test rest pattern in the index builder to zero. That lane reported this site and **declined it deliberately**, because repairing it is not mechanical — which is exactly why it is its own ticket rather than a line in that sweep.

**Fact — reported by that lane, NOT verified by the coordinator.** `crates/tiler-ir/src/index/builder.rs` carries `_ => {}` in `prepare_access`, matching over a `(AccessMode, TensorRole)` pair. Re-derive it at your base before acting; it is a worker report and secondhand.

**Why it is not the same as the sweep's other sites.** Every arm that sweep repaired was a record walk where binding the elided fields to `_` changed nothing and prevented a silent miss. This one is different: the wildcard is over a **pair of enums**, and a new `TensorRole` reaching `prepare_access` would be *silently admitted* rather than merely unvisited. Making it exhaustive therefore means **deciding what a hypothetical new role should do** — admit, refuse, or refuse by name — and that is a policy question about a vocabulary that does not exist yet.

**Inference — so the honest outcomes are not only "make it exhaustive".** Enumerating the current pairs with an explicit arm per role is one. Recording why a wildcard is correct here, with a reconsideration trigger tied to `TensorRole` gaining a variant, is another and may be the better one. A third is a typed refusal for the unenumerated case, which is fail-closed but adds a diagnostic nobody can currently reach — and this repository has repeatedly found that a check whose failing case is unreachable is worse than no check, because it reads as covered.

## Required work

- Re-audit the Fact at your base with a verdict, and read `prepare_access` in full before deciding anything.
- Establish whether a `TensorRole` variant can be added without touching this function — that is the question the whole ticket turns on. `core::mem::variant_count` over `TensorRole`, or an exhaustive match elsewhere that would already break, may settle it.
- **Decide by reading between the three outcomes above.** If a wildcard is genuinely correct, record why at the site with a reconsideration trigger; that is a valid close and better than an enumeration that guesses at admission policy.
- If you enumerate, **state what each new arm does and why** — an arm that silently mirrors the old wildcard has changed nothing except to make the next reader think a decision was taken.
- **Before trusting any refusal you add, state what it would take for it to fire and confirm that case is reachable.** If it is not reachable, say so rather than landing an unreachable diagnostic.
- State whether any identity value moves. Expected: none. Rederive rather than assume.

## Non-goals

The record walks already made exhaustive by `192153a2` and `f5f4cff1`. The identity encoders, done by `a0659d05`. Adding a `TensorRole` variant. Changing what `prepare_access` does for any pair that exists today.

## Closes when

`prepare_access` either enumerates its pairs with a stated reason per arm, or records why a wildcard is correct there with a reconsideration trigger tied to the vocabulary growing, no identity value has moved, and any refusal added has been shown reachable or declared unreachable on purpose.

## Worker verdict — 2026-08-23 by `worker-prepaccess` at base `6266dd92`

**Fact verdict: verified.** Read `prepare_access` in full at `crates/tiler-ir/src/index/builder.rs:1851-1918`. At base it read `match (mode, tensor_data.role) { (AccessMode::Read, TensorRole::Output) => ..., (AccessMode::Write, TensorRole::Input) => ..., _ => {} }`, matching over `(AccessMode, TensorRole)` exactly as reported. `git show 192153a2` touches `builder.rs` only in `expression_reads_environment` and `check_index_node_integers` (an `IndexNode` walk), never `prepare_access` — the sweep's own diff confirms the decline was deliberate, not an oversight.

**Can a `TensorRole` (or `AccessMode`) variant be added without touching `prepare_access`? Established empirically, not assumed.** Added a throwaway `PerturbationProbe` variant to each enum in `index/model.rs` in turn and ran `cargo check -p tiler-ir --all-targets`, then reverted before every subsequent step (final `git diff` against `model.rs` is empty).

- Pre-fix (base `_ => {}`): widening `TensorRole` broke exactly 3 sites — `builder/compact.rs:865`, `builder/identity.rs:191`, `builder/identity.rs:544` (all exhaustive `match role { Input => 1, Output => 2 }` canonical-byte encoders) — and did **not** touch `prepare_access`, which compiled clean through the new variant. Widening `AccessMode` broke exactly 1 site — `builder/identity.rs:580-581` — plus a test array-size mismatch in `builder/tests.rs:753` (`const TAGS: usize = DIRECT_MODES + ACCESS_KINDS - 1`), and likewise never touched `prepare_access`.
- So at base, the answer is **yes** — a new variant compiles straight through `prepare_access` while breaking elsewhere. The encoders force a byte-tag decision; nothing forces the read/write admission-policy decision this ticket is about. That is the exact danger the ticket names, confirmed rather than assumed.

**Outcome chosen: 1 — enumerate, not 2 (bare wildcard-with-comment).** Replaced `_ => {}` with the two remaining pairs named explicitly: `(AccessMode::Read, TensorRole::Input) | (AccessMode::Write, TensorRole::Output) => {}` (merged into one arm after `clippy::match_same_arms` flagged two identical `{}` bodies as separate arms — `make full`'s lint step caught this and is reported below). No behavior changes for any of the 4 pairs that exist today: the two error arms are untouched, and the two previously-wildcarded pairs still admit silently, now named instead of defaulted. A comment above the match states why (`AGENTS.md`'s ticket text framed enumeration as viable only if each arm states what it does and why — done inline).

This beats outcome 2 (bare wildcard plus comment) because a comment is not reachable by a future editor who doesn't read it, whereas an exhaustive match **is** reachable — verified below — and beats outcome 3 (typed refusal) because there is no unenumerated case to refuse: `AccessMode` and `TensorRole` are both closed 2-variant enums with no `#[non_exhaustive]`, so every value the match receives today is one of the 4 named pairs. A refusal arm would be dead code.

**Reachability check, repeated against the fix.** Re-ran the same `PerturbationProbe` perturbation on both enums with the repaired match in place. Both times `builder.rs:1869` (the `prepare_access` match) now appears in the error set alongside the pre-existing encoder sites — e.g. widening `TensorRole` now reports `error[E0004]: non-exhaustive patterns: (AccessMode::Read, TensorRole::PerturbationProbe) and (AccessMode::Write, TensorRole::PerturbationProbe) not covered --> builder.rs:1869`. So the "check" this outcome adds is compiler exhaustiveness, it fires the moment either enum gains a variant, and that case is directly reachable — demonstrated, not asserted. Negative control (base, unperturbed): `cargo check -p tiler-ir --all-targets` exits 0.

**Identity: no value moves.** The match's set of `(mode, role)` inputs and their outcomes (two errors, two no-ops) is unchanged for every pair that exists today; only the wildcard is replaced by naming what it already covered. `AccessData::Direct` construction and the canonical byte encoders (`compact.rs`, `identity.rs`) are untouched by this diff — confirmed by `git diff` touching only `builder.rs`.

**Gates.** `cargo fmt -p tiler-ir -- --check`: clean. `cargo clippy -p tiler-ir --all-targets -- -D warnings`: clean after merging the two OK-arms (see above). `git diff --check`: clean. `tkt lint`: `ok: no problems found`. `DEVELOPER_DIR=/Applications/Xcode.app TILER_REQUIRE_METAL_TOOLCHAIN=1 make full`: see commit for result. `tkt guard --base 6266dd92a7c1d2f48f71070c969ef068890e82be tkt/decide-what-prepare-access-does-with-an-unenumerated-access-mode-and-role-pair`: see commit.
