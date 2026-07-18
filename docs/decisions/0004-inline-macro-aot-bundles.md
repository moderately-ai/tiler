# 0004: Treat each inline macro invocation as an AOT bundle

**Status:** proposed

## Context

The required developer experience is an ordinary inline tensor macro. Requiring
consumer `build.rs`, duplicated kernel declarations, source scanning, a Cargo
subcommand, or a separate specification crate is unacceptable. Cargo build
scripts also execute before arbitrary proc-macro expansions and cannot naturally
collect their results.

Procedural macros can execute host tools during compilation. One invocation
contains enough semantic information to optimize its own operation and can
produce a metallib containing all entry points required by its one- or multi-
kernel program plans.

## Decision

Each inline macro invocation is an independent AOT compilation and embedding
unit. The proc macro invokes the Tiler compiler, emits a macro-local MSL
translation unit, compiles it with Apple's offline tools on a global content-
addressed cache miss, and embeds the canonical manifest and metallib as
byte-string literals in the returned Rust tokens.

Equivalent invocations share external compiler work through deterministic
content identity and a concurrency-safe cache. Generated runtime code never
depends on cache files. Crate-wide aggregation is not required for correctness
or usability.

## Consequences

- Inline call sites require no consumer build script or prebuild command.
- Runtime execution remains AOT and does not compile MSL source.
- One invocation can embed multiple entry points and multi-step plans.
- Fusion is limited to semantics visible inside one invocation; wider fusion
  requires an inline region frontend.
- Target selection is constrained by proc-macro host visibility; native macOS
  is the initial Metal AOT path.
- rustc memory, IDE behavior, embedded bundle size, and duplicate-binary storage
  require explicit measurement and budgets.
- The global compiler cache is disposable and must use cross-process locking,
  complete identity, validation, and atomic publication.

## Alternatives considered

Rejected primary workflows include explicit build registries, shared descriptor
files, enclosing collection solely for build discovery, Cargo subcommands,
source scanning, and separate specification crates. They may remain internal
tools or optional future optimizations but cannot be required developer
workflow.
