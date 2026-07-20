---
schema: "tiler-doc/v1"
id: "tiler.research.extensions.proc-macro-extension-visibility"
kind: "research"
title: "Proc-macro visibility of operation extensions"
topics: ["extensions", "proc-macro", "rust"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.operation-extensions"]
adopted_by: ["ADR-0045"]
ticket: "proc-macro-extension-visibility"
---

# Proc-macro visibility of operation extensions

**Question:** Can an inline stable Rust proc macro performing AOT compilation
discover and invoke operation-provider trait implementations defined in its
consumer crate?

## Primary-language constraints

The [Rust procedural macro reference](https://doc.rust-lang.org/reference/procedural-macros.html)
defines a proc macro as a function compiled in a `proc-macro` crate that accepts
and returns token streams. It may not be used in the crate where it is defined.
The macro implementation is therefore already compiled before it processes a
consumer invocation.

[Cargo build dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#build-dependencies)
and proc-macro dependencies execute for the host and do not become ordinary
target dependencies. Features can enable optional dependencies already named
by the proc-macro package, but a consumer cannot make the macro depend back on
the consumer: Cargo rejects that as a cyclic package dependency.

These are compilation-graph constraints, not missing reflection APIs. Stable
Rust has no mechanism by which a macro can resolve an arbitrary token path to a
consumer-local type, obtain its vtable, and execute its trait implementation
inside the already-built macro process.

## Executable experiment

[`spikes/extensions/proc-macro-visibility`](../../../spikes/extensions/proc-macro-visibility)
contains five crates and a negative dependency-cycle fixture. On 2026-07-19 it
was run with:

```text
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
host aarch64-apple-darwin
```

Run it with:

```sh
sh spikes/extensions/proc-macro-visibility/run.sh
```

Measured results:

| Probe | Result |
|---|---|
| Provider crate is a direct proc-macro dependency | Macro executes it during expansion |
| Optional provider is predeclared by the macro and enabled with a Cargo feature | Macro executes it during expansion |
| Consumer-local provider type is passed as invocation tokens | Macro observes only token spelling; generated target code can use the type later, but expansion cannot invoke it |
| Macro package identity versus generated `env!` identity | Expansion reports the host macro package; emitted tokens report the consumer package |
| Add a reverse macro-to-consumer dependency | Cargo reports a cyclic package dependency |
| Repeat the workspace test | Same provider keys and package observations |

Only `aarch64-apple-darwin` was installed. Cross-target environment details are
intentionally left to `macro-build-environment`; they cannot change the
dependency-direction result.

## Implications for the extension API

The public compiler API and inline macro have different provider-supply
surfaces over the same registry contract:

```text
ordinary compiler API
    caller constructs RegistryBuilder
    -> any statically linked provider in caller's dependency graph

inline proc macro
    macro constructs RegistryBuilder
    -> only providers in macro's host dependency graph
    -> plus semantic declarations fully represented by invocation tokens
```

The macro may generate target code that refers to a consumer type, but this is
too late for Tiler's expansion-time optimization and AOT artifact generation.
Deferring provider execution to that generated code would be runtime
compilation or an additional build phase, both outside the accepted product
contract.

## Smallest viable restriction

The initial inline macro freezes a deterministic provider set linked into its
own host dependency graph:

- Tiler built-ins and officially bundled operation packages require no
  auxiliary consumer workflow;
- optional Cargo features may select only provider packages already declared
  by that macro crate;
- invocation-local declarative semantics or exact decompositions may be
  admitted when the macro syntax contains the complete canonical definition;
- an arbitrary consumer-local Rust callback provider is supported by the
  ordinary compiler API, but not automatically by the separately compiled
  inline proc macro;
- an extension author may later ship a proc-macro frontend/wrapper that links
  its provider, provided it preserves the same registry, cache, artifact, and
  inline-DX contracts.

This restriction does not make the semantic operation set closed. It makes the
provider set for one proc-macro binary closed at its build boundary. Adding an
official operation continues to exercise the public registry path rather than
adding a private IR node.

## Rejected workarounds

- Linker inventories in the consumer populate the target binary, not the
  already compiled host macro process.
- Source scanning cannot reliably resolve Rust names, cfgs, macros, or trait
  implementations and violates the accepted workflow.
- A consumer `build.rs`, registry file, Cargo subcommand, or prepare step is an
  explicitly rejected primary DX.
- Runtime provider invocation occurs after the artifact was supposed to be
  generated and cannot participate in expansion-time planning.
- Depending from the macro back to the consuming crate is a Cargo cycle.

## Identity and diagnostics

The macro's complete linked/frozen registry remains compilation-request
provenance. Selected artifact identity uses only reached and selected providers
under ADR 0044. An unavailable provider is diagnosed during semantic admission
with its `OpKey` and the macro's provider-set identity; it is never treated as
an unknown opaque operation or silently delayed until runtime.
