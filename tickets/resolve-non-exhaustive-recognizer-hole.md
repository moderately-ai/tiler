---
id: resolve-non-exhaustive-recognizer-hole
title: Resolve the non-exhaustive recognizer hole before applying the convention
status: in-progress
priority: p1
dependencies: []
related: [harden-public-enums-non-exhaustive, extend-canonical-identity-encodings-for-reserved-variants, draft-public-api-conventions-adr, draft-public-boundary-approval-policy-adr]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, public-api, correctness]
claimed_from: todo
assignee: agent-resolve-non-exhaustive-recognizer-hole
lease_expires_at: 1784912788
---
Accepted ADR 0074's convention 5 says to mark public enums documented as growing
`#[non_exhaustive]` so a later variant "lands additively rather than breaking a
downstream `match`". It also says that forcing consumers to grow an explicit
reject-unknown arm is "the intended fail-closed posture, not a regression". Those
two statements are in tension for **recognizer** enums, and the tension is a real
hole rather than a wording nit.

## The hole

For an enum a *recognizer* matches on — `tiler_ir::schedule::ScalarProgram` is the
concrete case, matched by `tiler-compiler` to decide which programs it supports —
`#[non_exhaustive]` forces every cross-crate consumer to carry a wildcard arm.
Adding a variant then **compiles cleanly at every consumer** while silently
routing the new variant into `_ => reject`. Nothing breaks, nothing warns, and a
program the profile now supports is quietly refused.

The compile error was the feature. It is what forced each consumer to consciously
decide how to handle a newly supported case. `#[non_exhaustive]` converts a loud,
mechanical prompt into a silent behavioural change from accept to reject. It is
still fail-*closed* — it refuses rather than mishandles — but it is silently
**incomplete**, which is a different and harder failure to notice: the capability
exists in the IR and never reaches the consumer.

## Why it must be resolved before `harden-public-enums-non-exhaustive` runs

That ticket applies `#[non_exhaustive]` to exactly these enums, including
`ScalarProgram`. Running it against an unresolved rule would *install* the hole.
Add the dependency rather than trusting sequencing by luck.

It also interacts with `extend-canonical-identity-encodings-for-reserved-variants`,
which requires encoders to match **exhaustively** so a new variant is a compile
error at the encoding site. Note the interaction is not a contradiction: those
encoders live in the same crate as the enums, and `#[non_exhaustive]` only
constrains *other* crates. Verify that holds for every encoder before relying on
it — a same-crate assumption that later moves crates would break silently.

## What to decide

Amend ADR 0074's convention 5 to distinguish the cases rather than stating one
rule for all growing enums. Candidate distinction to evaluate, not to adopt
unexamined: `#[non_exhaustive]` suits enums consumers *produce or read* (errors,
provenance, descriptors), and is actively harmful for enums a consumer
*exhaustively recognizes* to decide support, where the compile break is the
mechanism that keeps recognizers complete. If that distinction survives scrutiny,
state how a recognizer enum announces growth instead — a versioned capability
list, a conformance test that fails when a variant lacks a recognizer arm, or an
explicit decision that such enums stay exhaustive and their growth is a
deliberate breaking change we accept while pre-alpha.

Whatever is chosen, amend the accepted ADR explicitly rather than leaving the
convention stated in a form we now know is wrong for one of its cases.
