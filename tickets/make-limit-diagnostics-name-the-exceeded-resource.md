---
id: make-limit-diagnostics-name-the-exceeded-resource
title: Make artifact limit diagnostics name the exceeded resource
status: todo
priority: p3
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [implementation/artifact]
shared_scopes: []
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
