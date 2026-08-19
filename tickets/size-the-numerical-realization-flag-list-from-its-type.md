---
id: size-the-numerical-realization-flag-list-from-its-type
title: Size the numerical realization flag list from its type
status: done
priority: p2
dependencies: []
related: [carry-required-compilation-selection-identity-on-compile-profile-contexts]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity, hardening, metal-aot]
---
## User-visible outcome

Adding a dimension to `NumericalRealization` is a compile error at the flag list rather than a silent omission, so two materially different compilations can never share one `CompilationIdentity` or one `CompilationSelectionIdentity`.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and verified first-hand by the coordinator at `de18ebdb`. **This is latent, not live** — no such field exists today, and nothing in the tree is currently misidentified. It is filed now because the failure it admits is a silent identity collision, which is the class AGENTS.md gives extra scrutiny.

**Fact — the flag list is hand-sized where its neighbours are type-sized.** `NumericalRealization` (`crates/tiler-metal-aot/src/input.rs`, anchor `pub struct NumericalRealization`) carries three fields: `math_mode`, `fp32_functions`, `fp_contract`. `flags` (same file, anchor `pub fn flags(self) -> [String; 3]`) reads those three by name and returns a hand-written three-element array.

**Fact — a new field is forced through the constructor but not through the flag list.** `NumericalRealization::new` (anchor `pub const fn new`) builds `Self { math_mode, fp32_functions, fp_contract }` with field-init shorthand, so a fourth field is a compile error there and the author is forced to touch `new`. Nothing forces them to touch `flags`. The omission compiles.

**Fact — the omission would reach identity.** The flag strings `flags` returns are what the compilation selection is derived from, so a numerical dimension wired through `new` and omitted from `flags` leaves both `CompilationIdentity` and `CompilationSelectionIdentity` silently. Two compilations differing only in that dimension would then encode identical selection bytes and pass the per-population equality check that `carry-required-compilation-selection-identity-on-compile-profile-contexts` landed — the check would confirm agreement on a projection that had stopped being total.

**Fact — the existing perturbation test would not catch it either.** `the_selection_excludes_source_and_toolchain_and_tracks_every_selection_field` enumerates six hand-written cases with no type-derived sizing, so it asserts over the same population that shrank. Its name claims totality that its body does not deliver.

## Required work

- Destructure irrefutably at the top of `flags` — `let Self { math_mode, fp32_functions, fp_contract } = self;` — so a widened struct is a compile error at this function, and state in a comment why the destructure exists rather than leaving it looking like style. The return type's `[String; 3]` should be derived or asserted against the field count rather than left as a second independent hand-written number, if a spelling that does so is available without inventing machinery.
- Apply the same test to the perturbation case list: make the enumeration fail loudly when the vocabulary widens, per AGENTS.md's "size enumerations from the type". Where the population genuinely cannot be typed, assert a floor and print the census.
- **Perturb the subject, not the assertion, and quote the failure text.** Add a fourth field to `NumericalRealization` locally, confirm the build now fails at `flags` and at the case list, quote both messages, and restore. A report that claims the guard can fail without showing its message has not demonstrated it. Perturb each guard separately — a perturbation that reddens everything cannot show which one is load-bearing.
- Check the sibling accessors on this type and on `CompileRequest` for the same shape before concluding; per AGENTS.md, finding one instance of a pattern obliges checking all siblings. Report what you found and what you found clean.

## Non-goals

Adding any numerical dimension, changing what `flags` emits for the current three fields, moving any identity domain or pin, and any repair outside `crates/tiler-metal-aot/`. **This change must not move a byte**: the emitted flag strings, `CompilationIdentity`, and `CompilationSelectionIdentity` are all expected to be unchanged, and the standard Metal pins in `crates/tiler-build` must recompute identical. If any of them moves, stop and report rather than repinning.

## Closes when

A widened `NumericalRealization` is a compile error at both the flag list and the perturbation case list with the failure text quoted for each, the sibling scan is reported, every current identity and pin is proven unmoved, and `cargo nextest run -p tiler-metal-aot` plus the touched-package Clippy and rustdoc gates are green.
