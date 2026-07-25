---
id: finish-consolidating-tiler-ir-length-framing
title: Finish consolidating the private length-framing copies in tiler-ir
status: todo
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
