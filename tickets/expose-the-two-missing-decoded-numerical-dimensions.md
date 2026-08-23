---
id: expose-the-two-missing-decoded-numerical-dimensions
title: Expose the two missing decoded numerical dimensions
status: in-progress
priority: p2
dependencies: []
related: [admit-an-explicit-non-arithmetic-region-and-delivery-state]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifacts, numerics, public-boundary]
claimed_from: todo
assignee: worker-decoded
lease_expires_at: 1787471800
---
## Outcome

A decoded artifact can report every numerical dimension it carries on the wire, so the decoded view stops being narrower than the record it views.

## Fact — two dimensions cross the wire and cannot be read back, at `41a018fbc7e33e9a573a63a61264f49e5f41717a`

`NumericalFacts` carries ten behaviour dimensions plus the profile key and the canonical NaN bits (`crates/tiler-artifact/src/program/codec/model.rs`, anchor `pub(crate) struct NumericalFacts {`). The encoder writes all twelve and the decoder reads all twelve.

The decoded view exposes ten accessors — `profile_key`, `canonical_arithmetic_nan_bits`, `input_subnormals`, `result_subnormals`, `contraction`, `reassociation`, `permutation`, `signed_zero`, `nan_assumptions`, `infinity_assumptions` — and none for `reciprocal_transform` or `approximate_intrinsics` (`crates/tiler-artifact/src/program/codec/view.rs`, anchor `pub struct DecodedNumerical`). Both were added to the record by the `19.0` manifest step; the accessors were not added with them.

This is a read-side gap rather than a correctness defect: the bytes are present, identity folds them, and the builder-side `EntryRealization` carries all ten. What is missing is any way for a consumer holding a decoded artifact to ask about two of the dimensions the artifact declares.

## Required delivery

Add the two accessors in the idiom of their eight siblings. Then size the population from the type rather than by hand, so the next widened dimension set is a build error at the view instead of a silently narrower one: the record already has `DIMENSION_COUNT` in `crates/tiler-ir/src/numerics.rs` (anchor `pub const DIMENSION_COUNT: usize = 11;`), and the ten-versus-eleven asymmetry with `MaterializationRounding` is deliberate — read `overlapping_behaviour` before assuming the two sets should be equal.

## Closes when

Every behaviour dimension `NumericalFacts` carries is readable from the decoded view; a test enumerates the dimension set from the type rather than from a hand-written list; and the deliberate exclusion of `MaterializationRounding` from the per-entry set is stated where a reader meets it.
