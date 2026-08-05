---
id: make-stage-retention-reachable-from-a-test
title: Make stage_retention reachable from a test
status: done
priority: p3
dependencies: []
related: [retain-succeeding-metal-stage-tool-output, carry-a-producer-stated-total-into-a-retained-run, cover-multi-position-stage-retention]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, testing]
---
## User-visible outcome

`stage_retention` — the function that assembles the only retention this product actually publishes — is exercised by a test, so its labelling, its per-stage attribution, and the total it now states are checked rather than merely compiled.

## Why this exists

**Withdrawn — "nothing reaches it" did not survive checking.** This premise read: "`crates/tiler-build/src/metal_cache.rs`'s `stage_retention` is private, takes `&[StageOutputs]`, and has no test in the workspace. Reproduce with `grep -rn "stage_retention" crates/ --include='*.rs'`: three hits, all in that file, none in a test." The grep is accurate about the *symbol* and wrong about *reachability*. `crates/tiler-build/src/metal_plan.rs`'s `a_succeeding_stages_output_returns_from_a_validated_cache_hit` and `a_silent_stage_is_retained_as_an_empty_run` reach `stage_retention` indirectly through `accept_or_publish_metal_plan` → `accept_or_publish_delivered_metal_artifact`, and they pin both governed labels against deliberately different per-stage sentences. They landed in `7bd91ec9`, the `retain-succeeding-metal-stage-tool-output` commit already named in this ticket's `related:`. **Measurement** — swapping the two stages inside `stage_retention` and running `cargo nextest run -p tiler-build` fails both of them; the swap was never invisible. This is the failure mode the research contract names: a name that does not appear is not a behaviour that is not tested.

**Fact — it cannot be constructed out of crate.** `tiler_metal_aot::record::StageOutputs` is `#[non_exhaustive]` with public fields and no constructor (`crates/tiler-metal-aot/src/record.rs`), and its only construction site is `crates/tiler-metal-aot/src/driver.rs:416`. A `tiler-build` test therefore cannot build one. This is accurate and, as it turned out, not the constraint that mattered: the third route the ticket did not enumerate is the *fake* toolchain — `Toolchain::with_launcher` over shell scripts standing in for `metal` and `metallib`, already in `metal_plan.rs` as `warning_toolchain`. It drives the whole real path, self-skips on nothing, and makes a stage's output an input the test chooses.

**Fact — the untested surface just grew.** `carry-a-producer-stated-total-into-a-retained-run` (commit `c39cb814`) changed `stage_retention` to state `ToolOutput::total_bytes()` through `DebugRetention::retaining_with_stated_total`. The retention side of that change is tested in `tiler-cache`; the producer side is carried by `cargo check` alone. The pairing of a stage's bytes with *that same stage's* total is exactly the kind of mistake a type does not catch.

**Fact — the untested half was the total, and only the total.** With the premises above corrected, the gap was measured rather than inferred. Perturbing `stage_retention` to pass `output.as_bytes().len()` in place of `output.total_bytes()` — a producer presenting a bounded prefix as the whole diagnostic — left all 83 pre-existing `tiler-build` tests green (`cargo nextest run -p tiler-build -E 'not test(metal_cache::)'`). Every retention fixture in the crate was short, where a stage's total and its retained length agree by construction, so nothing could tell the two apart.

## The elimination, and its survivor

Four remedy shapes, tested against correctness, coverage actually gained, and the surface each costs.

1. **A `#[cfg(test)]`-gated constructor** — eliminated on the facts: `cfg(test)` in `tiler-metal-aot` is not set when `tiler-build` compiles it as a dependency. A Cargo feature is the same item wearing a disguise, always on under workspace unification, and makes `cargo check -p tiler-metal-aot` and `-p tiler-build` see different surfaces.
2. **A public constructor on `StageOutputs` plus a unit test on `stage_retention`** — implemented and measured, then withdrawn. The shape that survived internal review was `from_stages(impl Fn(CompileStage) -> ToolOutput)` rather than `new(metal, metallib)`: both stages carry a `ToolOutput`, so a positional constructor lets a caller transpose them and still compile, reintroducing out of crate exactly the ordering convention the named fields exist to refuse, and it would spend the growth property `#[non_exhaustive]` protects. It worked — the unit tests caught all three perturbations below. It was withdrawn because of what it bought: see the survivor.
3. **The real Apple toolchain in a gated test** — eliminated. It self-skips without a qualified toolchain, and the one existing real-toolchain case (`a_real_front_end_warning_survives_a_succeeding_compilation`) documents why the linker stage cannot be asserted quiet: that is this toolchain's behaviour today, not a guarantee, and pinning it makes an Apple release a test failure.
4. **The fake toolchain already in `metal_plan.rs`** — the survivor. `warning_toolchain` takes each stage's stderr text as a parameter, so a stage that outwrites `MAX_RETAINED_OUTPUT_BYTES` is one longer argument, not a new fixture. It costs no public item, and it exercises capture, labelling, the stated total, cache encode, disk, decode, and re-validation rather than a synthetic call to a private function.

**Why 2 lost to 4 despite covering more.** Everything 2 covered and 4 does not — delivery positions past 0, and the `MAX_RETAINED_RUNS` elision branch — is unreachable in the product: `BoundMetalCompileDeclaration` has exactly one constructor, so every plan resolves at one delivery position. Option 2 would have bought Tom's acceptance of a permanent public boundary in exchange for coverage of configurations nothing can currently produce, while leaving the one real gap closable for free. `cover-multi-position-stage-retention` holds that remainder at `deferred` with the trigger that makes it real.

## Outcome

`a_stage_that_outwrote_the_capture_bound_states_the_total_it_had` in `crates/tiler-build/src/metal_plan.rs`: the fake front end writes `MAX_RETAINED_OUTPUT_BYTES + 512` bytes, the run is read back **from the cache hit** so the total is one the cache encoded and re-validated, and the quiet linker beside it pins the neighbour — a stage under the bound states its own length — so the assertion cannot be satisfied by a total that is simply always larger. No public item was added; `crates/tiler-metal-aot/` is untouched by this ticket, so `implementation/metal-aot` was dropped from `scopes:` rather than held exclusively against a diff that never reached it.

**Three watched-failing perturbations**, each reverted, all run as `cargo nextest run -p tiler-build`:

- *stage swap* (`stage_outputs.stage(stage)` → the other stage): 3 failures, including the new test.
- *stated total dropped* (`output.total_bytes()` → `output.as_bytes().len()`): 1 failure, the new test alone, naming the exact defect — `left: 16384, right: 16896`.
- *inflated constant total* (`output.total_bytes().max(MAX_RETAINED_OUTPUT_BYTES)`, correct for the truncated stage and wrong for the quiet one): 3 failures, the new test's neighbour assertion among them — `a stage under the bound states its own length: left: 16384, right: 31`.

Also corrected along the way: `crates/tiler-build/src/lib.rs` claimed "The Metal path retains nothing today because the AOT driver keeps a stage's output only when the stage fails", which `stage_retention` has contradicted since `7bd91ec9`.

## Closes when

A test observes `stage_retention` producing one run per stage per delivery position, under the governed labels, each carrying its own stage's bytes and its own stage's stated total — and a deliberate swap of the two stages fails it. Satisfied at the one delivery position the product can produce; the multi-position remainder is `cover-multi-position-stage-retention`. No new public item was added, so nothing here awaits Tom's acceptance.
