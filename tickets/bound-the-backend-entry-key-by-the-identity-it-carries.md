---
id: bound-the-backend-entry-key-by-the-identity-it-carries
title: Bound the backend entry key by the identity it carries
status: in-progress
priority: p0
dependencies: []
related: [route-the-runtime-proof-through-the-artifact-envelope]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, artifact, needs-tom]
claimed_from: todo
assignee: agent-artifact
lease_expires_at: 1785015441
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
