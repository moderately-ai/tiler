---
id: admit-the-tiler-facade-and-proc-macro-crate-boundary
title: Admit the tiler facade and proc-macro crate boundary
status: done
priority: p1
dependencies: []
related: [prototype-inline-proc-macro-frontend]
scopes: [implementation/frontend, implementation/workspace]
shared_scopes: [implementation/cargo-lock, project/tickets]
paths: []
tags: []
---
## User-visible outcome

Consumers import one ordinary `tiler` facade and call `tiler::tensor!`; the procedural implementation lives in a separate `tiler-macros` proc-macro crate, while normal runtime/frontend types remain available from the facade.

## Implementation keys

The approved surface fixes the public path as `tiler::tensor!`. A proc-macro crate cannot be the durable normal-type/runtime facade because Rust restricts what a proc-macro crate exports. A standalone `tiler-macros` crate would either force users to depend on internal crates named by generated tokens or change the approved import path. A normal `tiler` facade re-exporting the macro from `tiler-macros` is the standard dependency direction and keeps generated paths stable.

Admit both workspace members atomically. `tiler-macros` owns token parsing, span mapping, and expansion; `tiler` owns stable re-exports and the consumer-visible frontend/runtime traits selected by their dedicated boundary tickets. Neither crate creates a second semantic operation vocabulary, invokes runtime JIT, scans source, requires `build.rs`, or hides a generated dependency the consumer did not receive.

## Public boundary for Tom

Ratify the two-crate topology and public `tiler::tensor!` path before workspace admission. The exact manifests, dependency direction, re-export, minimal public module tree, and one compile-pass consumer are the review packet. A crate admission does not stabilize the macro grammar or runtime adapter beyond their separately accepted tickets.

## Ratification (2026-07-30)

Tom approved the `tiler` normal facade plus `tiler-macros` proc-macro implementation topology and the public `tiler::tensor!` path. Implementation may proceed with the dependency direction and exclusions above; the exact manifest/re-export diff remains part of acceptance evidence rather than a reopened topology choice.

## Accepted (2026-07-31)

Tom accepted the exact surface below without exception. The crate documentation now states the accepted status; the corpus amendment and admission ADR remain owned by `record-the-frontend-crate-admission-in-the-design-corpus`, which this acceptance unblocks.

## Review packet (2026-07-31) — accepted above

The implementation landed on `tkt/admit-the-tiler-facade-and-proc-macro-crate-boundary`. The ratification above settled the *topology*; the surface below is a concrete draft and **is not accepted until Tom accepts the exact interface**. ADR 0075 classifies both a new workspace member and a new crate-root `pub mod` as always requiring that review, and this diff is both. No document in the corpus describes this boundary as accepted, and `crates/tiler/src/lib.rs` says so in its own module documentation, per ADR 0074 §7.

The complete public surface added, which is what there is to accept or refuse:

- `tiler-macros`: `#[proc_macro] pub fn tensor(TokenStream) -> TokenStream`. Nothing else; a proc-macro crate can export nothing else.
- `tiler`: `pub use tiler_macros::tensor;` — the ratified `tiler::tensor!` path.
- `tiler`: `#[doc(hidden)] pub mod __private`, containing `pub struct ExpansionAnchor` (unit, `Debug + Clone + Copy + PartialEq + Eq`) and `pub const fn expansion_anchor() -> ExpansionAnchor`.

`__private` is the part worth arguing about. A proc macro has no `$crate`, so its expansion must spell an absolute path, and that path has to terminate somewhere public. Putting it in the facade is what makes "generate only paths reachable through the consumer's declared `tiler` dependency" true; the alternative — generated tokens naming `::tiler_ir::` or `::tiler_artifact::` directly, as the example at `docs/integration/frontends.md` still does — hands the consumer a dependency it never declared. It is `#[doc(hidden)]`, carries no compatibility claim, and is disclosed here rather than hidden behind that attribute.

Dependency direction is `tiler -> tiler-macros` and nothing else. The facade re-exports no frontend or runtime types yet: those are selected by `define-inline-symbol-binding-and-runtime-value-adaptation` and `promote-artifact-family-selection-for-the-frontend`, and re-exporting anything now would publish a boundary this ticket did not review. `tiler-macros` has an empty dependency closure — it uses only the compiler-provided `proc_macro` crate, so the lockfile diff adds no third-party package at all.

`tiler` is deliberately absent from `[workspace.dependencies]`: nothing in the workspace may depend on the facade, and an entry there would only make that edge easy to add by accident.

### Scope of what the macro does today

There is no grammar, and none was invented. Empty input expands to the anchor; any non-empty input is a spanned `compile_error!` naming the two tickets that own the grammar. Empty input is a sentinel for "no region yet", not a case the eventual grammar is expected to accept — those tickets replace the body rather than extend it. What the placeholder does buy is compiler-checked evidence for the two properties a later grammar would otherwise have to re-establish: that the re-export resolves, and that generated tokens reach the facade.

### Consequence for the corpus, filed not absorbed

Merging this makes `docs/status.md`'s reproducible absence block (`test ! -d crates/tiler-macros`, `! rg -n 'proc-macro\s*=\s*true' crates --glob Cargo.toml`) report the opposite of the prose around it, and makes `docs/architecture.md`'s "nine reusable libraries" and "deliberately omits frontend, proc-macro ... crates" false. `docs/` is outside this ticket's scopes, so `record-the-frontend-crate-admission-in-the-design-corpus` (p1) owns the amendment and the missing admission ADR — every other member has one, these two do not. It should land in the same batch or immediately after.

`resolve-the-generated-facade-path-under-crate-renaming` (p2) records the one bounded question the fixed absolute path leaves open.

## Closes when

Tom ratifies the topology; both members, lockfile, and ticketsplease scope ownership land atomically; dependency checks prove compiler/IR remain frontend-independent; a compile-pass fixture imports only the facade; a deliberate missing re-export or wrong generated path fails; and targeted checks plus `make full` pass.

## Graph maintenance

- Add scope mappings for `crates/tiler/**` and `crates/tiler-macros/**` in the same commit that admits the members; paths alone do not make later frontend work schedulable.
- Keep `define-inline-symbol-binding-and-runtime-value-adaptation` and `promote-artifact-family-selection-for-the-frontend` dependent on this admission rather than relying on shared-scope serialization.
- Do not close `prototype-inline-proc-macro-frontend` from this ticket; it consumes the admitted facade after its separate symbol/value and artifact-family prerequisites land.
