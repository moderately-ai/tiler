---
id: prototype-semantic-type-authority-v2
title: Correct semantic type authority and marker binding
status: todo
priority: p0
dependencies: [prototype-semantic-foundation-api-v2]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir, research/extensions, research/numerics]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions]
paths: []
tags: [implementation, semantics, registry, dtypes]
---
Implement the reviewed type-authority portion of the v2 semantic API.

- register nominal definitions, parameterized constructors, and
  encoded-numeric schemes independently of process-local Rust markers;
- bind markers optionally to already valid complete resolved types, while
  retaining one-marker/one-resolved-type coherence within a frozen snapshot;
- validate concrete instances on demand through bounded host-owned schemas and
  narrow immutable semantic validators;
- replace the overloaded type-argument/facts representation with distinct
  public newtypes over shared canonical machinery, including a validated
  `NormativeDefinitionRef`;
- stage provider output in a fresh registration batch and merge atomically,
  without cloning the complete builder; and
- make retained semantic definition objects and provider entry points
  dyn-compatible, deterministic, and `Send + Sync + 'static` while excluding
  Rust identities and implementation addresses from durable identity.

Migrate the governed F32 definition and an external nominal, constructor, and
encoded-scheme proof through the same path. Preserve the existing resolved-type
bounds and canonical encodings unless the reviewed ADR explicitly versions
them. Do not add operation definitions, typed handles, reference execution, or
backend capabilities.
