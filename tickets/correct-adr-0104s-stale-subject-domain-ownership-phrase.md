---
id: correct-adr-0104s-stale-subject-domain-ownership-phrase
title: Correct ADR 0104's stale statement that tiler-digest owns subject domains
status: todo
priority: p2
dependencies: []
related: [correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise, repair-the-artifact-abis-stale-cross-crate-no-prefix-argument]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity]
---

## Why this exists

**Fact — one accepted-tense phrase in ADR 0104 overstates the bottom crate's ownership.** `docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`, at the source anchor `owning `DigestAlgorithm`, `Digest`, the tag table, and the domains`, assigns subject-domain ownership to `tiler-digest`. The same ADR's executed-status paragraph instead says the crate owns `DigestAlgorithm`, `Digest`, `DIGEST_BYTES`, the tag table, and the two admitted pre-image shapes, and its execution paragraph delegates the moved ownership statements to the artifact ABI contract.

**Fact — the live ownership split is algorithm and pre-image discipline, not subject domains.** The later accepted correction in `docs/artifact-abi.md`, at the anchor `tiler-digest, which owns the algorithm`, states that `tiler-digest` deliberately knows no subject domain because each domain belongs to the authority that decides what it names. `crates/tiler-digest/src/lib.rs` and `docs/architecture.md` state the same split. The artifact and IR domain populations remain owned and pinned in their respective crates.

This residual was found by the independent review of [`correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise`](correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise.md). It does not invalidate that source-comment repair: the live sources and the later accepted ABI contract agree, while this older accepted-tense phrase is the stale statement.

## What closes this

Add a dated correction to ADR 0104 that preserves the historical accepted choice of a new bottom digest crate while withdrawing the claim that it owns subject domains. State the current split using the live crate docs and accepted artifact ABI contract: `tiler-digest` owns the governed algorithm, tag table, and admitted pre-image shapes; the authority naming each subject owns its domain. Preserve the identity-folding decision, its acceptance provenance, the crate boundary, dependency direction, re-exports, domain steps, measurements, and historical text. Update no source, domain byte, public API, schema, or identity.

Re-read the complete ADR, live digest crate, architecture and artifact ABI ownership passages, and both domain census modules before editing. Use source-safe anchors, run `make citations`, `tkt lint`, `git diff --check`, and exact-base `tkt guard`, and report any other live ADR statement that conflicts with the current ownership split rather than silently widening this ticket.
