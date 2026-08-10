---
id: correct-adr-0104s-stale-subject-domain-ownership-phrase
title: Correct ADR 0104's stale statement that tiler-digest owns subject domains
status: in-progress
priority: p2
dependencies: []
related: [correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise, repair-the-artifact-abis-stale-cross-crate-no-prefix-argument]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity]
claimed_from: todo
assignee: sol-adr0104-domain-ownership
lease_expires_at: 1786404386
---

## Why this exists

**Fact — one accepted-tense phrase in ADR 0104 overstates the bottom crate's ownership.** `docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`, at the source anchor `owning `DigestAlgorithm`, `Digest`, the tag table, and the domains`, assigns subject-domain ownership to `tiler-digest`. The same ADR's executed-status paragraph instead says the crate owns `DigestAlgorithm`, `Digest`, `DIGEST_BYTES`, the tag table, and the two admitted pre-image shapes, and its execution paragraph delegates the moved ownership statements to the artifact ABI contract.

**Fact — the live ownership split is algorithm and pre-image discipline, not subject domains.** The later accepted correction in `docs/artifact-abi.md`, at the source anchor `which owns the algorithm`, states that `tiler-digest` deliberately knows no subject domain because each domain belongs to the authority that decides what it names. `crates/tiler-digest/src/lib.rs` and `docs/architecture.md` state the same split. The artifact and IR domain populations remain owned and pinned in their respective crates.

## Fact audit — 2026-08-10

- **Verified:** ADR 0104 contains the first Fact's exact source anchor. Its executed-status paragraph names `DigestAlgorithm`, `Digest`, `DIGEST_BYTES`, the tag table, and the two admitted pre-image shapes; its answered-boundary paragraph at `The ownership statements this section names as moving` points the executed ownership split to the artifact ABI contract.
- **Imprecise, repaired above:** the second Fact's substance is verified, but its original rendered-form anchor `tiler-digest, which owns the algorithm` did not occur in the source because the inline-code closing backtick sits between `tiler-digest` and the comma. The replacement `which owns the algorithm` is a source-safe clause in the accepted artifact ABI contract. `This crate cannot hold that check` in `crates/tiler-digest/src/lib.rs`, `It owns no *domain*, deliberately` in `docs/architecture.md`, `GovernedDomain::ALL` in the artifact domain census, and `PINNED_IDENTITY_DOMAINS` in the IR census independently verify the stated split.

The repair changes no substantive Fact and does not change this ticket's purpose.

This residual was found by the independent review of [`correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise`](correct-the-two-source-comments-that-repeat-the-reversed-domain-dependency-premise.md). It does not invalidate that source-comment repair: the live sources and the later accepted ABI contract agree, while this older accepted-tense phrase is the stale statement.

## What closes this

Add a dated correction to ADR 0104 that preserves the historical accepted choice of a new bottom digest crate while withdrawing the claim that it owns subject domains. State the current split using the live crate docs and accepted artifact ABI contract: `tiler-digest` owns the governed algorithm, tag table, and admitted pre-image shapes; the authority naming each subject owns its domain. Preserve the identity-folding decision, its acceptance provenance, the crate boundary, dependency direction, re-exports, domain steps, measurements, and historical text. Update no source, domain byte, public API, schema, or identity.

Re-read the complete ADR, live digest crate, architecture and artifact ABI ownership passages, and both domain census modules before editing. Use source-safe anchors, run `make citations`, `tkt lint`, `git diff --check`, and exact-base `tkt guard`, and report any other live ADR statement that conflicts with the current ownership split rather than silently widening this ticket.

## Outcome — 2026-08-10

Added a dated correction immediately after ADR 0104's stale accepted-tense paragraph. The original decision-round sentence and its full ownership phrase remain byte-for-byte as history, while the correction withdraws only `and the domains` from the live allocation. It states that `tiler-digest` owns the governed algorithm, `Digest`, `DIGEST_BYTES`, the tag table, the two admitted pre-image shapes, and their domain-separation discipline, while deliberately owning no subject domain; each subject-naming authority owns its domain and local no-prefix obligation.

The correction explicitly preserves Tom's acceptance and relay provenance, the bottom-crate boundary and dependency direction, the `tiler_artifact::program` re-exports, the graph-identity fold, and both fully spelled coverage-domain `v1` → `v2` steps. A semantic residual read found no second live conflict in ADR 0104. The retired phrase occurs there only in the retained accepted sentence and the dated correction's historical quotation; its ticket occurrence states the defect rather than reasserting it. No source, public API, domain byte, identity, schema, pin, measurement, or catalog changed.

`make citations` passed with 1,189 pinned citations and 6,443 local links resolved; `tkt lint --format json` returned `ok: true` with no diagnostics; and `git diff --check` passed. The green `make full` published at `447c24923dca4294c6c5afaa2837423d9063f33c` carries: it is an ancestor of the exact base, and the exact-base delta contains only this accepted decision and ticket, touching none of the gate-invalidating paths. Fresh citation and ticket lint gates were run as the repository's carry rule requires.

The correction and audit are committed at `e6984925d36de780fe9018609532c7c176eafe3d`. Exact-base `tkt guard` with both `--base` and `--config-ref` set to `d5d5136eab64161533b61158a63d78a5a02cb5a5` reports `conflict: false`, no under-declared scope, and exactly the two declared affected scopes: direct `contracts/decisions` and shared `project/tickets`. Its overlap report is warning-only; the two other live claims are parallel-safe by pairwise `tkt why` and have no committed branch diff against this base.
