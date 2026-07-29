---
id: correct-the-stale-post-vertical-implementation-status
title: Correct the stale post-vertical implementation status
status: done
priority: p1
dependencies: []
related: [own-the-dtype-support-maturity-matrix, prototype-metal-aot-slice, prototype-metal-runtime-proof, prototype-public-compiler-api, prototype-structured-kir-slice, prototype-kernel-program-ir, prototype-artifact-program-model, prototype-metal-kir-lowering, prototype-apple-aot-driver, prototype-metal-bundle-assembly, prototype-runtime-artifact-validation, prototype-runtime-routing-commit, prototype-metal-runtime-execution, correct-stale-public-compiler-boundary-authorities, correct-stale-artifact-identity-and-delivery-authorities, correct-stale-post-vertical-integration-inventories]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, status, correctness]
---

## User-visible outcome

`docs/status.md` accurately distinguishes what the completed compiler, Metal AOT, artifact, and bounded runtime-proof tickets delivered from the inline frontend, Candle adapter, general backend, and production-runtime work that remains. A reader starting at the documentation portal no longer receives implementation claims already falsified by the source and completed graph.

## Why this is a correctness ticket

- **Fact:** `docs/status.md` still says Metal AOT and device execution are unimplemented and no compiler API is public, while `prototype-metal-aot-slice`, `prototype-metal-runtime-proof`, and `prototype-public-compiler-api` are done and the retained runtime proof records thirty bit-identical cases on one Apple M4 Max host.
- **Fact:** the status document describes schedule, structured-kernel, program, artifact identity, lowering, driver, and runtime layers as future, and names `tiler.kernel-program.v2`; the current construction authorities use kernel-program v5, artifact-program v9, and artifact schema 7.0.
- **Fact:** checked refinement and the general compiler path have landed, but inline macro/Candle/product breadth has not. Replacing every “future” with “done” would therefore be as false as preserving the current text.
- **Inference:** the status slice must be re-derived from construction sites and completed ticket outcomes as one coherent update. Patching the first stale sentence would leave the same false boundary repeated below.

## Implementation keys

- Read `docs/status.md` in full, then verify each implementation claim against its construction site and completed owning ticket rather than ticket title or memory.
- State bounded evidence precisely: distinguish target-neutral checked mechanisms, deterministic Metal AOT construction, the one-host prototype execution corpus, production runtime support, and portable target-family guarantees.
- Record current identity/schema versions from their minting constants and `docs/artifact-abi.md`; do not copy a version from this ticket.
- Preserve still-open inline frontend, Candle adapter, cache integration, generalized workload, and production backend/runtime work. Remove only claims made false by completed outcomes.
- Search the documentation corpus for duplicated versions and “unimplemented/future/no execution/no public API” wording tied to these exact authorities; update every normative or navigation statement whose truth changed, or file a narrower follow-up with a reproducible trigger.
- Any negative source claim includes a one-line reproducible check and is read at the construction site. Fault-prove every new check once.

## Closes when

The status document agrees with current source, accepted ADRs, artifact ABI, dtype/operation maturity ledgers, and completed ticket outcomes; bounded prototype execution is not promoted into production or portable support; every version is recomputed from its authority; local links are coherent; `tkt lint` and `git diff --check` pass; and one batch `make full` passes.

## Graph maintenance

- Link any newly discovered stale contract to the exact ticket that made it false; do not leave a prose-only cleanup note.
- If correcting a public boundary reveals an actually unsettled interface, stop at that boundary for Tom's review rather than accepting it through a status edit.
- Close this ticket when the status corpus is truthful; downstream implementation tickets remain open on their own outcomes rather than keeping this correction open.
