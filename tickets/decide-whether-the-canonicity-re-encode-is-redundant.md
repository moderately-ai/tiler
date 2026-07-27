---
id: decide-whether-the-canonicity-re-encode-is-redundant
title: Decide whether the canonicity re-encode is redundant
status: done
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

## Outcome

**Retained, on evidence rather than argument. The set of forms only the backstop covers is non-empty**, and one member of it is caught by *nothing else at all* the moment a governed component schema minor moves off zero.

### Per-form experiment

Method: every named check listed above was neutered *simultaneously* — one at a time would let a surviving named check mask the answer — and each form's forgery was then decoded. A form the backstop covers reports `NonCanonicalManifest`; a form it is blind to decodes successfully. The harness is the ordinary forging helpers in `crates/tiler-artifact/src/program/codec/tests.rs`; the neutering was temporary and is not in the diff, and every form's forgery is now a retained named test.

| Non-canonical form | Named check | With every named check neutered |
|---|---|---|
| section order | `validate.rs` `check_sections` | accepted — backstop blind |
| **section identifiers** | `decode.rs` `read_sections` | **`NonCanonicalManifest` — backstop is the only catcher** |
| provider order | `decode.rs` `read_providers` | accepted — backstop blind |
| payload order | `decode.rs` `read_payloads` | accepted — backstop blind |
| expression order | `decode.rs` `parse_expressions` | accepted — backstop blind |
| declared feature set | `validate.rs` `validate` | accepted — backstop blind |
| entry order | `validate.rs` `check_variant` | accepted — backstop blind |
| deferred predicate order | `validate.rs` `check_variant` | accepted — backstop blind |
| launch precondition order | `validate.rs` `check_entry` | accepted — backstop blind |
| **component schema minor** | *none* | **`NonCanonicalManifest`, with nothing neutered** |

### Why the split falls exactly there

**Fact, derived from the two files and confirmed by the table.** The eight forms the backstop is blind to are all *model-carried*: the parse reads them into `ArtifactEnvelope` in file order and the encoder writes them back in model order — `encode` does not re-canonicalize, because canonicalization happens in `ArtifactEnvelope::project` when a *verified artifact* is projected, not when bytes are read. So `encode ∘ parse` is the identity on them, a non-canonical spelling re-encodes to itself, and the byte comparison cannot see it. Their named checks are load-bearing and not redundant with anything.

The backstop's real coverage class is the complement: **fields the wire carries that the model does not.** A section identifier is one — a `Section` holds a purpose and bytes, and both the framing identifier and the descriptor's copy are re-derived from position — so a forged identifier is normalized away by the parse and reappears as a byte difference. Every such field has a named check today *except one*.

### The one with no named check, measured

`parse_component_schemas` admits `minor <= governed` and then deliberately returns `ArtifactSchema::GOVERNED` — the constant, not the values it read, with a comment saying why. Every governed component minor is `0` today, so nothing below the governed minor exists and the hole is unreachable. Raising the ABI expression component to `(1, 1)` locally and forging a manifest that declares minor `0` produced `NonCanonicalManifest` **with no check neutered**: the manifest is admitted, silently normalized, and caught only by the re-encode. Retiring the backstop would open that hole the first time any component takes a minor version.

### Cost, and why it is not the reason to remove it

The re-encode was measured at 49.2% of decode, which is what made it look expensive. Profiling shows that share is not the re-encode being intrinsically costly — **it is the governed SHA-256 running at ~120 MB/s**, and a decode drives roughly two envelope-equivalents through it. `crates/tiler-artifact/src/program/codec/digest.rs` `Sha256::compress` shifts its working state with `working.rotate_right(1)` on a `[u32; 8]`, which is the *slice* rotate and lowers to a `memmove` call, 64 times per 64-byte block; a sampling profile of a decode loop attributes 57% of active self time to `_platform_memmove` and a further 10% to `<[u32]>::rotate_right`. A standalone A/B of the same rounds — asserted to produce identical state — measures 280.7 MB/s for the shipped spelling against 407.2 MB/s for named-variable reassignment. Making the digest fast is worth several times what removing this check would buy, and it costs no guarantee at all.

## Closes when

Done. Every named form was tested against a neutered check; the set the backstop uniquely covers is recorded above and is non-empty; the backstop is retained on that evidence; and the forms that previously had no regression test (section identifiers, entry order, deferred predicate order, launch precondition order) now have one each.
