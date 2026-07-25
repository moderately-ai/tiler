---
id: decide-the-expansion-cache-owner-and-digest-authority
title: Decide the expansion cache owner and its digest authority
status: todo
priority: p1
dependencies: []
related: []
scopes: [contracts/decisions, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [cache, architecture, decision]
---
The expansion cache has no home that satisfies the records governing it, and every candidate is blocked by an authority a worker cannot resolve. `prototype-expansion-content-cache` hit this and landed only the half that is independent of the answer.

## The conflict, with the exact text

**Fact — the accepted contract assigns the cache to `tiler-metal-aot` and, in the same row, forbids it every dependency.** `docs/architecture.md`'s Component ownership table gives that crate "Expansion-time Apple tool invocation, cross-process content cache, atomic publication, byte embedding, …" and states its forbidden dependencies as "Every workspace and third-party dependency, Candle included: its empty closure is decided, not incidental". `scripts/check_workspace.py` pins `"tiler-metal-aot": []` mechanically.

**Inference — that row is internally unsatisfiable.** ADR 0050 requires readers to validate "bounded framing, embedded key, schemas, manifest, section lengths/digests, and required meanings on every hit". Section digests need a hash function. The governed one is `tiler.digest.sha-256.v1`, implemented in `crates/tiler-artifact/src/program/codec/digest.rs`, where `DigestAlgorithm`, `Digest`, and `DIGEST_BYTES` are all `pub(crate)`. So the assigned owner cannot reach the governed algorithm even if the closure were opened, and a local digest in the driver would make it a second identity authority over the same subject — the thing `crates/tiler-metal-aot/src/family.rs` and the digest module's own documentation both refuse.

**Fact — the proposed ADR says the opposite of the accepted table.** [ADR 0077](../docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) item 1 states the driver "does not emit MSL, does not assemble the target-neutral artifact bundle, and does not implement the expansion cache or the proc-macro layer." It is `decision_status: proposed`, so under `AGENTS.md` it is a hypothesis, not a commitment — but it is the record `docs/architecture.md` itself points at as the pending decision for that crate, and the two say different things about the same responsibility.

**Fact — a dedicated crate is doubly blocked.** [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) is accepted and puts "a new publicly reachable namespace — a new crate" in the always-ask-Tom category. Separately, `docs/architecture.md`'s accepted packaging profile says the profile "deliberately omits frontend, proc-macro, Candle, generalized cache, and reusable Metal-runtime crates until the proof reaches those boundaries", and ADR 0077 item 5 restates that omission.

## What is being asked

One atomic decision: **which component owns the expansion cache, and how does it reach the governed digest?** The three options, with what each enables and prevents:

**A. A new `tiler-cache` crate** depending on `tiler-artifact` for the governed digest, with `tiler-artifact` promoting a minimal digest surface. *Enables* the whole of ADR 0050 in one place with the governed algorithm and no second authority; keeps the driver's audited closure intact. *Prevents* nothing architecturally, but spends a crate admission and a promotion of `tiler-artifact` internals, and contradicts the packaging profile's "deliberately omits … generalized cache" clause until that clause is superseded.

**B. The cache lives in `tiler-artifact`**, beside the envelope and the digest it validates. *Enables* validation-on-every-hit with no new crate, no promotion, and no dependency edge — the digest and the envelope are already there. *Prevents* the ownership table's current allocation from standing, and puts filesystem locking and cross-process publication into a crate whose stated forbidden dependencies include "Metal device APIs" but whose responsibility is "encoding, compatibility, runtime fact binding" — a storage protocol is a different kind of thing.

**C. The cache stays in `tiler-metal-aot`** as the table says, and the empty closure is spent on a `tiler-artifact` edge. *Enables* the ownership table to stand unchanged. *Prevents* the property ADR 0077 item 2 calls decided rather than incidental — "a reader auditing what Tiler asks the Metal compiler to do reads one crate with nothing underneath it" — and that property is destroyed by the first dependency, not degraded by it. The cache protocol is also substantially more code than the driver it would sit beside.

**Recommendation: A.** The cache is not artifact encoding and it is not compiler invocation; it is a third responsibility with its own crash, race, and durability contract, which ADR 0050 and `docs/research/cache/crash-and-race-protocol.md` already treat as a subject in its own right. B is the cheapest and is a real alternative — it needs no crate and no promotion — but it merges a storage protocol into the encoding layer, and the next consumer that wants the cache without the artifact model has no way to take one and not the other. C is the only option that costs a decided property outright.

**The evidence behind the recommendation is the digest reachability, not a preference about layering.** Whatever is decided, it must state how the owner reaches `tiler.digest.sha-256.v1` without becoming a second authority for it, because that is the constraint every option is actually fighting.

## Also to decide, and dependent on the above

Whether `docs/architecture.md`'s `tiler-metal-aot` row is corrected to drop "cross-process content cache, atomic publication, byte embedding", and whether ADR 0077's acceptance or a superseding record carries that. The row and ADR 0077 item 1 cannot both stand.

## Not part of this question

The cache *key subject* is settled and landed: `crates/tiler-metal-aot/src/identity.rs` emits the driver's complete canonical compilation subject as bytes and leaves digesting to whichever component owns the governed algorithm, following `family.rs`'s precedent. That placement is correct under every option above, because the subject is a fact about the driver's own inputs.

## Decision — Tom, 2026-07-25

**Decided: a dedicated cache crate, depending on `tiler-artifact` for the governed digest.**

**The alternative was not merely worse, it was unsatisfiable.** `docs/architecture.md`'s ownership row assigns `tiler-metal-aot` the "cross-process content cache, atomic publication, byte embedding" while the *same row* forbids it "every workspace and third-party dependency", and `check_workspace.py` pins its closure empty. ADR 0050 requires section-digest validation on every hit, and the governed digest (`tiler.digest.sha-256.v1`) is `pub(crate)` in `tiler-artifact`. No implementation can satisfy that row as written.

`tiler-metal-aot` therefore stays dependency-free, which is the property ADR 0077 admitted it for.

**Amend the ownership table in the same change.** The accepted profile deliberately omits "generalized cache" crates, so this decision amends it rather than fitting inside it, and that amendment is part of the work rather than a follow-up.

**Carry the five correctness properties `AGENTS.md` names as the specification**: complete cache identity, validation on EVERY hit, immutable entries, atomic publication, and defined crash/race behaviour. The identity half already landed as canonical bytes in `tiler-metal-aot`; a corrupt or unreadable entry must fail loud rather than silently becoming a miss, unless you can argue otherwise and say so.
