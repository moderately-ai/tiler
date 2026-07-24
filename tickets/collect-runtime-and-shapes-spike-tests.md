---
id: collect-runtime-and-shapes-spike-tests
title: Collect the runtime and shapes spike tests in the repository gate
status: done
priority: p2
dependencies: []
related: [finish-spike-process-group-cleanup]
scopes: [implementation/workspace, research/runtime, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness, gate-reliability]
---
`finish-spike-process-group-cleanup` added process-group cleanup regression tests to four spike areas. Two of them are collected by the gate and two are not, because `spikes/runtime` and `spikes/shapes` are outside the canonical pytest `testpaths`:

- `spikes/runtime/test_measure_semantic_validation.py` — not collected
- `spikes/shapes/shape-evidence/test_shape_evidence_measure.py` — not collected
- `spikes/shapes/nightly-dependent-static-shapes/test_nightly_dependent_measure.py` — not collected

Each passes when run directly. **Inference:** an uncollected regression test protects nothing after the author stops running it by hand, which is the same shape of gap that left the gate flaky after `make-spike-process-group-cleanup-best-effort` stopped at its scope boundary.

The collection contract is deliberately governed in two places that must agree: `testpaths` in `pyproject.toml` and `EXPECTED_PYTEST_PATHS` in `scripts/check_repository.py`, both owned by `implementation/workspace`. Expanding it was left out of the cleanup ticket because it is a contract change in a scope that ticket did not hold, and because of the following unresolved wrinkle.

**Fact:** `spikes/shapes/shape-evidence/.gitignore` and `spikes/shapes/nightly-dependent-static-shapes/.gitignore` both ignore a local `/target/`, so a measured checkout has Cargo build trees under `spikes/shapes/**`. pytest's default `norecursedirs` does not exclude `target`, so adding `spikes/shapes` to `testpaths` makes collection walk those trees. Decide whether to add the two leaf directories, add a `norecursedirs` setting, or both — and mirror whatever is chosen into `check_repository.py`, whose `validate_python_config` requires the two to be exactly equal.

Done when the three tests above run in `uv run --locked python scripts/check_repository.py`, collection does not walk generated Cargo trees, and the gate's runtime cost of the addition is stated.

## Outcome

The three previously uncollected test files are now in the gate. `testpaths` gained the exact leaf directories `spikes/runtime`, `spikes/shapes/shape-evidence`, and `spikes/shapes/nightly-dependent-static-shapes`, and a new `norecursedirs` restates pytest's nine defaults plus `target`. Both are mirrored into `scripts/check_repository.py` as `EXPECTED_PYTEST_PATHS` and a new `EXPECTED_PYTEST_NORECURSEDIRS`, so the two authorities stay exactly equal as `validate_python_config` requires. The gate collects 188 items, up from 179, and all nine new tests appear by node id.

**Fact — the leaf-directory route is not a standalone fix, and this was measured rather than reasoned.** The leaf directory *is* the Cargo workspace root, so `target/` sits inside a path that must be collected. Canary modules planted at `spikes/shapes/nightly-dependent-static-shapes/target/`, `.../target/debug/`, `spikes/shapes/shape-evidence/target/`, and `spikes/extensions/non-exhaustive-visibility/target/` were all collected under pytest's defaults (91 items) and none with the `target` entry (87 items). The pattern is depth-independent, so it also covers the second self-contained Cargo workspace under `spikes/extensions/`, which matters because `compile-extension-spike-fixtures-in-the-gate` proposes adding `spikes/extensions` to `testpaths`; that route is now hazard-free without further work.

**Fact — the `norecursedirs` ini key replaces pytest's default list rather than extending it** (`_pytest/main.py`). Setting only `target` would have silently dropped `.*`, `build`, `dist`, `node_modules`, `venv`, and the rest, so all nine defaults are restated explicitly.

Exact leaf directories were chosen over a `spikes/shapes` parent because a parent path opts future sibling spikes in without a decision, which is the inverse of the gap this ticket closes. `collect_ignore` was unavailable because `--noconftest` is a pinned addopt, and `--ignore` would live in the gate's argv rather than in the declarative contract, so it would not help a contributor running pytest by hand.

**Measurement — the cost is real and is named.** Eight of eight complete gate runs green. The pytest phase moves from a 15.26 s median to 18.05 s (five runs each, zero failures either side), and the warm gate from 25.3–28.6 s to 28.7–30.1 s: about +2.8 s on the pytest phase and roughly +12% on the gate. Six of the nine added tests spawn a subprocess and two deliberately burn a one-second deadline.

**No other uncollected spike tests exist.** `git ls-files | grep -E '(^|/)(test_[^/]*\.py|[^/]*_test\.py)$'` returns exactly fifteen files repo-wide, all now under `testpaths`, and pytest reports 188 items from those fifteen.

`research/runtime` and `research/shapes` were added as scopes after guard flagged them, because the three module docstrings asserted that the gate did not collect them — the exact opposite of the new truth.

A genuine pre-existing defect surfaced during the work and was filed rather than fixed: `resolve-macro-environment-alarm-path-dependence`. `test_overall_alarm_reaps_child_after_capture_pipes_close` spawns a bare `python3` where every sibling uses `sys.executable`, and a pyenv shim's 281 ms startup deterministically loses its 200 ms alarm margin. The gate never sees it because `sanitized_environment` pins `.venv/bin` first, but `uv run --locked pytest spikes/macro-environment` is deterministically red for a contributor on a stock pyenv. That ticket absorbed a duplicate filed independently by `finish-spike-process-group-cleanup`.
