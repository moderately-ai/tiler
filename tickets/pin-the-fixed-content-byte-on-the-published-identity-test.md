---
id: pin-the-fixed-content-byte-on-the-published-identity-test
title: Pin the fixed-content byte on the published-identity test
status: review
priority: p3
dependencies: []
related: [attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget, decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifacts, measurement]
claimed_from: todo
assignee: agent-byte-pin
lease_expires_at: 1786070865
---
## User-visible outcome

An unexplained movement of the artifact envelope's fixed content fails a test with a superseded-value ledger, instead of being discovered days later by a research sweep — the budget answer [the manifest-growth record](../docs/research/artifacts/manifest-fixed-content-growth.md) recommends, implemented.

## Why this shape, from the record's own measurement

**Measurement.** Across the 107 landings the attribution swept, a fixed-content byte pin would have fired **3 times, and all three were identity-domain steps already rebaselining `the_standard_metal_path_publishes_its_recorded_identities`** (crates/tiler-build) — so the pin adds no rebaseline event to the workflow, only one number to a file that was already moving whenever it would fire. That test already pins two identities with a superseded-value ledger and regeneration mechanics; the fixed-content byte joins them in the same idiom.

**The counterpoint the record demonstrated, carried rather than dropped.** A fixed-fixture pin is blind to program-size growth: `36d05128` raised the governed `semantic_operations` budget 8 → 62 — admitting programs ~2.8× past the 1 MiB embedding ceiling — and moved the fixture by zero. The blind spot's coverage is [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md), which the record ranks first; this pin is the cheap second half, not a substitute.

## The work

Add the fixed-content byte count of a zero-object envelope of the standard Metal path's program to `the_standard_metal_path_publishes_its_recorded_identities`' pinned values, with the ledger paragraph stating what the number is, why it moves only at encoding changes, and the regeneration mechanics the test's existing pins use. Watch it fail under a deliberate perturbation (any one-byte manifest addition) before trusting it. If [`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md) lands first, the pin's initial value is taken after that step rather than before it.

## Closes when

The pin exists with its ledger, was watched failing, and the record's Section 6 recommendation row points at it as implemented.

## Outcome — 2026-08-06, delivered at `562b02e543e177509575d2f50a9a002e1bd78859`

**The pinned number is 64,699 bytes, and it is fixed content rather than envelope length.** `the_standard_metal_path_publishes_its_recorded_identities` gains `FIXED_CONTENT_BYTES: usize = 64_699`, asserted third after the two identity pins. The whole envelope this path publishes is **64,707** bytes and it carries **one** payload holding **8** object bytes — the fixture's fake toolchain emits `MTLBplan` — so the pin subtracts them and pins what the artifact would encode to carrying an empty object. Both figures were read from the tree, not derived: a probe assertion on the branch reported `(64707, 8, 1)` for `(envelope length, object bytes, payload count)`.

**The subtraction is exact rather than approximate, which is why the brief's fallback of pinning total length was not taken.** `SECTION_FRAMING_BYTES` in `crates/tiler-artifact/src/program/codec/encode.rs` is `size_of::<u32>() + size_of::<u64>()`, asserted fixed width at the site that writes it, so removing a section's content bytes removes nothing but them. Separating them keeps the pin from rebaselining on an edit to the fake toolchain's output text, which would say nothing about the encoding. The quantity is read through `AcceptedMetalPlanArtifact::decoded()` — the validated envelope the cache resolution carries — rather than by re-encoding the producer-side artifact, so the pinned value is the one a loading host receives.

**Watched failing under a one-byte manifest perturbation, and the failure has the exact shape the pin exists for.** Lengthening `MANIFEST_DOMAIN` from `b"tiler.artifact-envelope.manifest.v1\0"` to `b"tiler.artifact-envelope.manifest.v1x\0"` adds one byte to every manifest and leaves decoding valid, because `decode.rs:326` compares against the same constant. Under it the run **reached the third assertion** — both identity pins passed, since `encode_identity` reads the envelope and not the manifest — and failed with `left: 64700` against `right: 64699`. That is a size-moving wire change that moved no subject, which is precisely the class the two identity pins cannot catch and the class manifest schema `15.0` belonged to. The perturbation was reverted before the commit; `git status` is clean of it and the delivered diff touches one file.

Also watched failing first with a deliberately wrong constant of `0`, which reported `left: 64699` — that is how the value was taken, in the idiom the existing pins document.

**The ledger paragraph carries the counterpoint rather than dropping it**, and states three things the ticket asked for plus one it did not: what the number is and why the subtraction is exact; that it moves whenever the encoded *size* of manifest or section content moves, with `15.0` as the worked example of a move no identity pin sees; the regeneration mechanics, now three runs in assertion order rather than two; and — added because the pin would otherwise be read as covering more than it does — that a byte count is not a digest of the bytes, so a reordering or an equal-width field swap passes here. The counterpoint is the record's own: `36d05128` raised the governed `semantic_operations` budget 8 → 62 and moved a fixed fixture's fixed content by zero, with the note that the two folds since landed make the ~2.8× arithmetic historical while the blind spot is not, and that covering it belongs to `add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`. The superseded-value list is annotated so a reader does not misread its pairs as triples with a missing third value.

**Checks.** `cargo fmt --check`; `cargo clippy -p tiler-build --all-targets -- -D warnings`; `cargo nextest run -p tiler-build` (87 passed); `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-build`; `git diff --check`; `make full` on the branch. `tkt guard tkt/pin-the-fixed-content-byte-on-the-published-identity-test --format json` reports `"conflict": false`, `"under_declared": []`, and `"changed_files": ["crates/tiler-build/src/metal_plan.rs"]`; the listed collisions are declared-area overlaps with open siblings, which this repository configures as non-failing.

**Owed record pointer, for the coordinator — `docs/research/artifacts/manifest-fixed-content-growth.md` is `research/artifacts` and was not edited from here.** Section 6 (a) owes one line, landable verbatim:

> **Implemented 2026-08-06 at `562b02e543e177509575d2f50a9a002e1bd78859`.** `the_standard_metal_path_publishes_its_recorded_identities` pins its published envelope's fixed content — **64,699 bytes**, its 64,707-byte envelope less the eight object bytes the fixture's fake toolchain emits — beside its two identity pins, with the ledger paragraph, Section 7's counterpoint carried at the pin, and the check watched failing under a one-byte lengthening of `MANIFEST_DOMAIN` that left both identity assertions passing. That figure is this test's own fixture and not the ladder's: this note's 57,978 is `spikes/cache/hot-path-efficiency`'s governed serial-sum program, so the two are different programs and are not comparable point to point.

**What this does not establish.** One fixture, one program shape, one operation count, and one host. The pin detects movement; it prices nothing and refuses nothing, and it is silent on the program-size growth that the embedding-ceiling trigger owns.
