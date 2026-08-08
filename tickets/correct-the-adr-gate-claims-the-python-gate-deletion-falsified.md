---
id: correct-the-adr-gate-claims-the-python-gate-deletion-falsified
title: Retire the deleted Python gate's claims in ADRs 0074 and 0076
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, evidence, gate]
claimed_from: todo
assignee: w-correct-t
lease_expires_at: 1786165341
---
## Two accepted ADRs assert that the repository gate runs a spike harness, and no gate reaches `spikes/`

Verified 2026-08-07 at base `7c371155` by `verify-and-file-the-remaining-maturity-audit-leads`. Both ADRs are in `contracts/decisions`, share one root cause, and are filed together so a second worker does not re-derive the same gate fact against the same exclusive scope.

**Fact — the shared ground, and it is checkable in one read.** `e197176` replaced the Python gate with the root `Makefile`, and the `Makefile` header states the consequence in terms: `Makefile "Spikes deliberately have no target."`. There is no `scripts/` directory in the tree, so `scripts/check_rust.py` and `scripts/tests/test_research_harnesses.py` — the two mechanisms both ADRs' claims rest on — do not exist. Reproduce:

```sh
ls scripts                                  # No such file or directory
grep -n spike Makefile                      # the header comment, and nothing else
git log --oneline --all -- scripts | head -1  # e197176f Replace the Python gate with a Makefile of cargo commands
```

`spikes/extensions/README.md "Nothing runs either half automatically"` already records the corrected state, and eight passages across ADRs 0077, 0079, 0081, 0082, and 0088 already carry dated `e197176` corrections. These two records were missed by that sweep. This is the duplicated-authority failure the documentation contract names: a worker reading the spike README gets the right answer and a worker reading the accepted ADR gets the wrong one.

## ADR 0074 — two false sentences in one Evidence paragraph, and one overclaimed Measurement

The Evidence paragraph under the 2026-07-24 convention 5 amendment carries both:

- `docs/decisions/0074-use-explicit-public-api-conventions.md "which the repository gate runs, checks the retained diagnostics against that record without invoking Cargo"` — **false**. Nothing invokes `spikes/extensions/run.py --self-test`.
- `docs/decisions/0074-use-explicit-public-api-conventions.md "the gate now also compiles the workspace, so the fixtures are re-derived under the pinned toolchain on every invocation"` — **false**. No target compiles the spike workspace. The `Updated 2026-07-24 by compile-extension-spike-fixtures-in-the-gate` refresh that added this sentence was accurate when written and was falsified by `e197176` two days later.

**The conclusion versus its ground, and this is the part to get right.** Convention 5b's Measurement closes `docs/decisions/0074-use-explicit-public-api-conventions.md "the record's fail-closed channel comparison forces a fresh run at the next pin migration"`. **That sentence is imprecise rather than false, and the distinction decides the repair.** The comparison is real and intact — `verify_visibility_evidence` in `spikes/extensions/run.py` raises when the pinned channel is not among the recorded ones, and a self-test calls it with a moved pin and fails if that succeeds. What is false is the word *forces*: an intact mechanism nothing invokes compels nothing. The measurement therefore survives on a weaker ground than it states — it holds because the toolchain pin has not moved, not because anything would catch it if it did.

**Do not retire the measurement.** Its values were reproduced and are retained byte for byte under `spikes/extensions/non-exhaustive-visibility/`. What is wrong is the custody claim around them, not the numbers.

## ADR 0076 — a bullet that contradicts its own dated correction

The `Measured evidence` bullet opens with a correction and then closes by asserting what the correction denies, in the same bullet:

- The correction: `docs/decisions/0076-declare-target-honourable-numerical-realizations.md "they are no longer collected by anything"`, which names `e197176` and states that a toolchain change altering a measured value now invalidates the record silently.
- The surviving sentence, later in that same bullet: `docs/decisions/0076-declare-target-honourable-numerical-realizations.md "Every re-verified observation in this record is now reproduced by the harness the gate runs."`
- A second passage, in the `Measurement — environment` block: `docs/decisions/0076-declare-target-honourable-numerical-realizations.md "reproduces this same row through a checked-in harness the repository gate runs"`.

**The conclusion survives and the stale ground is the two sentences.** The 2026-07-26 correction is right; the two sentences predate it and were left standing. The additive-path observation the second sentence is about *was* genuinely re-established by `extend-the-numerical-probe-to-an-additive-path-kernel` — finding 20 of the Apple record owns the result — so the repair is to the custody clause alone. Rewriting it as "reproduced by nothing" would overshoot in the other direction: the harness and its assertions are intact and are re-run by hand with `uv run --with pytest pytest spikes/apple-targets`.

## Requirements

- Correct all five sentences above, in the house form the sibling ADRs already use: a dated correction that preserves the retired text rather than deleting it, names `e197176`, and states what the loss of the gate costs the record.
- **Do not rewrite the retained measurements.** Every value in both records was measured and is retained; only the claim about what re-checks them is wrong.
- State ADR 0074's convention 5b Measurement at its true strength — the comparison exists, nothing invokes it, so the bounded claim rests on the pin not having moved.
- A dated correction must quote a retired extent in prose or as a bare `:LINE` suffix, never pinned to a path, or `make citations` will demand it resolve.

## Out of scope, and separately filed

The same falsified claim appears across twenty-two passages of `docs/research/apple-targets/numerical-behaviour.md`, which is `research/apple-targets` and not this scope. Filed as `retire-the-gate-reproduction-claims-in-the-apple-numerical-record`. ADR 0076's `Measured evidence` bullet cites that record as its authority, so the two should be read together, but neither blocks the other.

## Closes when

Neither ADR asserts that any gate runs, collects, or compiles anything under `spikes/`; each correction is dated and names `e197176`; the retained measurements are unchanged; convention 5b's Measurement states its true strength; and `make citations` is green.
