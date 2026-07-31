---
id: resolve-the-generated-facade-path-under-crate-renaming
title: Resolve the generated facade path under crate renaming
status: todo
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

**Fact.** A procedural macro has no `$crate`. `crates/tiler-macros/src/lib.rs` therefore emits one fixed absolute path, `FACADE_ANCHOR_PATH = "::tiler::__private::expansion_anchor()"`, which resolves only while the consumer's dependency is literally named `tiler`.

**Fact.** The failure is loud, not silent: renaming produces `error[E0433]: failed to resolve: use of unresolved module or unlinked crate 'tiler'` at the invocation. Nothing is silently wrong, which is why this is p2 and not a defect.

**Fact.** The corpus contains no prior art here. Checked in the admission worktree at `7b1e3a7`: `grep -rn '\$crate' docs/ spikes/ crates/` matches only `macro_rules!` bodies under `spikes/shapes/**` and `crates/tiler-ir/src/shape/evidence.rs`; the string "proc-macro-crate" appears once, at `docs/research/embedding/embedded-artifact-costs.md`, as prose meaning "the proc-macro crate's fixed build cost" rather than the crates.io package.

Candidates, and what each costs:

- **Keep the fixed path.** Zero cost, fails loudly, and the diagnostic is poor. Defensible for pre-alpha with no external consumers.
- **Resolve via the `proc-macro-crate` package**, which reads the consumer's `Cargo.toml` at expansion time. Standard ecosystem answer, and it makes expansion depend on filesystem state outside the token stream — which is a cache-identity question here, not a style one. ADR 0050 requires a complete compilation key, and `spikes/macro-environment/**` already measured that a proc macro cannot observe `TARGET`/`CARGO_CFG_*`; whether a resolved crate name belongs in the key is exactly the same class of question and is unanswered.
- **Detect and reject with a spanned diagnostic** naming the manifest requirement. Keeps expansion self-contained, converts a confusing error into an explainable one. Needs a way to detect the rename, which lands back on the second option's machinery.

## Closes when

One candidate is chosen with its reasoning recorded, or the question is explicitly deferred with a reconsideration trigger (most likely: the first consumer that needs to rename, or the first artifact-identity work that must decide what the compilation key contains). If a mechanism lands, a fixture proves the renamed case, and the cache-identity consequence is stated rather than assumed absent.

## Graph maintenance

- Only `crates/tiler-macros/src/lib.rs` and the facade's fixtures are in scope; a cache-key change is a separate ticket against `implementation/cache`.
- `crates/tiler-macros/src/lib.rs` cites this ticket id by name at `FACADE_ANCHOR_PATH`. Renaming this ticket orphans that reference.
