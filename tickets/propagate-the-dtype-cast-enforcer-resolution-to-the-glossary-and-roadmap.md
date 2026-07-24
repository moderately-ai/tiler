---
id: propagate-the-dtype-cast-enforcer-resolution-to-the-glossary-and-roadmap
title: Propagate the dtype-cast enforcer resolution to the glossary and roadmap
status: todo
priority: p2
dependencies: []
related: [reconcile-dtype-cast-enforcer-with-boundary-properties]
scopes: [contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, numerics]
---
`reconcile-dtype-cast-enforcer-with-boundary-properties` settled that a dtype cast is not a boundary enforcer and removed it from the enforcer list in `docs/compiler/optimizer.md`. An enforcer may change only how a boundary value is stored, addressed, placed, or delivered, never which values it carries; a cast is a semantic operation carrying a resolved typed conversion contract under ADR 0010, so a schedule may neither insert one nor elide one. Two documents outside that ticket's scope still assert the old list.

`docs/glossary.md` defines "Boundary enforcer" as "Explicit materialization, layout conversion, cast, or copy that satisfies a boundary requirement." Drop `cast` from that definition; materialization, layout conversion, and copy are all value-preserving and the entry reads correctly without it.

`docs/roadmap.md` states, in the Milestone 6 evidence block, that the optimizer "lists contiguous materialization, layout conversion, and dtype cast as enforcers that supply a missing required property at a cost". That sentence is labelled **Fact** and is now a misquotation of the contract. Its surrounding inference — that layout conversion is already an enforcer rather than a new mechanism — is unaffected and should survive the correction.

Check no other occurrence survives with `grep -rn -i "enforcer" docs/` and confirm each remaining hit describes a value-preserving stage.
