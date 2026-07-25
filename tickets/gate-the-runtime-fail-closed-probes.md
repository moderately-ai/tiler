---
id: gate-the-runtime-fail-closed-probes
title: Run the runtime fail-closed probes in the gate
status: in-progress
priority: p1
dependencies: []
related: [prototype-runtime-routing-commit, bound-the-backend-entry-key-by-the-identity-it-carries]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, routing, correctness, testing]
claimed_from: todo
assignee: agent-probes
lease_expires_at: 1785016755
---
The routing commit's fail-closed classification is measured but not gate-enforced. Make it a checked property.

## What is already true

`prototype-runtime-routing-commit` landed `probe_fail_closed` in `prototypes/serial-sum-run/src/main.rs`. It perturbs the real envelope five ways and asserts the *class* of each refusal: a flipped byte, a truncation, a foreign expected identity, a host offering another target profile descriptor, and a host stating another backend family. Measured on an Apple M4 Max against a 32,449-byte artifact, all five refuse under distinct classes and none becomes a route miss.

The one-way commit itself *is* gate-enforced, by three `cargo test --doc` examples on `Preflight::commit` that pin `E0382` and `E0277`.

## The gap

The probes run only when the proof binary is invoked with `--artifact <path>` and a Metal device. `scripts/check_rust.py` runs `cargo test --workspace`, which has neither, so nothing in the gate would notice if a refusal silently changed class — a corrupt file starting to report `NoApplicableVariant` instead of `artifact.integrity` would pass CI. Class, not refusal, is the property: it decides whether a reader re-fetches bytes or rebuilds a plan.

## Why it was not simply done

The probes need a *valid* artifact and this workspace's only producer of one is a separate binary, `tiler-prototype-compile`, which lives in `implementation/metal-aot`. The two candidate closures both have a real cost, and choosing between them is the work:

- **A checked-in artifact fixture.** Cheap, and goes stale against the very encoder it exists to exercise — a fixture recorded before an envelope format change tests the old format and still passes. `AGENTS.md`'s rule about retained golden artifacts applies: a claim on disk outlives whatever produced it.
- **Building the artifact inside the test.** No staleness, but it needs the producer's bundle assembly, which is out of the runtime scope, so it means either a shared fixture crate or moving the assembly. That is a boundary question, not a test-plumbing question.

The probe classes themselves are device-free, so whichever closure is chosen, the resulting test needs no GPU and can run on both CI profiles.

## What closes this

A gate-run test asserting each of the five refusal classes, plus a stated decision on which closure was taken and why the other was eliminated. Reuse the `LoadRejection` variants the probes already match on; do not weaken them to a boolean.
