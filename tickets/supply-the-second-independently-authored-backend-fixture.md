---
id: supply-the-second-independently-authored-backend-fixture
title: Supply the second independently authored backend fixture
status: in-progress
priority: p2
dependencies: []
related: [publish-the-backend-provider-conformance-suite]
scopes: [implementation/build, implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance, backend-providers, evidence]
claimed_from: todo
assignee: worker-fixture
lease_expires_at: 1787424892
---
## User-visible outcome

A second, independently authored backend fixture exists that shares the portfolio's neutral, non-self-certifying structural and execution subjects — so `publish-the-backend-provider-conformance-suite`'s release condition names a deliverable that something in the graph actually owns.

## Why this exists

Filed 2026-08-22 by the coordinator from a **graph gap** the deferred/blocked sweep found: a p1's release trigger names a deliverable with **no node in the graph**.

**Fact (reported by the sweep, not re-derived by me — verify it first).** `publish-the-backend-provider-conformance-suite` (p1) states its release trigger as "one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects". No ticket owns producing that fixture, so the p1 cannot become ready by any path currently in the graph.

**Fact (reported, unverified by me).** `.ticketsplease/decision-queue.md` item 14 (accepted 2026-08-18) specifies a bounded extraction that would supply exactly this. Read that item as the specification rather than inventing a shape.

**Why "independently authored" is the whole point, and the trap to avoid.** A conformance suite validated only by the fixture that motivated it proves the suite agrees with itself. The value is a *second* author's reading of the same neutral subjects. So a fixture derived by copying `crates/tiler-build/tests/custom_backend` and renaming it would satisfy the words and defeat the purpose. **If you conclude the only tractable route is a derivative of the existing fixture, stop and report** — that is a decision about what the conformance claim is worth, not an implementation detail.

**Prior art that is genuinely independent, and worth reading before designing.** `restore-multi-family-metal-delivery-evidence-under-per-family-profiles` established that the scalar-host backend can hold two artifact families *honestly* — it runs no target compiler, its payload is an image its own in-process translator writes, and one profile key covers both triples with every declared axis holding for both. That is the shape of a neutral, non-self-certifying subject.

## Source-first Fact audit — 2026-08-22 at base `e20ed09e0c40ec777e22a2cd43ec70cb0c5ccdea`

Both Facts were reported by the coordinator's read-only graph sweep and marked un-re-derived. Both were re-read here, in full, at this base. Every anchor below was run with `grep -c` against the file the citation names before being written down.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The p1 states its release trigger as "one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects" | **Verified, verbatim** | `tickets/publish-the-backend-provider-conformance-suite.md`, `## Trigger check log`, anchor `one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects` returns 1. The source sentence is `The trigger is one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects.` |
| No node in the graph owns producing that fixture | **Verified** | `grep -l "independently authored" tickets/*.md` returns seven files; the four that are not this ticket, the p1, or its owning decision are `own-or-close-the-adr-internal-open-questions` (`done`, about target profiles), `reconsider-registered-quantitative-capability-axis-schemas` (`deferred`, target-profile trigger), `refuse-empty-live-domains-before-routing-commit` (`done`, launch preconditions), and `revise-adr-0108-with-a-complete-data-dependent-index-vertical` (`done`, index descriptions). None owns a backend fixture. |
| The p1's one-line log entry is the whole trigger | **False — it is a compressed restatement, and the authority is stronger** | The p1 says of itself `This carrier defers on that decision's named reopening trigger rather than dispatching on the decision ticket's completion`. That decision is `decide-the-backend-provider-conformance-harness-public-surface`, whose Trigger 1 reads `A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject` (anchor returns 1) and continues `It also proves typed host unavailability, caller-owned execution policy` (anchor returns 1), after which the same sentence names tiler-reference as the sole mathematical oracle and adapter-owned terminal resource lifetime. A fixture whose existence is asserted but which proves none of those four does **not** fire the trigger. This ticket's Required work did not name them; they are added below. |
| `.ticketsplease/decision-queue.md` item 14 "specifies a bounded extraction" that should be read "as the specification rather than inventing a shape" | **False as stated** | Item 14 is four bullets and carries no module, type, container, entry, or subject spelling at all. Its only sentence about the extraction is the Recommendation's `One bounded two-fixture extraction with typed host unavailability, caller-owned policy` (anchor returns 1), which continues by naming tiler-reference as sole mathematical oracle and adapter-owned terminal lifetime as sufficient to reopen a partial facade independently. That names four properties the extraction must **prove**; it does not specify a shape, so the instruction to read it as the specification is not followable. Item 14's own `Presentation order and release trigger` bullet is about when to present the packet to Tom, not about a fixture. The only usable specification is the decision ticket's Trigger 1 plus the source seams themselves. |
| The p1's reproduce command can observe the trigger firing | **False — the check cannot say yes** | The p1's log line ends `Reproduce: enumerate independently authored fixtures under `crates/tiler-conformance/` and compare their structural/execution subjects` (anchor `enumerate independently authored fixtures under` returns 1). At this base `crates/tiler-conformance/` holds `Cargo.toml` and `src` and no `tests` directory, and the owning decision itself locates every existing out-of-crate fixture elsewhere — `crates/tiler-compiler/tests/external_physical_provider.rs`, `crates/tiler-build/tests/custom_backend/main.rs`, the runtime adapter fixtures, and `spikes/runtime/backend-provider-portfolio/`. An enumeration rooted at `crates/tiler-conformance/` therefore returned the empty set for a reason unrelated to the trigger. This work makes the command's root non-empty rather than leaving it a check that could only ever report absence. |

**Consequence for this ticket.** The premise survives: a p1's release condition names a deliverable no node owns, and this ticket owns it. What changes is the bar. The deliverable is not "a second fixture exists" but a fixture that expresses **one device-free structural subject and one execution subject** and proves **typed host unavailability, caller-owned execution policy, `tiler-reference` as the sole mathematical oracle, and adapter-owned terminal resource lifetime**, none of which the ticket named before this audit.

## Scope added — 2026-08-22

`implementation/conformance` was added with `tkt set --add-scope`, and the reason is a dependency fact rather than a preference. `crates/tiler-build`'s manifest lists `tiler-artifact`, `tiler-cache`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, and `tiler-metal-aot` and **neither `tiler-runtime` nor `tiler-reference`**, so no test target under `implementation/build` can route a plan or compare against the oracle: the execution subject and the sole-oracle proof are unreachable from the originally declared scope. `crates/tiler-conformance`'s manifest already lists all ten crates the vertical crosses, including both, so an integration test there reaches every seam with **no manifest and no `Cargo.lock` change**. It is also the crate whose header names its subject as cross-layer executed evidence, and the crate the p1's own reproduce command roots at. No live lane holds `implementation/conformance`: the two other live claims declare `implementation/{ir,reference,compiler}` plus `contracts/*` and `research/target-profiles`.

## Required work

- Re-audit both Facts above at your actual base and report a per-Fact verdict before designing. Quote the release trigger verbatim from `publish-the-backend-provider-conformance-suite` and decision-queue item 14.
- Derive what "neutral and non-self-certifying" requires of the fixture, from the portfolio's own subjects rather than from this ticket's summary of them. State the subjects it must share and the ones it must **not** inherit.
- Build the fixture, and state plainly what makes it independently authored rather than a rename.
- **Perturb the subject:** show the conformance suite failing against a fixture that certifies itself, and quote the failure. A suite that passes both a sound and an unsound fixture has not been demonstrated.
- **Added by the Fact audit above, because the trigger's real authority names them and this ticket did not.** The fixture must additionally prove all four properties the owning decision's Trigger 1 requires of the extraction: a typed host-unavailable outcome that cannot compare equal to a pass; an execution policy owned by the caller rather than read from an ambient environment variable; `tiler-reference` as the sole mathematical oracle, with no expected value restated in the fixture; and adapter-owned terminal resource lifetime, meaning the adapter retains its resources through terminal device use and the route mints no second completion authority.

## Non-goals

Publishing the conformance suite — that is `publish-the-backend-provider-conformance-suite`'s own work, and this ticket exists to make it reachable. Any Metal second-family route: the iOS families are hardware- and decision-gated and `second_artifact_family_fixture` was deleted in `1f6ec214`, so that path is closed. Changing the portfolio's neutral subjects.

## Closes when

The fixture exists, its independence is argued rather than asserted, the self-certifying perturbation is quoted failing, `publish-the-backend-provider-conformance-suite`'s release trigger is satisfied or its remaining gap is stated exactly, and the touched package's gates are green.
