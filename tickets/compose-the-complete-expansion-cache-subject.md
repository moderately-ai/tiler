---
id: compose-the-complete-expansion-cache-subject
title: Compose the complete expansion cache subject
status: todo
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/metal-aot]
shared_scopes: []
paths: []
tags: [cache, identity, correctness]
---
`tiler-cache` keys an entry by the governed digest of a canonical subject the producer supplies, and a bundle carries a **whole artifact envelope**. A conforming subject must therefore determine every byte of that envelope, which is what `docs/backends/metal.md` already requires when it says "full artifact identity is the key".

**No component emits that subject as one canonical byte run.** `crates/tiler-metal-aot/src/identity.rs` emits the half that determines the `metallib` — source, target, exact ordered compile and link flags, SDK and tool versions, evidence class — and says nothing about the plan variants, ABI bindings, routing, or interface wrapped around it. Two artifacts that agree on the compilation and differ in their plan portfolio would hash to one key, and the cache would serve either for the other. That is a silently wrong result, not a lost hit.

`tiler-cache` deliberately did not invent the composition: it cannot compose a subject without becoming an authority over encodings it does not own, and `crates/tiler-cache/src/expansion/key.rs` states the obligation and this gap in terms rather than assuming a caller will meet it.

## What this ticket owes

- Decide where the composed subject is derived. The artifact layer already derives `CanonicalArtifactProgramIdentity` from a verified artifact, and the driver already derives its compilation subject; the open question is which component composes them and whether the composition happens before compilation, which it must, since the key is needed on a miss.
- Establish the composition by a mechanism rather than by vigilance, as `identity.rs` does — a new identity-bearing input must fail to compile until it reaches the subject.
- Keep it domain-separated and length-prefixed, so no two component subjects can be re-split into a third.
- State whether the composed subject subsumes `tiler-metal-aot`'s or wraps it, and preserve its `SameHost` reuse bound either way.

Until this lands, a caller passing the driver's subject alone is under-keying and `tiler-cache` cannot detect it.
