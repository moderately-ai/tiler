---
id: add-checked-closure-convenience-for-shared-ir-builders
title: Add checked closure convenience for shared IR builders
status: in-progress
priority: p1
dependencies: [prototype-canonical-index-region-slice]
related: [prototype-shared-compiler-ir-ownership, update-adr-0071-schedule-builder-boundary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dx]
claimed_from: todo
assignee: agent-add-checked-closure-convenience
lease_expires_at: 1784861597
---
# Add checked closure convenience for shared IR builders

## Goal

Complete ADR 0071's accepted ergonomic layer with a closure-based convenience
that delegates to the same transactional builder and consuming `build()`
verifier. Keep the mutable draft scoped to the closure and return only an
immutable verified product.

## Work

- Decide one error composition that preserves both closure/admission failures
  and recoverable whole-object verification diagnostics without erasing either.
- Add the convenience first for `IndexRegionBuilder`; make the pattern reusable
  by later schedule, kernel, and program builders without a generic untyped IR
  abstraction.
- Document ordinary builder and closure call sites side by side.

## Acceptance

- The successful closure path produces the same canonical region as manual
  construction followed by `build()`.
- Admission and whole-region failures retain their typed distinctions and do
  not expose or forge verified storage.
- ADR 0071 and the public API docs no longer describe an unimplemented
  convenience as part of the implemented static slice.

## Outcome

**Fact.** Implemented the closure convenience `IndexRegionBuilder::build_with` in
`crates/tiler-ir/src/index/builder.rs`. It is an associated function taking a
`FrozenScalarRegistry` and an authoring closure
`FnOnce(&mut IndexRegionBuilder) -> Result<(), IndexBuildError>`, returning
`Result<VerifiedIndexRegion, CheckedBuildError<IndexBuildError, IndexRegionBuildError>>`.
It constructs the builder via the existing `new`, runs the closure to author the
draft, then consumes it through the same `build()` verifier. The mutable draft
never escapes to the caller; on success only the immutable `VerifiedIndexRegion`
is returned.

**Fact.** The one error composition is the new public generic
`tiler_ir::CheckedBuildError<Admission, Verification>` in
`crates/tiler-ir/src/convenience.rs`, with variants `Admission(Admission)` and
`Verification(Verification)`. For the index layer it is instantiated as
`CheckedBuildError<IndexBuildError, IndexRegionBuildError>`. The `Verification`
variant wraps the intact `IndexRegionBuildError`, so `.diagnostics()` and
`.into_parts()` (recoverable builder) remain available — neither failure kind is
erased or collapsed. It implements `Display` and `Error` (with a `source()`
chain) under the obvious bounds.

**Fact.** The shared, reusable shape is `CheckedBuildError` plus the crate-private
generic combinator `build_checked(builder, assemble, verify)`. Later schedule,
kernel, and program builders reuse both by instantiating the two type parameters
with their own admission/verification error types — no untyped abstraction and no
universal builder trait. A `#[cfg(test)]` toy builder in `convenience.rs` with
distinct error types proves the shape composes generically.

**Fact.** Acceptance evidence (all green under the full repository gate):
- `closure_convenience_matches_manual_construction` asserts the closure path and
  manual `new`/author/`build()` path produce byte-identical
  `canonical_identity()` (owner-independent canonical identity, matching the
  established `constant_region` idiom).
- `closure_admission_failure_surfaces_typed_error_without_verification` shows a
  `WriteToInput` insertion failure surfaces as `CheckedBuildError::Admission`
  with verification never reached.
- `closure_verification_failure_preserves_recoverable_diagnostic` shows a
  no-output region surfaces `CheckedBuildError::Verification`, that
  `.diagnostics()`/`.into_parts()` recover the intact builder and the
  `NoOutputs` diagnostic, and that the recovered builder can be amended and
  rebuilt.
- Compile-fail `tests/index-region/fail/closure_cannot_build.rs` proves the
  closure (holding only `&mut IndexRegionBuilder`) cannot consume the draft
  through `build()` (E0507), so it cannot obtain the verified product except via
  the checked path. The pre-existing `forge_verified.rs` already proves the
  opaque `VerifiedIndexRegion` cannot be fabricated by struct literal.

**Fact.** Added `# no longer unimplemented` documentation: `build_with` is
documented in tiler-ir rustdoc with the manual and closure call sites side by
side, and the shared shape carries module and type rustdoc. No tiler-ir source
rustdoc described this convenience as unimplemented prior to this change.

**Inference.** Because `build_with` owns construction through verification and the
closure receives only `&mut`, the convenience is exactly as safe as the manual
path: there is no additional way to reach or forge verified storage.

**Deferred.** The ADR 0071 decision-document status line (`docs/decisions/0071*.md`
"The accepted closure convenience is not part of this first implementation") is
OUT of this ticket's `implementation/ir` scope and is owned by
`update-adr-0071-schedule-builder-boundary`. Not edited here.

**Measurement.** `uv run --locked python scripts/check_repository.py` passed
("complete repository validation passed"); `git diff --check` clean;
`ticketsplease guard` verdict ok. Toolchain nightly-2026-07-19, macOS arm64.
