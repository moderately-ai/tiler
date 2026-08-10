---
id: resolve-the-generated-facade-path-under-crate-renaming
title: Resolve the generated facade path under crate renaming
status: deferred
priority: p2
dependencies: [admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [prototype-inline-proc-macro-frontend]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A consumer that renames its Tiler dependency — `tensor = { package = "tiler", version = "..." }`, or two Tiler versions side by side — either compiles or is told exactly why it cannot. Today it gets an unresolved-path error inside macro-generated tokens, which points at the call site rather than at the manifest that caused it.

## Implementation keys

**Fact.** A procedural macro has no `$crate`. `crates/tiler-macros/src/lib.rs` therefore emits fixed absolute facade paths headed by `FACADE_ENTRY_PATH = "::tiler::__private::bind_and_build"` and `FACADE_ROUTE_ENTRY_PATH = "::tiler::__private::bind_route_and_build"`; retained diagnostics also route through the exact `::tiler::__private::__tiler_compile_error!` authority. Every route still resolves only while the consumer's dependency is literally named `tiler`.

**Fact.** The failure is loud, not silent: renaming produces `error[E0433]: failed to resolve: use of unresolved module or unlinked crate 'tiler'` at the invocation. Nothing is silently wrong, which is why this is p2 and not a defect.

**Fact.** The corpus contains no prior art here. Checked in the admission worktree at `7b1e3a7`: `grep -rn '\$crate' docs/ spikes/ crates/` matches only `macro_rules!` bodies under `spikes/shapes/**` and `crates/tiler-ir/src/shape/evidence.rs`; the string "proc-macro-crate" appears once, at `docs/research/embedding/embedded-artifact-costs.md`, as prose meaning "the proc-macro crate's fixed build cost" rather than the crates.io package.

Candidates, and what each costs:

- **Keep the fixed path.** Zero cost, fails loudly, and the diagnostic is poor. Defensible for pre-alpha with no external consumers.
- **Resolve via the `proc-macro-crate` package**, which reads the consumer's `Cargo.toml` at expansion time. Standard ecosystem answer, and it makes expansion depend on filesystem state outside the token stream — which is a cache-identity question here, not a style one. ADR 0050 requires a complete compilation key, and `spikes/macro-environment/**` already measured that a proc macro cannot observe `TARGET`/`CARGO_CFG_*`; whether a resolved crate name belongs in the key is exactly the same class of question and is unanswered.
- **Detect and reject with a spanned diagnostic** naming the manifest requirement. Keeps expansion self-contained, converts a confusing error into an explainable one. Needs a way to detect the rename, which lands back on the second option's machinery.

## Explicitly deferred (2026-07-31)

The question is deferred with its reconsideration triggers armed, and the elimination is recorded so the deferral is a position rather than a postponement. Keeping the fixed `::tiler::` path is the only candidate that embeds no unanswered question: it fails loudly (`E0433` at the invocation) and never silently, which satisfies the fail-closed bar. Resolving via the `proc-macro-crate` package would make expansion depend on filesystem state outside the token stream, and whether a resolved crate name belongs in the ADR 0050 compilation key is exactly the class of cache-identity question the macro-environment spike showed cannot be waved through — answering it now, with zero consumers who rename, would be designing the key against an imagined caller. The detect-and-reject candidate needs that same machinery and inherits the same question.

Reconsideration triggers, either of which reopens this: the first real consumer that renames its Tiler dependency (the loud `E0433` is the signal), or the first artifact-identity work that must decide what the expansion-time compilation key contains — at that point the resolved-name question must be answered anyway, and this ticket's mechanism choice falls out of that answer rather than preceding it.

## Closes when

One candidate is chosen with its reasoning recorded, or the question is explicitly deferred with a reconsideration trigger (most likely: the first consumer that needs to rename, or the first artifact-identity work that must decide what the compilation key contains). If a mechanism lands, a fixture proves the renamed case, and the cache-identity consequence is stated rather than assumed absent.

## Graph maintenance

- Absolute `::tiler::__private::…` emission sites for a future rename-resolution mechanism: the four `FACADE_*` constants and related emission in `crates/tiler-macros/src/lib.rs`, plus hardcoded paths in `binding.rs` (RegionFacts / OperandFacts / SymbolFacts), `aot.rs` (`RouteFacts::source`), and `delivery.rs` (retained diagnostics); the facade's fixtures pin the fixed-path spellings. A cache-key change is a separate ticket against `implementation/cache`.
- `crates/tiler-macros/src/lib.rs` cites this ticket id by name immediately above `FACADE_ENTRY_PATH`. Renaming this ticket orphans that reference.

## Trigger check log

- 2026-08-04 — **not fired, and trigger 2's premise is refuted rather than merely unmet.** Trigger 1 is unmet: no consumer renames its Tiler dependency. Trigger 2 named "the first artifact-identity work that must decide what the expansion-time compilation key contains", and that work **landed** — `crates/tiler-cache/src/expansion/key.rs` derives the key from a `ComposedSubject`, and `SubjectFacets` is exactly `{ backend_compilations, artifact_program }` (`crates/tiler-cache/src/expansion/subject.rs:155-166`). It decided the key's complete contents **without needing the resolved-name question answered**, because a consumer's dependency spelling changes no byte of either facet. So the prediction that the question "must be answered anyway" is false, and the surviving trigger is trigger 1 alone. Recheck: `grep -n 'struct SubjectFacets' -A 12 crates/tiler-cache/src/expansion/subject.rs`.
- 2026-08-04 — **stale citation corrected.** Graph maintenance above says `crates/tiler-macros/src/lib.rs` cites this ticket at `FACADE_ANCHOR_PATH`. That constant no longer exists; the anchor is now four constants — `FACADE_ENTRY_PATH`, `FACADE_ROUTE_ENTRY_PATH`, `FACADE_FACTS_TYPE`, `FACADE_ROUTE_FACTS_TYPE` — plus the absolute `::tiler::__private::RouteFacts` path emitted from `RouteFacts::source` in `crates/tiler-macros/src/aot.rs`. The ticket id is still cited by name immediately above `FACADE_ENTRY_PATH` in `crates/tiler-macros/src/lib.rs`, so renaming this ticket still orphans a reference; the file is out of this sweep's scopes, so the correction is recorded here rather than made there.
- 2026-08-09 — **not fired.** The unsafe-authority repair added another exact generated facade route for retained diagnostics, but it deliberately kept the same absolute `::tiler` namespace and did not add a renamed-dependency consumer. The repository manifests contain no dependency declared with `package = "tiler"` under another local name. Trigger 1 remains the first real renamed consumer; the compilation-key trigger remains retired as explained above.
- 2026-08-10 — **line citations in the 2026-08-04 stale-citation entry were themselves stale.** At audit base `c99ac54950f2` the four `FACADE_*` constants, the ticket-id comment, and the `RouteFacts::source` absolute-path format string had all moved relative to the line numbers that entry recorded (`:105`/`:113`/`:116`/`:119`, `lib.rs:103`, `aot.rs:270`). Those line numbers are dropped; searchable anchors (constant names, the ticket-id comment above `FACADE_ENTRY_PATH`, and `RouteFacts::source`) remain the authority. Graph maintenance above is also tightened so a future mechanism worker does not scope only to `lib.rs`.
