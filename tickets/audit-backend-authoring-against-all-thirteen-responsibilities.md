---
id: audit-backend-authoring-against-all-thirteen-responsibilities
title: Audit backend authoring against all thirteen responsibilities
status: review
priority: p1
dependencies: []
related: [specify-the-consumer-neutral-backend-provider-composition-contract, publish-the-backend-provider-conformance-suite, expose-explicit-backend-provider-and-selection-policy-composition, accept-the-neutral-build-orchestration-boundary, disclose-the-physical-provider-environment-a-compilation-was-offered, make-explain-dispositions-assertable-by-a-conformance-suite, refresh-adr-0090-source-anchors-after-the-seams-moved, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [research/extensions, research/program-planning, research/artifacts, research/runtime, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, backend-providers, audit, conformance]
claimed_from: todo
assignee: agent-backend-audit
lease_expires_at: 1785951341
---
## User-visible outcome

An independent backend author can determine, at compile time and without Metal or
Candle types entering the core, every contract that must be supplied to compile,
package, select, bind, and execute a backend program.

Audit the accepted compositional design across all thirteen responsibilities:
semantic authority; index lowering; scalar lowering; physical implementations;
opaque calls; target profile; emitter; build orchestration; backend family and
representation identity; payload provenance; entry mapping; runtime adapter; and
live context. For each, record the public typed seam, construction/registration path,
validation and identity obligations, explain behavior, conformance evidence, and
current maturity. A monolithic `Backend` trait is not the goal; completeness of the
composed authoring contract is.

Trace the existing provider chain, including the still-private physical provider,
explicit provider/policy composition, multi-family portfolio, build/runtime join,
and conformance suite. File missing design or implementation nodes with correct
dependencies. Define what a backend-author conformance kit must prove, including
deliberate invalid providers and identity/routing failures. Any new public trait or
module boundary remains Tom's to accept.

## Closes when

All thirteen rows have one authoritative owner and maturity state; an external
backend's end-to-end path has no undocumented repository-private hook; the
conformance-suite dependency graph covers negative as well as successful paths; and
remaining unsupported responsibilities reject explicitly.

## Outcome

**The audit landed as a dated section in [the backend-provider composition record](../docs/research/extensions/backend-provider-composition.md), at base `51e9374a`.** It carries the thirteen-row maturity table with a per-row citation, the three rows that moved since that record's own base `e6a47d9`, the row-4 obligations that did not move, and the conformance-kit definition. It lives in that record rather than a new one because it re-verifies that record's own matrix: no new frontmatter, no new catalog row, and the audit's scopes do not reach `contracts/navigation`. Every maturity claim was read from the file cited beside it.

**Where the thirteen rows stand.** Nine rows hold a tested guarantee under the operation-extension contract's own invariant — a component outside the defining crate's governed set drove the surface through the ordinary path. Row 3 (scalar lowering) holds implemented support with no compile-path resolution, unchanged and correctly recorded in the contract already. Row 4 (physical implementation) is an internal architectural seam whose three missing obligations — visibility, installation, observability — are all still missing. Row 5 (opaque calls) is implemented and tested in crate, and its ADR sentence is stale. Row 12's tested guarantee is a guarantee about the seam: no `RuntimeAdapter` implementor lives in any `crates/*/src` production path, which is correct for a device-free loader and is recorded so it is not over-read.

**The finding that most needed filing.** The build-orchestration seam an external backend author must call is implemented, tested by a non-Metal producer sharing no code with the Metal path, and **not accepted by Tom** — it carries a coordinator's provisional acceptance from 2026-08-01 under names it has since outgrown, and Tom's 2026-08-05 live review accepted five other boundaries and not this one. ADR 0090 item 11 routes it to him by name. Filed as [`accept-the-neutral-build-orchestration-boundary`](accept-the-neutral-build-orchestration-boundary.md) at `awaiting-decision`.

**Corrections landed in scope.** [The architecture contract](../docs/architecture.md)'s packaging section said the row-8 promotion was "not yet implemented" and that the dependency block "remains the live profile until that promotion lands"; the promotion had landed, and the correction separates what it removed (the authoring choice) from what it did not (the packaging cost — `tiler-build` still names `tiler-metal` and `tiler-metal-aot` unconditionally, and no workspace member declares a `[features]` section). The research record's `implementation_status` moved `not-started` → `partial`, which moves no catalog view because the research catalog line renders `disposition`, evidence classes, `informs`, and experiments and not that field. This ticket's sibling [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) had four stale anchors in its blocking analysis and one refuted sentence; refreshed in place, with the blockers themselves unchanged.

**Gaps filed rather than absorbed: four.** The acceptance node above; [`disclose-the-physical-provider-environment-a-compilation-was-offered`](disclose-the-physical-provider-environment-a-compilation-was-offered.md) for a doc comment the source refutes and the artifact-provenance consequence it now has; [`make-explain-dispositions-assertable-by-a-conformance-suite`](make-explain-dispositions-assertable-by-a-conformance-suite.md) at `deferred` with a trigger log, because `ExplainReport::render` is the only accessor and its own documentation forbids parsing it; and [`refresh-adr-0090-source-anchors-after-the-seams-moved`](refresh-adr-0090-source-anchors-after-the-seams-moved.md) as the carrier for `contracts/decisions`, which this ticket's scopes do not reach.

**No live correctness defect was found in shipped code.** The two candidates were examined and neither is one: the `offered_providers` incompleteness cannot be observed while exactly one physical provider exists and nothing can vary it, and the one-payload-per-delivery-position bound is stated in `crates/tiler-build/src/lib.rs:43-47` as a bounded scope rather than an unhandled case.

**Measurement boundary.** Inspected source at one commit on one host. Nothing was compiled or executed for this audit: every maturity claim is a reading of a declaration, a call site, or a test's existence, and "a test exists" is weaker than "a test passes at this base". No claim here rests on a grep alone — the row-4 and row-5 reproduction returning empty was resolved by reading `frontier.rs` and `pipeline.rs`, which is what found the `PhysicalAuthorities` change the empty result concealed.
