---
id: measure-artifact-decoder-allocation-amplification
title: Measure and reduce artifact-decoder allocation amplification
status: done
priority: p2
dependencies: [bind-stage-coverage-to-index-refinement-identity, bind-the-artifact-variant-abi-to-the-program-abi, carry-the-byte-offset-of-a-partial-binding-view, wire-the-delivered-realization-record-into-the-artifact]
related: [prototype-neutral-artifact-codec, replace-the-codec-arena-content-key-with-the-existing-comparator, stop-copying-the-carried-payload-through-the-envelope-projection]
scopes: [research/artifacts, implementation/artifact]
shared_scopes: [project/tickets, contracts/navigation]
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

## User-visible outcome

Where amplification is material, validate through bounded borrowed views and
take ownership only at the boundary that requires it. Preserve every size
limit, typed rejection, canonical re-encoding, and decoded-lifetime invariant.
Do not trade a copy for an unbounded borrow or unchecked offset.

## Closes when

The report states the exact allocation boundary, the implementation meets an
explicit peak-amplification target or records why it already does, adversarial
inputs remain bounded, and the full gate passes.

## Outcome

Measured first, then reduced. The instrument is
[`spikes/artifacts/decoder-allocation/`](../spikes/artifacts/decoder-allocation/README.md)
— a counting `GlobalAlloc` driving the public `decode_artifact` over real
envelopes — and the report is
[`docs/research/artifacts/decoder-allocation-amplification.md`](../docs/research/artifacts/decoder-allocation-amplification.md).
Allocation counts are properties of the program rather than of the host, so
there is no variance to report: the harness measures every call twice after a
warm-up and asserts the two readings are identical.

**The exact allocation boundary.** *Peak live during a decode is the decoded
artifact's own footprint plus a term that does not depend on how many section
bytes the envelope carries.* Measured, that term is **220,531 bytes at 1 MiB,
16 MiB, and 64 MiB of carried object — identical to the byte across a 64-fold
change in section size.* Before this ticket the same difference read 1,383,121 /
17,111,761 / 67,443,409, one whole extra envelope every time.

**The peak-amplification target, met.** `decode_artifact` peak live fell from
2.00x the envelope to **1.00x** at a 64 MiB object (134,614,747 to 67,391,869
bytes), 2.01x to 1.01x at 16 MiB, 2.15x to 1.15x at 1 MiB, and 4.03x to 2.48x
for the manifest-only envelope. Total bytes requested by a 64 MiB decode fell
135,091,020 to 67,582,130.

**What changed, and nothing else did.** The canonicity backstop re-encoded the
validated envelope into a second buffer and compared the two; it now derives the
same encoding and compares it against the bytes run by run
(`encode::matches_canonical_encoding`, sharing one `encode_head` and one
`push_section_framing` with `encode_with_identity`, so the two cannot drift).
Every size limit, typed rejection, canonical-form guarantee and decoded-lifetime
invariant is unchanged, and the envelope budget is now checked *before* the
encoding is assembled rather than after. `validate`'s two ordering checks stopped
copying what they compare — measured at exactly two allocations per variant and
two per precondition-bearing entry, confirmed by the call-count deltas 263 -> 261
and 1,843 -> 1,839. The bytes those copies held are stage subjects and content
keys, small at this fixture and manifest-sized at the governed bound.

**The refusal was watched failing before it was trusted.**
`the_canonicity_backstop_refuses_every_run_it_walks` perturbs one byte in each of
the four runs the comparison walks plus both length disagreements; each of the
four comparisons was neutered in turn and watched failing at its own offset — 0,
69, 74343, 97077.

**Adversarial inputs remain bounded.** Seven malformed inputs per shape, each
asserted to be a refusal and reported with its own verdict. The five that carry a
forged length or count are refused in 8-76 bytes at every envelope size, which is
the observation of `budget.rs`'s claim that a forged length reserves nothing. The
two that reach the deepest paths — a flipped object byte, and a flipped identity
byte with the manifest digest repaired so it passes framing, integrity, canonical
form and the whole of `validate` — peak at 1.00-1.56x the envelope, at or below a
valid decode.

**Two amplifications were found that this ticket could not take.** Both are
filed with their measured tables rather than absorbed:

- [`replace-the-codec-arena-content-key-with-the-existing-comparator`](replace-the-codec-arena-content-key-with-the-existing-comparator.md)
  (p1). A **226,214-byte** envelope carrying a 4,000-node ABI expression chain
  makes a decode allocate **1,569,620,906 bytes**, 6,939x the envelope, and a
  forged envelope that is *rejected* allocates the identical peak. The growth is
  quadratic in arena depth, pinned at each of five doublings, with the last point
  at `MAX_ABI_EXPRESSIONS` rather than extrapolated to. The cause is
  `expr_key`'s nested subtree framing; the fix is
  `tiler_ir::program::abi::compare_expr_nodes`, which is public, already used by
  the identity encoder, and documented with this exact rationale. It is filed
  rather than done because switching changes which byte string is *the* canonical
  encoding: a `MANIFEST_SCHEMA` major step and every pinned envelope byte in the
  workspace, across scopes this ticket does not hold.
- [`stop-copying-the-carried-payload-through-the-envelope-projection`](stop-copying-the-carried-payload-through-the-envelope-projection.md)
  (p2). `VerifiedArtifactProgram::encode` peaks at **4.99x** the envelope,
  because `ArtifactEnvelope::project` copies each carried object four times
  before the encoder writes it a fifth. That is the producer path, not the
  decoder this ticket names.

**No contract text changed, and that is a finding rather than an omission.**
`docs/artifact-abi.md`'s statement that the decoder "re-encodes the validated
envelope and requires byte equality" describes the guarantee, which is exactly
what still holds; only the buffer it used to need is gone.

`contracts/navigation` was added as a shared scope for two additive catalog
lines — the research note in `docs/research/README.md` and the spike in
`spikes/README.md` — which the corpus requires to be updated with the metadata
they describe.
