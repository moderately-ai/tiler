---
id: pin-the-serial-sum-producer-runner-shape-interface
title: Pin the serial-sum producer/runner shape interface the way its filenames are pinned
status: done
priority: p2
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, prototype-metal-runtime-proof]
scopes: [implementation/runtime, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, metal, prototype, testing]
---
## User-visible outcome

A drift between what `prototypes/serial-sum-compile` publishes and what `prototypes/serial-sum-run` expects fails in the repository gate, rather than only when someone runs the hardware proof by hand.

## Why this is worth a ticket

**Fact — it already happened, and stayed hidden for a month.** `prove_member` compiled `serial_sum_program(ROWS, columns)` from the *runner's* own `ROWS = 4` and routed it against artifacts the producer publishes with `ROWS = 1`. Every packaged program was therefore foreign, and the whole matrix pass — six members, thirty operand cases — could not prove a single one. It was introduced when 0b7e59d (2026-07-30, `Defer Metal workgroup limits to prepared pipelines`) moved the producer to one row, three days after 1f4b7fc added the matrix pass. `construct-and-bind-the-first-authoritative-metal-compile-profile` found it by running the proof and fixed it: `prove_member` now reads the shape from the artifact, as the deep proof already did.

**Fact — the same class of drift is already defended against, for filenames only.** The two crates share no code and no Cargo edge, so the member names and the `.proof` suffix are each pinned by a test *in both crates* that names the other side, precisely because a rename once broke the slice end to end for a whole commit under a green gate. The shape has no such pin.

**Inference — reading the shape from the artifact is the right fix and is not the whole fix.** It removes this instance and makes the runner correct for any shape a producer publishes. It does not make a *disagreement* detectable: a producer that published a shape the runner's operand patterns cannot fill would still only fail on hardware.

## Work

Choose and implement one, stating the elimination:

- A pinned pair like the filename pair: both crates assert the published shape matrix in a test naming the other side. Cheap, and it re-couples two crates that deliberately share nothing.
- A gate-reachable fixture: the runner's test module already assembles an envelope from the live builder for the fail-closed probes, and `prove_member`'s shape handling could run against one. This is the option that would have caught the defect.
- An explicit statement in the sidecar the runner validates its own expectations against, making the shape part of the interface both halves already read.

## Closes when

A producer/runner shape disagreement is a red gate, the chosen option's cost is recorded, and the prototypes' own documentation says which mechanism holds the interface.

## Outcome

**Chosen: the gate-reachable fixture, with the pinned pair kept as its validity condition rather than as a second mechanism. The sidecar statement is eliminated.**

### The elimination, derived rather than asserted

**Fact — the sidecar statement cannot be a red gate, and cannot see the drift class this ticket names.** The artifact already declares the input shape and the runner already reads it (`bind_interface`); a shape restated in the sidecar would be the *producer* on both sides of its own comparison, so it can only catch a producer-internal inconsistency, never a producer/runner disagreement. It also fails only when a published artifact is read, which happens on hardware and never in the gate — the two prototype crates share no code and no Cargo edge, and `prototypes/serial-sum-compile` is a `[[bin]]`-only package the runner cannot link, so no gate-reachable published artifact exists. Adding a shape field to `ProofSidecar` would additionally edit `crates/tiler-artifact`, outside this ticket's scopes.

**Fact — the pinned pair alone is blind to the defect that actually happened.** The defect was not a wrong constant. Both halves stated one row and would have agreed; the defect was `prove_member` *consuming* `ROWS = 4` instead of the shape it had just read. A pair of assertions over stated values cannot observe which value the code used.

**Inference — a name is only ever compared, and a shape is consumed, so the mechanisms are not interchangeable.** The filename pair works because a filename has no second life beyond the comparison. The fixture is the only candidate that runs the consuming code.

**Inference — the pair is still required, as the fixture's validity condition.** The fixture must assemble at *the producer's* shapes to be about the interface at all, and its shapes come from a runner-side statement. Without a matching producer-side assertion, a producer that moved to another shape would leave the fixture quietly assembling envelopes nobody publishes — the stale-fixture failure the runner's own test-module header already warns about. That is one assertion per side in an idiom both crates already carry twice, not a second mechanism; it is recorded as such in both crates' documentation.

### What landed

- `prototypes/serial-sum-run/src/proof.rs`: the device-free half of `prove_member` is now two named functions, `compile_for_declared_shape` (shape read from the artifact, program compiled for it) and `require_derived_program` (packaged kernel-program identity must be one this build derived). `prove_member` calls both; nothing else changed on the hardware path except that its per-member line now prints the declared shape.
- New gate case `the_published_shape_matrix_survives_this_builds_shape_handling`: for each published reduction class it assembles an envelope at `PUBLISHED_ROWS × extent` through the module's existing live-builder assembler, runs those two functions, asserts the bound shape is the artifact's, and **requires the same check to refuse** when this crate's own `ROWS` is substituted. The packaged identity is taken from a `prepare`, so no device and no Metal toolchain is involved.
- New pinned pair: `the_published_shape_matrix_is_the_one_the_producer_writes` (runner) and `the_published_shape_matrix_is_the_one_the_runner_expects` (producer), over the published rows and the `(class, reduced extent)` matrix, each naming the other side. The runner's `REDUCTION_CLASSES` extents were previously stated and never read (`for (class, _reduced_extent)`); they are now load-bearing.
- `PUBLISHED_ROWS` lives under `#[cfg(test)]` in the runner, so a constant naming the producer's rows is unreachable from the routed path by construction. Its inequality with `ROWS` is asserted, because that inequality is what makes a substitution detectable.
- Documentation: both crates' module docs gained a section naming the mechanism that holds the shape and why it is not the filename mechanism, including the eliminations above.
- Corrected three stale claims found while reading: the runner's `COLUMNS` and `bind_interface` docs said the producer's and the direct path's shapes "now coincide" (the reduced extents coincide; the rows deliberately do not, since 0b7e59d), and the test module's `FIXTURE_COLUMNS` still deferred to `bound-the-backend-entry-key-by-the-identity-it-carries` as open. That ticket is `done` and bounded `BackendEntryKey` at `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`, so any reduced extent constructs now; `FIXTURE_COLUMNS` stays 1 for the loader cases by choice, and the published extents are covered by the new case instead.

### Measurement — the chosen option's cost

Host: Apple M4 Max, macOS, this checkout's pinned toolchain, `cargo nextest run -p tiler-prototype-run --status-level all` on the `test` profile.

| case | wall |
| --- | --- |
| `the_published_shape_matrix_survives_this_builds_shape_handling` | 25 ms |
| `the_published_shape_matrix_is_the_one_the_producer_writes` | 12 ms |

That is three envelope assemblies and six compilations (one per class, plus the substituted one per class) against a whole-package run of 0.96 s for 46 tests. Source cost: +2 tests and +2 shared functions in the runner, +1 test in the producer; no new dependency, no new public boundary, no change to the hardware path's behaviour.

### Measurement — the checks were watched failing

Each perturbation was applied, run, and reverted; the verbatim failure lines:

1. **The defect itself**, `compile_for_declared_shape` changed to `serial_sum_program(ROWS, columns)`:
   `empty-domain: the artifact declares 1x0 and this build compiled something else for it: the artifact packages a kernel program whose 2063-byte identity matches none of the 2 alternative(s) this process compiled for the artifact's own declared shape; the two prototypes have drifted`
2. **Producer rows drift**, producer `ROWS` 1 → 2:
   ``assertion `left == right` failed: `prototypes/serial-sum-run` assembles its published-shape fixture at one row; changing the published rows means changing its `PUBLISHED_ROWS` too / left: 2 / right: 1``
3. **Producer extent drift**, producer `("singleton", 1)` → `("singleton", 2)`:
   ``assertion `left == right` failed: `prototypes/serial-sum-run` assembles one fixture per class from its own copy of this matrix; a class or extent changed here must change there too / left: [("empty-domain", 0), ("singleton", 2), ("nontrivial", 3)] / right: [("empty-domain", 0), ("singleton", 1), ("nontrivial", 3)]``

The refusal half of case 1 is additionally permanent, not only a perturbation: the new case compiles the substituted shape on every run and requires `ForeignProgram`.

### Measurement boundary and what is not covered

**Measurement.** The fixture substitutes a synthetic carried payload for an `xcrun` link, exactly as the module's existing loader fixtures do, and the kernel-program identities it compares are the compiler's — so this is evidence about the runner's shape handling and the artifact layer, and none about what `xcrun` emits or what a device executes. The hardware proof remains the only thing that runs a published member.

**Unsupported.** A producer that published a shape whose *rows* equalled the runner's `ROWS` would make the substitution undetectable; that is why the inequality is asserted rather than assumed, but the assertion names the condition rather than removing it. A producer/runner disagreement in a dimension other than the input shape — a changed input or output key, a rank change — is caught by `bind_interface` at run time and is not covered by a gate fixture here.

**Not done, deliberately.** `cargo clippy -p tiler-prototype-run --all-targets -- -D warnings` reports three findings (one `redundant_closure_for_method_calls`, two `err_expect`); all three are present unchanged at the base commit `8ad0773`, all three are in code this ticket does not touch, and `make lint` excludes `prototypes/` by policy. They were left rather than swept into this commit.
