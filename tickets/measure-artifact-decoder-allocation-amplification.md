---
id: measure-artifact-decoder-allocation-amplification
title: Measure and reduce artifact-decoder allocation amplification
status: todo
priority: p2
dependencies: [bind-stage-coverage-to-index-refinement-identity, bind-the-artifact-variant-abi-to-the-program-abi, carry-the-byte-offset-of-a-partial-binding-view, carry-the-data-flow-of-a-stage-dependency, carry-the-honourability-fact-provenance-into-the-artifact-record]
related: [prototype-neutral-artifact-codec]
scopes: [research/artifacts, implementation/artifact]
shared_scopes: []
paths: []
tags: [artifact, codec, measurement, performance]
---
Bound the peak memory required to validate and expose a large artifact envelope,
then remove allocation amplification that the measurement demonstrates.

The current decoder clones manifest and payload data at several ownership
transitions. Static inspection cannot establish whether those copies are
material relative to the envelope's configured limits, and the wire model is
still changing under the listed prerequisites.

## Measurement

After the current envelope-shaping wave lands, construct valid envelopes at
representative and maximum admitted section/manifest sizes. Record exact input
shape, toolchain/profile, peak live bytes, total allocated bytes, and allocation
sites for decode, validation, and view construction. Include malformed inputs
that reach the deepest bounded validation paths.

## Outcome

Where amplification is material, validate through bounded borrowed views and
take ownership only at the boundary that requires it. Preserve every size
limit, typed rejection, canonical re-encoding, and decoded-lifetime invariant.
Do not trade a copy for an unbounded borrow or unchecked offset.

## Closes when

The report states the exact allocation boundary, the implementation meets an
explicit peak-amplification target or records why it already does, adversarial
inputs remain bounded, and the full gate passes.
