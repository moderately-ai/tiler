---
id: reconcile-dtype-cast-enforcer-with-boundary-properties
title: Reconcile the dtype-cast enforcer with the boundary-property list
status: todo
priority: p2
dependencies: []
related: [qualify-contraction-association-reassociation-permission, implement-boundary-property-model, implement-boundary-property-enforcers]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, numerics]
---
`docs/compiler/optimizer.md` lists "dtype cast" as an enforcer that "supplies a missing required property at a cost", beside contiguous materialization and layout conversion. Its own boundary-contract list two sections later does not contain dtype: the initial boundary contracts are storage layout class and contiguous axes, alignment and vectorizable width, materialized buffer / alias-view / opaque runtime value, and device and address space. The same paragraph states that logical shape, accumulation semantics, and numerical policy are semantic traits or optimization-context constraints, not properties supplied by a schedule.

Either dtype is a boundary property missing from that list, or a dtype cast is not an enforcer. The distinction is observable. ADR 0010 makes every semantic numeric conversion carry a resolved typed conversion contract, forbids a backend silently substituting a different conversion, and its context explicitly warns that fusion may remove the store and reload that happened to realize a conversion. An enforcer that introduces a narrowing absent from the semantic graph changes the program's values; one that merely realizes a conversion already in the graph supplies no missing property and is therefore not an enforcer.

Decide which, and state it in the enforcer section. If dtype becomes a boundary property, its requirement/guarantee vocabulary, subsumption, and dominance owe the same treatment alignment already has, and the enforcer owes a typed conversion contract rather than a bare cast.

Found while auditing the optimizer rule lists for transformations stated with no declared numerical permission (`qualify-contraction-association-reassociation-permission`).
