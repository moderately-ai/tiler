---
id: correct-the-numerical-contract-spelling-outside-the-restored-spike-scopes
title: Correct the composed numerical contract's spelling in ADR 0011 and the apple-targets probe quotation
status: review
priority: p3
dependencies: []
related: [restore-the-spikes-against-the-composed-numerical-contract]
scopes: [contracts/decisions, research/apple-targets]
shared_scopes: [project/tickets]
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

`project/tickets` was added as a shared scope so this Outcome could be written; the two editing scopes are unchanged.

## Outcome

**2026-08-06.** Both filed sites corrected at base `6cc4c242`. The diff touches `docs/decisions/`, `spikes/apple-targets/`, and `tickets/` and nothing else — no crate, prototype, manifest, or configuration file — so no repository gate is reachable from it.

**Fact — the realization paragraph was rewritten from the source, not from this ticket's enumeration.** `crates/tiler-compiler/src/session.rs:1331-1894` and `crates/tiler-compiler/src/request.rs:233-525` were read in full at base. The ticket's list of seven associated constants is exact and unchanged, but the sentence had drifted on two counts the body did not name, both found by reading:

- `governed_profile` no longer exists anywhere in `crates/` — `grep -rn 'governed_profile' crates/` returns only `governed_profile_source`, an unrelated target-fact provenance helper. Admission is `StrictF32NumericalContract::is_governed` (`request.rs:421`), and it is deliberately *not* set membership: its own documentation records that "membership in a table of four is deliberately not among them: that test is what made an unnamed corner unreachable". The successor of the named set, `named_profile` (`request.rs:391`), is `#[cfg(test)]` and is documented as "documentation and test population, no longer an admission authority". So "a contract outside the registered set is rejected" was wrong in kind, not only in spelling.
- The dependency direction is inverted from what the sentence claimed. `NumericalContract::resolve` (`session.rs:1624`) composes the internal record field by field and ends at `keyed()`; it is `request.rs`'s four `#[cfg(test)]` named constructors — `governed_flush_to_zero`, `governed_relaxed`, `governed_reassociating`, `governed_flush_and_reassociate` — that resolve *through* the public constants, which `session.rs:1620` states is why a named vector is spelled once.

The three call sites the old sentence named do survive, and each was read: `verify_request` (`request.rs:3500`, the request boundary), `VerifiedRequest::for_target` (`request.rs:3061`, the per-target verification), and `verify_schedule_with_feasibility` (`physical.rs:2183`, the physical schedule verifier). The struct's eleven dimensions were counted against the fields of `NumericalContract` (`session.rs:1408-1418`) and cross-checked against the exhaustive `behaviour` match over `NumericalDimension` (`request.rs:437-465`), which projects exactly those eleven and deliberately not `key`, `arithmetic`, or `canonical_arithmetic_nan_bits`.

The rewritten paragraph is `docs/decisions/0011-per-operation-numerical-permissions.md:78`, followed at `:80` by a dated correction marker in the ADR 0090 house style — bold `**Corrected <date> by [ticket](…):**`, quoting the superseded wording, naming each way it drifted, and stating what the item still claims. The Decision, Consequences, Alternatives, both `Unrealized` items, the final `Consequently unrealized` item, and the whole frontmatter including `decision_status: accepted` and `implementation_status: partial` are byte-identical to base; `git diff --stat` on the file is `4 +++-`, two lines removed and four added.

**Fact — the quotation was repaired against its source's current text.** `spikes/apple-targets/code-domain-integer-decode/test_decode_probe.py:121-125` now reads:

> "if the scale is a normal F32, the decode is bit-identical under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` and under a subnormal-preserving F32" is the derivation the profile record states over 256 codes and 256 zero points. Here it is evaluated over all 65,536 cells for every normal scale in the corpus.

The source is `docs/research/numerics/first-quantized-lm-profile.md:134`, whose current sentence is "**Inference — exhaustive over the finite code domain: if the scale is a normal F32, the decode is bit-identical under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` and under a subnormal-preserving F32.**" — corrected there by `restore-the-spikes-against-the-composed-numerical-contract` at `d5960e81`. Two characters beyond the identifier changed, and deliberately: the quoted fragment begins mid-sentence in the source, so the docstring's sentence-initial `If` became the source's `if`. Quoting a fragment verbatim was the standard this ticket asked for, and the capitalization was the remaining infidelity; the surrounding docstring sentence is unchanged and reads the same with a lower-case opening.

**Measurement — the spike still runs, device-free.** `uv run --with pytest pytest spikes/apple-targets -q` from the worktree root: **165 passed, 1 skipped in 68.24s**. The skip is the device-dispatch case, which needs the Metal toolchain. This is evidence the file still imports and collects; the docstring is not read by any assertion, so it could not have changed a verdict.

**Fact — the closing grep, and why it is a weak check.** `grep -rn 'NumericalContract::[A-Z][a-z]' docs/decisions spikes/apple-targets` reports **no match** at completion. It also reported no match at base: both filed sites spelled the stale name *bare* — `FlushSubnormalsToZeroF32`, not `NumericalContract::FlushSubnormalsToZeroF32` — so the qualified pattern never matched either of them. The check as filed is a regression guard against the qualified form, not a discriminator for the work; what verified this ticket was reading the two sites and their sources. The discriminating grep is `grep -rn 'FlushSubnormalsToZeroF32' docs/decisions spikes/apple-targets`, which returned two lines at base — ADR 0011:78 and `test_decode_probe.py:122` — and returns one at completion: ADR 0011:80, the correction marker quoting the wording it supersedes, which is the same convention the ADR 0090 markers use.

**Fact — the excluded sites were left alone.** `spikes/numerics/bf16-second-dtype/README.md` and the three migration-explaining paragraphs under `spikes/` are untouched; `git status --porcelain` lists exactly three modified files and no others.

**Checks.** `git diff --check` clean; `tkt lint` clean; `tkt guard` reports only the declared scopes.
