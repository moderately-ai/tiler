---
id: prototype-optimizer-conformance-gate
title: Gate the target-neutral optimizer conformance profile
status: in-progress
priority: p0
dependencies: [enforce-repository-validation-gate-integrity, prototype-artifact-program-model]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, conformance, milestone-0b]
claimed_from: todo
assignee: agent-prototype-optimizer-conformance-gate
lease_expires_at: 1784917189
---
Exercise an externally registered operation through the ordinary compiler path,
not a test-only shortcut. Cover at least two non-isomorphic graph shapes plus
fan-out or ordered multi-output behavior: generic occurrences, checked
refinement, region enumeration, legality evidence, complete selection, verified
KIR, neutral and artifact program construction, typed stable explain,
deterministic identity, and the correct failure taxonomy. Remove proof-only
candidate lists and downstream `cfg(test)` isolation after interface review.

Include identity conformance for provider-only revision changes, identical
region/index/schedule structure at distinct occurrences, occurrence-specific
refinements, and complete-plan coverage. Assert identity and selected-provider
provenance at every implemented layer. Each change must affect only the identity
and provenance subjects governed by ADR 0072.

The reviewed draft authorities this gate must wire into the ordinary path are now
concrete: `capability`, `legality`, `fusion_legality`, `frontier`, `cover`, and
`selection` (plus the pre-existing `explain`/`feasibility` drafts). Each carries a
module-level `#![allow(dead_code, reason = "reviewed draft authority; not yet
wired…")]` that must be removed as it is wired — a still-present allow after this
gate is a sign the authority is not actually on the compile path. Two concrete
deferrals recorded at their review to settle here: promote `cover`'s draft-local
`CoverBudgets` into the live `request::DeterministicBudgets` (it is local today
only to avoid fields read solely under `cfg(test)`), and emit the draft
authorities' typed events through the explain vocabulary rather than leaving them
explain-silent.
