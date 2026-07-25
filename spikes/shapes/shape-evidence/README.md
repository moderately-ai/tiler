---
schema: "tiler-doc/v1"
id: "tiler.spike.shapes.shape-evidence"
kind: "experiment"
title: "Stable-Rust shape-evidence feasibility spike"
topics: ["shapes", "rust", "semantics", "diagnostics"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.shapes.stable-rust-shape-evidence", "tiler.research.shapes.public-static-shape-spelling"]
entrypoints: ["spikes/shapes/shape-evidence/src/lib.rs", "spikes/shapes/shape-evidence/measure.py", "spikes/shapes/shape-evidence/measure.sh", "spikes/shapes/shape-evidence/measure-spellings.sh", "spikes/shapes/shape-evidence/verify_evidence.py"]
last_verified: "2026-07-21"
ticket: "prototype-shape-evidence-spike"
---

# Stable-Rust shape-evidence feasibility spike

This isolated Rust 1.89 crate tests optional shape refinements over graph-owned
typed values. The graph remains authoritative: only it constructs refined
handles after checking canonical metadata, and all evidence-preserving
operations delegate to the unrefined admission path before rechecking results.

The model covers `Rank<R>`, `Exact<S>`, explicit weakening, pointwise
propagation, statically checked reduction axes, graph-owned same-shape
witnesses, foreign-proof rejection, evidence-neutral canonical identity, and
downstream static-shape descriptions which grant no authority by themselves.
Trybuild cases retain the Rust 1.89 diagnostics for rank/exact mismatches,
invalid or duplicate axes, sealed-evidence implementation, and handle forgery.

Run the correctness and diagnostic suite:

```sh
cargo +1.89.0 test --manifest-path spikes/shapes/shape-evidence/Cargo.toml
cargo +1.89.0 clippy --manifest-path spikes/shapes/shape-evidence/Cargo.toml --all-targets -- -D warnings
```

## Custody of the retained diagnostics

This is the repository's sole off-pin spike, and that posture decides how its evidence is checked. `AGENTS.md` compiles a spike Cargo workspace in the Rust gate exactly when it retains a `trybuild` `.stderr` captured on the toolchain `rust-toolchain.toml` pins. These six were captured on stable 1.89.0, which is not that pin, so `scripts/check_rust.py` names this directory in `OFF_PIN_SPIKE_WORKSPACES` and never compiles it: re-deriving them needs a compiler the gate has no authority to install, and re-recording them on the nightly pin would destroy the stable-Rust claim the spike exists to make.

Excluding a workspace from *reproduction* is not the same as leaving its evidence unchecked. [`results/2026-07-24-macos-arm64.json`](results/2026-07-24-macos-arm64.json) is the other half. It records, for each compile-fail case, the exact toolchain, the expected first line, the ordered sequence of diagnostic codes the file emits, the fragments that must appear, and — where two cases share an error code — the fragments that must not; for each compile-pass case it records that no diagnostic may be retained beside it. It also digests every input that determines what those diagnostics say: the manifest, the lockfile, `src/lib.rs`, `tests/ui.rs`, and each fixture. [`verify_evidence.py`](verify_evidence.py) checks all of that without invoking Cargo, and `test_shape_evidence_record.py` runs it inside the repository gate through the canonical pytest `testpaths` entry, alongside twenty-four cases that each corrupt a copy of the spike and require the exact refusal that corruption should produce.

Two rules follow from being off-pin rather than gated, and both differ from the equivalent for `spikes/extensions/non-exhaustive-visibility`.

The channel comparison is inverted. That probe requires its record to name the repository pin, because the gate recompiles its fixtures and a moved pin must force a fresh run. Here the recorded channel must *not* be the pin, and must equal the selector `measure.py` executes with, the release series `Cargo.toml` declares, and the one this README documents. Evidence that has drifted onto the pin has been re-recorded into meaninglessness, so it fails rather than passing quietly.

The input digests are load-bearing, not belt and braces. A compiled spike gets its total check from compilation; nothing ever compiles this one, so a diagnostic that quotes `src/lib.rs` would otherwise sit unchanged on disk after the code beneath it moved. Editing any recorded input therefore fails the gate until the suite is re-run on 1.89.0 and the record is refreshed in the same commit — which is the honest cost of a claim about a compiler the gate cannot run.

Refresh a diagnostic with `TRYBUILD=overwrite` **only** after deciding the claim still holds on 1.89.0, then re-run the suite above and update the record's claims and digests in the same commit. Do not add a `rust-toolchain.toml` here: the repository pin is the sole toolchain authority for everything the gate compiles, and a directory-local file would silently select another compiler for this evidence.

Check the retained evidence on its own:

```sh
uv run --locked python spikes/shapes/shape-evidence/verify_evidence.py
uv run --locked pytest spikes/shapes/shape-evidence/test_shape_evidence_record.py
```

Regenerate the 1/10/100/1,000-shape workloads and repeat the bounded host
measurement:

```sh
spikes/shapes/shape-evidence/measure.sh
```

Raw run products are ignored. The compact checked-in result is
[`measurements/summary.json`](measurements/summary.json). The command derives
that file directly from its fresh subprocess results and retains each raw
stdout/stderr stream under the ignored `measurements/raw/` directory. One
sample per case is sufficient to reject catastrophic scaling, not to estimate
a production compile-time cost distribution.

Compare downstream descriptors, library-owned arity families, and dimension
tuples with five isolated samples per case:

```sh
spikes/shapes/shape-evidence/measure-spellings.sh
```

The checked-in summary retains all five samples, derives its medians, and
records the exact host/toolchain in
[`measurements/spelling-summary.json`](measurements/spelling-summary.json).
Both entrypoints impose a 300-second overall process-group deadline on every
subprocess. They support macOS with BSD `/usr/bin/time -lp` and Linux with GNU
`/usr/bin/time -v`; other platforms fail closed. Binary sizes come from
Python's portable `Path.stat()` rather than incompatible `stat` command flags.
