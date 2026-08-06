---
id: correct-the-numerical-contract-spelling-outside-the-restored-spike-scopes
title: Correct the composed numerical contract's spelling in ADR 0011 and the apple-targets probe quotation
status: in-progress
priority: p3
dependencies: []
related: [restore-the-spikes-against-the-composed-numerical-contract]
scopes: [contracts/decisions, research/apple-targets]
shared_scopes: []
paths: []
tags: [maintenance, numerics, docs]
claimed_from: todo
assignee: agent-contract-spelling
lease_expires_at: 1786051648
---
## User-visible outcome

No retained record outside the spikes describes the composed `NumericalContract` as the preset enumeration it replaced, so a reader who greps for a named contract finds the spelling the compiler actually accepts.

## Why this exists

**Fact.** `restore-the-spikes-against-the-composed-numerical-contract` repaired every *code* site and the six research-record sites its own enumeration named, and confirmed the code is clean: `grep -rn 'NumericalContract::[A-Z][a-z]' --include='*.rs' spikes docs crates prototypes` at `d5960e81` returns exactly one line, `IncoherentNumericalContract::UnfoundedValueDomainProvenance` in `crates/tiler-compiler/src/session.rs:1808`, which is a different type matched only because the pattern is unanchored.

That ticket's stated closing check was the same grep *without* `--include`, reporting no match at all. It cannot: the residue below is prose, and two of the three sites are outside the six research scopes that ticket holds.

**Fact — the sharp one, and the reason this is filed rather than noted.** [`docs/decisions/0011-per-operation-numerical-permissions.md`](../docs/decisions/0011-per-operation-numerical-permissions.md) line 78 describes the *current* implementation in the present tense and is now wrong on three counts:

> `crates/tiler-compiler/src/session.rs` exposes a `NumericalContract` enum with two named user-facing modes, `StrictF32` and `FlushSubnormalsToZeroF32`, which `resolve` maps to `StrictF32NumericalContract::governed` and `governed_flush_to_zero` in `crates/tiler-compiler/src/request.rs`.

`NumericalContract` is a `struct` (`session.rs:1332`), not an enum; its named points are associated constants (`STRICT_F32`, `FLUSH_SUBNORMALS_TO_ZERO_F32`, `RELAXED_F32`, `REASSOCIATE_F32`, `FLUSH_AND_REASSOCIATE_F32`, `STRICT_BF16`, `FLUSH_SUBNORMALS_TO_ZERO_BF16`), so "two named user-facing modes" is wrong in kind and in count; and a caller composes arbitrary points through `NumericalContractBuilder`, which the sentence does not mention. This is an **accepted ADR** whose realization note is read as fact, and it sits in `contracts/decisions`, which the restoration ticket did not hold.

The ADR's *decision* is untouched — ADR 0011 holds that one permission never implies another, and the composed record strengthens that rather than superseding it. Only the "Realized" paragraph describing today's code needs correcting, so this is a contract-sentence sweep and not a supersession.

**Fact — the second site, a stale quotation.** `spikes/apple-targets/code-domain-integer-decode/test_decode_probe.py:122` quotes `docs/research/numerics/first-quantized-lm-profile.md` verbatim in a docstring, including the old bare `FlushSubnormalsToZeroF32`. The record it quotes was corrected at `d5960e81`, so the quotation no longer matches its source. `research/apple-targets` was not among the restoring ticket's scopes. The claim the test makes is unaffected — it recomputes the exhaustive-finite result over all 65,536 cells rather than citing it — so this is a quotation-fidelity repair, not a numerical one.

**Inference — one site is deliberately *not* in scope for this ticket.** `spikes/numerics/bf16-second-dtype/README.md:91` spells `NumericalContract::{StrictF32, …}` and says "Four presets". That row is a **measured survey of the surface at `59a2fe2`**, scoped by the spike's own `verified_at_commit`, and its prediction — "A BF16 contract is a fifth key, not a widened fourth" — was *borne out* by the BF16 contract work that has since landed. Editing a measured snapshot to match a later tree would falsify what the survey recorded. What that spike wants is a re-run against the current tree, which is `re-run-the-bf16-second-dtype-spike-against-the-landed-bf16-contract` work rather than a spelling sweep; the three prose paragraphs under `spikes/` that quote the old spelling to *explain* the migration are correct as they stand for the same reason.

## Closes when

ADR 0011's realization paragraph describes the composed `NumericalContract` struct, its associated constants, and `NumericalContractBuilder` as they are, with the ADR's decision and rationale unchanged and its `decision_status` untouched; the `test_decode_probe.py` docstring quotes its source record's current text; and `grep -rn 'NumericalContract::[A-Z][a-z]' docs/decisions spikes/apple-targets` reports no match.

## Graph maintenance

Do not sweep the three migration-explaining paragraphs under `spikes/` or the `bf16-second-dtype` survey row — the reasoning above is why each is correct as it stands. If a later reader wants the survey row refreshed, that is a re-run of that spike and needs its own ticket.
