---
id: collect-runtime-and-shapes-spike-tests
title: Collect the runtime and shapes spike tests in the repository gate
status: todo
priority: p2
dependencies: []
related: [finish-spike-process-group-cleanup]
scopes: [implementation/workspace]
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
