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

**Correction — 2026-08-10.** The two sentences above are filing motivation. On the tree they remain only as the historical residue of the dated-correction idiom: each is immediately followed by a **Corrected 2026-08-05** successor that states the probe, the stderr shape, non-refusal, `off` behaviour, and the owner implementer at `done`. They are not live claims. ADR 0089 Consequences and `docs/research/cache/root-policy.md` Unsupported cases still live-assert the probe is uncalled; those sites were outside this ticket's scopes and Required delivery (see Fact audit).

**Fact — the frontend contract is silent rather than wrong.** `docs/integration/frontends.md`'s "Compiler cache" section specifies the root policy and the whole eviction policy, including the shape of the eviction's refusal line. It says nothing about the root probe, so a reader learns the eviction's diagnostic exists and not the probe's — even though the probe's line was deliberately written to that same shape.

**Correction — 2026-08-10.** No longer silent. The same section now states the probe beside the eviction refusal (anchor: `The resolved root is probed in that same shape, and for that same reason it never refuses`).

**Why this is a separate ticket.** The implementing ticket's scope is `implementation/frontend`. `docs/status.md` and `docs/open-questions.md` are `contracts/navigation` and `docs/integration/**` is `contracts/integrations`, so the worker could not correct them in the same change and filed this rather than absorbing it.

## Required delivery

- Correct `docs/status.md`'s expansion-cache bullet so its stated reproduction command supports what it claims, in that document's own "Corrected <date> —" idiom rather than by deletion.
- Correct the residue sentence in `docs/open-questions.md`'s Q-ART-004 closure record the same way, and state that its owner ticket is complete.
- State the probe in `docs/integration/frontends.md`'s "Compiler cache" section beside the eviction's refusal it matches: that a resolved root is probed once per build process before the cache is used, that an unsuitable answer is one attributable line on standard error naming the root and each property that did not answer, that a refuted property and an unprobed one are distinguished, that the expansion is never refused over it, and that `TILER_EXPANSION_CACHE_DIR=off` probes nothing.

## Non-goals

- Changing the diagnostic, the trigger, or the amortization. Those are implemented and tested; this ticket describes them.
- Re-opening whether the probe should refuse a build. It must not, for the reason the eviction's refusal must not.
- Correcting ADR 0089 Consequences or `docs/research/cache/root-policy.md`. Those were never in Required delivery or in the scopes below; they are residual documentation debt after this ticket's three-document outcome.

## Closes when

The three documents named in User-visible outcome and Required delivery no longer live-assert that the probe is uncalled: `docs/status.md` and `docs/open-questions.md` carry dated corrections whose stated reproduction commands support the corrected claim, and `docs/integration/frontends.md` states the probe beside the eviction refusal whose shape it matches.

## Outcome — landed by 2026-08-05 (ticket body recorded 2026-08-10)

Three-document delivery under scopes `contracts/navigation` and `contracts/integrations`:

- `docs/status.md` cache-root chooser bullet: after the retired "still not called" sentence, **Corrected 2026-08-05 — the probe landed too** states the probe from `crates/tiler-macros/src/preflight.rs`, the attributable stderr line, non-refusal, `TILER_EXPANSION_CACHE_DIR=off` behaviour, `grep -rn report_unsuitable_root crates/tiler-macros/src/`, the named preflight tests, and [`call-the-expansion-cache-preflight-on-the-resolved-root`](call-the-expansion-cache-preflight-on-the-resolved-root.md) at `done`.
- `docs/open-questions.md` Q-ART-004: **Corrected 2026-08-05 — that residue is closed and its owner ticket is `done`, so Q-ART-004 carries none**, with the same behavioural restatement and a pointer to the frontend contract.
- `docs/integration/frontends.md` Compiler cache: full probe paragraph beside the eviction refusal (anchor: `The resolved root is probed in that same shape, and for that same reason it never refuses`).

No crate change was required by this ticket; code authority remains the related implementer.

## Fact audit — 2026-08-10

**Verified.** `report_unsuitable_root` is defined in `crates/tiler-macros/src/preflight.rs` and called from `aot::open_cache` on the `Directory` arm; Disabled skips the probe without spending the process gate. Reproduce: `grep -rn report_unsuitable_root crates/tiler-macros/src/`.

**Verified.** Required delivery and User-visible outcome for the three named documents are satisfied at the present tree (Outcome above).

**Correction.** The original Closes when said "No document asserts the probe is uncalled", which overclaimed relative to Required delivery and scopes. ADR 0089 Consequences still contains `ExpansionCache::preflight` is still not called on a resolved root, and `docs/research/cache/root-policy.md` Unsupported cases still contains `ExpansionCache::preflight` is not called — neither has a Corrected successor. Closes when is narrowed to the three documents this ticket owned. Remainder (new ticket or reopen with `contracts/decisions` plus the research path) is product debt outside this wave; link related to this ticket and to `call-the-expansion-cache-preflight-on-the-resolved-root` when filed.

**Status.** `done` retained for the three-document outcome.
