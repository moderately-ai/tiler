---
id: make-runtime-routing-commit-authority-one-shot
title: Make runtime fallback authority consumable exactly once
status: todo
priority: p1
dependencies: []
related: [prototype-metal-runtime-preflight, prototype-metal-runtime-execution, preflight-every-entry-of-a-multi-stage-route]
scopes: [implementation/runtime]
shared_scopes: []
paths: []
tags: [runtime, correctness, routing, fallback]
---
After Tiler commits to an artifact route, a caller must not be able to recover
or mint another authority that permits semantic fallback for the same attempt.

## Fact

`DecodedProgram::preflight` is callable through `&self`, and the decoded program
is clonable. A non-clone `Preflight` value is therefore not unique: callers can
mint more than one before consuming any of them.

## Outcome

Successful preflight yields one route-level authority whose consumption is the
only way to cross the routing commit. The authority covers every stage that may
execute and cannot be recreated from a retained or cloned decoded program.
Precommit refusals leave fallback legal; after consumption, allocation,
encoding, submission, validation, or execution failure is terminal.

## Closes when

The type and call-site structure make two commits or post-commit fallback
unrepresentable for one route, and negative tests prove repeated preflight or
commit cannot recover the authority.
