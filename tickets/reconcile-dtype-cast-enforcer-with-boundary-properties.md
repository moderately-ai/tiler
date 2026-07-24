---
id: reconcile-dtype-cast-enforcer-with-boundary-properties
title: Reconcile the dtype-cast enforcer with the boundary-property list
status: done
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

## Outcome

Resolved against the Enforcers list: a dtype cast is not an enforcer, and dtype is not a missing boundary property. Adding dtype to the property list would have been the cheap edit and would have been wrong in two independent ways.

The first is direct. `docs/numerical-semantics.md` states that "Casts are semantic operations with resolved, typed conversion contracts", that a fused implementation avoiding materialization of a typed edge "must still reproduce every semantic conversion on that edge", and — most directly — that "Narrowing, flushing, or NaN rewriting in scratch is an explicit semantic conversion, never a cost-only storage choice". An enforcer is by construction a cost-bearing schedule-level insertion, so that last sentence forbids exactly the thing the bullet licensed. ADR 0009 adds that "Physical storage representation and allocation are separate decisions and cannot introduce or erase semantic rounding", and ADR 0010 forbids a later phase substituting a different conversion. A conversion already in the graph is realized by ordinary lowering of that operation and supplies no missing property; a conversion absent from the graph may not be introduced at all. Neither case is an enforcer.

The second is structural and is why dtype cannot be admitted to the property list even as a corrective. Satisfaction over that list is a subsumption order — the document's own example is "16-byte alignment satisfies a 4-byte requirement". The dtype analogue is a producer keeping `f32` where the boundary calls for `f16`, which is precisely the erased narrowing ADR 0009 and ADR 0010 forbid. The ordering relation the property system runs on is unsound over dtype, so dtype is absent from that list by construction rather than by omission.

As with the sibling finding, the document lacked the general obligation and not just the one word, so the fix closes the class:

- `dtype cast` is removed from the Enforcers list, whose last item now terminates with `.` like every other list in the file (the trivial defect the audit noted in passing, fixed inline rather than deferred);
- a list-wide scope statement now says an enforcer may change only how a boundary value is stored, addressed, placed, or delivered, never which values that boundary carries, deriving the constraint from ADR 0001 — its separation of semantic planning from physical scheduling holds only because several physical schedules implement one semantic group identically;
- a following paragraph states the dtype resolution with the two arguments above;
- a third paragraph routes the adjacent question, wider computation or accumulator precision inside a region, to its actual gate: the implementation rules already require each candidate's machine-checkable numerical guarantee to refine every effective operation contract, which is conformance checked on an implementation rather than a property supplied at a boundary;
- the boundary-property section's "not properties supplied by a schedule" sentence now names resolved value dtype beside logical shape, accumulation semantics, and numerical policy, so the document states where dtype belongs instead of leaving its absence to inference.

The asymmetry was original rather than introduced. `git log -S` over `docs/compiler/optimizer.md` for `- dtype cast;`, `An enforcer supplies a missing required property`, and `Logical shape, accumulation semantics, and numerical policy` each returns only `9acca0d`, so both lists entered in the initial commit and neither was later edited into disagreement.

Two independent contracts corroborate the split rather than contradict it. ADR 0047's accepted enforcer families are "transfer/import, materialization/repacking, or legal recomputation" — dtype conversion is not among them, and the ADR separately requires that "Transfer does not silently convert encoding". The unstarted ticket pair mirrors it exactly: `implement-boundary-property-model` enumerates the properties as "layout, alignment, materialization, placement, memory domain, ordering, and synchronization" with no dtype, while `implement-boundary-property-enforcers` implements dtype conversion as an emittable operation. Dtype absent from properties, present as an operation, is the same shape this ticket ratifies, so neither needed changing.

Three follow-ups filed:

- `propagate-the-dtype-cast-enforcer-resolution-to-the-glossary-and-roadmap` — `docs/glossary.md` defines a boundary enforcer as "Explicit materialization, layout conversion, cast, or copy", and `docs/roadmap.md` quotes the old three-item list under a **Fact** label. Both are outside `contracts/optimizer`.
- `reconcile-the-transfer-taxonomy-convertdtype-label-with-the-enforcer-definition` — the transfers research proposal labels `ConvertDtype` an enforcer in its taxonomy table. Its intent agrees (its verifier requires dtype changes to use separately typed stages precisely so a transfer cannot fold one in); only the word conflicts, and the document is a proposal no accepted contract has incorporated.
- `decide-whether-storage-encoding-is-a-missing-boundary-property` — applying this ticket's admission test surfaced a candidate that passes it. Bit-packed versus unpacked sub-byte integers (ADR 0028) and quantized companion storage (ADRs 0029, 0030) are producer-side choices that preserve represented values, and ADR 0047 already accepts repacking as an enforcer family whose property the optimizer contract does not name.
