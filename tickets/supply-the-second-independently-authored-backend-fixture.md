---
id: supply-the-second-independently-authored-backend-fixture
title: Supply the second independently authored backend fixture
status: todo
priority: p2
dependencies: []
related: [publish-the-backend-provider-conformance-suite]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [conformance, backend-providers, evidence]
---
## User-visible outcome

A second, independently authored backend fixture exists that shares the portfolio's neutral, non-self-certifying structural and execution subjects — so `publish-the-backend-provider-conformance-suite`'s release condition names a deliverable that something in the graph actually owns.

## Why this exists

Filed 2026-08-22 by the coordinator from a **graph gap** the deferred/blocked sweep found: a p1's release trigger names a deliverable with **no node in the graph**.

**Fact (reported by the sweep, not re-derived by me — verify it first).** `publish-the-backend-provider-conformance-suite` (p1) states its release trigger as "one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects". No ticket owns producing that fixture, so the p1 cannot become ready by any path currently in the graph.

**Fact (reported, unverified by me).** `.ticketsplease/decision-queue.md` item 14 (accepted 2026-08-18) specifies a bounded extraction that would supply exactly this. Read that item as the specification rather than inventing a shape.

**Why "independently authored" is the whole point, and the trap to avoid.** A conformance suite validated only by the fixture that motivated it proves the suite agrees with itself. The value is a *second* author's reading of the same neutral subjects. So a fixture derived by copying `crates/tiler-build/tests/custom_backend` and renaming it would satisfy the words and defeat the purpose. **If you conclude the only tractable route is a derivative of the existing fixture, stop and report** — that is a decision about what the conformance claim is worth, not an implementation detail.

**Prior art that is genuinely independent, and worth reading before designing.** `restore-multi-family-metal-delivery-evidence-under-per-family-profiles` established that the scalar-host backend can hold two artifact families *honestly* — it runs no target compiler, its payload is an image its own in-process translator writes, and one profile key covers both triples with every declared axis holding for both. That is the shape of a neutral, non-self-certifying subject.

## Required work

- Re-audit both Facts above at your actual base and report a per-Fact verdict before designing. Quote the release trigger verbatim from `publish-the-backend-provider-conformance-suite` and decision-queue item 14.
- Derive what "neutral and non-self-certifying" requires of the fixture, from the portfolio's own subjects rather than from this ticket's summary of them. State the subjects it must share and the ones it must **not** inherit.
- Build the fixture, and state plainly what makes it independently authored rather than a rename.
- **Perturb the subject:** show the conformance suite failing against a fixture that certifies itself, and quote the failure. A suite that passes both a sound and an unsound fixture has not been demonstrated.

## Non-goals

Publishing the conformance suite — that is `publish-the-backend-provider-conformance-suite`'s own work, and this ticket exists to make it reachable. Any Metal second-family route: the iOS families are hardware- and decision-gated and `second_artifact_family_fixture` was deleted in `1f6ec214`, so that path is closed. Changing the portfolio's neutral subjects.

## Closes when

The fixture exists, its independence is argued rather than asserted, the self-certifying perturbation is quoted failing, `publish-the-backend-provider-conformance-suite`'s release trigger is satisfied or its remaining gap is stated exactly, and the touched package's gates are green.
