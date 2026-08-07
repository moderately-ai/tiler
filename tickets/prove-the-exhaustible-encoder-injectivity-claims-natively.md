---
id: prove-the-exhaustible-encoder-injectivity-claims-natively
title: Prove the exhaustible encoder-injectivity claims natively
status: in-progress
priority: p2
dependencies: []
related: [spike-kani-bounded-verification-on-one-inexhaustible-encoder]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [verification, identity, injectivity, evidence-upgrade]
claimed_from: todo
assignee: w-prove-the-
lease_expires_at: 1786140348
---

## User-visible outcome

Every canonical-encoding injectivity claim whose input domain is small enough to enumerate is backed by an exhaustive test rather than by prose reasoning — the claim's evidence class moves from a comment's argument to exhaustive-finite — and the encoders whose domains defeat exhaustion are enumerated by name with their domain sizes, which is the input the bounded-verification spike needs to pick its target.

## Why this exists (claims-ledger design work with Tom, 2026-08-06)

**Fact.** The identity discipline's per-tag injectivity claims are carried today by reasoning recorded at encoding sites plus mutation tests that each check one collision, not all of them. **Fact — some of these domains are tiny.** `push_synchronization_subject` (`crates/tiler-ir/src/schedule/model.rs:2265`) writes six bytes from small tag enums; its full input domain is a few hundred values, and exhaustive injectivity (all pairs, encode-equal implies value-equal) is a cheap native test under the repo's existing exhaustive-finite evidence class. **Inference.** Every such encoder can have its injectivity *proved* today with no new toolchain; leaving those claims on prose while the domain is enumerable is unspent evidence.

## The work

1. Enumerate every encoder participating in a canonical identity (schedule/kernel/program/semantic/index encodings in `tiler-ir`, the artifact codec in `tiler-artifact`; the compiler's explain-subject encoding is out of this ticket's scopes — enumerate it in the report, land nothing there). For each: the exact input domain and whether it is exhaustible at test-time cost (rule of thumb: full pair-set comparable in well under a second; state the count).
2. For each exhaustible encoder: land one exhaustive injectivity test beside it, in the existing test idiom, deterministic, with the population counted in the test so a shrunk domain fails rather than silently passing. Watch each fail under a deliberately introduced collision before trusting it.
3. For each inexhaustible encoder: record name, domain character (which fields blow the domain: u32/u64 ordinals, data-dependent loops), and the per-tag reasoning that currently carries it — this list is the spike's target menu and gets recorded in the ticket Outcome.
4. Do not weaken or replace the existing mutation tests; the exhaustive tests sit beside them.

## Closes when

The enumeration is complete with each encoder classified and counted, every exhaustible encoder has a passing exhaustive-injectivity test that was watched failing on a planted collision, and the inexhaustible list with domain characterizations is in the Outcome.
