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
//! # The pipeline, and why it is four modules
//!
//! An expansion runs `tokens` → `grammar` → `region` → emission, and each step
//! is separated from the next by what it is allowed to know:
//!
//! - `tokens` copies the invocation into a span-generic form, because
//!   `proc_macro::Span` cannot be constructed and `TokenStream::from_str` cannot
//!   be called outside an expanding macro, so anything written directly against
//!   those types has diagnostics no test can observe.
//! - `grammar` decides the *shape* of the approved region text and nothing about
//!   what any name means, so a syntax refusal lands on the token that is wrong.
//! - `region` resolves every name — element types, operands, and the governed
//!   semantic operations `*` and `+` denote — derives the result, drives
//!   `binding`'s symbol unification, and constructs the region as a public
//!   logical program wherever the fixed-extent semantic layer can represent it.
//! - emission, below, turns that into tokens, keeping each operand's own span on
//!   the identifier that names the Rust value it will be supplied from.
//!
//! `binding` owns what `sym n;` means, `delivery` owns the artifact-family
//! policy an expansion states, and `cache_root` owns where an expansion would
//! look for an expansion cache.
//!
//! # What an invocation evaluates to
//!
//! `Result<A::Value, tiler::value::BindError<A::Error>>`, where `A` is the
//! adapter the supplied operands carry. It is a `Result` because the checks a
//! region owes — operand count, rank, stored scalar, and every symbol's equality
//! obligations — are decidable only against the values the invocation is handed,
//! and a region that cannot honour its declared interface must refuse rather
//! than return a value derived from a shape it did not verify.
//!
//! # Expansion is self-contained
//!
//! Expansion runs entirely inside this process from the tokens it is given. It
//! does not scan the consumer's sources, require a `build.rs`, consult a
//! registry, or compile anything at runtime, and the only crate the generated
//! tokens name is the `tiler` facade the consumer already declared.
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

use core::fmt;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

mod binding;
mod cache_root;
mod delivery;
mod family_cfg;
mod grammar;
mod region;
mod tokens;

use delivery::DeliveryPlan;
use region::{Expansion, RegionError};

/// The path generated tokens use to reach the facade's expansion entry point.
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
const FACADE_ENTRY_PATH: &str = "::tiler::__private::bind_and_build";

/// The type generated tokens name for the region facts they carry.
const FACADE_FACTS_TYPE: &str = "::tiler::__private::RegionFacts";

/// The name of the block-local constant holding one region's facts.
///
/// Block-scoped, so it cannot collide with anything outside the expansion, and
/// spelled unlike a name a consumer would write, so it does not shadow one
/// inside the block either.
const REGION_FACTS_BINDING: &str = "__TILER_REGION_FACTS";

/// Expands an inline Tiler tensor region.
///
/// # The region
///
/// A declaration block followed by one result expression:
///
/// ```text
/// tiler::tensor! {
///     sym n;
///     in a: f32[n], b: f32[n], c: f32[n];
///     out (a * b) + c
/// }
/// ```
///
/// `sym` declares one symbolic extent, unified from every operand axis naming
/// it. `in` declares the region's operands, each with its element type and its
/// axes; an axis is a declared symbol or a literal extent. `out` names the
/// single result expression, built from the declared operands with `*` and `+`.
/// Both statements may be repeated, and `out` is terminal.
///
/// # What it evaluates to
///
/// `Result<A::Value, tiler::value::BindError<A::Error>>` — the consumer's own
/// tensor type, or a typed refusal naming the operand and axis that failed.
///
/// # Refusals
///
/// Every refusal carries the span of the token that caused it: a rejected
/// element type lands on the element type, an undeclared symbol on the axis that
/// names it, an unsupported operator on the operator.
#[proc_macro]
pub fn tensor(input: TokenStream) -> TokenStream {
    let region = Span::call_site();
    match expand(&tokens::read(input), region) {
        Ok(expanded) => expanded,
        Err(refusal) => spanned_compile_error(refusal.span(), &refusal.to_string()),
    }
}

/// Expands one region, or returns the first refusal with its span.
fn expand(trees: &[tokens::Tree<Span>], region: Span) -> Result<TokenStream, Refusal> {
    let syntax = grammar::parse(trees, region).map_err(RegionError::from)?;
    let expansion = region::lower(&syntax)?;

    // The delivery policy is stated and validated before any token is produced,
    // and a policy this expansion cannot deliver returns the refusal instead of
    // the region. Emitting anyway would be the one thing ADR 0053 forbids
    // outright: a selected family "cannot silently turn a selected-family build
    // failure into fallback on the matching target".
    let delivery = delivery::stated_delivery(delivery::stated_policy())
        .and_then(delivery::stated_plan)
        .map_err(|source| Refusal::Delivery {
            span: region,
            source,
        })?;

    emit(&expansion, &delivery, region)
}

/// Why an invocation did not expand.
enum Refusal {
    /// The tokens are not a region this frontend admits.
    Region(RegionError<Span>),
    /// The artifact-family delivery policy this expansion states is not one it
    /// can deliver.
    Delivery {
        /// The invocation.
        span: Span,
        /// The delivery module's own refusal.
        source: delivery::DeliveryRefusal,
    },
    /// This crate produced tokens it cannot itself lex.
    ///
    /// A defect in `tiler-macros`, routed to a spanned error rather than to a
    /// panic: a panic inside an expansion aborts rustc with "proc macro
    /// panicked" and no span, which is the worst diagnostic this crate could
    /// produce.
    MalformedEmission {
        /// The invocation.
        span: Span,
        /// The source text that failed to lex.
        source: String,
    },
}

impl Refusal {
    /// Returns the span this refusal must be reported at.
    const fn span(&self) -> Span {
        match self {
            Self::Region(source) => *source.span(),
            Self::Delivery { span, .. } | Self::MalformedEmission { span, .. } => *span,
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(source) => source.fmt(formatter),
            Self::Delivery { source, .. } => source.fmt(formatter),
            Self::MalformedEmission { source, .. } => write!(
                formatter,
                "`tiler-macros` produced source it cannot lex (`{source}`); this is a defect in \
                 `tiler-macros`, not in the invocation"
            ),
        }
    }
}

impl From<RegionError<Span>> for Refusal {
    fn from(source: RegionError<Span>) -> Self {
        Self::Region(source)
    }
}

/// Builds the block one region expands to.
///
/// The shape is fixed:
///
/// ```text
/// {
///     const __TILER_REGION_FACTS: ::tiler::__private::RegionFacts = …;
///     <the delivery plan's `#[cfg]`-gated items, if it has any>
///     ::tiler::__private::bind_and_build(&__TILER_REGION_FACTS, &[&a, &b, &c])
/// }
/// ```
///
/// The delivery items are `delivery::DeliveryPlan::items_source`'s, placed in
/// the same block so a `#[cfg]`-gated payload selector is scoped to the one
/// region that selected it. Every expansion states `FallbackOnly` today, whose
/// plan contributes nothing, so the block is exactly the two statements above.
///
/// The operand identifiers carry the spans the region's own `in` list wrote
/// them at, so a value that is not in scope is reported at the declaration that
/// named it rather than at the invocation as a whole. Everything else is
/// scaffolding and carries the call site.
fn emit(
    expansion: &Expansion<Span>,
    delivery: &DeliveryPlan,
    region: Span,
) -> Result<TokenStream, Refusal> {
    let facts = lex(
        &format!(
            "const {REGION_FACTS_BINDING}: {FACADE_FACTS_TYPE} = {};",
            expansion.facts
        ),
        region,
    )?;
    let items = delivery.items_source();
    let delivered = if items.is_empty() {
        TokenStream::new()
    } else {
        lex(&items, region)?
    };
    let entry = lex(FACADE_ENTRY_PATH, region)?;
    let facts_reference = lex(&format!("&{REGION_FACTS_BINDING}"), region)?;

    let mut operands = TokenStream::new();
    for (position, operand) in expansion.operands.iter().enumerate() {
        if position != 0 {
            operands.extend([TokenTree::Punct(spanned_punct(',', region))]);
        }
        operands.extend([
            TokenTree::Punct(spanned_punct('&', operand.span)),
            TokenTree::Ident(Ident::new(&operand.text, operand.span)),
        ]);
    }
    let mut operand_slice = TokenStream::new();
    operand_slice.extend([
        TokenTree::Punct(spanned_punct('&', region)),
        TokenTree::Group(spanned_group(Delimiter::Bracket, operands, region)),
    ]);

    let mut arguments = TokenStream::new();
    arguments.extend(facts_reference);
    arguments.extend([TokenTree::Punct(spanned_punct(',', region))]);
    arguments.extend(operand_slice);

    let mut body = TokenStream::new();
    body.extend(facts);
    body.extend(delivered);
    body.extend(entry);
    body.extend([TokenTree::Group(spanned_group(
        Delimiter::Parenthesis,
        arguments,
        region,
    ))]);

    let mut expanded = TokenStream::new();
    expanded.extend([TokenTree::Group(spanned_group(
        Delimiter::Brace,
        body,
        region,
    ))]);
    Ok(expanded)
}

/// Lexes one piece of generated source, carrying the call site on every token.
fn lex(source: &str, span: Span) -> Result<TokenStream, Refusal> {
    let stream: TokenStream = source.parse().map_err(|_| Refusal::MalformedEmission {
        span,
        source: source.to_owned(),
    })?;
    Ok(respan(stream, span))
}

/// Replaces every span in a stream, so generated scaffolding is attributed to
/// the invocation rather than to a source file the consumer cannot open.
fn respan(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|tree| match tree {
            TokenTree::Group(group) => TokenTree::Group(spanned_group(
                group.delimiter(),
                respan(group.stream(), span),
                span,
            )),
            mut other => {
                other.set_span(span);
                other
            }
        })
        .collect()
}

/// Builds one punctuation token at a span.
fn spanned_punct(character: char, span: Span) -> Punct {
    let mut punct = Punct::new(character, Spacing::Alone);
    punct.set_span(span);
    punct
}

/// Builds one delimited group at a span.
fn spanned_group(delimiter: Delimiter, stream: TokenStream, span: Span) -> Group {
    let mut group = Group::new(delimiter, stream);
    group.set_span(span);
    group
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

    let mut bang = Punct::new('!', Spacing::Alone);
    bang.set_span(span);

    let mut expanded = TokenStream::new();
    expanded.extend([
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(bang),
        TokenTree::Group(spanned_group(Delimiter::Brace, body, span)),
    ]);
    expanded
}
