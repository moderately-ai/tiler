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
//! deliver macos 26.0, ios 26.0; // a family list, when a floor must be stated
//! ```
//!
//! A fifth statement follows Tom's 2026-08-01 decision that no numerical
//! contract may be assumed: `contract` states the numerical contract the region
//! compiles under, beside the other three, and names it with one identifier.
//!
//! ```text
//! contract flush_subnormals_to_zero_f32;
//! ```
//!
//! This module owns exactly the shape of that text. It knows nothing about the
//! semantic operation registry, the shape environment, artifact delivery, the
//! numerical contract vocabulary, or what any name means — [`crate::region`]
//! decides all of that, [`crate::delivery`] decides which profiles and families
//! exist, and [`crate::numerics`] decides which contracts do. The split is what
//! keeps a syntax error reported at the token that is wrong rather than at
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
//! # Statements repeat; `deliver` and `contract` do not; the body terminates
//!
//! `sym` and `in` may each appear more than once, because nothing is gained by
//! forcing one line and a region that grows an operand should not have to
//! rewrite a list. `deliver` and `contract` may each appear at most once,
//! because two of either would be two answers to one question — which families
//! this invocation builds for, and what its arithmetic means — and nothing about
//! the text says which wins. `out` is terminal: it takes the rest of the
//! invocation as one expression, so a second `out`, a stray `;`, or anything
//! after the body is a refusal at that token rather than a silently ignored
//! tail.
//!
//! No `deliver` statement is not a missing one. Its absence resolves to
//! `FallbackOnly`, the policy every region stated before this statement existed,
//! so a region written without it expands to exactly the tokens it did before.
//!
//! No `contract` statement *is* a missing one, and that asymmetry is Tom's
//! decision rather than an oversight: an absent delivery policy builds nothing,
//! while an absent numerical contract would have to be filled in with a meaning
//! nobody wrote. This module still parses such a region — absence is a shape it
//! can represent, and the refusal names a vocabulary this module does not own —
//! so the refusal is [`crate::numerics`]'s.
//!
//! # A body reduces, and it names constants
//!
//! Candidate B also approved "named calls for operations without an operator
//! spelling", and this grammar fills exactly one of them:
//! [`STRICT_SERIAL_SUM_CALL`], over a bracketed list of axis *names*. The
//! bracket is the same one an operand's shape is written in, because it delimits
//! the same thing — axes — and a reader who has read one `in` statement already
//! knows what `[cols]` is.
//!
//! ```text
//! in x: f32[rows: 2, cols: 2];
//! out strict_serial_sum(x * 2.0 + 1.0, [cols])
//! ```
//!
//! An axis acquires a name in the shape that declares it: `f32[n]` names its
//! axis `n` — the symbol *is* the name — and `f32[cols: 8]` names an axis whose
//! extent is a literal. Without the second form no region with fixed extents
//! could reduce at all, and a fixed extent is exactly what an ahead-of-time
//! compilation needs, so the two forms are one feature rather than a convenience
//! beside it.
//!
//! `2.0` is the other half. A reduction that cannot be preceded by scalar
//! arithmetic denotes a whole program no build compiles, so a region would parse
//! and then fail; the scalar constant is what makes the reduction reachable. It
//! is written as a plain real number, and a whole one still carries its point,
//! because `x * 2` is not something the surrounding Rust would have accepted
//! either. [`crate::region`] decides what element type it has and what it
//! rounds to; this module decides only that the token is a real literal.
//!
//! Nothing here spells a *plan*. A region says which axes it sums; how many
//! kernels that becomes, and whether anything is materialized between them, is
//! the optimizer's to decide and has no spelling in this grammar.

use core::fmt;

use crate::tokens::{Delimiter, Tree};

/// The keyword introducing a symbol declaration.
const SYM_KEYWORD: &str = "sym";
/// The keyword introducing an operand declaration.
const IN_KEYWORD: &str = "in";
/// The keyword introducing the artifact-family delivery statement.
const DELIVER_KEYWORD: &str = "deliver";
/// The keyword introducing the region's numerical contract statement.
///
/// `contract` rather than `numerics`, and the difference is what the statement
/// denotes: a region does not state a field of study, it states one contract —
/// the same word the compiler's own [`NumericalContract`] type is named for, and
/// the same word `docs/integration/frontends.md` uses for the thing an expansion
/// compiles under. Every admissible name ends in the arithmetic type it speaks
/// for, so nothing about the statement reads as unqualified.
///
/// [`NumericalContract`]: tiler_compiler::session::NumericalContract
const CONTRACT_KEYWORD: &str = "contract";
/// The keyword introducing the region's result expression.
const OUT_KEYWORD: &str = "out";

/// The one named operation call this profile registers.
///
/// Named rather than spelled `sum`, and the reason is the key it resolves to:
/// the governed profile registers `tiler::strict-serial-sum-f32@1` and nothing
/// else that sums, and *strict serial* is a numerical guarantee — the result is
/// defined by a left fold in ascending contributor order — rather than an
/// implementation note. A region spelled `sum` would commit a consumer to that
/// fold without saying so, and would have to be respelled the day a
/// reassociating sum is registered beside it.
const STRICT_SERIAL_SUM_CALL: &str = "strict_serial_sum";

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

/// What fixes one declared axis's extent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AxisExtentSyntax<S> {
    /// An extent naming a declared symbol.
    Symbol(Name<S>),
    /// An extent fixed by a literal.
    Literal {
        /// The extent.
        value: u64,
        /// The token it was written at.
        span: S,
    },
}

/// One declared axis of an operand: what it is called, and how big it is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AxisSyntax<S> {
    /// The name written before the extent, as in `cols: 8`.
    pub(crate) label: Option<Name<S>>,
    /// What fixes the extent.
    pub(crate) extent: AxisExtentSyntax<S>,
}

impl<S> AxisSyntax<S> {
    /// Returns the name this axis is known by inside the region.
    ///
    /// A written label first, and a symbolic extent's own name otherwise: `n` in
    /// `f32[n]` is already a name a consumer wrote for that axis, so requiring
    /// `f32[n: n]` to reduce over it would be ceremony for nothing. A literal
    /// extent with no label has no name, and a reduction naming it is refused
    /// rather than resolved by position.
    pub(crate) const fn name(&self) -> Option<&Name<S>> {
        match (&self.label, &self.extent) {
            (Some(label), _) => Some(label),
            (None, AxisExtentSyntax::Symbol(symbol)) => Some(symbol),
            (None, AxisExtentSyntax::Literal { .. }) => None,
        }
    }
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
    /// `deliver macos 26.0, ios 26.0;` — a family list, in written order, which
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

/// One `contract` statement as its tokens spell it.
///
/// One identifier and a terminator, unlike [`DeliverySyntax`]: a contract name
/// is a single ordinary identifier, so nothing here joins hyphens and the name
/// carries the whole of what a consumer wrote. That is what lets an unknown name
/// be refused at exactly the token that names it, rather than at the first
/// fragment of a name Rust's lexer split.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ContractSyntax<S> {
    /// The `contract` keyword, which is the token a refusal about the statement
    /// as a whole is reported at.
    pub(crate) keyword: S,
    /// The contract's name, whose vocabulary [`crate::numerics`] owns.
    pub(crate) name: Name<S>,
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

/// One scalar constant as its tokens spell it.
///
/// The source text rather than a parsed number, for the reason
/// [`DeploymentMinimumSyntax`] is read from text: what a literal *means* depends
/// on the element type it is a constant of, and this module knows no element
/// types. [`crate::region`] resolves the text against the profile's registered
/// scalar-constant operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScalarSyntax<S> {
    /// The literal as a number is written: any leading `-`, and no digit
    /// separators.
    pub(crate) text: String,
    /// The literal token, which is where a refusal about the value lands.
    pub(crate) span: S,
}

/// The region's result expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Expression<S> {
    /// A reference to a declared operand.
    Operand(Name<S>),
    /// A scalar constant.
    Scalar(ScalarSyntax<S>),
    /// A strict serial sum over named axes of one subexpression.
    Reduction {
        /// The call name, which is where a refusal about the reduction lands.
        span: S,
        /// The subexpression being reduced.
        operand: Box<Expression<S>>,
        /// The axes named for reduction, in the order written. Never empty: the
        /// grammar admits no `[]`, so a reduction over nothing cannot be
        /// spelled.
        axes: Vec<Name<S>>,
    },
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
    /// The one `contract` statement, or nothing when the region states none.
    ///
    /// `Option` because this module reports the *shape* of the text and a region
    /// with no such statement is a shape it can read. That it is not a region
    /// anything may expand is [`crate::numerics`]'s refusal, made where the
    /// admissible names it must offer are known.
    pub(crate) contract: Option<ContractSyntax<S>>,
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
    /// A literal in the body is not a plain real number.
    MalformedScalarLiteral {
        /// The literal as written.
        text: String,
        /// The token.
        span: S,
    },
    /// A reduction's axis list is missing, so what it reduces is unstated.
    ExpectedReductionAxes {
        /// What was found.
        found: String,
        /// Where.
        span: S,
    },
    /// A reduction's axis list names no axis.
    EmptyReductionAxes {
        /// The bracketed list.
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
    /// The region states more than one `contract` statement.
    RepeatedContractStatement {
        /// The second `contract` keyword.
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
    /// The body spells a named operation call this profile does not register.
    ///
    /// The approved syntax reserves named calls for operations without an
    /// operator spelling, and this profile fills exactly one of them. Refusing
    /// at the name keeps the rest of the reservation open without inventing what
    /// it would mean.
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
                "expected `{SYM_KEYWORD}`, `{IN_KEYWORD}`, `{DELIVER_KEYWORD}`, \
                 `{CONTRACT_KEYWORD}`, or `{OUT_KEYWORD}` and found {found}; a region is a \
                 declaration block followed by one `{OUT_KEYWORD}` expression"
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
                "expected an axis — a declared symbol, a literal extent, or either of those under a \
                 name, as in `cols: 8` — and found {found}"
            ),
            Self::MalformedExtent { text, .. } => write!(
                formatter,
                "`{text}` is not a literal extent; a fixed axis is written as a plain non-negative \
                 integer with no suffix, as in `f32[4]`"
            ),
            Self::MalformedScalarLiteral { text, .. } => write!(
                formatter,
                "`{text}` is not a scalar constant; a region writes one as a plain real number with \
                 no suffix, as in `2.0`, `-1.5`, or `1e-6`, and a whole number still carries its \
                 point because `x * 2` is not what the surrounding Rust would have accepted either"
            ),
            Self::ExpectedReductionAxes { found, .. } => write!(
                formatter,
                "expected the bracketed axis names `{STRICT_SERIAL_SUM_CALL}` reduces and found \
                 {found}; a reduction states which axes it sums, as in `{STRICT_SERIAL_SUM_CALL}(x \
                 * 2.0 + 1.0, [cols])`"
            ),
            Self::EmptyReductionAxes { .. } => write!(
                formatter,
                "`{STRICT_SERIAL_SUM_CALL}` is given no axis to reduce; name at least one axis of \
                 the expression it sums, as in `[cols]`, because a reduction over nothing is the \
                 expression itself"
            ),
            Self::ExpectedDeliverySpecifier { found, .. } => write!(
                formatter,
                "expected `;` after a delivery profile name, or a deployment minimum such as \
                 `14.0` after an artifact family, and found {found}; a `{DELIVER_KEYWORD}` \
                 statement names either one profile, as in `{DELIVER_KEYWORD} macos-and-ios;`, or \
                 a family list stating each family's own floor, as in `{DELIVER_KEYWORD} macos \
                 26.0, ios 26.0;`"
            ),
            Self::ExpectedDeploymentMinimum { found, .. } => write!(
                formatter,
                "expected an artifact family's deployment minimum, written as `<major>.<minor>`, \
                 and found {found}; a family list states one for every family it names, as in \
                 `{DELIVER_KEYWORD} macos 26.0, ios 26.0;`"
            ),
            Self::MalformedDeploymentMinimum { text, .. } => write!(
                formatter,
                "`{text}` is not a deployment minimum; it is written as `<major>.<minor>` with no \
                 suffix and no digit separators, as in `{DELIVER_KEYWORD} ios 26.0;`"
            ),
            Self::RepeatedDeliveryStatement { .. } => write!(
                formatter,
                "this region already states a `{DELIVER_KEYWORD}` statement; a region states its \
                 artifact-family delivery policy once, because two statements would be two \
                 policies for one invocation and nothing here says which one delivers"
            ),
            Self::RepeatedContractStatement { .. } => write!(
                formatter,
                "this region already states a `{CONTRACT_KEYWORD}` statement; a region states the \
                 numerical contract it compiles under once, because two statements would be two \
                 answers to what one region's arithmetic means and nothing here says which one it \
                 computes"
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
                "expected a declared operand, a scalar constant, a `{STRICT_SERIAL_SUM_CALL}(…)`, \
                 or a parenthesized expression and found {found}"
            ),
            Self::UnsupportedOperator { operator, .. } => write!(
                formatter,
                "`{operator}` is not an operation this region vocabulary carries; the governed \
                 semantic profile registers `*` and `+` over `f32`, and an operation with no \
                 public logical spelling is refused rather than approximated"
            ),
            Self::NamedOperationCall { name, .. } => write!(
                formatter,
                "`{name}(…)` is a named operation call this profile does not register; \
                 `{STRICT_SERIAL_SUM_CALL}` is the one it does, and the approved syntax reserves \
                 the form for operations without an operator spelling rather than inventing a \
                 meaning the public logical program cannot express"
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
            | Self::MalformedScalarLiteral { span, .. }
            | Self::ExpectedReductionAxes { span, .. }
            | Self::EmptyReductionAxes { span }
            | Self::ExpectedDeliverySpecifier { span, .. }
            | Self::ExpectedDeploymentMinimum { span, .. }
            | Self::MalformedDeploymentMinimum { span, .. }
            | Self::RepeatedDeliveryStatement { span }
            | Self::RepeatedContractStatement { span }
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
    let mut contract = None;

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
            Tree::Ident { name, raw, span } if !raw && name == CONTRACT_KEYWORD => {
                let keyword = *span;
                let _keyword = cursor.advance();
                // Refused before the statement is read, for the reason a second
                // `deliver` is: the repetition is what is wrong, and it is
                // reported at the keyword that repeats rather than at whatever
                // its name turns out to be.
                if contract.is_some() {
                    return Err(SyntaxError::RepeatedContractStatement { span: keyword });
                }
                contract = Some(parse_contract_statement(&mut cursor, keyword)?);
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
                    contract,
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

/// Reads `deliver macos-and-ios;` or `deliver macos 26.0, ios 26.0;` after its
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

/// Reads `contract flush_subnormals_to_zero_f32;` after its keyword.
///
/// One ordinary identifier, then the terminator every declaration statement
/// ends with. There is no list and no second production, because a region
/// compiles under one numerical contract: a list would be several meanings for
/// one program, and this grammar has no spelling for choosing between them.
///
/// The name is taken by [`Cursor::name`] rather than by
/// [`parse_hyphenated_name`], which is what keeps a refusal about the vocabulary
/// on the whole name: a hyphenated spelling would arrive as several tokens and
/// carry only the first one's span, because joining spans needs the unstable
/// `Span::join`.
fn parse_contract_statement<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    keyword: S,
) -> Result<ContractSyntax<S>, SyntaxError<S>> {
    let name = cursor.name("a numerical contract name", keyword)?;
    let _terminator = cursor.punct(';', "after a numerical contract name", name.span)?;
    Ok(ContractSyntax { keyword, name })
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
        axes.push(parse_axis(&mut cursor, shape)?);

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

/// Reads one axis: an optional name, then what fixes its extent.
///
/// The two productions are told apart by the token *after* the first
/// identifier, and by nothing else: a `:` makes it a name, and anything else
/// makes it a symbolic extent. One token of lookahead, decided without knowing
/// which symbols exist, which is what keeps `sym` resolution in
/// [`crate::binding`].
fn parse_axis<S: Copy>(
    cursor: &mut Cursor<'_, S>,
    shape: S,
) -> Result<AxisSyntax<S>, SyntaxError<S>> {
    let label = if names_an_axis(cursor) {
        let label = cursor.name("an axis name", shape)?;
        let _colon = cursor.punct(':', "after an axis name", label.span)?;
        Some(label)
    } else {
        None
    };
    let previous = label.as_ref().map_or(shape, |label| label.span);

    let extent = match cursor.peek() {
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
            let declared = AxisExtentSyntax::Symbol(Name {
                text: name.clone(),
                span: *span,
            });
            let _consumed = cursor.advance();
            declared
        }
        Some(Tree::Literal { text, span }) => {
            let declared = AxisExtentSyntax::Literal {
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
            let (found, span) = cursor.found_or(previous);
            return Err(SyntaxError::ExpectedAxis { found, span });
        }
    };
    Ok(AxisSyntax { label, extent })
}

/// Reports whether the cursor sits on `<name> :`, which names an axis.
///
/// A raw identifier is deliberately not matched, so `r#type: 4` reaches the
/// extent arm and is refused as the raw identifier it is rather than accepted as
/// a name this frontend cannot re-emit.
fn names_an_axis<S: Copy>(cursor: &Cursor<'_, S>) -> bool {
    matches!(cursor.peek(), Some(Tree::Ident { raw: false, .. }))
        && matches!(
            cursor.peek_at(1),
            Some(Tree::Punct {
                character: ':',
                joint: false,
                ..
            })
        )
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

/// Reads one scalar constant's source text as a real number, separators removed.
///
/// The admitted form is Rust's own `<digits>[.<digits>][e[+-]<digits>]` with at
/// least a fraction or an exponent, and digit separators are accepted because
/// `1_000.0` is what a consumer writes. A suffix, a radix prefix, a sign inside
/// the mantissa, and a bare integer are all refused, for the reason
/// [`literal_extent`] refuses its own near misses and one more: `x * 2` is not
/// something the surrounding Rust would have accepted against an `f32`, so a
/// region that accepted it would read like Rust and mean something Rust does
/// not.
///
/// What this does *not* decide is the value. Rounding a decimal to a binary
/// format is a question about that format, and this module knows no element
/// types; [`crate::region`] converts the text it returns.
fn real_literal(text: &str) -> Option<String> {
    let stripped: String = text.chars().filter(|character| *character != '_').collect();
    let bytes = stripped.as_bytes();
    let digits = |from: usize| {
        let end = bytes[from..]
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .map_or(bytes.len(), |offset| from.saturating_add(offset));
        (end > from).then_some(end)
    };

    let mut at = digits(0)?;
    let mut real = false;
    if bytes.get(at) == Some(&b'.') {
        at = digits(at.saturating_add(1))?;
        real = true;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at = at.saturating_add(1);
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at = at.saturating_add(1);
        }
        at = digits(at)?;
        real = true;
    }
    (real && at == bytes.len()).then_some(stripped)
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

/// Reads one operand reference, scalar constant, reduction, or parenthesized
/// subexpression.
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
            // An identifier immediately followed by `( … )` is a named call.
            // Exactly one is registered; every other name is refused at the name
            // rather than given a meaning here.
            if let Some(Tree::Group {
                delimiter: Delimiter::Parenthesis,
                trees,
                span,
            }) = cursor.peek()
            {
                let (trees, group) = (trees.clone(), *span);
                let _consumed = cursor.advance();
                if reference.text != STRICT_SERIAL_SUM_CALL {
                    return Err(SyntaxError::NamedOperationCall {
                        name: reference.text,
                        span: reference.span,
                    });
                }
                return parse_reduction(&trees, reference.span, group);
            }
            Ok(Expression::Operand(reference))
        }
        // A leading `-` belongs to the literal it signs rather than to an
        // operator: negation of an arbitrary expression is not an operation this
        // profile registers, and admitting it here would invent one. `a - b` is
        // unaffected, because a `-` in operator position never reaches this
        // function.
        Some(Tree::Punct {
            character: '-',
            span,
            ..
        }) => {
            let sign = *span;
            let _consumed = cursor.advance();
            let Some(Tree::Literal { text, span }) = cursor.peek() else {
                return Err(SyntaxError::UnsupportedOperator {
                    operator: "-".to_owned(),
                    span: sign,
                });
            };
            let (text, span) = (text.clone(), *span);
            let _consumed = cursor.advance();
            Ok(Expression::Scalar(scalar_literal(&text, span, true)?))
        }
        Some(Tree::Literal { text, span }) => {
            let (text, span) = (text.clone(), *span);
            let _consumed = cursor.advance();
            Ok(Expression::Scalar(scalar_literal(&text, span, false)?))
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

/// Reads one scalar constant from a literal's source text.
fn scalar_literal<S: Copy>(
    text: &str,
    span: S,
    negated: bool,
) -> Result<ScalarSyntax<S>, SyntaxError<S>> {
    let Some(number) = real_literal(text) else {
        return Err(SyntaxError::MalformedScalarLiteral {
            // The sign is part of what the consumer wrote, so a refusal quotes
            // it: `-2` and `2` are different mistakes to read past.
            text: if negated {
                format!("-{text}")
            } else {
                text.to_owned()
            },
            span,
        });
    };
    Ok(ScalarSyntax {
        text: if negated {
            format!("-{number}")
        } else {
            number
        },
        span,
    })
}

/// Reads `(<expression>, [<axis>, …])` after the reduction's call name.
///
/// `keyword` is the call name, which is where a refusal about the reduction as a
/// whole lands, and `group` is its parenthesized arguments, which is where one
/// about a missing argument does.
fn parse_reduction<S: Copy>(
    trees: &[Tree<S>],
    keyword: S,
    group: S,
) -> Result<Expression<S>, SyntaxError<S>> {
    let mut cursor = Cursor::new(trees);
    if cursor.peek().is_none() {
        return Err(SyntaxError::ExpectedOperandReference {
            found: Delimiter::Parenthesis.as_str().to_owned(),
            span: group,
        });
    }
    // The `,` ends the expression rather than being refused as an operator,
    // because `STATEMENT_PUNCT` holds it — the same rule that lets `out a;`
    // report a trailing token instead of an unregistered operator.
    let operand = parse_expression(&mut cursor, group)?;
    let comma = cursor.punct(',', "between a reduction's operand and its axes", group)?;

    let Some(Tree::Group {
        delimiter: Delimiter::Bracket,
        trees: named,
        span,
    }) = cursor.peek()
    else {
        let (found, span) = cursor.found_or(comma);
        return Err(SyntaxError::ExpectedReductionAxes { found, span });
    };
    let (named, list) = (named.clone(), *span);
    let _consumed = cursor.advance();
    if let Some(extra) = cursor.peek() {
        return Err(SyntaxError::TrailingTokens {
            found: extra.describe(),
            span: extra.span(),
        });
    }

    Ok(Expression::Reduction {
        span: keyword,
        operand: Box::new(operand),
        axes: parse_reduced_axes(&named, list)?,
    })
}

/// Reads the comma-separated axis names inside a reduction's brackets.
fn parse_reduced_axes<S: Copy>(trees: &[Tree<S>], list: S) -> Result<Vec<Name<S>>, SyntaxError<S>> {
    let mut cursor = Cursor::new(trees);
    if cursor.peek().is_none() {
        return Err(SyntaxError::EmptyReductionAxes { span: list });
    }
    let mut axes = Vec::new();
    loop {
        axes.push(cursor.name("an axis name to reduce", list)?);
        match cursor.peek() {
            None => return Ok(axes),
            Some(Tree::Punct {
                character: ',',
                joint: false,
                ..
            }) => {
                let _comma = cursor.advance();
                // A trailing comma closes the list, matching an operand's axes.
                if cursor.peek().is_none() {
                    return Ok(axes);
                }
            }
            Some(other) => {
                return Err(SyntaxError::ExpectedPunct {
                    expected: ',',
                    role: "between two reduced axes",
                    found: other.describe(),
                    span: other.span(),
                });
            }
        }
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

    /// Returns the token `offset` positions ahead without consuming anything.
    fn peek_at(&self, offset: usize) -> Option<&'a Tree<S>> {
        self.trees.get(self.at.saturating_add(offset))
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
