---
id: decide-whether-the-canonicity-re-encode-is-redundant
title: Decide whether the canonicity re-encode is redundant
status: todo
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets, contracts/artifacts]
paths: []
tags: [performance, artifact, correctness]
---
**Measured: the re-encode at `decode.rs:113` is 50% of decode time — 274 µs of 548 µs on a 26 KB envelope — and it runs on every artifact load and every cache hit.**

## What it is, stated fairly before it is questioned

```rust
if encode(&envelope)? != bytes {
    return Err(ArtifactCodecError::NonCanonicalManifest);
}
```

Its comment calls it "the backstop that makes one artifact have one byte identity: any well-formed but non-canonical spelling a named check did not already catch fails here rather than being silently normalized on the way in."

**It is the only thing enforcing one-artifact-one-spelling.** A non-canonical spelling of the same content derives the *same* identity, so the identity check at `decode.rs:105` cannot catch it, and the digests cannot either — they cover the bytes as written. Removing it on the argument that "the named checks cover it" would be exactly the cheap-option-that-drops-the-guarantee this repository warns about.

## Resolve it by experiment, not by argument

For each named non-canonical form — section order (`validate.rs:201`), section ids (`decode.rs:191`), provider order (`decode.rs:393`), payload order (`decode.rs:506`), expression order (`decode.rs:598`), feature set (`validate.rs:71`), entry order (`validate.rs:459`), deferred and precondition order (`validate.rs:432`, `:504`) — neuter that named check and observe whether the backstop is what catches the form.

That produces the set of forms **only** the backstop covers:

- **empty** — it is genuinely redundant and can retire, with the experiment recorded as the evidence;
- **non-empty** — those forms need their own named checks, and the backstop retires behind them, which is a better outcome than keeping it: a named check reports *which* canonicity rule was violated, where the backstop reports only that some spelling differed.

Either way the guarantee survives and the diagnostic improves.

## Note

`stop-recomputing-pure-derivations-in-the-codec` removes the duplicated identity derivation inside the re-encode regardless of this outcome, so the two are independent.

## Closes when

Each named non-canonical form has been tested against a neutered check; the set the backstop uniquely covers is recorded; the backstop is retired or retained on that evidence rather than on argument; and if retired, every form it covered has a named check that names it.
