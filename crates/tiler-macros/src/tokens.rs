//! A span-generic reading of the tokens one invocation was handed.
//!
//! # Why the tokens are copied instead of parsed in place
//!
//! `proc_macro::Span` cannot be constructed outside an expanding macro and
//! `proc_macro::TokenStream::from_str` panics there too, so a parser written
//! directly against those types compiles and can never be run by a test. Its
//! diagnostics would then be evidence of nothing — which is the same reason
//! [`crate::binding`] is generic over its span rather than naming
//! `proc_macro::Span`.
//!
//! [`Tree`] is therefore the shape the grammar reads: the same four token
//! kinds the stable proc-macro API exposes, carrying an opaque span type. The
//! expansion supplies `proc_macro::Span` through [`read`]; the tests supply a
//! marker they can assert on. [`read`] itself is the one part no test reaches,
//! and it is deliberately the smallest possible part — a total, span-preserving
//! copy that decides nothing.
//!
//! # Raw identifiers are refused rather than carried
//!
//! `proc_macro::Ident::to_string` renders `r#type` with its `r#` prefix, and
//! `Ident::new` panics on a name spelled that way. Carrying one would mean a
//! region that parsed and then aborted rustc with no span at all during
//! emission. [`Tree::Ident`] records the prefix instead, and the grammar
//! refuses it at the token that carries it.

use proc_macro::{Delimiter as ProcDelimiter, Span, TokenStream, TokenTree};

/// The delimiter of one token group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Delimiter {
    /// `( … )`.
    Parenthesis,
    /// `{ … }`.
    Brace,
    /// `[ … ]`.
    Bracket,
    /// An invisible group, produced by another macro's expansion.
    Invisible,
}

impl Delimiter {
    /// Returns the spelling a diagnostic should use for this delimiter.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Parenthesis => "( … )",
            Self::Brace => "{ … }",
            Self::Bracket => "[ … ]",
            Self::Invisible => "an invisible group",
        }
    }
}

/// One token of an invocation, carrying the span it was written at.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Tree<S> {
    /// An identifier or keyword.
    Ident {
        /// The name as written, without any raw prefix.
        name: String,
        /// Whether it was written as a raw identifier (`r#name`).
        raw: bool,
        /// The token.
        span: S,
    },
    /// One punctuation character.
    Punct {
        /// The character.
        character: char,
        /// Whether the next token continues this operator (`+=` is two joint
        /// puncts). Retained so a two-character operator is refused as one
        /// operator rather than as two unrelated ones.
        joint: bool,
        /// The token.
        span: S,
    },
    /// A literal, kept as its source text because the only literals this
    /// grammar admits are unsuffixed non-negative integers.
    Literal {
        /// The literal's source text.
        text: String,
        /// The token.
        span: S,
    },
    /// A delimited group.
    Group {
        /// Its delimiter.
        delimiter: Delimiter,
        /// Its contents.
        trees: Vec<Tree<S>>,
        /// The group as a whole.
        span: S,
    },
}

impl<S: Copy> Tree<S> {
    /// Returns the span this token was written at.
    pub(crate) const fn span(&self) -> S {
        match self {
            Self::Ident { span, .. }
            | Self::Punct { span, .. }
            | Self::Literal { span, .. }
            | Self::Group { span, .. } => *span,
        }
    }

    /// Returns a short description a diagnostic can name this token by.
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Ident { name, raw, .. } => {
                if *raw {
                    format!("`r#{name}`")
                } else {
                    format!("`{name}`")
                }
            }
            Self::Punct { character, .. } => format!("`{character}`"),
            Self::Literal { text, .. } => format!("`{text}`"),
            Self::Group { delimiter, .. } => delimiter.as_str().to_owned(),
        }
    }
}

/// Copies one invocation's tokens into the span-generic form the grammar reads.
///
/// Total and lossless for the four token kinds the stable API exposes: nothing
/// here rejects, normalizes, or reorders, so a grammar failure is always the
/// grammar's and never this function's.
pub(crate) fn read(stream: TokenStream) -> Vec<Tree<Span>> {
    stream
        .into_iter()
        .map(|tree| match tree {
            TokenTree::Ident(ident) => {
                let rendered = ident.to_string();
                let (name, raw) = match rendered.strip_prefix("r#") {
                    Some(stripped) => (stripped.to_owned(), true),
                    None => (rendered, false),
                };
                Tree::Ident {
                    name,
                    raw,
                    span: ident.span(),
                }
            }
            TokenTree::Punct(punct) => Tree::Punct {
                character: punct.as_char(),
                joint: punct.spacing() == proc_macro::Spacing::Joint,
                span: punct.span(),
            },
            TokenTree::Literal(literal) => Tree::Literal {
                text: literal.to_string(),
                span: literal.span(),
            },
            TokenTree::Group(group) => Tree::Group {
                delimiter: match group.delimiter() {
                    ProcDelimiter::Parenthesis => Delimiter::Parenthesis,
                    ProcDelimiter::Brace => Delimiter::Brace,
                    ProcDelimiter::Bracket => Delimiter::Bracket,
                    ProcDelimiter::None => Delimiter::Invisible,
                },
                span: group.span(),
                trees: read(group.stream()),
            },
        })
        .collect()
}
