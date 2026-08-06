---
id: stop-copying-the-carried-payload-through-the-builder-assemble
title: Stop copying the carried payload through the builder's assemble
status: todo
priority: p2
dependencies: []
related: [stop-copying-the-carried-payload-through-the-envelope-projection, measure-artifact-decoder-allocation-amplification]
scopes: [implementation/artifact]
shared_scopes: [research/artifacts]
paths: []
tags: [artifact, codec, performance]
---
`ArtifactProgramBuilder::build` copies every carried object once more, on the
same publication path
[`stop-copying-the-carried-payload-through-the-envelope-projection`](stop-copying-the-carried-payload-through-the-envelope-projection.md)
reduced to one.

`build(self)` calls `assemble(&self, declared)`, which writes
`payload_content: self.payload_content.clone()` into the `ArtifactProgramData`
it returns (`crates/tiler-artifact/src/program/builder.rs`). The borrow is
forced by the method's own contract: `build` returns the **intact builder**
inside `ArtifactVerificationError` when verification fails, so nothing may be
moved out of it before the diagnostics are known — and the diagnostics are
derived from the assembled data.

So a producer that builds and then encodes one artifact carrying an `n`-byte
object holds `n` in the builder, `n` in the artifact data, `n` in the projected
section table, and `n + manifest` in the encoder's output buffer. The projection
ticket took the third of those from four copies down to one; this is the second,
and it is untouched because it sits in a different function under a different
constraint.

## Why this is a design question rather than a mechanical fix

Three shapes are available and they are not equivalent:

1. `assemble(&mut self, ...)` taking the content with `std::mem::take`. Cheapest,
   and it silently guts the builder the error path promises to return intact — a
   caller that recovered from a failed `build` would find its payloads gone.
2. Assemble by move and reconstruct the builder on the error path. The builder
   holds `subject`, `expression_types`, and interning state the data does not, so
   this is not a reconstruction that exists today.
3. Move the object bytes only, behind a type that states the builder is spent
   for payloads. A public-boundary change on `ArtifactVerificationError`'s
   recoverability contract.

Tom owns the third; the first is a correctness regression stated as a
performance win.

## Measure it first

The retained spike rows do not cover this: the harness builds its fixture
*outside* the measured window, so
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md)
reports the encode and not the build. A `build` phase has to be added to the
harness before the size of this is a measurement rather than a reading of the
source.

## Closes when

The publication path's build step is measured, the copy is either removed under
a contract that stays true or the retention is recorded with its reason, and
`make full` passes.
