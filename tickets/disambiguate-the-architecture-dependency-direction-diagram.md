---
id: disambiguate-the-architecture-dependency-direction-diagram
title: Disambiguate the architecture dependency-direction diagram
status: done
priority: p2
dependencies: []
related: [record-metal-aot-in-architecture-crate-profile]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, architecture]
---
**Fact — the "Dependency direction" diagram in `docs/architecture.md` mixes two different arrow meanings, and the sixth crate made the difference load-bearing.** The diagram's first row, `frontend integrations ─► tiler-ir ◄─ tiler-compiler`, is a Cargo dependency arrow: both sides depend on `tiler-ir`. Its last two rows, `backend emitters` above `target AOT tools` and `tiler-artifact` above `runtime adapters`, read as pipeline/data flow: emitted source flows into the AOT tools, and an artifact flows into a runtime adapter. As a Cargo edge the emitter/AOT pair points the opposite way from the direction the workspace deliberately reserves — `crates/tiler-metal/src/target.rs` and `crates/tiler-metal-aot/src/input.rs` both record that `tiler-metal-aot` → `tiler-metal` is the eventual *production* direction, and that the existing `tiler-metal` → `tiler-metal-aot` edge is development-only precisely to keep it available.

**Why it matters now rather than before.** While AOT invocation was a module inside `tiler-metal` there was no edge to get backwards. There is now, the accepted packaging profile in the same document pins it as a development dependency with a stated reason, and a reader who takes the diagram as a Cargo statement would conclude the opposite of what the profile says.

**What closes this.** Decide what the arrows denote and make the document say so. Either split the illustration into a data-flow diagram and a dependency-edge statement, or annotate the arrow kinds inline and point the dependency claims at the packaging profile block, which is the checked authority (`scripts/check_workspace.py` pins every package's exact normal and development dependency list). Do not "fix" it by redrawing the emitter/AOT arrow as a Cargo edge in either direction: the production direction is reserved and unbuilt, and asserting it here would make this document a second authority over the profile block.

Found while editing the same section under `record-metal-aot-in-architecture-crate-profile`; deliberately not changed there because choosing the arrows' meaning is an interpretive change to an accepted contract's illustration that the ticket did not scope, and a one-line "this diagram is data flow" clarification would have been false of its first row.

## Outcome

**Done, by the ticket's first option.** The illustration is now a single-meaning production-and-consumption pipeline, stated as such in the sentence above it, and the dependency claims are a prose paragraph that defers to the packaging profile rather than a second drawing.

**Correction — the diagram carried three arrow meanings, not two, and the ticket's reading of one row was wrong.** Reading the block character by character rather than by shape gives eight arrows in three senses:

- *Cargo dependency:* `frontend integrations → tiler-ir`, `tiler-compiler → tiler-ir`, and — this is the one the ticket misread — `runtime adapters → tiler-artifact`.
- *Production and consumption:* `tiler-compiler → verified IR products`, `verified IR products → tiler-artifact`, `verified IR products → backend emitters`, `backend emitters → target AOT tools`.
- *Registration:* `public op definitions → tiler-ir`, which is neither, since a definition is contributed to a registry rather than depended on or piped.

The ticket groups the last two rows together as pipeline flow. They point in opposite senses. The connector between `tiler-artifact` and `runtime adapters` carries its arrowhead at the *top* — `▲`, at column 42 under `tiler-artifact` — so it runs adapter → artifact, which is the dependency direction and the reverse of how an artifact travels. The emitter/AOT connector carries `▼` at the bottom, so it runs emitter → tooling, which is the flow direction. Two adjacent connectors, drawn identically, meaning opposite things. That makes the section harder to repair than the ticket assumed and is why annotating arrow kinds inline — the ticket's second option — was rejected: it would have required three annotations on eight arrows in a twelve-line block, and the two bottom connectors would have carried different labels while looking identical.

**Why production-and-consumption was the sense kept.** The Cargo edges already have an authority one section above: the accepted packaging profile pins every intra-workspace edge and `scripts/check_workspace.py` pins each package's complete normal and development dependency list, so redrawing them here would create an unchecked second copy of a checked contract. Flow has no other home in this document. Keeping flow and pointing the dependency claims at the profile therefore removes a duplicated authority rather than adding one.

**The emitter/AOT arrow was not redrawn as a Cargo edge, as the ticket requires.** The prose states the two facts and reserves the third: the `tiler-metal` → `tiler-metal-aot` edge is development-only for the two reasons the profile gives, and the `tiler-metal-aot` → `tiler-metal` production direction is reserved and unbuilt. Verified in source, not inferred from the profile alone — `crates/tiler-metal/src/target.rs:36-42` records that a normal edge "puts Apple tool discovery into every consumer's build graph, and Cargo's cycle rule would then forbid the eventual `tiler-metal-aot` → `tiler-metal` production direction outright", and `crates/tiler-metal-aot/src/input.rs:26-30` records the same reservation from the other side.

**One thing added beyond the ticket.** The section's four constraint sentences — runtime adapter must not link the optimizer, backend emitters do not own frontend syntax, and so on — now open with a sentence saying what they are: constraints on which components may know about which, *including roles no workspace crate has yet*. They were previously indistinguishable from commentary on the diagram, and three of the four constrain roles (`runtime adapter`, `backend emitters`, `target AOT tooling`) rather than packages, which is exactly why they are not redundant with the profile and must not be deleted as such by a later reader.

**Nodes still mix crates with roles**, deliberately: `tiler-ir`, `tiler-compiler`, and `tiler-artifact` are packages, while `frontend integrations`, `backend emitters`, `runtime adapters`, and `target AOT tools` are roles with no admitted crate. The profile above owns the package view. Renaming the roles to prospective crate names would assert a packaging decision this document does not hold and that the profile explicitly defers.

**Evidence.** Connector and label columns checked programmatically, not visually, after the edit: spine at column 25, the top join at 10/41 and the split at 14/36, with every node label centred within one column of its connector. `uv run --locked python scripts/docs.py render`; full repository gate green.
