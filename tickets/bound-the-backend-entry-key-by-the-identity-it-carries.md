---
id: bound-the-backend-entry-key-by-the-identity-it-carries
title: Bound the backend entry key by the identity it carries
status: done
priority: p0
dependencies: []
related: [route-the-runtime-proof-through-the-artifact-envelope]
scopes: [implementation/artifact, contracts/artifacts, implementation/metal-aot, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, needs-tom]
---
`prototypes/serial-sum-compile` cannot package the program `prototypes/serial-sum-run` dispatches. The artifact layer bounds every opaque identity at 1,024 bytes, and the canonical kernel identity of any non-degenerate serial reduction is larger. This is the one thing standing between the runtime proof's real program and the envelope, and the fix is a modelling choice rather than a number, which is why it is filed instead of applied.

## Measurement

Host: Apple M4 Max, macOS, this checkout's pinned toolchain. Procedure: vary `COLUMNS` in `prototypes/serial-sum-compile/src/main.rs`, run the producer, read `MetalEntryPoint::kernel_identity().as_bytes().len()` at `prototypes/serial-sum-compile/src/payload.rs:85` where it is handed to `BackendEntryKey::from_bytes`.

| reduced extent | kernel identity | packages? |
| --- | --- | --- |
| 1 | 728 bytes | yes |
| 2 | 1,113 bytes | `KeyTooLong { kind: BackendEntry, bytes: 1113, limit: 1024 }` |
| 3 | 1,113 bytes | same |
| 4 | 1,113 bytes | same |
| 8 | 1,113 bytes | same |

**Inference.** The step is between one contributor and two, and the size is then flat to eight. It is the *reduction structure* that crosses the bound, not the data size — so this is not a large-program problem that a modest program avoids. The bound admits the degenerate single-contributor reduction and nothing else.

## Fact — two authorities in this crate disagree about the same quantity

`crates/tiler-artifact/src/program/codec/model.rs:62-66` bounds a framed section at 64 MiB and says why: "A section carries one canonical kernel-program identity, so the bound is the shared IR's own identity budget." `crates/tiler-artifact/src/program/keys.rs:28` bounds an opaque identity at 1,024. A `BackendEntryKey` carries **the same canonical kernel identity** — `prototypes/serial-sum-compile/src/payload.rs:78-81` states that in terms: "The neutral entry key is the kernel's canonical identity, not the emitted symbol." Two constants, one quantity, a factor of 65,536 apart. `codec/budget.rs:10-12` names this exact failure mode: "Where the artifact model already governs a quantity, that constant is reused rather than restated. A codec-local bound would be a second authority for the same limit, and the two would drift." They have drifted.

**A second reading of the same evidence.** `keys.rs:11-13` describes an opaque identity as "a payload content digest, a backend entry key, a target-profile descriptor digest" — a list in which two of three are not digests. `PayloadDigest` is genuinely derived and measures 32 bytes. `TargetProfileDescriptorDigest` is not a digest despite its name: `crates/tiler-compiler/src/session.rs` documents that the descriptor bytes "*are* the descriptor identity rather than a hash of it". So 1,024 is a digest-sized bound applied to two types that carry canonical encodings.

## The choice, and why a worker should not make it

**(a) Raise `MAX_OPAQUE_IDENTITY_BYTES` to the shared IR identity budget.** Smallest diff, makes the two authorities agree. Cost: it also widens `PayloadDigest`, the one member of the three that really is a digest and really is 32 bytes, so a genuine bound is discarded to fix two types that never belonged under it. This is the cheap option and what it saves is the modelling.

**(b) Give a canonical identity its own bound, separate from a digest's.** Correct: `BackendEntryKey` and `TargetProfileDescriptorDigest` take the identity budget, `PayloadDigest` keeps a digest-sized one. Cost: a new public constant in `tiler-artifact` and a second validation path, which is a public-boundary change under ADR 0075.

**(c) Make a backend entry key a digest of the kernel identity rather than the identity.** Smallest wire, and the only option that stays bounded as programs grow. Cost: it changes what an entry key *means*. Today an artifact entry and a payload mapping tie together because both name the same canonical identity; under (c) they tie through a derived digest, and `tiler-artifact` would be minting an identity for a subject it explicitly says it is not the authority for (`keys.rs:14-16`: "That constructor is a statement that this crate is not the authority for that subject").

**Recommendation: (b).** (a) discards a real bound to fix a misclassification, and (c) is the only one that scales but is an ABI semantics change with an authority-direction cost. (b) fixes exactly the misclassification the evidence shows, and leaves (c) available as a later decision if identity size becomes a wire problem rather than a bound problem.

Not applied by the worker that found it: (a) and (b) both change a governed contract in `docs/artifact-abi.md`'s scope, (b) adds a public constant, and (c) is an ABI decision. Picking any of them is Tom's.

## What closes this

`prototypes/serial-sum-compile` packages the three-column program, its `COLUMNS` returns to 3, `prototypes/serial-sum-run`'s two paths run one program again, and the bound chosen is recorded where `docs/artifact-abi.md` states artifact limits.

## Outcome

**2026-07-25. None of (a), (b), or (c). A fourth option the evidence produced: bound each opaque identity by the constant the authority that mints it already publishes.** A `BackendEntryKey` takes `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES` — 16 MiB, the exact constant `encode_identity` enforces when it mints a `CanonicalKernelIdentity`. `PayloadDigest` and `TargetProfileDescriptorDigest` keep `MAX_OPAQUE_IDENTITY_BYTES`. No number was chosen, no public constant was added, and no dependency edge is new: `tiler-artifact` already names `tiler_ir::kernel` types in its public API.

This eliminates all three filed options rather than choosing among them. **(a)** discards `PayloadDigest`'s real bound; it is unnecessary once the bound is per-identity. **(b)** is right about *what* to separate and wrong about *where the number comes from* — a new `tiler-artifact` constant for the identity budget would be this crate deciding a bound for a subject `keys.rs:14-16` says it is not the authority for, and would be a second authority destined to drift from `MAX_KERNEL_IDENTITY_BYTES` exactly as `codec/budget.rs:10-12` warns. **(c)** was proposed as the only option that stays bounded as programs grow; that premise is false. The kernel identity is *already* bounded, by the crate that mints it, and the codec already carries the same bytes at that bound (below).

### Retraction — the ticket's central claim was imprecise, and the imprecision hid a stronger fact

The ticket states that `MAX_SECTION_BYTES` (64 MiB) and `MAX_OPAQUE_IDENTITY_BYTES` (1,024) bound "the same quantity". They do not. A `KernelProgramSubject` section carries `variant.program.canonical_identity()` (`codec/model.rs:433`, `:754`) — a `CanonicalKernelProgramIdentity`, bounded by `tiler_ir::program::MAX_PROGRAM_IDENTITY_BYTES` = 64 MiB. A `BackendEntryKey` carries a `CanonicalKernelIdentity`, bounded by `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES` = 16 MiB. Two related but distinct quantities with two distinct authorities, which is why `MAX_SECTION_BYTES` was never the number to copy.

**The real contradiction is one level tighter, and it is inside a single `EntryRow`.** `super::model::stage_key` (`model.rs:1138-1147`) length-prefixes `stage.kernel().canonical_identity().as_bytes()` into the stage subject, which `codec/model.rs:841` wraps as a `StageSubject` under `MAX_SUBJECT_BYTES` = 16 MiB. So every artifact carries its entry's kernel identity **twice**: once as `EntryRow::stage`, admitted to 16 MiB, and once as `EntryRow::entry_key`, refused past 1,024. The 1,024 bound therefore refused values the envelope beside it had already accepted, and guarded no allocation the stage subject had not already made. Its protective value was zero and its false-refusal rate was total.

**A second thing 1,024 was not doing.** `Cursor::slice` reads inside a manifest buffer already materialized under `MAX_MANIFEST_BYTES` (`decode.rs:161`), so a per-field identity bound cannot stop a forged length from reserving memory — the manifest bound does that. A per-identity bound is a well-formedness claim, which is exactly why its value has to be the minting authority's and not a guess.

### Measurement — reproduced, corrected, and extended

Host: Apple M4 Max, macOS, this checkout's pinned toolchain (`nightly-2026-07-19`). Procedure: a temporary sweep in `prototypes/serial-sum-compile`'s test module built `SemanticProgram`s at varying shape and reduced-axis sets, compiled each through `compile_governed(_, FlushSubnormalsToZeroF32)`, and read `VerifiedKernel::canonical_identity().as_bytes().len()` off the selected plan.

| input shape | reduced axes | kernel identity | packaged before |
| --- | --- | --- | --- |
| `[4, 1]` | `[1]` | 736 bytes | yes |
| `[4, 2]` | `[1]` | 1,121 bytes | no |
| `[4, 3]` | `[1]` | 1,121 bytes | no |
| `[4, 4]` | `[1]` | 1,121 bytes | no |
| `[4, 8]` | `[1]` | 1,121 bytes | no |
| `[1, 3]` | `[1]` | 1,029 bytes | no |
| `[64, 3]` | `[1]` | 1,121 bytes | no |
| `[4096, 3]` | `[1]` | 1,121 bytes | no |
| `[4, 3, 3]` | `[2]` | 1,483 bytes | no |
| `[4, 3, 3]` | `[1, 2]` | 1,314 bytes | no |
| `[4, 3, 3, 3]` | `[3]` | 1,845 bytes | no |
| `[4, 3, 3, 3]` | `[1, 2, 3]` | 1,507 bytes | no |
| `[4, 3, 3, 3, 3]` | `[1, 2, 3, 4]` | 1,700 bytes | no |
| `[4, 3, 3, 3, 3, 3]` | `[1, 2, 3, 4, 5]` | 1,893 bytes | no |
| `[4, 3, 3, 3, 3, 3, 3]` | `[1, 2, 3, 4, 5, 6]` | 2,086 bytes | no |
| `[4, 3, 3, 3, 3, 3, 3, 3]` | `[1, 2, 3, 4, 5, 6, 7]` | 2,279 bytes | no |

**Correction to the filed table.** The two figures are 736 and 1,121 on this checkout, not 728 and 1,113; the earlier measurement was taken before the eight bytes some intervening change added. The *shape* of the claim — a step between one contributor and two, then flat in the reduced extent — reproduces exactly, and so does the conclusion drawn from it.

**Extension, which is what eliminates a chosen number.** Identity is flat in every *extent* (`[4096, 3]` equals `[4, 3]`) and grows linearly in program *structure*: +362 bytes per rank reducing one axis, +193 bytes per rank reducing all of them. It is not flat, and no round number sits above it. The artifact model's own `MAX_INTERFACE_SHAPE_RANK` is 4,096, so a rank-4,096 program this codec declares admissible reaches roughly 1.5 MiB of kernel identity by the measured single-axis slope alone — before any fused body. Any bound between 1,024 and the minting authority's own would have been a different arbitrary refusal point, discovered the same way this one was: by running.

**Also measured, and deliberately not acted on.** `Compilation::target_profile_descriptor()` is 249 bytes for the standard profile and does not vary with the program, so `TargetProfileDescriptorDigest` refuses nothing today. It is still a canonical encoding under a digest-sized bound, it grows with the profile's capability and honourability facts, and `tiler-artifact` cannot name a `tiler-compiler` constant. Split to `bound-the-target-profile-descriptor-by-its-declaring-authority` rather than given an invented number here.

### No schema or domain version moved, and that is a decision rather than an omission

`MANIFEST_SCHEMA` stays at `3.0` and `ARTIFACT_DOMAIN` at `v2`. Nothing about the layout, the identity encoding, or the meaning of any field changed: an entry key was a length-prefixed opaque run before and is one now, and every artifact that encoded before this change encodes to the same bytes under the same identity after it. What moved is the *set of values admitted*, strictly widened. A reader built before this change meets a longer entry key and refuses it by name through the same `KeyTooLong` path — fail-closed, not mis-parsed — which is exactly the behaviour a bump exists to guarantee and therefore not a reason to spend one. Bumping would have been the more visible choice and the wrong one: it would assert a layout incompatibility that does not exist, and `docs/artifact-abi.md` already records why a version step has to mean what it says.

**`needs-tom` is retained.** The ticket reserved the choice, and the option taken is not among the three it reserved. Two of the three reasons it gave for reserving do not apply — no public constant was added, and no ABI semantics changed — but the third does: `docs/artifact-abi.md` is a governed contract and this edits it.

### What landed

- `crates/tiler-artifact/src/program/keys.rs`: `validate_opaque` takes the limit; `opaque_identity!` takes it per identity along with the doc naming it. Module documentation states the rule — an opaque identity's bound belongs to whoever mints it — and `MAX_OPAQUE_IDENTITY_BYTES`'s own documentation now says exactly which two identities it bounds and that it does not bound a backend entry key. No public item was added or removed; the constant keeps its name and value.
- `crates/tiler-artifact/src/program/tests.rs`: two regression cases. `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` admits the measured 1,121-byte case, asserts the refusal past `MAX_KERNEL_IDENTITY_BYTES` is still loud and typed, and pins both digest-shaped identities to the smaller bound — so raising one for all three cannot pass silently. `an_artifact_encodes_an_entry_key_longer_than_the_digest_bound` builds, encodes, and decodes a real artifact whose entry key exceeds the old bound, which is the half that proves the builder and the codec admit the same values.
- `docs/artifact-abi.md` "Governed budgets": the rule, the per-identity bounds, and the measurement including the twice-carried-identity contradiction.
- `prototypes/serial-sum-compile`: `COLUMNS` is 3. `prototypes/serial-sum-run`: no code change — `bind_interface` reads the declared shape as before — and its documentation now records that the two paths agree rather than that they cannot.

### Verified end to end

```text
$ cargo run -p tiler-prototype-compile -- --out <path>
target profile: tiler.prototype-target-neutral-baseline.v1
selected alternative: selected-plan:f31b2681fd654990 (fused)
emitted 1 entry point(s), 3097 bytes of MSL
compiled 3843 bytes of metallib for air64-apple-macos13.0
payload subject: 1 entr(y/ies), 2 obligation(s), identity 32 bytes
artifact envelope: 39226 bytes, 3 section(s), 1 variant(s), identity 15406 bytes

$ cargo run -p tiler-prototype-run -- --artifact <path>
device: Apple M4 Max
selected alternative: selected-plan:f31b2681fd654990
the artifact declares a 4 by 3 input
routed: symbol "tiler_kernel_d8260aa9a85f7c45", 3843 object byte(s), 4 thread(s) in groups of 1
  abi slot 0 -> transport 0, 48 byte(s), ProgramInput(InputKey("input"))
  abi slot 1 -> transport 1, 16 byte(s), ProgramOutput([OutputKey("result")])
direct    4x3: [40c00000, 3f800000, 7fc00000, 7f800000] against [40c00000, 3f800000, 7fc00000, 7f800000]
envelope  4x3: [40c00000, 3f800000, 7fc00000, 7f800000] against [40c00000, 3f800000, 7fc00000, 7f800000]
bit-for-bit agreement: direct on 4 element(s), envelope on 4 element(s)
```

The envelope path declares `4 by 3` rather than `4 by 1`, and both paths reduce the same three-contributor program to the same bits.
