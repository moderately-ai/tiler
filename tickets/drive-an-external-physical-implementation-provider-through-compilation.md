---
id: drive-an-external-physical-implementation-provider-through-compilation
title: Drive an external physical implementation provider through compilation
status: in-progress
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary]
related: [prototype-complete-physical-plan-selection, wire-capability-and-refinement-into-compile-path]
scopes: [implementation/compiler, implementation/ir, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, compiler]
claimed_from: todo
assignee: coord
lease_expires_at: 1786178177
---
## User-visible outcome

An out-of-crate caller can install a physical implementation provider into the ordinary compiler session, have its candidates reverified and considered additively, and observe exact selected-provider provenance in the resulting plan and explain output.

## Implementation keys

- Promote only the exact physical-provider facade accepted by the composition ADR.
- Let providers propose bodies, applicability, and estimates through bounded writers; derive provider identity from registration and derive resource/boundary facts from verified output.
- Retain several valid providers' implementations side by side for cost-based selection.
- Preserve the asymmetry with lowering: two lowering authorities for one occurrence are ambiguous, while two physical implementations of one verified region are alternatives.
- Reject malformed provider output as a provider/compiler defect rather than silently treating it as an empty offer.
- Keep empty offer, hard rejection, unknown analysis, provider defect, and cost disadvantage distinct in explain output.
- Add an out-of-crate compile fixture and perturb installation, identity, region coverage, target applicability, and verifier bypass attempts.
- Review the exact public trait/module/session boundary with Tom before acceptance.

## Closes when

An external provider reaches `enumerate_frontier` through `session::compile`, the selected plan records its non-forgeable identity, every negative control fails for the intended reason, targeted nextest and Clippy pass, and one final `make full` passes for the batch.

## What blocks this today

Measured by `prototype-a-forkless-custom-metal-physical-provider` at commit `7b1e3a7e15b09dd3ea65c88759699655c462be4a`, with retained compile-fail evidence at [`spikes/extensions/forkless-physical-provider/`](../spikes/extensions/forkless-physical-provider/README.md). Two independent changes are needed, and the second is the one a reader is likely to miss.

Visibility: `crates/tiler-compiler/src/lib.rs:24` declares `mod frontier;` private, and the closure a provider needs reaches four more private modules — `request::VerifiedTargetRequest`, `region::SemanticMemberId`, `physical::{pointwise_region, verify_schedule_with_feasibility}`, and `pipeline::compile` (`lib.rs:34`, `:35`, `:38`, `:39`). The governed cost-model key `tiler.cost.structural.v1` is a private constant (`frontier.rs:100`) and the only admissible one, so it needs a public spelling too.

Installation: publishing the trait alone would still leave a provider uninstallable, and the internal `CompilationRequest` (`request.rs:1024`) carries no provider field. The out-of-crate compile fixture this ticket asks for should keep the spike's pairing: a compile-fail case for the absent physical installation method beside a compiling one for `CompileRequest::with_capabilities`, so the fixture states the asymmetry rather than a bare absence.

**Anchors refreshed 2026-08-05 at base `51e9374a` by [`audit-backend-authoring-against-all-thirteen-responsibilities`](audit-backend-authoring-against-all-thirteen-responsibilities.md); the blockers themselves are unchanged.** This section previously cited `lib.rs:19`, `frontier.rs:80`, `request.rs:542`, and "a hardcoded one-element literal at `crates/tiler-compiler/src/pipeline/planning.rs:171`". That literal no longer exists: the provider list and the opaque-call registry are now composed into `PhysicalAuthorities` (`frontier.rs:2893`), installed as `PhysicalAuthorities::governed()` at `pipeline.rs:604` and consumed at `pipeline/planning.rs:292`. That is an internal composition improvement and moves none of the three obligations this ticket exists to discharge — visibility, installation from outside the crate, and observability all still fail, the last at `session.rs:2092-2093` where `offered_providers` is populated from the lowering registry alone. Read the change as narrowing what has to be built, not as partial completion.

The spike also establishes what does *not* need work: `tiler-metal` is reusable unchanged by an out-of-tree provider, the proposal body type is already public, and the schedule axis a specialization varies is free under the intrinsic verifier and folded into canonical identity.

## Graph maintenance

- Unblock payload production and final provider composition only through the accepted public seam.
- Keep semantic-equivalence trust limitations explicit; structural verification cannot prove arbitrary replacement mathematics.
- Update ADR 0078's implementation status and governed seam inventory only after the path is genuinely external and exercised.
