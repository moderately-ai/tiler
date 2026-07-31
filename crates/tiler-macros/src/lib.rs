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
//! Only the expansion entry point and the two behaviours [`tensor`] documents.
//! There is no grammar here: token parsing, span mapping onto a tensor
//! program, and ahead-of-time expansion are owned by
//! `define-inline-symbol-binding-and-runtime-value-adaptation` and
//! `prototype-inline-proc-macro-frontend`, and this crate rejects rather than
//! guesses at input those tickets have not defined.
//!
//! What the current expansion does prove is the part a later grammar cannot
//! re-litigate cheaply: that `tiler::tensor!` resolves through the facade's
//! re-export, and that generated tokens reach a stable path inside the facade
//! the consumer already depends on.
//!
//! # This crate never becomes a dependency of the compiler
//!
//! Nothing under `crates/tiler-ir`, `crates/tiler-compiler`, or any other
//! member may depend on this crate or on `tiler`. The frontend sits at the top
//! of the workspace graph so the compiler and IR stay consumer-agnostic; the
//! `dependency_direction` test in the `tiler` crate checks the resolved graph
//! and fails if an inward edge ever appears.

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

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
/// - Empty input expands to an inert anchor value —
///   `::tiler::__private::expansion_anchor()` — which carries no tensor
///   semantics and exists so the facade re-export and the generated path are
///   compiler-checked.
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
        // Not `expect`: a panic here aborts rustc with "proc macro panicked"
        // and no span, which is the worst diagnostic this crate could produce.
        // The path is a fixed valid expression and this branch is not expected
        // to be reachable, but the cost of routing it to a real error is one
        // line and the cost of getting it wrong is a useless compiler message.
        None => FACADE_ANCHOR_PATH.parse().unwrap_or_else(|_| {
            spanned_compile_error(
                Span::call_site(),
                "`tiler-macros` failed to lex its own facade anchor path; this is a defect in \
                 `tiler-macros`, not in the invocation",
            )
        }),
        Some(first) => spanned_compile_error(
            first.span(),
            "`tiler::tensor!` has no grammar yet, so this input is rejected rather than \
             guessed at; the region syntax and its expansion are owned by \
             `define-inline-symbol-binding-and-runtime-value-adaptation` and \
             `prototype-inline-proc-macro-frontend`",
        ),
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
