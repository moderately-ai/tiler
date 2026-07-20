# 0049: Select artifact families explicitly at inline invocations

**Status:** accepted

## Context

An inline procedural macro executes as a host compiler plugin. Stable Cargo and
proc-macro contracts do not provide it the consumer-target environment promised
to build scripts, and local measurement found `TARGET` and `CARGO_CFG_TARGET_*`
absent. Cargo also does not track a selected Xcode installation as an input that
automatically invalidates an otherwise fresh expansion.

Inferring the artifact platform from the macro host can silently confuse macOS,
iOS device, iOS simulator, Catalyst, and non-Apple targets. Requiring a consumer
build script or prepare workflow would violate the accepted inline experience.

## Decision

Every inline AOT compilation request contains a canonical, typed
`ArtifactFamilySelection`. It names the governed artifact families to build and
fully participates in explain output and content identity. The proc macro does
not infer a family from its host environment.

A frontend may expose an ergonomic literal default profile, but the resolved
profile is explicit Tiler input. Each family retains its own platform, SDK,
deployment, compiler, payload, and compatibility metadata.

Changing the selected Apple toolchain is a documented rebuild boundary. When
expansion runs, the complete compiler fingerprint changes the cache key. Cargo
may not rerun expansion merely because Xcode or the external cache changed, so
users and CI must force a rebuild after changing toolchains.

## Consequences

- Native and cross builds cannot silently receive a host-family metallib.
- One invocation can deliberately embed several independently identified
  families without a registry or source scan.
- Unselected SDK families incur no mandatory compiler work.
- Cache deletion cannot break already generated code.
- Tiler cannot promise automatic incremental invalidation for external
  toolchain changes on stable Rust.
- Rust-analyzer behavior is a performance concern, not a correctness branch.

## Alternatives considered

- infer the consumer family from process environment;
- treat the macro host as the consumer target;
- always compile all installed Apple SDK families;
- require a build script, registry, scan, or prepare step;
- use an undocumented IDE-analysis mode.
