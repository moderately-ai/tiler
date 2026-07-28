---
id: bind-the-cache-subject-to-the-carried-payload-provenance
title: Bind the cache subject to the carried payload provenance
status: awaiting-decision
priority: p2
dependencies: [compose-the-complete-expansion-cache-subject]
related: [implement-the-expansion-cache-protocol]
scopes: [implementation/cache, implementation/artifact, implementation/frontend]
shared_scopes: []
paths: []
tags: [cache, identity, correctness]
---
A `tiler-cache` bundle proves two things about its key on every hit: that the bundle was published under the key it is filed at, and that the key is the governed digest of the subject the bundle carries. It does **not** prove that subject describes the artifact beside it.

A writer that derived `K` from one subject and packaged an artifact compiled from another would produce a bundle every reader accepts. Nothing in the cache can catch it, because catching it means parsing the producer's subject encoding, which would make the cache a second authority over an encoding it does not own.

The carried envelope does record its own compilation subject: `PayloadMetadata` holds the exact source, target, flags, and toolchain provenance, and `decode_artifact` already proves the payload descriptor's digest equals `payload_identity` of those bytes. So the material for a cross-check exists on both sides; what is missing is a component that may read both.

## User-visible outcome

A cache result is never used as though it were built from a different
compilation subject. The cache validates framing, cardinality, ordering, and
artifact integrity. The orchestrator that legitimately understands both the
producer's compilation facts and artifact metadata validates their semantic
correspondence before publication and before accepting a hit.

A mismatch is a typed producer/protocol defect, not an ordinary cache miss:
rebuilding and republishing the same mismatch would hide and repeat the bug.
The cache must not parse the foreign inner subject encoding; that would make it
a second authority.

## What composing the subject changed, and what it did not

`compose-the-complete-expansion-cache-subject` landed and this ticket is **not** made unnecessary by it. Composition decides what a key covers; this ticket decides whether the covered subject describes the artifact beside it. A writer that composed a correct subject and packaged an envelope from a different compilation still produces a bundle every reader accepts.

The cache now owns the composed outer frame, so it can check cardinality and
ordering without parsing a producer encoding. The facts-level comparison
belongs to the orchestrator that owns both inputs.

## Blocked on Tom — where the orchestrator lives (2026-07-27)

The ticket is well specified; what it needs is a component that may read both sides, and that component has no home.

**Measured state of the dependency graph.** `tiler-cache` depends on `tiler-artifact`, so both inputs are already reachable from one crate. **Nothing depends on `tiler-cache`** — checked across every `crates/*/Cargo.toml` and `prototypes/*/Cargo.toml`; it has no consumer at all. `crates/tiler-compiler/src/session.rs` is the compiler's public boundary and says in its own header that it is scoped to *reaching* execution, with "bundle assembly, execution" named as downstream of it. So the orchestrator is downstream of everything that exists, and nothing would call it today.

**The question.** Where does the component that validates the correspondence between the producer's compilation facts and the artifact's `PayloadMetadata` live?

Three candidates, and each is eliminated or reserved rather than merely weighed:

- **In `tiler-cache`.** Mechanically possible — it already depends on `tiler-artifact`. Eliminated by this ticket's own accepted text: the cache must not become a second authority over the producer's subject encoding. Worth stating explicitly because it is the cheapest option and it *compiles*, which is what makes it tempting.
- **In `tiler-compiler`** (behind `session.rs`). Would make the compiler depend on `tiler-cache`, inverting the direction — the compiler produces artifacts and would then also consume the cache that stores them. It also puts a publication/acceptance protocol inside the crate whose public boundary is a reviewed draft explicitly scoped *not* to be the finished API.
- **A new crate that owns publication and acceptance.** The right dependency direction: it depends on the compiler, the artifact, and the cache, and none of them depends on it. **This is what makes the question yours** — `AGENTS.md` reserves crate scaffolding: "Do not scaffold crates, stabilize APIs, or begin production kernel implementation unless Tom explicitly moves the project into that phase."

**Recommendation: the new crate**, because it is the only candidate whose dependency direction is correct, and because the mismatch this ticket catches is a *publication-time* and *acceptance-time* check — both of which are that component's job rather than the cache's or the compiler's. The counterpoint is real: it is the first crate scaffolded for something other than a compiler stage, and it will accrete the rest of the run path, so it is a larger commitment than one cross-check warrants on its own.

**What a decision must cover:** whether to scaffold now, and if so whether its scope is this cross-check alone or the whole publication/acceptance path. The typed mismatch being a producer/protocol defect rather than a cache miss is settled by the ticket and is not part of the question.

