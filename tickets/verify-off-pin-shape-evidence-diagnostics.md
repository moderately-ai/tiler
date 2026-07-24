---
id: verify-off-pin-shape-evidence-diagnostics
title: Verify the off-pin shape-evidence diagnostics against a record
status: in-progress
priority: p2
dependencies: []
related: [compile-extension-spike-fixtures-in-the-gate]
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [testing, gate-reliability, harness]
claimed_from: todo
assignee: agent-verify-off-pin-shape-evidence-diagnostics
lease_expires_at: 1784929885
---
`compile-extension-spike-fixtures-in-the-gate` decided the rule for spike Cargo workspaces that retain compiler-produced golden artifacts: compile them in the Rust sub-gate when the artifact was captured on the toolchain `rust-toolchain.toml` pins, and record an explicit off-pin exclusion when it was not. `spikes/shapes/shape-evidence` is the sole off-pin case — `measure.py:30` sets `TOOLCHAIN = "1.89.0"`, `Cargo.toml` declares `rust-version = "1.89"`, and its README documents `cargo +1.89.0 test` — so its six retained `.stderr` files are correctly not reproduced by the gate. Reproducing them would need a toolchain `AGENTS.md` forbids installing without Tom's authorization, and re-recording them on the pin would destroy the stable-Rust claim the spike exists to make.

**The gap that leaves.** Excluding a workspace from *reproduction* is not the same as leaving its evidence unchecked, and shape-evidence currently has neither. `spikes/extensions/non-exhaustive-visibility` has both halves: the gate compiles it, and `spikes/extensions/run.py --self-test` verifies each retained diagnostic against a JSON record naming the exact toolchain, the expected first line, the diagnostic code, the fragments that must appear, and the fragments that must not. `spikes/shapes/shape-evidence` has no record at all. Its gated pytest module, `test_shape_evidence_measure.py`, tests the measurement harness's deadline and process-group cleanup paths and reads none of the six diagnostics. `spikes/shapes/nightly-dependent-static-shapes` has the reproduction half but likewise no record; it needs the answer to a narrower question, since compiling it already compares fixture to diagnostic.

**Fact — verified by reading, at `89b2eed`.** `find spikes/shapes/shape-evidence -name '*.stderr' -not -path '*/target/*'` returns six files; no JSON record accompanies them; `grep -rn 'stderr' spikes/shapes/shape-evidence/test_shape_evidence_measure.py` returns nothing. So a `TRYBUILD=overwrite` refresh, a hand edit, or a deleted case under that directory changes checked-in evidence with nothing in the gate reporting it — the same decay `preserve-non-exhaustive-visibility-probe` closed for the extension probe.

**Scope of the work.** Give the off-pin spike the record half of the pattern the visibility probe established: a `results/*.json` capturing the exact stable toolchain each diagnostic was taken on, a verifier that checks first lines, codes, required and forbidden fragments, and fixture-set equality in both directions, and a gate entry that runs it without Cargo. `spikes/shapes/shape-evidence` is already in `testpaths`, so the gate entry costs no `pyproject.toml` change. Reuse `spikes/extensions/run.py`'s `verify_visibility_evidence` shape rather than inventing a second one; note that its channel comparison is against `rust-toolchain.toml`, which is exactly wrong for an off-pin spike, so the off-pin form must compare against the toolchain the record itself names and must not silently accept the repository pin.

**Also decide, and record either way:** whether `spikes/shapes/nightly-dependent-static-shapes` needs a record too. Compiling it already proves fixture and diagnostic agree, so a record would add attribution and a fixture inventory rather than reproduction. That is a smaller gap than the off-pin one and may be worth closing in the same pass or explicitly declining.

Do not weaken `scripts/check_rust.py`'s custody predicate or move `shape-evidence` into `GATED_SPIKE_WORKSPACES` to make this easier; installing a second Rust toolchain for the gate needs Tom's authorization and is a different decision.
