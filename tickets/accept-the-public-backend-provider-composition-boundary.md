---
id: accept-the-public-backend-provider-composition-boundary
title: Accept or revise the public backend-provider composition boundary
status: awaiting-decision
priority: p1
dependencies: [draft-the-backend-provider-composition-adr]
related: [draft-public-extension-seam-ownership-adr]
scopes: [contracts/decisions, contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, decision, needs-tom]
---
## User-visible outcome

Tom receives one evidence-backed atomic decision packet for the backend-provider composition boundary, and no production or public implementation conditional on that model becomes dispatchable before acceptance.

## Decision boundary

Present the exact proposed ADR after eliminating every option that cannot preserve target-independent semantics, re-verification, deterministic identity, partial provider composition, AOT build/runtime separation, routing safety, and long-term multi-backend evolution. State what each surviving option enables and prevents, its counterpoint, and the recommendation.

This node is not research or implementation work. It remains parked until the proposed record exists and Tom accepts it or requests revisions.

## Closes when

Tom accepts or revises the ADR; its status, acceptance date, body, catalogs, governed contracts, and implementation boundary agree; proposal-only disclosures are removed or corrected; and all dependent implementation tickets are released by this node becoming `done`.

## Graph maintenance

- Only Tom approves or revises the decision. After his answer, the implementing agent records it durably, applies every acceptance consequence, runs the checks, and closes this node.
- If the ADR is revised, amend the still-proposed record rather than creating a superseding accepted fiction.
- Keep multi-device, dynamic-library plugins, untrusted providers, and stable plugin ABI outside the accepted initial boundary unless the evidence unexpectedly forces one of them into scope.

## The packet, complete as of 2026-07-31

The record is [ADR 0090: Compose backends per responsibility rather than per backend](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), `decision_status: proposed`, drafted by [`draft-the-backend-provider-composition-adr`](draft-the-backend-provider-composition-adr.md) from the completed [consumer-neutral backend-provider composition record](../docs/research/extensions/backend-provider-composition.md) and the two spikes behind it. This ticket moved from `todo` to `awaiting-decision` on that record's existence, and it is the only node that releases the implementation work conditional on it.

**The eleven atomic decisions collapsed to one surviving candidate each, so this is not a choice among options.** That elimination is written out in the ADR's "Why this is one record rather than several" section and in the drafting ticket's outcome, and it is stated so it can be refuted rather than only disbelieved. Three things nonetheless need Tom and cannot be derived:

1. **Whether the sharpened trigger in ADR 0078 item 5 restates his intent or narrows it** — one sentence, about his own prior decision rather than about the tree, so no research can supply it. The literal reading of the original clause may already be satisfied; the sharpened reading is not, because no out-of-crate physical provider has reached `enumerate_frontier` through `compile()`.
2. **Whether refining his recorded deferral is what he meant** — ADR 0090 proposes that target-specific scheduling knowledge is a checked combination split at feasibility, with profiles declaring what a target can do, providers proposing what to do with it, and the host performing every comparison. What would refute it is a specialization fully determined by declared target facts.
3. **The exact public boundaries**, each his under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) regardless of research quality: the installed-physical-provider registry and its request-installation method; the separate offered-versus-selected disclosure accessors; and the promoted build-time `assemble_artifact` function with its closure parameter, which is also the one item that changes the accepted packaging profile rather than describing it. No shape in the record has ever compiled or carried an out-of-crate fixture, so each is a sketch to argue with rather than a proposed interface.

**What is not in scope for this acceptance and must stay out.** Multi-device, sharding, collectives, cross-device transfers, queue affinity, and multiple command streams stay with [`multi-device-and-sharding-scope-gate`](multi-device-and-sharding-scope-gate.md). Dynamic loading, a stable plugin ABI, adapter discovery, hot reload, untrusted or sandboxed providers, cross-process callbacks, and runtime source compilation stay jointly deferred with no reserved seam. The two target-profile key grammars stay with [`reconcile-the-two-target-profile-key-grammars`](reconcile-the-two-target-profile-key-grammars.md), the missing host-process availability phase with [`name-a-host-process-availability-phase`](name-a-host-process-availability-phase.md), and the compile-path opaque-call registry with [`register-opaque-calls-on-the-compile-path`](register-opaque-calls-on-the-compile-path.md), which is `related` rather than dependent for the reason recorded there.

**On acceptance, the disclosures become wrong and correcting them is part of closing this node.** Seven documents carry a sentence that exists only because the record is proposed: `docs/architecture.md` (four sites), `docs/operation-extensions.md` (two), `docs/artifact-abi.md`, `docs/backends/cpu.md`, `docs/glossary.md`'s Provider row, `docs/decisions/0078-name-the-intended-public-extension-seams.md`'s two Tom-owned open questions, and `docs/decisions/README.md`'s two catalog rows and its "exactly one record is proposed" paragraph. Reproduce the set with `grep -rln "0090-compose-backends-per-responsibility-rather-than-per-backend" docs/`. Nothing checks any of them, which is why the closing condition above names them.
