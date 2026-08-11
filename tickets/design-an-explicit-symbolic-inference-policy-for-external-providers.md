---
id: design-an-explicit-symbolic-inference-policy-for-external-providers
title: Design an explicit symbolic-inference policy for external providers
status: deferred
priority: p2
dependencies: [narrow-symbolic-inference-and-restore-host-owned-refusals]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [research/extensions, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, extensions, semantic-graph, decision, needs-tom, public-boundary]
---

## User-visible outcome

If external operation providers later gain symbolic inference, every operation explicitly declares which inference contract it uses and host preflight enforces that choice. No operation receives a silent default, and an incompatible provider/policy/environment combination cannot proceed to inference under another approach.

## Deferred question

Design the full public symbolic-provider seam only when a real consumer needs it. The minimum candidate is a closed, required policy such as `LiteralOnly | EnvironmentAware` supplied by every operation definition, with no `Default` implementation and no omitted field. The host validates the policy before callback, owns all environment comparison/refusal facts, and includes every behavior-affecting declaration in frozen registry identity.

## Required decisions when fired

- The exact required policy vocabulary and where every operation definition states it.
- A custom-registry-plus-shape-environment constructor whose one environment is fixed before any symbolic value exists.
- The opaque host request/proof API by which a provider asks equality/admission questions without receiving authority to fabricate their answers.
- Preflight behavior for absent capability, missing environment, foreign/late symbols, literal-only operations, and policy/schema mismatch. Every disposition is typed and none falls back to another policy.
- Registry identity/domain/version consequences and compatibility of older registry snapshots.
- An out-of-crate integration provider that drives one environment-aware operation through construction, verification, identity, and explanation.

## Non-goals

Do not reopen the narrow built-in elementwise result rule, introduce dynamic loading or runtime source compilation, or infer policy from operand contents. A provider must not be called and then asked after the fact whether it understood the symbol.

## Trigger check log

- 2026-08-11 — **not fired.** `SemanticProgramBuilder` has `try_standard_with_shape_environment` and `try_new(custom_registry)`, but no custom-registry-plus-environment constructor; the workspace has no second out-of-crate symbolic operation family. Reproduce with `rg -n 'try_.*shape_environment|try_new' crates/tiler-ir/src/semantic/program.rs` and the complete `impl SemanticProgramBuilder` read. Reconsider on the first proposed custom-registry symbolic constructor or second external symbolic family.

## Closes when

Tom accepts the exact required policy/preflight/identity surface and the first external provider proves it end to end, or rejects external symbolic inference and records the public seam as permanently static-only.
