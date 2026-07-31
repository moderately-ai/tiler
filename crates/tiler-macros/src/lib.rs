//! The procedural macro implementation behind Tiler's inline tensor regions.
//!
//! Rust forbids a `proc-macro` crate from exporting anything but macros, so
//! this crate cannot be the durable facade a consumer imports. It is the
//! implementation half of a two-crate pair: consumers depend on `tiler`, which
//! re-exports [`tensor`] from here, and generated tokens name paths under
//! `tiler` rather than under this crate. A consumer therefore never names
//! `tiler-macros` — not in a manifest, not in an import, and not through a
//! path this macro expands to.
//!
//! # What this crate implements today
//!
//! The expansion entry point, the two behaviours [`tensor`] documents, the
//! frontend's statement of its artifact-family delivery policy, in the
//! crate-private `delivery` module, and its statement of where an expansion
//! looks for the expansion cache, in the crate-private `cache_root` module.
//! There is no grammar here: token parsing, span mapping onto a tensor
//! program, and ahead-of-time expansion are owned by
//! `define-inline-symbol-binding-and-runtime-value-adaptation` and
//! `prototype-inline-proc-macro-frontend`, and this crate rejects rather than
//! guesses at input those tickets have not defined.
//!
//! # The cache root is chosen here, and opened nowhere yet
//!
//! `tiler-cache` takes a root from its caller and never consults the
//! environment, so choosing one is the frontend's. `cache_root` states that
//! choice — an override variable, a per-user macOS default, and a typed refusal
//! for every root that is unusable or not private — as a pure function of an
//! environment snapshot. Today's expansion opens no cache, so the resolver's
//! only caller is its own test module; `prototype-inline-proc-macro-frontend` is
//! the slice that calls it. Its consumer-visible spellings are a reviewed draft
//! under ADR 0075 until Tom accepts them.
//!
//! What the current expansion does prove is the part a later grammar cannot
//! re-litigate cheaply: that `tiler::tensor!` resolves through the facade's
//! re-export, that generated tokens reach a stable path inside the facade the
//! consumer already depends on, and that the delivery policy an expansion
//! performs is a stated, validated, canonical value rather than an unstated
//! consequence of what the expansion happens to do.
//!
//! # Why this crate, and not the facade, depends on the offline driver
//!
//! ADR 0049 requires every inline AOT request to carry a canonical typed
//! `ArtifactFamilySelection`, and its one canonical encoder is
//! [`tiler_metal_aot::family`] — copying it into the frontend would create a
//! second authority over one identity subject, and moving it below the driver
//! would spend the empty dependency closure ADR 0077 item 2 decides. So the
//! frontend depends on the driver. It is *this* crate that holds the edge
//! rather than `tiler`, because a `proc-macro` crate and its dependencies are
//! built for the host and never reach a consumer's target build graph, whereas
//! the same edge on the facade would link a process-spawning Apple toolchain
//! driver into every consumer on every platform — the cost ADR 0077 item 4
//! already refused for `tiler-metal`. `dependency_direction` in the `tiler`
//! crate is what keeps the facade free of it.
//!
//! # This crate never becomes a dependency of the compiler
//!
//! Nothing under `crates/tiler-ir`, `crates/tiler-compiler`, or any other
//! member may depend on this crate or on `tiler`. The frontend sits at the top
//! of the workspace graph so the compiler and IR stay consumer-agnostic; the
//! `dependency_direction` test in the `tiler` crate checks the resolved graph
//! and fails if an inward edge ever appears.

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

mod cache_root;
mod delivery;

/// The path generated tokens use to reach the facade.
///
/// It is absolute and leading-`::` so it resolves the same way from any
/// consumer module, and it names `tiler` rather than this crate because
/// `tiler-macros` is an implementation detail a consumer never declares.
///
/// A proc-macro has no `$crate`, so this path is the literal one every
/// expansion emits. It resolves only while the consumer's dependency is named
/// `tiler`; a consumer that renames it in `[dependencies]` gets a resolution
/// error at the call site rather than a wrong result. See
/// `resolve-the-generated-facade-path-under-crate-renaming` for the bounded
/// question of whether that stays acceptable.
const FACADE_ANCHOR_PATH: &str = "::tiler::__private::expansion_anchor()";

/// Expands an inline Tiler tensor region.
///
/// # Current behaviour
///
/// The grammar is not defined yet, and this macro does not invent one:
///
/// - Empty input states its artifact-family delivery policy
///   (`delivery::stated_policy`, `FallbackOnly` today), validates it into a
///   canonical `ArtifactFamilySelection`, and — because that selection invokes
///   no backend compiler — expands to an inert anchor value,
///   `::tiler::__private::expansion_anchor()`, which carries no tensor
///   semantics and exists so the facade re-export and the generated path are
///   compiler-checked. A policy this expansion cannot deliver becomes a
///   spanned compile error rather than a silent fallback.
/// - Any non-empty input is a compile error spanned at its first token. The
///   message names the tickets that own the grammar.
///
/// Both behaviours are placeholders. Empty input is a sentinel for "no region
/// yet", not a case the eventual grammar is expected to accept, and the
/// grammar tickets replace this entire body rather than extending it.
///
/// # Expansion is self-contained
///
/// Expansion runs entirely inside this process from the tokens it is given. It
/// does not scan the consumer's sources, require a `build.rs`, consult a
/// registry, or compile anything at runtime, and the only crate the generated
/// tokens name is the `tiler` facade the consumer already declared.
#[proc_macro]
pub fn tensor(input: TokenStream) -> TokenStream {
    match input.into_iter().next() {
        None => expand_region(),
        Some(first) => spanned_compile_error(
            first.span(),
            "`tiler::tensor!` has no grammar yet, so this input is rejected rather than \
             guessed at; the region syntax and its expansion are owned by \
             `define-inline-symbol-binding-and-runtime-value-adaptation` and \
             `prototype-inline-proc-macro-frontend`",
        ),
    }
}

/// Expands one region, after stating and validating its delivery policy.
///
/// The policy is stated before any tokens are produced, and a policy this
/// expansion cannot deliver returns the refusal instead of the anchor. Emitting
/// the fallback anyway would be the one thing ADR 0053 forbids outright: a
/// selected family "cannot silently turn a selected-family build failure into
/// fallback on the matching target".
fn expand_region() -> TokenStream {
    match delivery::stated_delivery(delivery::stated_policy()) {
        // Not `expect`: a panic here aborts rustc with "proc macro panicked"
        // and no span, which is the worst diagnostic this crate could produce.
        // The path is a fixed valid expression and this branch is not expected
        // to be reachable, but the cost of routing it to a real error is one
        // line and the cost of getting it wrong is a useless compiler message.
        Ok(_no_backend_work) => FACADE_ANCHOR_PATH.parse().unwrap_or_else(|_| {
            spanned_compile_error(
                Span::call_site(),
                "`tiler-macros` failed to lex its own facade anchor path; this is a defect in \
                 `tiler-macros`, not in the invocation",
            )
        }),
        Err(refusal) => spanned_compile_error(Span::call_site(), &refusal.to_string()),
    }
}

/// Builds `compile_error! { "<message>" }` with every token carrying `span`.
///
/// The span is what makes the rejection usable: it puts the diagnostic on the
/// offending token inside the invocation rather than on the macro call as a
/// whole.
fn spanned_compile_error(span: Span, message: &str) -> TokenStream {
    let mut literal = Literal::string(message);
    literal.set_span(span);

    let mut body = TokenStream::new();
    body.extend([TokenTree::Literal(literal)]);

    let mut group = Group::new(Delimiter::Brace, body);
    group.set_span(span);

    let mut bang = Punct::new('!', Spacing::Alone);
    bang.set_span(span);

    let mut expanded = TokenStream::new();
    expanded.extend([
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(bang),
        TokenTree::Group(group),
    ]);
    expanded
}
