---
id: correct-metallib-usability-claims
title: Stop treating a linked metallib as proven runtime-usable
status: done
priority: p3
dependencies: []
related: [prototype-metal-aot-slice, declare-a-required-gpu-family-in-the-artifact]
scopes: [implementation/metal-aot, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, aot, documentation, correctness]
---
Keep offline compilation evidence separate from runtime compatibility evidence.

`tiler-metal-aot` currently describes one successful-output condition as a
“usable Metal library.” A successful `metallib` link proves that the offline
toolchain produced a library for the requested compilation target. It does not
prove that every device or deployment target named by surrounding metadata can
load or execute it.

## Outcome

Use “produced” or “linked” artifact language at the AOT boundary. Reserve
“runtime-compatible” or “usable on device” for evidence that includes the
declared family/profile checks and successful runtime preparation required by
the runtime contract.

Correct code documentation, diagnostics, backend contracts, and examples that
currently cross that evidence boundary. Do not weaken a genuine
output-validation failure merely to change its wording.

## Closes when

No offline-only result claims runtime usability, runtime compatibility claims
name their evidence boundary, diagnostics remain actionable, and the
documentation corpus agrees.

## Outcome — corrected at four sites, with the check left intact (2026-07-27)

**Fact — what the driver actually validates.** `driver.rs` reads the linker's output and requires it to begin with `METALLIB_MAGIC`, which is `*b"MTLB"`. That is the whole test. Nothing in `tiler-metal-aot` opens a Metal device, so the crate has no way to form a runtime-compatibility opinion and never did.

**Corrected.** The three sites the ticket implied, plus the crate root:

- `driver.rs` — `compile`'s error list said "when the linker yields no usable Metal library"; it now says the output does not begin with the `MTLB` magic, and states that a success is offline evidence only.
- `diagnostic.rs` — `DriverError::EmptyArtifact`'s doc said "produced no usable Metal library"; it now names the shape check and says explicitly that it is not a compatibility verdict.
- `diagnostic.rs` — the `Display` string "offline compilation produced no usable metallib" became "offline compilation produced no metallib-shaped output". Still actionable: it names the stage, the condition, and carries the same `detail`.
- `lib.rs` — the crate root repeated the claim and is the AOT boundary this ticket is about, so it now carries the evidence boundary once, in full: what a `CompiledArtifact` proves, what decides runtime compatibility instead, and why merging the two would cross a line the rest of the workspace holds.

**The validation is unchanged.** Only wording moved; the magic-byte test, its position, and the typed error it raises are identical. The ticket warned against weakening a genuine output check to change its wording, and nothing here touches control flow.

**Two near-misses were left alone deliberately.** `ToolchainPhase::VersionProbe` says a tool "did not report a usable version banner" — that is about parsing a version string, not about a library's runtime fitness. `docs/artifact-abi.md` uses "usable" twice about a digest being usable as a key outside an envelope. Neither crosses the offline/runtime evidence boundary, and rewriting them would be the overcorrection the ticket warned against.

**The variant name is now the weakest part and is left as a question.** `EmptyArtifact` describes neither what is checked (a magic-byte prefix) nor what fails (a non-`metallib` output, which need not be empty). Renaming a public variant is a public-boundary change reserved to Tom under ADR 0075, and this ticket asked for wording rather than API, so it is raised here rather than done.
