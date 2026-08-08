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

## Outcome

Done 2026-08-08 on base `209013bd`. All five sentences corrected; no convention, no decision, and no measured value changed.

**Per-Fact audit — the filing's Facts re-verified at this base.** The shared ground holds exactly as stated: `ls scripts` reports no such file, `e197176f2214d989579f505f8e78637aeb343e3d` is dated 2026-07-26 with the subject the ticket quotes, and `grep -n spike Makefile` returns line 7 and nothing else. The whole `Makefile` was read rather than grepped: its seven targets are `check citations fmt build lint test doc full` and no recipe names `spikes/` at all. The two named mechanisms are gone with the directory — `scripts/tests/test_research_harnesses.py` invoked the self-test and `scripts/check_rust.py` carried the compile phase, per this repository's own [`compile-extension-spike-fixtures-in-the-gate`](compile-extension-spike-fixtures-in-the-gate.md) outcome. One count in the filing is imprecise and load-bearing on nothing: `grep -n e197176 docs/decisions/[0-9]*.md` returns seventeen passages across ADRs 0075, 0076, 0077, 0079, 0081, 0082, and 0088, not eight across five — the filing omits 0075, which carries two. The house form it points at is correct and abundant.

**The conclusion-versus-ground split, per corrected sentence.**

- ADR 0074, evidence paragraph, "which the repository gate runs": the *conclusion* — the self-test checks retained diagnostics against the record without invoking Cargo — is true and unchanged (`spikes/extensions/run.py "def verify_visibility_evidence(root: Path, channel: str)"`, reached from `--self-test`). Only the relative clause naming a gate was false. Corrected by retiring the clause, not the sentence.
- ADR 0074, "the gate now also compiles the workspace ... on every invocation": conclusion and ground both false, because the conclusion *is* a claim about the gate. Retired whole, verbatim, with its accurate-when-written dating preserved.
- ADR 0074, convention 5b Measurement, "forces a fresh run at the next pin migration": imprecise, not false, exactly as filed. The comparison exists and is intact — `verify_visibility_evidence` refuses an off-pin record and `spikes/extensions/run.py "def visibility_self_test()"` proves that rejection fires by calling it with a moved pin. What is false is *forces*. The measurement is restated at its true strength: it survives because `rust-toolchain.toml "nightly-2026-07-19"` still pins the channel `spikes/extensions/non-exhaustive-visibility/results/2026-07-24-macos-arm64.json "nightly-2026-07-19"` records it against — survival by an unmoved pin, not by a mechanism.
- ADR 0076, `Measured evidence`, "reproduced by the harness the gate runs": coverage conclusion true (finding 20 closed the additive-path gap); custody clause false, and it contradicted the 2026-07-26 correction opening the same bullet. Custody clause alone retired.
- ADR 0076, `Measurement — environment`, "through a checked-in harness the repository gate runs": checked-in and reproducing both true; the gate clause false. Gate clause alone retired.

**Method.** Every retired extent is preserved verbatim inside its dated correction, quoted in prose rather than pinned to a path, so the record keeps the text and the checker cannot demand it resolve. ADR 0074's `**Status:**` line gained one sentence, because it already narrates this record's correction history and a reader had something to un-learn; ADR 0076's did not, matching how its own 2026-07-26 correction of this same class stayed local. `docs/decisions/README.md` needs no edit: its two rows carry status, contracts, and evidence links, none of which move.

**Sibling scan.** `grep -n "gate runs\|the gate now\|gate collects\|repository gate" docs/decisions/*.md` and `grep -n "spikes/" docs/decisions/[0-9]*.md` were read site by site. Three other hits are correct and were left alone: ADR 0077's `crates/tiler-metal/src/golden_compilation.rs` genuinely is compiled by `make test`; ADR 0074's `unknown_lints` sentence is about a workspace crate under `-D warnings`, not a spike; and ADR 0106 already says a spike is run by hand. No further in-scope defect of this class exists.

**Checks.** `make citations` green at 964 citations, up 8 from the 956 baseline — exactly the eight anchors added, so every new anchor is reached and counted. Anchor reach demonstrated deliberately: breaking `"def visibility_self_test()"` to `"def visibility_self_test_XXBREAKXX()"` produced `FAIL ... anchor occurs nowhere in spikes/extensions/run.py` and exit 2; reverting restored green. The checker also caught a real defect mid-run — the retired 0076 quote was one trailing period short of byte-exact, which broke this ticket's own citation of it — and that was repaired rather than worked around.
