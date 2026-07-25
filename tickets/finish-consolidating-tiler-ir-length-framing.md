---
id: finish-consolidating-tiler-ir-length-framing
title: Finish consolidating the private length-framing copies in tiler-ir
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, identity]
---
`crates/tiler-ir/src/identity.rs` was introduced to be the one definition of canonical length framing, and its own module documentation states why: "two encoders that disagree by one byte name the same subject with two different identities — and nothing downstream can tell that from two genuinely different subjects." That consolidation was incomplete. Three private copies remain inside `tiler-ir`, and one of them is written in the exact form `identity.rs` names as the hazard it removed.

**Fact (inspected source, base `f286289`).** `crate::identity::{push_len, push_slice}` is imported by `shape/env.rs`, `shape/env/constraint.rs`, `program/abi.rs`, `program/model.rs`, and `kernel/model.rs`. Reproducible as `grep -rn "crate::identity" crates/tiler-ir/src/`, which returns exactly those five. The following encoders frame lengths themselves instead:

- **`crates/tiler-ir/src/schedule/model.rs` — narrowing `as` casts, four sites.** `encode_identity` and its helpers write `(shape.rank() as u64).to_be_bytes()` (line 464), `(axes.len() as u64)` (471), `(region.index.accesses.len() as u64)` (678), and `(region.index.bounds_proofs.len() as u64)` (682). `identity.rs` documents that the copy it replaced "narrowed with an `as` cast where the others used a checked conversion", and that `push_len` converts checked "so that a future 128-bit host fails loudly here instead of silently truncating a length and colliding two distinct subjects onto one identity". This module still has the form that was fixed.
- **`crates/tiler-ir/src/semantic/types.rs` — a private checked pair.** `fn encode_len` at line 1350 and `fn encode_bytes` below it duplicate `push_len`/`push_slice` exactly, including the checked conversion.
- **`crates/tiler-ir/src/index/scalar.rs` — a private checked pair.** `pub(super) fn encode_len` and `pub(super) fn encode_bytes` do the same, and are re-used across the `index` module.

**Inference — latent, not live.** On the 64-bit little-endian profiles the Rust gate admits, `usize` is `u64`, so `as u64` and `u64::try_from(..).expect(..)` emit identical bytes. No identity is currently wrong. That is precisely the hazard `identity.rs` describes: the divergence is invisible until a host or a length changes, and a silent digest change is indistinguishable in a cache from a real one.

**What closes this.**

1. Route all three through `crate::identity::{push_len, push_slice}`, deleting the private copies. Keep the `encode_bytes`/`push_slice` naming consistent at each call site rather than aliasing.
2. Confirm the change is byte-identical. Every existing identity test should pass unchanged; a moved identity would mean one of the copies was *not* equivalent, which is a stronger finding than this ticket assumes and should be reported rather than rebaselined.
3. Add whatever check keeps a fourth copy from appearing. A test that no identity encoder in the crate formats a length outside `identity.rs` is hard to write directly; naming the rule in `identity.rs`'s documentation and in the module docs of each converted encoder is the cheap version, and is what this ticket should deliver unless a mechanical check turns out to be practical.

Found while reading `schedule/model.rs::encode_identity` for `unify-schedule-index-region-with-verified-index-region`, which is a different question about the same file and should not silently rebaseline identity encoders as a side effect.

## Outcome

**Done, and there were five copies rather than three.** `crates/tiler-ir/src/identity.rs`, `schedule/model.rs`, `semantic/{types,registry,identity}.rs`, `index/{scalar,builder}.rs` changed. Byte-identical: all 768 workspace tests pass unchanged, including every identity fixture.

**Gate status.** `uv run --locked python scripts/check_rust.py` passes in full; `tkt lint` and `git diff --check` are clean. `uv run --locked python scripts/check_repository.py` exits 1 on two `scripts/docs.py validate` errors about `accept-adr-0077-metal-aot-crate-admission` and `accept-adr-0078-public-extension-seams`, both reproduced at base `63b02ec` in a clean detached worktree and both outside `implementation/ir`. Not introduced here, not fixed here.

### Correction: this ticket's own search undercounted, and the reason generalizes

**Fact.** The two copies this ticket did not name are `crates/tiler-ir/src/semantic/identity.rs:346` (`fn encode_len`, used by `encode_string` and `encode_shape` and by six sites in the semantic-graph encoder) and `crates/tiler-ir/src/semantic/registry.rs:2391` (`fn encode_len` and `fn encode_bytes`, used by the registry snapshot and projection encoders).

**Why they were missed.** This ticket's stated reproduction was `grep -rn "crate::identity" crates/tiler-ir/src/`, which enumerates the encoders that *already comply*. Whoever does not import the shared module is invisible to it. The search that finds copies is `grep -rn "fn encode_len\|fn push_len" crates/tiler-ir/src/`, which returns all five at base `63b02ec`. This is the corpus's own rule about absence claims — a failed search is evidence the search was wrong — applied to a search that did not fail loudly but silently answered a different question.

### What changed

- **`schedule/model.rs`** — the four narrowing `as u64` casts in `push_shape`, `push_axes`, and `encode_identity` now call `push_len`. This is the form `identity.rs` documents as the one it removed.
- **`semantic/types.rs`, `semantic/registry.rs`, `index/scalar.rs`** — private `encode_len`/`encode_bytes` pairs deleted; every call site now spells `push_len`/`push_slice` directly rather than aliasing, as the ticket asked.
- **`semantic/identity.rs`** — private `encode_len` deleted. Its `encode_string` wrapper survives, reimplemented as `push_slice(output, value.as_bytes())`, which is byte-identical because `str::len` *is* the UTF-8 byte length. The wrapper exists for the `&str` conversion, not for a second framing rule.
- **`index/builder.rs`** — 49 call sites repointed from `super::scalar` to `crate::identity`.

**Byte-identical confirmed, so nothing was rebaselined.** All five copies were equivalent in effect: the four helpers used the same checked conversion and the same eight-byte big-endian form, and `schedule/model.rs`'s casts agree with a checked conversion on the 64-bit profiles the Rust gate admits. The ticket asked for a stronger finding to be reported if an identity moved; none did.

### Point 3: a mechanical check, because prose had already failed twice

The ticket offered documentation as the cheap version. The evidence argues against it: `identity.rs` already existed, already stated the rule in its module documentation, and five copies grew anyway — two of them invisible even to the ticket written to remove them.

`crate::identity::tests::length_framing_has_exactly_one_definition_in_this_crate` walks the crate's own `src/` tree and fails on a definition of `push_len`/`push_slice`/`encode_len`/`encode_bytes` outside `identity.rs`, or on an open-coded `.len() as u64` / `.rank() as u64`. Both forms are taken from copies that actually existed — four were a helper pair, the fifth had no helper at all. Verified to have teeth rather than assumed: the same rule run against base `63b02ec` reports all nine production sites (`git grep -n -e "fn encode_len(" -e ".len() as u64" -e ".rank() as u64" 63b02ec -- crates/tiler-ir/src/`).

**Bound stated in the test itself.** It reads each file only to its first `#[cfg(test)]` line, so it governs production encoders and leaves test expectations alone. That exemption is deliberate: `shape/env.rs` asserts its identity begins with the domain's eight-byte length prefix by spelling that prefix out *independently*, and that independence is exactly what would catch `identity.rs` changing the framing width. A test that checked the encoder using the encoder's own helper could not.
