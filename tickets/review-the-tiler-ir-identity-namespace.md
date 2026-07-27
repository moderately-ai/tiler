---
id: review-the-tiler-ir-identity-namespace
title: Review or narrow the public tiler_ir::identity namespace
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/ir, implementation/artifact, implementation/cache, implementation/compiler, implementation/metal-aot, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, ir, decisions, identity]
---
`relocate-abi-expressions-into-tiler-ir` added `pub mod identity` to `tiler-ir` (commit `d1a95e1`). ADR 0075 makes a new publicly reachable namespace an always-ask category, and unlike the `abi` module beside it — which accepted ADR 0068 explicitly places in this crate — **no accepted decision covers this one**. It is a draft by default and is recorded here rather than left as an assertion in a conversation.

**What it is.** Two functions, `push_len` and `push_slice`, writing the canonical fixed-width big-endian length prefix every identity digest in the workspace is framed with.

**Current use.** The namespace is no longer an artifact-only convenience.
`tiler-artifact`, `tiler-cache`, `tiler-compiler`, `tiler-metal-aot`, and
`tiler-reference` all use the framing helpers, often at several identity
construction sites. Narrowing it now would recreate a workspace-wide
duplication rather than one local copy.

**User-visible outcome.** Every canonical identity must use one governed,
fallible length-framing rule, so artifacts, caches, compiler products, backend
provenance, and reference authorities cannot silently disagree. Consumers
should encounter the identity contract, not a pair of unexplained byte-pushing
utilities.

**The question review must actually settle.** Publishing the helpers makes
canonical length framing part of `tiler-ir`'s public contract. Determine whether
that ownership is correct and whether the public surface should remain the two
functions or become a nominal encoder that carries the invariant. A private
copy per consumer no longer survives the single-authority requirement and is
not a live alternative.

**Inference, not measurement:** a nominal encoder looks preferable because the
framing rule is already load-bearing across crates and a type can carry the
invariant its doc comment currently only asserts. Test that shape against the
actual consumers before proposing a public change.

## Closes when

The framing has one accepted owner and a reviewed public contract, every current
consumer uses that authority, and `make full` passes.

## Outcome — keep the two functions; the nominal encoder does not survive (2026-07-27)

The ticket asked to test the nominal-encoder shape against the actual consumers before proposing it. Tested; it fails on two independent grounds.

**Fact: two consumers cannot import `tiler_ir::identity` at all, by accepted decision.** [ADR 0077](../docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) item 2 pins `tiler-metal-aot`'s dependency closure empty, and [ADR 0082](../docs/decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) item 2 decides `tiler-cache`'s closure is exactly `tiler-artifact`, saying in terms that `tiler-ir` is "an edge this record decides the crate does not have". Both therefore restate the framing, each in one admitted copy, and would continue to whatever shape the public surface took. **Inference: the stated outcome — one governed rule every identity uses — is not reachable through the type system while those closures stand.** Changing two functions into a type would improve ergonomics for the reachable consumers and leave the invariant exactly where it is.

**Fact: an encoder owning only the framed writes would misrepresent its own authority.** Every consumer interleaves framed writes with direct fixed-width ones — `push_len(&mut bytes, n)` and `push_slice(&mut bytes, s)` beside `bytes.extend_from_slice(&value.to_be_bytes())` for tags, versions, counts, and canonical positions. A nominal encoder that owned only the first kind would read as *the* identity encoder while half the bytes of any real identity bypassed it. That is worse than the present arrangement, which claims nothing it does not do. An encoder owning both kinds is a different and much larger proposal: it would have to enumerate every scalar width the workspace encodes, and it is not what this ticket scoped.

**Measurement of the surface:** 31 files across `tiler-ir` (14), `tiler-compiler` (11), `tiler-artifact` (4), and `tiler-reference` (2) call the public helpers; the two closed crates hold one admitted copy each.

**Decision: the public surface stays `push_len` and `push_slice`.** `tiler-ir` owns the rule; three restatements are admitted by ADR; and nothing checks any of it.

## What was actually wrong, and is fixed

Two doc comments asserted a gate that does not exist, in the present tense:

- `crates/tiler-cache/src/expansion/subject.rs` and `crates/tiler-metal-aot/src/identity.rs` each claimed `scripts/check_workspace.py` "admits exactly this definition … so a second copy in this crate fails the gate rather than growing quietly". That script was deleted by `e197176` with the rest of the Python tooling and has no successor.
- Both also cited `scripts/check_rust.py` as admitting only 64-bit profiles, "which is what makes the conversion total" — **a load-bearing safety argument for an `expect()`, resting on a deleted file.** What actually makes it total is the supported-platform policy: `AGENTS.md` states Tiler develops on macOS only and every admitted target is 64-bit.

Both now say what is true, and say plainly that a third copy is caught only by review. `tiler-ir/src/identity.rs` was already correct — it describes the deleted check in the past tense and explains what a replacement would need.

## Remaining, and explicitly not closed here

Nothing mechanically prevents a second framing definition. `identity.rs`'s module documentation already specifies what a replacement check must do — recognize a *shape* (a `&mut Vec<u8>` sink plus one `usize`/`&[u8]`/`&str` payload, or one statement that both reads a length and writes it as fixed-width bytes) rather than a list of names, because every name list tried had been incomplete. That is a `implementation/workspace` engineering task, not a namespace review, and this ticket does not pretend to have done it.
