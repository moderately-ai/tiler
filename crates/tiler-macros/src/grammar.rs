//! The approved inline region grammar, read from tokens into typed syntax.
//!
//! # What this module decides, and what it deliberately does not
//!
//! Tom approved candidate B on 2026-07-30: an expression-position
//! `tiler::tensor!` with a leading declaration block, symbolic extents declared
//! once, typed operands, ordinary Rust operators where they carry the intended
//! logical operation, and `out` naming the single result expression.
//!
//! ```text
//! sym n;
//! in a: f32[n], b: f32[n], c: f32[n];
//! out (a * b) + c
//! ```
//!
//! Tom accepted a fourth statement on 2026-07-31 under
//! `accept-the-inline-artifact-family-profile-syntax`: `deliver` states the
//! region's artifact-family delivery policy, in the declaration block beside
//! `sym` and `in`, and in either of two spellings.
//!
//! ```text
//! deliver macos-and-ios;        // a named profile
//! deliver macos 14.0, ios 17.0; // a family list, when a floor must be stated
//! ```
//!
//! This module owns exactly the shape of that text. It knows nothing about the
//! semantic operation registry, the shape environment, artifact delivery, or
//! what any name means — [`crate::region`] decides all of that, and
//! [`crate::delivery`] decides which profiles and families exist. The split is
//! what keeps a syntax error reported at the token that is wrong rather than at
//! whatever later stage first noticed: an unknown profile name is refused at the
//! name by the module that owns the vocabulary, and a malformed deployment
//! minimum is refused at the version by this one.
//!
//! # Every refusal names one token
//!
//! That is the property Tom's decision turned on: candidate C was eliminated
//! because "a typed error can name the region but not the operand", and the
//! declaration block was accepted partly because it "retains token-level spans
//! for typed diagnostics". So [`SyntaxError`] carries a span on every variant,
//! and the spans are the ones the consumer's own tokens carry — the dtype error
//! lands on the dtype, not on the operand and not on the invocation.
//!
//! # Statements repeat; `deliver` does not; the body terminates
//!
//! `sym` and `in` may each appear more than once, because nothing is gained by
//! forcing one line and a region that grows an operand should not have to
//! rewrite a list. `deliver` may appear at most once, because two `deliver`
//! statements would be two delivery policies for one invocation and nothing
//! about the text says which wins. `out` is terminal: it takes the rest of the
//! invocation as one expression, so a second `out`, a stray `;`, or anything
//! after the body is a refusal at that token rather than a silently ignored
//! tail.
//!
//! No `deliver` statement is not a missing one. Its absence resolves to
//! `FallbackOnly`, the policy every region stated before this statement existed,
//! so a region written without it expands to exactly the tokens it did before.

use core::fmt;

use crate::tokens::{Delimiter, Tree};

/// The keyword introducing a symbol declaration.
const SYM_KEYWORD: &str = "sym";
/// The keyword introducing an operand declaration.
const IN_KEYWORD: &str = "in";
/// The keyword introducing the artifact-family delivery statement.
const DELIVER_KEYWORD: &str = "deliver";
/// The keyword introducing the region's result expression.
const OUT_KEYWORD: &str = "out";

/// The punctuation that separates and terminates declarations.
///
/// Neither character can appear in a region's result expression, so meeting one
/// where an operator would go means the expression has *ended* rather than that
/// an unregistered operator was spelled. Without the distinction, `out a; ` would
/// be refused as "`;` is not an operation this region vocabulary carries", which
/// names the wrong mistake.
const STATEMENT_PUNCT: [char; 2] = [';', ','];

/// One name as the region spelled it, and the token it was spelled at.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Name<S> {
    /// The name as written.
    pub(crate) text: String,
    /// The token it was written at.
    pub(crate) span: S,
}

/// One declared axis of an operand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AxisSyntax<S> {
    /// An axis naming a declared symbol.
    Symbol(Name<S>),
    /// An axis fixed by a literal extent.
    Literal {
        /// The extent.
        value: u64,
        /// The token it was written at.
        span: S,
    },
}

/// One `in` operand: its name, its element type, and its shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OperandSyntax<S> {
    /// The operand's name, which is also its stable interface key.
    pub(crate) name: Name<S>,
    /// The element type as written, resolved by [`crate::region`].
    pub(crate) dtype: Name<S>,
    /// The declared axes, outermost first.
    pub(crate) axes: Vec<AxisSyntax<S>>,
}

/// One artifact family's deployment minimum, as `<major>.<minor>`.
///
/// The components are read from the literal's *source text* rather than from a
/// parsed float: `14.10` and `14.1` are different deployment minimums and the
/// same `f64`, so a version that round-tripped through a float would silently
/// become another one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeploymentMinimumSyntax<S> {
    /// The major component.
    pub(crate) major: u16,
    /// The minor component.
    pub(crate) minor: u16,
    /// The literal it was written at.
    pub(crate) span: S,
}

/// One entry of a `deliver` family list: a family and its deployment minimum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FamilyMinimumSyntax<S> {
    /// The artifact family as written, resolved by [`crate::delivery`].
    pub(crate) name: Name<S>,
    /// The deployment minimum stated for it.
    pub(crate) minimum: DeploymentMinimumSyntax<S>,
}

/// What one `deliver` statement states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StatedDelivery<S> {
    /// `deliver macos-and-ios;` — one named profile, whose families and
    /// governed floors [`crate::delivery`] fixes.
    Profile(Name<S>),
    /// `deliver macos 14.0, ios 17.0;` — a family list, in written order, which
    /// [`crate::delivery`] canonicalizes. Never empty: the grammar admits no
    /// `deliver ;`, so an empty selection cannot be spelled.
    Families(Vec<FamilyMinimumSyntax<S>>),
}

/// One `deliver` statement as its tokens spell it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeliverySyntax<S> {
    /// The `deliver` keyword, which is the token a refusal about the statement
    /// as a whole is reported at.
    pub(crate) keyword: S,
    /// What it states.
    pub(crate) stated: StatedDelivery<S>,
}

/// One operator the region body spells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Operator {
    /// `*`.
    Multiply,
    /// `+`.
    Add,
}

impl Operator {
    /// Returns the character the region spells this operator with.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Multiply => "*",
            Self::Add => "+",
        }
    }
}

/// The region's result expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Expression<S> {
    /// A reference to a declared operand.
    Operand(Name<S>),
    /// One binary operation over two subexpressions.
    Binary {
        /// The operator.
        operator: Operator,
        /// The token it was written at.
        span: S,
        /// Its left operand.
        left: Box<Expression<S>>,
        /// Its right operand.
        right: Box<Expression<S>>,
    },
}

/// One region as its tokens spell it, before any name means anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionSyntax<S> {
    /// The invocation as a whole, for refusals that name no single token.
    pub(crate) region: S,
    /// Every `sym` declaration, in the order written.
    pub(crate) symbols: Vec<Name<S>>,
    /// Every `in` operand, in the order written, which is the interface order.
    pub(crate) operands: Vec<OperandSyntax<S>>,
    /// The one `deliver` statement, or nothing when the region states none.
    pub(crate) delivery: Option<DeliverySyntax<S>>,
    /// The `out` keyword, so a body-level refusal has a token.
    pub(crate) out: S,
    /// The result expression.
    pub(crate) body: Expression<S>,
}

/// Why an invocation's tokens are not a region.
///
/// Every variant carries the span of the token that caused it. A region-wide
/// span appears only where no single token is responsible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SyntaxError<S> {
    /// The invocation is empty.
    EmptyRegion {
        /// The invocation.
        span: S,
    },
    /// A statement was expected and something else was found.
    ExpectedStatement {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// A raw identifier was used where a region name is required.
    RawIdentifier {
        /// The name without its prefix.
        name: String,
        /// The token.
        span: S,
    },
    /// A name was expected and something else was found.
    ExpectedName {
        /// What the name would have been.
        role: &'static str,
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// A specific punctuation token was expected.
    ExpectedPunct {
        /// The character that was required.
        expected: char,
        /// Why it was required.
        role: &'static str,
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// An operand's bracketed shape is missing.
    ExpectedShape {
        /// The operand's name.
        operand: String,
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// An axis was expected and something else was found.
    ExpectedAxis {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// A literal extent is not a plain non-negative integer.
    MalformedExtent {
        /// The literal as written.
        text: String,
        /// The token.
        span: S,
    },
    /// A `deliver` statement's name is followed by neither its terminator nor a
    /// deployment minimum, so neither production is what was written.
    ExpectedDeliverySpecifier {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// A family list names a family and states no deployment minimum for it.
    ExpectedDeploymentMinimum {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// A deployment minimum is not a plain `<major>.<minor>` version.
    MalformedDeploymentMinimum {
        /// The literal as written.
        text: String,
        /// The token.
        span: S,
    },
    /// The region states more than one `deliver` statement.
    RepeatedDeliveryStatement {
        /// The second `deliver` keyword.
        span: S,
    },
    /// The region declares no `out` body.
    MissingBody {
        /// The invocation.
        span: S,
    },
    /// The `out` keyword is followed by nothing.
    EmptyBody {
        /// The `out` keyword.
        span: S,
    },
    /// An operand reference was expected in the body.
    ExpectedOperandReference {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// The body spells an operator this region vocabulary does not carry.
    ///
    /// A refusal rather than a second vocabulary: the operators this grammar
    /// admits are the ones the governed semantic operation profile registers,
    /// and inventing a spelling for an operation the public logical program
    /// cannot express is exactly what the frontend must not do.
    UnsupportedOperator {
        /// The operator as written.
        operator: String,
        /// The token.
        span: S,
    },
    /// The body spells a named operation call.
    ///
    /// The approved syntax reserves named calls for operations without an
    /// operator spelling, and this profile registers none. Refusing at the name
    /// keeps the reservation open without inventing what it would mean.
    NamedOperationCall {
        /// The name as written.
        name: String,
        /// The token.
        span: S,
    },
    /// A delimited group appears where this grammar admits none.
    UnsupportedGroup {
        /// The delimiter, as a diagnostic spells it.
        delimiter: &'static str,
        /// Why a group is not admitted here.
        role: &'static str,
        /// The group.
        span: S,
    },
    /// Tokens follow an expression that was already complete.
    TrailingTokens {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
}

impl<S> fmt::Display for SyntaxError<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegion { .. } => formatter.write_str(
                "`tiler::tensor!` was given no region; a region declares its operands and its \
                 result, as in `sym n; in a: f32[n], b: f32[n]; out a * b`",
            ),
            Self::ExpectedStatement { found, .. } => write!(
                formatter,
                "expected `{SYM_KEYWORD}`, `{IN_KEYWORD}`, `{DELIVER_KEYWORD}`, or \
                 `{OUT_KEYWORD}` and found {found}; a region is a declaration block followed by \
                 one `{OUT_KEYWORD}` expression"
            ),
            Self::RawIdentifier { name, .. } => write!(
                formatter,
                "`r#{name}` is a raw identifier, which a region cannot use as a name; a symbol, an \
                 operand, and an element type are each spelled as an ordinary identifier"
            ),
            Self::ExpectedName { role, found, .. } => {
                write!(formatter, "expected {role} and found {found}")
            }
            Self::ExpectedPunct {
                expected,
                role,
                found,
                ..
            } => write!(formatter, "expected `{expected}` {role} and found {found}"),
            Self::ExpectedShape { operand, found, .. } => write!(
                formatter,
                "operand `{operand}` declares no shape; write its axes in brackets, as in \
                 `{operand}: f32[n]`, and found {found}"
            ),
            Self::ExpectedAxis { found, .. } => write!(
                formatter,
                "expected an axis — a declared symbol or a literal extent — and found {found}"
            ),
            Self::MalformedExtent { text, .. } => write!(
                formatter,
                "`{text}` is not a literal extent; a fixed axis is written as a plain non-negative \
                 integer with no suffix, as in `f32[4]`"
            ),
            Self::ExpectedDeliverySpecifier { found, .. } => write!(
                formatter,
                "expected `;` after a delivery profile name, or a deployment minimum such as \
                 `14.0` after an artifact family, and found {found}; a `{DELIVER_KEYWORD}` \
                 statement names either one profile, as in `{DELIVER_KEYWORD} macos-and-ios;`, or \
                 a family list stating each family's own floor, as in `{DELIVER_KEYWORD} macos \
                 14.0, ios 17.0;`"
            ),
            Self::ExpectedDeploymentMinimum { found, .. } => write!(
                formatter,
                "expected an artifact family's deployment minimum, written as `<major>.<minor>`, \
                 and found {found}; a family list states one for every family it names, as in \
                 `{DELIVER_KEYWORD} macos 14.0, ios 17.0;`"
            ),
            Self::MalformedDeploymentMinimum { text, .. } => write!(
                formatter,
                "`{text}` is not a deployment minimum; it is written as `<major>.<minor>` with no \
                 suffix and no digit separators, as in `{DELIVER_KEYWORD} ios 17.0;`"
            ),
            Self::RepeatedDeliveryStatement { .. } => write!(
                formatter,
                "this region already states a `{DELIVER_KEYWORD}` statement; a region states its \
                 artifact-family delivery policy once, because two statements would be two \
                 policies for one invocation and nothing here says which one delivers"
            ),
            Self::MissingBody { .. } => write!(
                formatter,
                "this region declares no result; add `{OUT_KEYWORD} <expression>`, which is the \
                 value the invocation evaluates to"
            ),
            Self::EmptyBody { .. } => write!(
                formatter,
                "`{OUT_KEYWORD}` is followed by no expression; it names the region's single result"
            ),
            Self::ExpectedOperandReference { found, .. } => write!(
                formatter,
                "expected a declared operand or a parenthesized expression and found {found}"
            ),
            Self::UnsupportedOperator { operator, .. } => write!(
                formatter,
                "`{operator}` is not an operation this region vocabulary carries; the governed \
                 semantic profile registers `*` and `+` over `f32`, and an operation with no \
                 public logical spelling is refused rather than approximated"
            ),
            Self::NamedOperationCall { name, .. } => write!(
                formatter,
                "`{name}(…)` is a named operation call, and this profile registers none; the \
                 approved syntax reserves the form for operations without an operator spelling, \
                 and no such operation is admitted yet"
            ),
            Self::UnsupportedGroup {
                delimiter, role, ..
            } => write!(formatter, "{delimiter} is not admitted {role}"),
            Self::TrailingTokens { found, .. } => write!(
                formatter,
                "found {found} after an expression that was already complete; a region declares \
                 exactly one result, and `{OUT_KEYWORD}` takes the rest of the invocation"
            ),
        }
    }
}

impl<S> SyntaxError<S> {
    /// Returns the span this refusal must be reported at.
    pub(crate) const fn span(&self) -> &S {
        match self {
            Self::EmptyRegion { span }
            | Self::ExpectedStatement { span, .. }
            | Self::RawIdentifier { span, .. }
            | Self::ExpectedName { span, .. }
            | Self::ExpectedPunct { span, .. }
            | Self::ExpectedShape { span, .. }
            | Self::ExpectedAxis { span, .. }
            | Self::MalformedExtent { span, .. }
            | Self::ExpectedDeliverySpecifier { span, .. }
            | Self::ExpectedDeploymentMinimum { span, .. }
            | Self::MalformedDeploymentMinimum { span, .. }
            | Self::RepeatedDeliveryStatement { span }
            | Self::MissingBody { span }
            | Self::EmptyBody { span }
            | Self::ExpectedOperandReference { span, .. }
            | Self::UnsupportedOperator { span, .. }
            | Self::NamedOperationCall { span, .. }
            | Self::UnsupportedGroup { span, .. }
            | Self::TrailingTokens { span, .. } => span,
        }
    }
}

/// Reads one invocation's tokens as a region.
///
/// `region` is the span a refusal that names no single token is reported at.
///
/// # Errors
///
/// Returns the first [`SyntaxError`], carrying the span of the token that
/// caused it. One refusal rather than a list, because a stable proc macro's only
/// diagnostic channel is `compile_error!` in expression position, which is one
/// expression.
pub(crate) fn parse<S: Copy>(
    trees: &[Tree<S>],
    region: S,
) -> Result<RegionSyntax<S>, SyntaxError<S>> {
    if trees.is_empty() {
        return Err(SyntaxError::EmptyRegion { span: region });
    }

    let mut cursor = Cursor::new(trees);
    let mut symbols = Vec::new();
    let mut operands = Vec::new();
    let mut delivery = None;

    loop {
        let Some(tree) = cursor.peek() else {
            return Err(SyntaxError::MissingBody { span: region });
        };
        match tree {
            Tree::Ident { name, raw, span } if !raw && name == SYM_KEYWORD => {
                let _keyword = cursor.advance();
                parse_symbol_statement(&mut cursor, &mut symbols, *span)?;
            }
            Tree::Ident { name, raw, span } if !raw && name == IN_KEYWORD => {
                let _keyword = cursor.advance();
                parse_operand_statement(&mut cursor, &mut operands, *span)?;
            }
            Tree::Ident { name, raw, span } if !raw && name == DELIVER_KEYWORD => {
                let keyword = *span;
                let _keyword = cursor.advance();
                // Refused before the statement is read, so a second `deliver`
                // is reported at the keyword that repeats rather than at
                // whatever its own list turns out to disagree about.
                if delivery.is_some() {
                    return Err(SyntaxError::RepeatedDeliveryStatement { span: keyword });
                }
                delivery = Some(parse_delivery_statement(&mut cursor, keyword)?);
            }
            Tree::Ident { name, raw, span } if !raw && name == OUT_KEYWORD => {
                let out = *span;
                let _keyword = cursor.advance();
                let body = parse_body(&mut cursor, out)?;
                if let Some(extra) = cursor.peek() {
                    return Err(SyntaxError::TrailingTokens {
                        found: extra.describe(),
                        span: extra.span(),
                    });
                }
                return Ok(RegionSyntax {
                    region,
                    symbols,
                    operands,
                    delivery,
                    out,
                    body,
                });
            }
            other => {
                return Err(SyntaxError::ExpectedStatement {
                    found: other.describe(),
                    span: other.span(),
                });
            }
        }
    }
}

/// Reads `sym a, b;` after its keyword.
fn parse_symbol_statement<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    symbols: &mut Vec<Name<S>>,
    keyword: S,
) -> Result<(), SyntaxError<S>> {
    loop {
        symbols.push(cursor.name("a symbol name", keyword)?);
        match cursor.take_separator("or `,` after a symbol declaration", keyword)? {
            Separator::Comma => {}
            Separator::Terminator => return Ok(()),
        }
    }
}

/// Reads `in a: f32[n], b: f32[n];` after its keyword.
fn parse_operand_statement<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    operands: &mut Vec<OperandSyntax<S>>,
    keyword: S,
) -> Result<(), SyntaxError<S>> {
    loop {
        operands.push(parse_operand(cursor, keyword)?);
        match cursor.take_separator("or `,` after an operand declaration", keyword)? {
            Separator::Comma => {}
            Separator::Terminator => return Ok(()),
        }
    }
}

/// Reads `deliver macos-and-ios;` or `deliver macos 14.0, ios 17.0;` after its
/// keyword.
///
/// The two productions are told apart by the token *after* the first name and
/// by nothing else: `;` ends a profile, and a literal opens a family list. That
/// is decidable with one token of lookahead and without knowing which names
/// exist, which is what keeps the vocabulary in [`crate::delivery`] where a
/// widened profile list changes no parsing rule.
fn parse_delivery_statement<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    keyword: S,
) -> Result<DeliverySyntax<S>, SyntaxError<S>> {
    let name = parse_hyphenated_name(cursor, "a delivery profile or an artifact family", keyword)?;
    match cursor.peek() {
        Some(Tree::Punct {
            character: ';',
            joint: false,
            ..
        }) => {
            let _terminator = cursor.advance();
            Ok(DeliverySyntax {
                keyword,
                stated: StatedDelivery::Profile(name),
            })
        }
        Some(Tree::Literal { .. }) => {
            let mut families = Vec::new();
            let mut family = name;
            loop {
                let minimum = parse_deployment_minimum(cursor, family.span)?;
                families.push(FamilyMinimumSyntax {
                    name: family,
                    minimum,
                });
                match cursor.take_separator(
                    "or `,` after an artifact family's deployment minimum",
                    minimum.span,
                )? {
                    Separator::Terminator => {
                        return Ok(DeliverySyntax {
                            keyword,
                            stated: StatedDelivery::Families(families),
                        });
                    }
                    // No trailing comma, matching `sym` and `in` rather than an
                    // axis list: a `deliver` entry is a statement's declaration.
                    Separator::Comma => {
                        family = parse_hyphenated_name(cursor, "an artifact family", minimum.span)?;
                    }
                }
            }
        }
        _ => {
            let (found, span) = cursor.found_or(name.span);
            Err(SyntaxError::ExpectedDeliverySpecifier { found, span })
        }
    }
}

/// Reads one name that may carry hyphens, as `fallback-only` does.
///
/// Rust's lexer admits no hyphen inside an identifier, so `macos-and-ios`
/// arrives as five tokens. Joining them here is what lets the accepted profile
/// vocabulary be spelled the way Tom accepted it rather than in a second
/// underscored spelling invented to suit the lexer.
///
/// The name carries the span of its *first* identifier: joining several tokens'
/// spans needs `Span::join`, which is unstable, and this crate holds only the
/// accepted stable proc-macro contracts. A hyphen followed by anything but an
/// identifier is refused at that token rather than ending the name, so
/// `deliver macos-;` names the mistake instead of reporting an unknown profile
/// called `macos`.
fn parse_hyphenated_name<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    role: &'static str,
    previous: S,
) -> Result<Name<S>, SyntaxError<S>> {
    let mut name = cursor.name(role, previous)?;
    while let Some(Tree::Punct {
        character: '-',
        joint: false,
        ..
    }) = cursor.peek()
    {
        let hyphen = cursor.advance().map_or(name.span, Tree::span);
        let continued = cursor.name(role, hyphen)?;
        name.text.push('-');
        name.text.push_str(&continued.text);
    }
    Ok(name)
}

/// Reads one `<major>.<minor>` deployment minimum.
fn parse_deployment_minimum<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    previous: S,
) -> Result<DeploymentMinimumSyntax<S>, SyntaxError<S>> {
    let Some(Tree::Literal { text, span }) = cursor.peek() else {
        let (found, span) = cursor.found_or(previous);
        return Err(SyntaxError::ExpectedDeploymentMinimum { found, span });
    };
    let (text, span) = (text.clone(), *span);
    let _consumed = cursor.advance();
    let Some((major, minor)) = literal_deployment_minimum(&text) else {
        return Err(SyntaxError::MalformedDeploymentMinimum { text, span });
    };
    Ok(DeploymentMinimumSyntax { major, minor, span })
}

/// Reads one deployment minimum from a literal's source text.
///
/// Both components are plain decimal digits fitting `u16`, which is the width
/// the driver's `DeploymentMinimum` carries. A suffix, a sign, a third
/// component, a missing component, and a digit separator are all refused: a
/// version that is *nearly* a deployment minimum decides which OS versions a
/// consumer's build excludes, so guessing at one is guessing at that.
fn literal_deployment_minimum(text: &str) -> Option<(u16, u16)> {
    let (major, minor) = text.split_once('.')?;
    let decimal = |component: &str| {
        (!component.is_empty() && component.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| component.parse().ok())
            .flatten()
    };
    Some((decimal(major)?, decimal(minor)?))
}

/// Reads one `name: dtype[axes]` operand.
fn parse_operand<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    keyword: S,
) -> Result<OperandSyntax<S>, SyntaxError<S>> {
    let name = cursor.name("an operand name", keyword)?;
    cursor.punct(':', "after an operand name", name.span)?;
    let dtype = cursor.name("an element type", name.span)?;

    let Some(Tree::Group {
        delimiter: Delimiter::Bracket,
        trees,
        span,
    }) = cursor.peek()
    else {
        let (found, span) = cursor.found_or(dtype.span);
        return Err(SyntaxError::ExpectedShape {
            operand: name.text.clone(),
            found,
            span,
        });
    };
    let (trees, span) = (trees.clone(), *span);
    let _shape = cursor.advance();

    Ok(OperandSyntax {
        name,
        dtype,
        axes: parse_axes(&trees, span)?,
    })
}

/// Reads the comma-separated axes inside one operand's brackets.
fn parse_axes<S: Copy>(trees: &[Tree<S>], shape: S) -> Result<Vec<AxisSyntax<S>>, SyntaxError<S>> {
    let mut cursor = Cursor::new(trees);
    let mut axes = Vec::new();
    if cursor.peek().is_none() {
        return Ok(axes);
    }
    loop {
        axes.push(match cursor.peek() {
            Some(Tree::Ident {
                raw: true,
                name,
                span,
            }) => {
                return Err(SyntaxError::RawIdentifier {
                    name: name.clone(),
                    span: *span,
                });
            }
            Some(Tree::Ident { name, span, .. }) => {
                let declared = AxisSyntax::Symbol(Name {
                    text: name.clone(),
                    span: *span,
                });
                let _consumed = cursor.advance();
                declared
            }
            Some(Tree::Literal { text, span }) => {
                let declared = AxisSyntax::Literal {
                    value: literal_extent(text).ok_or_else(|| SyntaxError::MalformedExtent {
                        text: text.clone(),
                        span: *span,
                    })?,
                    span: *span,
                };
                let _consumed = cursor.advance();
                declared
            }
            _ => {
                let (found, span) = cursor.found_or(shape);
                return Err(SyntaxError::ExpectedAxis { found, span });
            }
        });

        match cursor.peek() {
            None => return Ok(axes),
            Some(Tree::Punct {
                character: ',',
                joint: false,
                ..
            }) => {
                let _comma = cursor.advance();
                // A trailing comma closes the list, matching Rust's own lists.
                if cursor.peek().is_none() {
                    return Ok(axes);
                }
            }
            Some(other) => {
                return Err(SyntaxError::ExpectedPunct {
                    expected: ',',
                    role: "between two axes",
                    found: other.describe(),
                    span: other.span(),
                });
            }
        }
    }
}

/// Reads one literal extent: a plain non-negative integer with no suffix.
///
/// Rust's own digit separators are accepted because `f32[1_024]` is what a
/// consumer writes; a sign, a suffix, a radix prefix, or a float is refused, so
/// a literal that is *nearly* an extent is a refusal rather than a guess.
fn literal_extent(text: &str) -> Option<u64> {
    if text.is_empty()
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'_')
    {
        return None;
    }
    let digits: String = text.chars().filter(|character| *character != '_').collect();
    digits.parse().ok()
}

/// Reads the region's result expression, which runs to the end of the region.
fn parse_body<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    out: S,
) -> Result<Expression<S>, SyntaxError<S>> {
    if cursor.peek().is_none() {
        return Err(SyntaxError::EmptyBody { span: out });
    }
    parse_expression(cursor, out)
}

/// Reads one expression, `+` binding less tightly than `*`, as in Rust.
fn parse_expression<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    context: S,
) -> Result<Expression<S>, SyntaxError<S>> {
    let mut left = parse_product(cursor, context)?;
    while let Some(span) = cursor.take_operator(Operator::Add)? {
        let right = parse_product(cursor, span)?;
        left = Expression::Binary {
            operator: Operator::Add,
            span,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

/// Reads one `*`-joined run.
fn parse_product<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    context: S,
) -> Result<Expression<S>, SyntaxError<S>> {
    let mut left = parse_atom(cursor, context)?;
    while let Some(span) = cursor.take_operator(Operator::Multiply)? {
        let right = parse_atom(cursor, span)?;
        left = Expression::Binary {
            operator: Operator::Multiply,
            span,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

/// Reads one operand reference or parenthesized subexpression.
fn parse_atom<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    context: S,
) -> Result<Expression<S>, SyntaxError<S>> {
    match cursor.peek() {
        Some(Tree::Ident {
            name,
            raw: true,
            span,
        }) => Err(SyntaxError::RawIdentifier {
            name: name.clone(),
            span: *span,
        }),
        Some(Tree::Ident { name, span, .. }) => {
            let reference = Name {
                text: name.clone(),
                span: *span,
            };
            let _consumed = cursor.advance();
            // An identifier immediately followed by `( … )` is a call, which the
            // approved syntax reserves and this profile does not fill.
            if let Some(Tree::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            }) = cursor.peek()
            {
                return Err(SyntaxError::NamedOperationCall {
                    name: reference.text,
                    span: reference.span,
                });
            }
            Ok(Expression::Operand(reference))
        }
        Some(Tree::Group {
            delimiter: Delimiter::Parenthesis,
            trees,
            span,
        }) => {
            let (trees, span) = (trees.clone(), *span);
            let _consumed = cursor.advance();
            let mut inner = Cursor::new(&trees);
            if inner.peek().is_none() {
                return Err(SyntaxError::ExpectedOperandReference {
                    found: Delimiter::Parenthesis.as_str().to_owned(),
                    span,
                });
            }
            let expression = parse_expression(&mut inner, span)?;
            if let Some(extra) = inner.peek() {
                return Err(SyntaxError::TrailingTokens {
                    found: extra.describe(),
                    span: extra.span(),
                });
            }
            Ok(expression)
        }
        Some(Tree::Group {
            delimiter, span, ..
        }) => Err(SyntaxError::UnsupportedGroup {
            delimiter: delimiter.as_str(),
            role: "in a region's result expression; group a subexpression with `( … )`",
            span: *span,
        }),
        Some(other) => Err(SyntaxError::ExpectedOperandReference {
            found: other.describe(),
            span: other.span(),
        }),
        None => Err(SyntaxError::ExpectedOperandReference {
            found: "the end of the region".to_owned(),
            span: context,
        }),
    }
}

/// What ended one comma-separated declaration list.
enum Separator {
    /// Another entry follows.
    Comma,
    /// The statement's `;`.
    Terminator,
}

/// A position in one flat token run.
struct Cursor<'a, S> {
    trees: &'a [Tree<S>],
    at: usize,
}

impl<'a, S: Copy> Cursor<'a, S> {
    /// Opens a cursor at the start of `trees`.
    const fn new(trees: &'a [Tree<S>]) -> Self {
        Self { trees, at: 0 }
    }

    /// Returns the token at the cursor without consuming it.
    fn peek(&self) -> Option<&'a Tree<S>> {
        self.trees.get(self.at)
    }

    /// Consumes the token at the cursor.
    fn advance(&mut self) -> Option<&'a Tree<S>> {
        let tree = self.trees.get(self.at);
        if tree.is_some() {
            self.at = self.at.saturating_add(1);
        }
        tree
    }

    /// Describes the token at the cursor, or the end of the run.
    fn found_or(&self, span: S) -> (String, S) {
        self.peek().map_or_else(
            || ("the end of the region".to_owned(), span),
            |tree| (tree.describe(), tree.span()),
        )
    }

    /// Consumes one ordinary identifier.
    fn name(&mut self, role: &'static str, previous: S) -> Result<Name<S>, SyntaxError<S>> {
        match self.peek() {
            Some(Tree::Ident {
                name,
                raw: true,
                span,
            }) => Err(SyntaxError::RawIdentifier {
                name: name.clone(),
                span: *span,
            }),
            Some(Tree::Ident { name, span, .. }) => {
                let taken = Name {
                    text: name.clone(),
                    span: *span,
                };
                let _consumed = self.advance();
                Ok(taken)
            }
            _ => {
                let (found, span) = self.found_or(previous);
                Err(SyntaxError::ExpectedName { role, found, span })
            }
        }
    }

    /// Consumes one required punctuation character.
    fn punct(
        &mut self,
        expected: char,
        role: &'static str,
        previous: S,
    ) -> Result<S, SyntaxError<S>> {
        match self.peek() {
            Some(Tree::Punct {
                character,
                joint: false,
                span,
            }) if *character == expected => {
                let span = *span;
                let _consumed = self.advance();
                Ok(span)
            }
            _ => {
                let (found, span) = self.found_or(previous);
                Err(SyntaxError::ExpectedPunct {
                    expected,
                    role,
                    found,
                    span,
                })
            }
        }
    }

    /// Consumes the `,` or `;` that ends one declaration in a statement.
    ///
    /// `role` completes "expected `;` …" for the statement being read, and is
    /// passed rather than derived from the keyword so a statement's own
    /// diagnostic lives beside its own parser.
    fn take_separator(
        &mut self,
        role: &'static str,
        previous: S,
    ) -> Result<Separator, SyntaxError<S>> {
        match self.peek() {
            Some(Tree::Punct {
                character: ',',
                joint: false,
                ..
            }) => {
                let _comma = self.advance();
                Ok(Separator::Comma)
            }
            Some(Tree::Punct {
                character: ';',
                joint: false,
                ..
            }) => {
                let _terminator = self.advance();
                Ok(Separator::Terminator)
            }
            _ => {
                let (found, span) = self.found_or(previous);
                Err(SyntaxError::ExpectedPunct {
                    expected: ';',
                    role,
                    found,
                    span,
                })
            }
        }
    }

    /// Consumes one binary operator when it is the one asked for.
    ///
    /// A punct that begins a longer operator — `*=` and `+=` are two joint
    /// tokens — is refused as that whole operator rather than accepted as its
    /// first character, so `a += b` cannot read as `a + (= b)`.
    ///
    /// [`STATEMENT_PUNCT`] ends the expression instead of being refused as an
    /// operator, so the caller reports what actually went wrong.
    fn take_operator(&mut self, wanted: Operator) -> Result<Option<S>, SyntaxError<S>> {
        let Some(Tree::Punct {
            character,
            joint,
            span,
        }) = self.peek()
        else {
            return Ok(None);
        };
        let (character, joint, span) = (*character, *joint, *span);
        if !joint && STATEMENT_PUNCT.contains(&character) {
            return Ok(None);
        }
        if joint {
            return Err(SyntaxError::UnsupportedOperator {
                operator: self.joint_operator(),
                span,
            });
        }
        let found = match character {
            '*' => Operator::Multiply,
            '+' => Operator::Add,
            _ => {
                return Err(SyntaxError::UnsupportedOperator {
                    operator: character.to_string(),
                    span,
                });
            }
        };
        if found != wanted {
            return Ok(None);
        }
        let _consumed = self.advance();
        Ok(Some(span))
    }

    /// Renders the whole multi-character operator starting at the cursor.
    fn joint_operator(&self) -> String {
        let mut rendered = String::new();
        for offset in self.at..self.trees.len() {
            let Some(Tree::Punct {
                character, joint, ..
            }) = self.trees.get(offset)
            else {
                break;
            };
            rendered.push(*character);
            if !joint {
                break;
            }
        }
        rendered
    }
}

#[cfg(test)]
mod tests;
