---
id: refresh-the-device-free-test-floor-s-prose-census
title: Refresh the device-free test floor s prose census
status: in-progress
priority: p3
dependencies: []
related: [date-the-conformance-measurement-bullet-s-all-runs-claim]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: w-terra-conformance-census
lease_expires_at: 1786214147
---

`DEVICE_FREE_TEST_FLOOR`'s doc comment retained a historical census after the live population grew. The initial prose-only premise was false: the unchanged floor also ceased to reject removal of a two-test device-free module.

## Fact audit, Terra, 2026-08-08 at `0339d28e`

**Verified — the historical prose was true when written.** `git log -S'declares 76 tests' -- crates/tiler-conformance/src/portability.rs` locates `9c46b5ae`; there, `git grep -F '#[test]' 9c46b5ae -- crates/tiler-conformance/src` returns 76 and `dispatch.rs` returns 3, so the `declares 76 tests` / `runs 73` sentence was true. It must be dated rather than silently rewritten.

**Verified — one later device-free test caused the drift.** `fe282f1e` adds `the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor` in `serial_sum/tests.rs`, moving that file's `#[test]` population from 17 to 18 while leaving `portability.rs` unchanged.

**Verified — the live census is 77 total, 74 device-free, and 3 macOS-gated; the floor remains 72.** `git grep -F '#[test]' HEAD -- crates/tiler-conformance/src` returns 77 and `dispatch.rs` returns 3. The qualified test filter `portability::a_non_apple_host_still_runs_the_device_free_test_population -- --exact --nocapture` prints `74 device-free test(s) and 3 in the macOS-gated module(s)` and passes. The unqualified exact filter ran zero tests and is not evidence.

**False — the original prose-only premise.** At 74 device-free tests, the floor of 72 sits two below the population. Each of `device_preflight`, `lints`, and `publication`'s own tests has two tests, so moving one behind the recognized macOS gate leaves 72 and produces a false green. The comment's current `smallest collapse fail` and `removing two device-free tests for any reason turns this red` claims are therefore false.

**Verified — the source selects the dominant maintenance answer.** The nearby governing rule says `Raising the floor with the population is the ordinary edit; lowering it is a decision about what a non-Apple host is held to`. Raising 72 to 73 restores the documented one-below relation and makes a two-test removal produce 72, below the floor. The purpose consequently changed from prose repair to guard restoration; it does not authorize lowering the requirement.

**Verified — `would remove twelve runs` was true only at its origin.** At `dd8f43db2`, `bf16_vertical/tests.rs`, `envelope/tests.rs`, and `serial_sum/tests.rs` each had twelve test attributes, so the warning described a real one-module gate loss then. It is not a current population authority; the scanner's `portability census:` output remains that authority.

**False — `It last rose 67 → 72` after this ticket's restoration.** `fe282f1e` added the eighteenth serial-sum test, and this ticket raises the floor 72 → 73 to restore its one-below sensitivity. The 67 → 72 movement remains historical, but no longer ends the chronology.

**False when introduced — inline modules carry no tests.** `dd8f43db2` already has `applicability.rs`'s inline `mod tests` with six test attributes. Inline tests are counted in their parent source file, so they need no child-path resolution; the per-file macOS-predicate mention check is what rejects an individually gated test from a device-free file.

## Why it is worth fixing rather than shrugging at

A floor plus a prose census is a deliberate pairing: the floor stops the population silently shrinking, and the census tells a reader what the floor is protecting. Here, the stale count hid a stale threshold: the unchanged assertion stayed green after its smallest claimed loss.

## What closes this

Raise `DEVICE_FREE_TEST_FLOOR` from 72 to 73. Keep the live population authority in the scanner's `portability census:` output rather than restating a current count. Preserve the 76/73 census and the twelve-run warning as dated historical observations, substitute the never-true inline-module explanation, and make every named removal arithmetic follow the 74-device-free population: 2 → 72, retained-record → 70, applicability → 68, publication proof → 65, bf16 vertical → 61, serial sum → 56, and envelope → 57.

Demonstrate the guard is load-bearing by temporarily putting an actual two-test device-free source behind the scanner's recognized macOS gate, capturing the resulting 72-versus-73 failure, then restoring it before the unperturbed census and normal gates.

**Establish the treatment from history** with `git show <commit>:<file>`: true when written → dated beside; never true → substituted with the retired wording quoted. Repository practice, stated in several ADRs while applying it and decided by none — cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**; say inline that a later hit lands inside your note.

**Check the crate's other prose censuses and name the count.** The independent `date-the-conformance-measurement-bullet-s-all-runs-claim` follow-up owns the `docs/status.md` Measurement bullet; this ticket must not edit `docs/status.md`.

**`crates/**` is gated, so run `make full`** and report its exit — **read the log tail rather than trusting a reported code**; a worker this session had exit 2 reported as 0 because the exit line went through `tee`.

Cite by searchable anchor, run its grep before committing, and use `grep -F`.
