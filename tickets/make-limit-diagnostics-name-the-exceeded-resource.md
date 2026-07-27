---
id: make-limit-diagnostics-name-the-exceeded-resource
title: Make artifact limit diagnostics name the exceeded resource
status: done
priority: p3
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, diagnostics, correctness]
---
A malformed or oversized artifact must tell the user which governed resource
exceeded its limit.

## Fact

The program decoder bounds stage-dependency edges with
`MAX_STAGE_DEPENDENCIES` but classifies that vector through
`CodecLimitKind::Entries`. A caller therefore receives a nearby entry-limit
diagnostic rather than the dependency-edge limit that actually rejected the
bytes.

## Outcome

Give every independently bounded codec collection a diagnostic kind that names
that collection. Audit neighboring limit call sites for the same
misclassification, without renaming limits whose resource genuinely is shared.

## Closes when

A stage-dependency overflow reports the dependency resource and its actual and
maximum counts, negative neighbors prove entry and dependency limits remain
distinct, and the full gate passes.

## Outcome — one real misclassification of forty-eight pairings (2026-07-27)

**The audit was exhaustive over a counted population, not a look at the neighbours.** A scan of every `MAX_* , CodecLimitKind::*` pairing across `codec/{decode,encode,budget,payload,model}.rs` — including multi-line calls, which a line-oriented grep misses — found **48** pairings. Exactly one misclassifies:

- `MAX_STAGE_DEPENDENCIES → CodecLimitKind::Entries` at `decode.rs`'s `parse_dependencies`. Now `CodecLimitKind::StageDependencies`.

**What the defect actually produced, measured.** Reverting the classification and running the new test prints `Limit { resource: Entries, actual: 18446744073709551615, limit: 65536 }`. The pairing is the tell: `65_536` is `MAX_STAGE_DEPENDENCIES`, while `MAX_VARIANT_ENTRIES` is `4_096`. A caller was told the *entry* resource had exceeded a limit that is not the entry limit — so both halves of the diagnostic were wrong together, and neither the resource nor the number would lead a reader to the collection that actually rejected the bytes.

**One near-miss was left alone deliberately, and it is the case this ticket warned about.** `MAX_INTERFACE_ENTRIES → CodecLimitKind::BindingTargetKeys` (`decode.rs`, `budget.rs`) pairs an interface-entry *constant* with a binding-target-keys *kind*. That is not a misclassification: the kind names the collection that overflowed, which is what a reader needs, and the constant is genuinely shared between two collections that are bounded alike. The ticket's instruction not to rename limits whose resource is genuinely shared covers exactly this, so the only change would have been to make the diagnostic worse.

**Negative neighbour, as required.** `a_stage_dependency_overflow_names_the_dependency_resource` drives `parse_dependencies` over an absurd count and asserts `StageDependencies`, then drives the entry budget over the *identical* bytes and asserts `Entries`. The second half is what makes it a regression guard rather than a snapshot: collapsing the two kinds back together fails it. `parse_dependencies` became `pub(super)` to allow this, matching `parse_expression_arena`, which is already `pub(super)` for the same reason.

**The failure path was confirmed reachable** by reverting the classification and watching the test fail on exactly that assertion, then restoring it. `CodecLimitKind` is `pub(crate)` and nothing maps it totally, so the new variant is additive and changes no public surface.
