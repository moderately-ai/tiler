---
id: gate-the-runtime-fail-closed-probes
title: Run the runtime fail-closed probes in the gate
status: done
priority: p1
dependencies: []
related: [prototype-runtime-routing-commit, bound-the-backend-entry-key-by-the-identity-it-carries]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, routing, correctness, testing]
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

## Outcome

Seven fail-closed cases now run in the repository gate, in `prototypes/serial-sum-run/src/main.rs`'s `#[cfg(test)] mod tests`. The crate's `[[bin]]` declares `test = true`, so `cargo test --workspace --locked` — the exact command `scripts/check_rust.py` runs — builds and runs them. They need no device and no Metal toolchain, so they hold on both CI profiles.

### A retraction, of a claim this ticket inherited

**"It perturbs the real envelope five ways and asserts the *class* of each refusal" was true of three of the five and false of two.** The landed `probe_fail_closed` matched `LoadRejection::Artifact(_)` for both the flipped byte and the truncation, which is the artifact *layer* rather than a class: an integrity failure silently becoming `Malformed`, `Unsupported`, `Invalid`, or `Limit` would have passed the probe unchanged. The measurement that produced `artifact.integrity` and `artifact.malformed` was real; the code did not assert it. Both are now pinned to the exact `ArtifactCodecFailure` variant, with the class *derived* from the encoder's framing rather than observed:

- **Damage → `IntegrityFailure`.** The encoder writes the header, then the manifest, then each section as its ordinal, its length, and its content, so the last section's content ends the envelope — checked with `bytes.ends_with(content)` rather than assumed. A changed content byte can only be caught by a digest comparison, and every such comparison classifies as an integrity failure.
- **Truncation → `Malformed`.** The framing header carries the envelope's own total length as a derived field. No proper prefix satisfies it, so a prefix long enough to hold the header refuses as a total-length disagreement and a shorter one as truncation; both classify as malformed, for any cut.

The original midpoint flip is **retained beside** the derived one as `probe_damaged_interior_byte`, and deliberately still asserts only `Artifact(_)`, because at an arbitrary offset the class is a function of where the byte lands. **Measurement**, Apple M4 Max against the producer's 32,449-byte envelope: the midpoint lands in the manifest and refuses as `ManifestDigestMismatch`. That is one envelope's arithmetic and is not asserted.

### Every rejection is paired with its accepted neighbour

`probe_accepted_baseline` runs first and requires the *unperturbed* subject to reach a `Preflight`, asserting the launch geometry the artifact's own expression evaluates to. Without it a harness that produced garbage would refuse every perturbation under a plausible class and report a fail-closed loader while measuring nothing. `probe_fail_closed` now takes a `ProbeSubject` holding the four inputs, so each probe's signature shows it changes one of them.

### Which closure was taken, and why the other two were eliminated

**Taken: assemble the envelope in the test, from the live builder.** Nothing can go stale — the artifact is minted by the current builder through the current encoder in the same compilation as the loader under test, so a builder or encoder change is a build failure rather than a fixture quietly describing yesterday's format. The fixture substitutes a synthetic carried payload for a real `xcrun` link, which the loader can neither observe nor interpret: a payload's object bytes are opaque to every check `DecodedProgram` performs. It keys its entry on the same canonical kernel identity the producer uses and therefore inherits the producer's `MAX_OPAQUE_IDENTITY_BYTES` constraint rather than routing around it, which is why it reduces one column.

**Eliminated: a checked-in envelope fixture.** The cheapest option, and it is a claim on disk that outlives whatever produced it, with no predicate in the repository comparing the two.

**Eliminated by scope, not by design: a unit test inside `tiler-runtime`, which is the better home.** `ArtifactProgramBuilder::new` takes a `tiler_ir::semantic::SemanticProgram` and `tiler-runtime` depends on `tiler-artifact` alone, so an in-crate test needs a `tiler-ir` dev-dependency. `Cargo.lock` records per-package dependency lists, so that edit falls in `implementation/cargo-lock`, which this ticket does not hold, and `cargo test --locked` would refuse it. Relocating these cases into the crate later is a move, not new evidence, so no follow-up ticket was split for it.

### Gate-enforced versus measured-only, after this change

Gate-enforced, each as its own case pinning both the matched variant and the rendered class prefix: an accepted route; `artifact.integrity` on a damaged section; `artifact.malformed` on a truncation; `runtime.program-mismatch`; `runtime.incompatible-target` with `TargetDeclaration::Variant` and `TargetCompatibility::DescriptorMismatch`; `runtime.unexecutable-payload`; and the artifact layer refusing an arbitrary interior flip.

Still measured-only, and unchanged by this ticket: that a *real* `xcrun`-produced 32,449-byte envelope refuses the same way. The gate cannot reach a Metal toolchain on both CI profiles, so the binary keeps carrying the same probe functions onto hardware. Neither subsumes the other.

**Verification that the cases are not vacuous.** Each class pin was inverted and the corresponding case observed to fail naming the real class — `TotalLengthMismatch { declared: 25908, actual: 12954 }` for the truncation and `SectionDigestMismatch { section: 2 }` for the damaged section — before being reverted.
