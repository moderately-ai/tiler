---
id: disambiguate-the-architecture-dependency-direction-diagram
title: Disambiguate the architecture dependency-direction diagram
status: todo
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
