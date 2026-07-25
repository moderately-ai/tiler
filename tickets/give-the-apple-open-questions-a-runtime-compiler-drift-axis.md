---
id: give-the-apple-open-questions-a-runtime-compiler-drift-axis
title: Give the Apple open questions a runtime-compiler drift axis
status: done
priority: p3
dependencies: []
related: [record-metal-runtime-compiler-provenance-gap]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, metal, numerics, open-questions]
---
Q-ART-007 in `docs/open-questions.md` is "Apple cross-machine and patch-toolchain evidence", closing on a "reproducibility and compatibility matrix across machines and toolchain patch versions". Its axis is the toolchain — that is, Xcode. The measured row shows that is not the only axis that moves.

**Measurement.** On one machine (Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113) the offline compiler is `metalfe-32023.883` from the Xcode MetalToolchain asset, the macOS runtime compiler is `metalfe-32023.921` from the OS-shipped `GPUCompiler.framework`, and a booted iOS 26.0 Simulator (build 23A8464) loads `metalfe-32023.830.1` from its own bundled copy. The runtime compilers move with the **OS build** and with the **simulator runtime version**, independently of Xcode.

**Consequence for the question.** A host that never changes Xcode can still change two of its three Metal compilers. Q-ART-007's close condition, read literally, would be satisfied by a matrix that holds the OS constant, and the numerical harness would then decline to compare rather than confirm — it announces an environment-row difference and does not compare, by design.

## The work

Add the OS build and the simulator runtime version as named axes of Q-ART-007, or record a sibling entry for runtime-compiler drift with its own trigger. State which measurement closes it: a re-run of `spikes/apple-targets/numerical_probe.py` on a host whose OS build differs and whose Xcode build does not, with the resulting `environment.family.<name>.runtime_compiler_build` rows compared against the retained record.

`docs/backends/metal.md` already records the measurement and the bounded cross-path agreement; this ticket only makes the open-question index name the axis. Do not restate the measurement there.

## Closes when

`docs/open-questions.md` names the OS-build and simulator-runtime axes with an explicit closing measurement, and the renderer and repository gate pass.

## Outcome

**Decided: extend Q-ART-007 in place rather than open a sibling.** Q-ART-007 now names four independent axes — machine and GPU, Xcode toolchain patch version, OS build, and installed simulator runtime version — and carries a separate `Closing measurement:` bullet. Its heading is now "Apple cross-machine, patch-toolchain, and runtime-compiler evidence", because the old title named only the two axes the entry actually asked for and a reader scanning headings would not have known the other two were in scope.

**Why not a sibling.** A sibling entry for runtime-compiler drift would close on a strict subset of Q-ART-007's evidence: one re-run of the numerical probe on a differing host produces both the cross-machine rows and the runtime-compiler rows, so two questions would be opened and closed by one measurement, and each would have to restate the other's scope to stay honest. The four axes are dimensions of one compatibility matrix, and Q-ART-007 already owns that matrix and names the Metal backend as its track. The counterpoint — that a runtime compiler is not a "toolchain patch version" in the sense the original title used, since it moves with an OS update the developer does not choose — is real, and it is answered by renaming the question rather than by splitting it.

**What the entry now forecloses.** The old close condition, read literally, was satisfiable by a matrix that varies machines and Xcode while holding the OS constant — a run in which no runtime compiler moves at all. The new `Closing measurement:` bullet states that a re-run whose `environment.family.<name>.runtime_compiler_build` rows are unchanged has not exercised the axis and does not close the question, so the null result can no longer be mistaken for evidence.

**Not restated.** The measured row itself — which compiler build each family resolves and where each ships from — stays in `docs/backends/metal.md`, which owns it. The index links its section anchor and names the axes only.

Gate: `uv run --locked python scripts/docs.py render`, then `uv run --locked python scripts/check_repository.py`, both green.
