---
id: reconsider-registered-quantitative-capability-axis-schemas
title: Reconsider registered quantitative capability-axis schemas
status: deferred
priority: p3
dependencies: []
related: [own-or-close-the-adr-internal-open-questions, prototype-a-bounded-scalar-cpu-backend-vertical, declare-cpu-vector-realization-facts-in-the-target-profile, construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [research/target-profiles, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, target-profiles, extensions, deferred]
---
## User-visible outcome

If real backend evidence outgrows the compiler-owned quantitative-axis
vocabulary, Tiler re-evaluates a governed extension schema from concrete rows
rather than either blocking ecosystem work indefinitely or stabilizing an
extension protocol speculatively.

## Accepted starting point

Tom decided on 2026-08-03 in the T3 Code orchestration conversation that
quantitative capability axes remain compiler-owned and exhaustive for the
initial profile. [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
records the decision and its grounds. A required quantitative fact is therefore
added as a reviewed typed compiler variant with a host-owned comparison.

The alternative preserved here is not an arbitrary provider-defined key or an
opaque comparison callback; those fail correctness and deterministic identity
and remain eliminated. The only surviving alternative was a frozen per-request
registry binding a governed axis key to a host-validated quantity, relation,
validation rule, and canonical identity encoding, with the host retaining every
comparison.

## Activation triggers

Deferred rather than dispatchable. Reopen this question only when either
evidence threshold is met:

1. An independently authored target profile is blocked by a genuinely
   quantitative target fact that the compiler does not name; or
2. A second backend demonstrates a quantitative row whose validation,
   comparison, and identity schema is materially shared with a CPU row.

A request for vector width, mask/tail behaviour, or scalable-vector realization
does not fire either trigger: ADR 0093 derives those as one exact atomic
realization subject rather than an ordered quantity.

## Work after activation

- Preserve the concrete rows and their producer/consumer evidence before
  proposing an abstraction.
- Compare adding typed compiler variants against a registered host-validated
  schema on correctness, deterministic identity, total-map maintenance,
  out-of-tree authorship, and review cost.
- Specify registry freezing, duplicate/conflict refusal, canonical encoding,
  unknown-schema rejection, and which crate owns each public type before
  drafting implementation.
- Return every consequential public registration, trait, type, or call-site
  boundary to Tom under ADR 0075; this ticket records no advance acceptance.

## Dependency policy

This deferred reconsideration is not a prerequisite for existing target-profile
or backend work. Current work adds measured quantitative rows through the
accepted compiler-owned vocabulary. If an activation trigger fires, the blocked
consumer may depend on this ticket then; no speculative dependency is added now.

## Closes when

After a trigger fires, the concrete multi-backend evidence eliminates all but
one ownership model, the result is recorded in a durable contract or accepted
ADR, every identity and validation consequence is enumerated, and any public
boundary has Tom's explicit acceptance.
