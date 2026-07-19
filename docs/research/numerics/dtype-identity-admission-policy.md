# Dtype identity admission policy

**Status:** namespace governance accepted

## Problem

Recognition does not imply execution, but adding a canonical type identity is
still a durable compatibility commitment. Tiler needs a policy for deciding:

- which exact formats enter the built-in vocabulary;
- which retain an external project/vendor identity;
- which remain provider-defined extensions;
- how normative ownership and specification revisions appear in a `TypeKey`.

ADR 0027 requires one canonical identity and forbids renaming an external type
when Tiler later bundles support. It does not decide namespace authority.

## Precedents

DLPack and ONNX centrally enumerate exact tensor scalar formats and add new
values through versioned releases. StableHLO similarly versions its portable
type vocabulary and rejects unknown features outside its compatibility window.
MLIR places custom types in their owning dialect namespace. Arrow reserves its
own namespace for canonical types and requires third-party extension names to
be qualified; canonical admission includes specified semantics, serialization,
ambiguity analysis, and implementation evidence.

Primary sources: [DLPack C API](https://dmlc.github.io/dlpack/latest/c_api.html),
[ONNX IR](https://onnx.ai/onnx/repo-docs/IR.html),
[StableHLO compatibility](https://openxla.org/stablehlo/compatibility),
[MLIR language reference](https://mlir.llvm.org/docs/LangRef/),
and [Arrow canonical extensions](https://arrow.apache.org/docs/format/CanonicalExtensions.html).

## Proposed admission gates

A format enters Tiler's built-in recognized vocabulary only when all of these
are available:

1. an authoritative stable public definition;
2. an unambiguous bit/value contract including special values;
3. meaningful multi-ecosystem interchange or a foundational Tiler use case;
4. canonical encode/decode and adversarial conformance vectors;
5. a reviewed canonical descriptor and alias mapping;
6. an accepted serialization and compatibility commitment.

Backend or operation support is not an admission gate. ADR 0026 keeps those as
separate capabilities.

External project/vendor identities remain external even when Tiler ships
first-party providers. Frontend names are aliases resolved before semantic
admission. A semantic change creates a new identity/version; an implementation
change updates provider provenance.

## Preliminary classification

### Already accepted Tiler built-ins

- `bool`;
- signed `i2/i4/i8/i16/i32/i64`;
- unsigned `u2/u4/u8/u16/u32/u64`.

### Standards-backed candidates for built-in admission

- IEEE binary16, binary32, binary64, and binary128;
- bfloat16, pinned to a public exact cross-architecture definition;
- OCP OFP8 E4M3 and E5M2;
- OCP MX E2M3, E3M2, E2M1, and E8M0 constituents;
- OCP MX block-format scheme identities, separately from scalar `TypeKey`s.

IEEE decimal32/64/128 meet the stability test but have weaker tensor/GPU
relevance. Whether they enter the initial built-in catalog is product scope,
not a semantic ambiguity.

### Recognized external owner-namespaced candidates

- MLIR/StableHLO IEEE-convention `f8E3M4` and `f8E4M3`;
- StableHLO/MLIR, AMD, or Graphcore-qualified FNUZ variants;
- IBM HFP8 `f8E4M3B11FNUZ`;
- target ABI formats such as x86 extended precision and PPC double-double.

Equivalence edges require exact bit/value and conversion conformance. Similar
suffixes or widths never establish identity.

### Non-`TypeKey` identities

- TF32 is a computation/operand precision contract;
- packing is a `StorageEncodingKey`;
- MX, NVFP4, NF4, nested codecs, and other compound encoded tensors use a
  `QuantSchemeKey` plus storage encoding;
- resource, token, PRNG-key, and shape/index values use their own value kinds.

### Initial extension-only families

- `i128/u128` and arbitrary-width integers under the existing extension rule;
- posit/quire, logarithmic, unum, rational, and arbitrary-precision families;
- fixed-point, decimal fixed-point, UNORM/SNORM;
- project codecs and learned/vector codebooks without an admitted exact
  canonical descriptor;
- nonnumeric element domains such as strings, temporal, categorical, record,
  object, and variant values.

Extension-only means a registered provider may make the identity fully
verifiable and executable. It does not mean opaque or permanently unsupported.

## Accepted namespace policy

Two coherent canonical-key policies were considered:

1. Tiler-governed built-in keys such as `tiler::f32@1`, whose canonical
   descriptor contains a mandatory normative reference to IEEE 754-2019.
2. Authority-qualified keys such as `ieee::binary32@2019` and
   `ocp::ofp8_e4m3@1.0`, even though Tiler does not control those organizations'
   namespace or IR-version policy.

A project-controlled URI-style authority registry is a third spelling of the
second approach, but it retains the same governance burden.

**Accepted by Tom on 2026-07-19:** formats deliberately admitted into Tiler's
built-in vocabulary use Tiler-governed canonical keys. Their immutable
canonical descriptors carry mandatory normative-definition references. Actual
project/vendor-owned published identities retain their external keys even when
Tiler bundles first-party support.

Forward-compatibility rules:

- a published key resolves to one immutable semantic descriptor;
- an incompatible meaning change requires a new key semantic version;
- a later standards revision proven semantically identical may be recorded as
  additional provenance/equivalence evidence without changing the key;
- aliases and source-format spellings are frontend/import metadata, not
  canonical identities;
- before minting a built-in key, admission checks that Tiler has not already
  recognized an external canonical identity for the same format;
- an already-published external identity is supported in place and is never
  rekeyed into the Tiler namespace;
- exact external equivalence is explicit, versioned, and conformance-tested;
  equal width, name, or descriptor shape is insufficient;
- canonical serialization records the key and validates the registered
  descriptor fingerprint; Rust enum discriminants and provider addresses never
  participate;
- namespace display syntax and exact Rust structures may evolve without
  changing the abstract identity tuple.

These rules are recorded in ADR 0034. Namespace registration and collision
governance for external providers remain an API-design task, but the ownership
direction is fixed.
