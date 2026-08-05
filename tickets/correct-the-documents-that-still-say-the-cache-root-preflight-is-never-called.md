---
id: correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called
title: Correct the documents that still say the cache-root preflight is never called
status: done
priority: p3
dependencies: []
related: [call-the-expansion-cache-preflight-on-the-resolved-root]
scopes: [contracts/navigation, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [cache, frontend, diagnostics, documentation]
---
## User-visible outcome

The three documents a reader consults about the expansion cache agree with the code: the frontend contract states that a resolved root is probed and what the line says, and neither the status page nor the open-questions record still asserts that the probe is never taken.

## Why this exists

**Fact — the code moved and the documents did not.** `call-the-expansion-cache-preflight-on-the-resolved-root` landed the call: `crates/tiler-macros/src/preflight.rs` probes the resolved root once per process from `open_cache`, and an unsuitable verdict writes one attributable line to the expanding process's standard error without failing the build. Reproduce with `grep -rn 'report_unsuitable_root' crates/tiler-macros/src/`, which now returns the definition and the call site.

**Fact — two documents assert the opposite, in words.** `docs/status.md` says "`ExpansionCache::preflight` is still not called on a resolved root, so the filesystem properties the publication protocol assumes are unprobed by any expansion", and `docs/open-questions.md`'s Q-ART-004 closure record says "one residue survives the closure … `ExpansionCache::preflight` is still not called on a resolved root", offering `grep -rn preflight crates/tiler-macros/src/` as its reproduction. That command's output no longer supports either sentence.

**Fact — the frontend contract is silent rather than wrong.** `docs/integration/frontends.md`'s "Compiler cache" section specifies the root policy and the whole eviction policy, including the shape of the eviction's refusal line. It says nothing about the root probe, so a reader learns the eviction's diagnostic exists and not the probe's — even though the probe's line was deliberately written to that same shape.

**Why this is a separate ticket.** The implementing ticket's scope is `implementation/frontend`. `docs/status.md` and `docs/open-questions.md` are `contracts/navigation` and `docs/integration/**` is `contracts/integrations`, so the worker could not correct them in the same change and filed this rather than absorbing it.

## Required delivery

- Correct `docs/status.md`'s expansion-cache bullet so its stated reproduction command supports what it claims, in that document's own "Corrected <date> —" idiom rather than by deletion.
- Correct the residue sentence in `docs/open-questions.md`'s Q-ART-004 closure record the same way, and state that its owner ticket is complete.
- State the probe in `docs/integration/frontends.md`'s "Compiler cache" section beside the eviction's refusal it matches: that a resolved root is probed once per build process before the cache is used, that an unsuitable answer is one attributable line on standard error naming the root and each property that did not answer, that a refuted property and an unprobed one are distinguished, that the expansion is never refused over it, and that `TILER_EXPANSION_CACHE_DIR=off` probes nothing.

## Non-goals

- Changing the diagnostic, the trigger, or the amortization. Those are implemented and tested; this ticket describes them.
- Re-opening whether the probe should refuse a build. It must not, for the reason the eviction's refusal must not.

## Closes when

No document asserts the probe is uncalled, every reproduction command a corrected sentence offers supports it, and the frontend contract states the probe beside the eviction refusal whose shape it matches.
