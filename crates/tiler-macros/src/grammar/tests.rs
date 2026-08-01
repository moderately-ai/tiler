//! The approved region grammar, decided from both sides.
//!
//! Every refusal below has an accepting neighbour differing in one token, so an
//! acceptance is evidence about the rule rather than about the fixture. The
//! spans are integers a test can assert on, which is the whole reason
//! [`super::parse`] is generic over its span: `proc_macro::Span` cannot be
//! constructed outside an expanding macro and `TokenStream::from_str` panics
//! there, so a parser written against those types would have diagnostics no test
//! could observe.

use crate::tokens::{Delimiter, Tree};

use super::{
    AxisExtentSyntax, AxisSyntax, DeploymentMinimumSyntax, Expression, FamilyMinimumSyntax,
    Operator, RegionSyntax, ScalarSyntax, StatedDelivery, SyntaxError, parse,
};

/// A span a test can construct and assert on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct At(u32);

/// The span a refusal naming no single token is reported at.
const REGION: At = At(0);

/// One refusal case: what it is called, the tokens that cause it, and the
/// refusal they must produce.
type RefusalCase = (&'static str, Vec<Tree<At>>, SyntaxError<At>);

fn ident(name: &str, at: u32) -> Tree<At> {
    Tree::Ident {
        name: name.to_owned(),
        raw: false,
        span: At(at),
    }
}

fn raw_ident(name: &str, at: u32) -> Tree<At> {
    Tree::Ident {
        name: name.to_owned(),
        raw: true,
        span: At(at),
    }
}

fn punct(character: char, at: u32) -> Tree<At> {
    Tree::Punct {
        character,
        joint: false,
        span: At(at),
    }
}

fn joint(character: char, at: u32) -> Tree<At> {
    Tree::Punct {
        character,
        joint: true,
        span: At(at),
    }
}

fn literal(text: &str, at: u32) -> Tree<At> {
    Tree::Literal {
        text: text.to_owned(),
        span: At(at),
    }
}

fn group(delimiter: Delimiter, trees: Vec<Tree<At>>, at: u32) -> Tree<At> {
    Tree::Group {
        delimiter,
        trees,
        span: At(at),
    }
}

fn bracket(trees: Vec<Tree<At>>, at: u32) -> Tree<At> {
    group(Delimiter::Bracket, trees, at)
}

fn paren(trees: Vec<Tree<At>>, at: u32) -> Tree<At> {
    group(Delimiter::Parenthesis, trees, at)
}

/// `sym n;` at spans 1 and 2.
fn sym_n() -> Vec<Tree<At>> {
    vec![ident("sym", 1), ident("n", 2), punct(';', 3)]
}

/// `in <name>: f32[n]` at the given base span, occupying five slots.
fn operand(name: &str, base: u32) -> Vec<Tree<At>> {
    vec![
        ident(name, base),
        punct(':', base + 1),
        ident("f32", base + 2),
        bracket(vec![ident("n", base + 3)], base + 4),
    ]
}

/// The approved example: `sym n; in a: f32[n], b: f32[n], c: f32[n]; out (a * b) + c`.
fn approved_region() -> Vec<Tree<At>> {
    let mut trees = sym_n();
    trees.push(ident("in", 10));
    trees.extend(operand("a", 11));
    trees.push(punct(',', 16));
    trees.extend(operand("b", 17));
    trees.push(punct(',', 22));
    trees.extend(operand("c", 23));
    trees.push(punct(';', 28));
    trees.push(ident("out", 30));
    trees.push(paren(
        vec![ident("a", 32), punct('*', 33), ident("b", 34)],
        31,
    ));
    trees.push(punct('+', 35));
    trees.push(ident("c", 36));
    trees
}

fn parsed(trees: &[Tree<At>]) -> RegionSyntax<At> {
    parse(trees, REGION).expect("the region parses")
}

fn refused(trees: &[Tree<At>]) -> SyntaxError<At> {
    parse(trees, REGION).expect_err("the region is refused")
}

/// The approved region parses to exactly the declarations and body it states,
/// with every name carrying the token it was written at.
#[test]
fn the_approved_region_parses_to_its_declarations_and_body() {
    let region = parsed(&approved_region());

    assert_eq!(region.symbols.len(), 1);
    assert_eq!(region.symbols[0].text, "n");
    assert_eq!(region.symbols[0].span, At(2));

    let names: Vec<&str> = region
        .operands
        .iter()
        .map(|operand| operand.name.text.as_str())
        .collect();
    assert_eq!(
        names,
        ["a", "b", "c"],
        "the interface order is written order"
    );

    // The property Tom's decision turned on: an operand's element type has its
    // own token, so a dtype refusal can land on the dtype.
    assert_eq!(region.operands[0].name.span, At(11));
    assert_eq!(region.operands[0].dtype.text, "f32");
    assert_eq!(region.operands[0].dtype.span, At(13));
    assert_eq!(region.operands[1].dtype.span, At(19));
    assert_eq!(region.operands[2].dtype.span, At(25));
    assert_eq!(
        region.operands[0].axes,
        vec![AxisSyntax {
            label: None,
            extent: AxisExtentSyntax::Symbol(super::Name {
                text: "n".to_owned(),
                span: At(14),
            }),
        }],
    );

    assert!(
        region.delivery.is_none(),
        "the approved region states no `deliver`, which is what makes its absence the default",
    );

    assert_eq!(region.out, At(30));
    let Expression::Binary {
        operator,
        span,
        left,
        right,
    } = &region.body
    else {
        panic!("the body is `(a * b) + c`, whose root is `+`");
    };
    assert_eq!(*operator, Operator::Add);
    assert_eq!(*span, At(35));
    assert_eq!(
        **right,
        Expression::Operand(super::Name {
            text: "c".to_owned(),
            span: At(36),
        })
    );
    let Expression::Binary { operator, span, .. } = &**left else {
        panic!("the left operand is `a * b`");
    };
    assert_eq!(*operator, Operator::Multiply);
    assert_eq!(*span, At(33));
}

/// An empty invocation is refused at the invocation, which is the one refusal
/// that has no token of its own.
#[test]
fn an_empty_invocation_is_refused_at_the_invocation() {
    assert_eq!(refused(&[]), SyntaxError::EmptyRegion { span: REGION },);
}

/// A statement must open with one of the three keywords.
#[test]
fn a_statement_must_open_with_a_region_keyword() {
    let mut trees = vec![ident("let", 1)];
    trees.extend(approved_region());
    assert_eq!(
        refused(&trees),
        SyntaxError::ExpectedStatement {
            found: "`let`".to_owned(),
            span: At(1),
        },
    );
}

/// A declaration list ends at `;`, and anything else is refused there.
#[test]
fn a_declaration_list_must_be_terminated() {
    // `sym n in a: f32[n]; out a` — the `;` after `n` is missing.
    let mut trees = vec![ident("sym", 1), ident("n", 2), ident("in", 3)];
    trees.extend(operand("a", 11));
    trees.push(punct(';', 16));
    trees.push(ident("out", 17));
    trees.push(ident("a", 18));
    assert_eq!(
        refused(&trees),
        SyntaxError::ExpectedPunct {
            expected: ';',
            role: "or `,` after a symbol declaration",
            found: "`in`".to_owned(),
            span: At(3),
        },
    );

    // The accepting neighbour differs only in the terminator.
    let region = parsed(&{
        let mut trees = sym_n();
        trees.push(ident("in", 10));
        trees.extend(operand("a", 11));
        trees.push(punct(';', 16));
        trees.push(ident("out", 17));
        trees.push(ident("a", 18));
        trees
    });
    assert_eq!(region.symbols.len(), 1);
}

/// An operand with no bracketed shape is refused, naming the operand.
#[test]
fn an_operand_declares_its_shape_in_brackets() {
    let trees = vec![
        ident("in", 1),
        ident("a", 2),
        punct(':', 3),
        ident("f32", 4),
        punct(';', 5),
        ident("out", 6),
        ident("a", 7),
    ];
    assert_eq!(
        refused(&trees),
        SyntaxError::ExpectedShape {
            operand: "a".to_owned(),
            found: "`;`".to_owned(),
            span: At(5),
        },
    );
}

/// A rank-0 operand is written with empty brackets and parses.
#[test]
fn a_rank_zero_operand_parses() {
    let trees = vec![
        ident("in", 1),
        ident("s", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(Vec::new(), 5),
        punct(';', 6),
        ident("out", 7),
        ident("s", 8),
    ];
    assert_eq!(parsed(&trees).operands[0].axes, Vec::new());
}

/// A literal extent is a plain non-negative integer; a near miss is refused at
/// the literal rather than guessed at.
#[test]
fn a_literal_extent_must_be_a_plain_integer() {
    let with_extent = |text: &str| {
        vec![
            ident("in", 1),
            ident("a", 2),
            punct(':', 3),
            ident("f32", 4),
            bracket(vec![literal(text, 5)], 6),
            punct(';', 7),
            ident("out", 8),
            ident("a", 9),
        ]
    };

    let rejected = ["4u64", "0x10", "1.5", "-4", "\"4\"", "4_u8"];
    assert_eq!(
        rejected.len(),
        6,
        "the population this test covers is every near-miss shape, counted",
    );
    for text in rejected {
        assert_eq!(
            refused(&with_extent(text)),
            SyntaxError::MalformedExtent {
                text: text.to_owned(),
                span: At(5),
            },
            "`{text}` must be refused as an extent",
        );
    }

    // The accepting neighbours: a plain integer, and one with Rust's own digit
    // separators, because `f32[1_024]` is what a consumer writes.
    for (text, value) in [("4", 4_u64), ("1_024", 1024), ("0", 0)] {
        assert_eq!(
            parsed(&with_extent(text)).operands[0].axes,
            vec![AxisSyntax {
                label: None,
                extent: AxisExtentSyntax::Literal { value, span: At(5) },
            }],
        );
    }
}

/// A trailing comma closes an axis list, as it does in Rust's own lists.
#[test]
fn a_trailing_comma_closes_an_axis_list() {
    let trees = vec![
        ident("sym", 1),
        ident("n", 2),
        punct(';', 3),
        ident("in", 4),
        ident("a", 5),
        punct(':', 6),
        ident("f32", 7),
        bracket(vec![ident("n", 8), punct(',', 9)], 10),
        punct(';', 11),
        ident("out", 12),
        ident("a", 13),
    ];
    assert_eq!(parsed(&trees).operands[0].axes.len(), 1);
}

/// `*` binds more tightly than `+`, as it does in Rust.
#[test]
fn precedence_matches_rust() {
    let body = |trees: Vec<Tree<At>>| {
        let mut region = vec![
            ident("in", 1),
            ident("a", 2),
            punct(':', 3),
            ident("f32", 4),
            bracket(Vec::new(), 5),
            punct(',', 6),
            ident("b", 7),
            punct(':', 8),
            ident("f32", 9),
            bracket(Vec::new(), 10),
            punct(',', 11),
            ident("c", 12),
            punct(':', 13),
            ident("f32", 14),
            bracket(Vec::new(), 15),
            punct(';', 16),
            ident("out", 17),
        ];
        region.extend(trees);
        parsed(&region).body
    };

    // `a * b + c` groups as `(a * b) + c`.
    let Expression::Binary { operator, left, .. } = body(vec![
        ident("a", 20),
        punct('*', 21),
        ident("b", 22),
        punct('+', 23),
        ident("c", 24),
    ]) else {
        panic!("the root is a binary operation");
    };
    assert_eq!(operator, Operator::Add);
    assert!(matches!(
        *left,
        Expression::Binary {
            operator: Operator::Multiply,
            ..
        }
    ));

    // `a + b * c` groups as `a + (b * c)`.
    let Expression::Binary {
        operator, right, ..
    } = body(vec![
        ident("a", 20),
        punct('+', 21),
        ident("b", 22),
        punct('*', 23),
        ident("c", 24),
    ])
    else {
        panic!("the root is a binary operation");
    };
    assert_eq!(operator, Operator::Add);
    assert!(matches!(
        *right,
        Expression::Binary {
            operator: Operator::Multiply,
            ..
        }
    ));

    // Parentheses override it: `a * (b + c)`.
    let Expression::Binary {
        operator, right, ..
    } = body(vec![
        ident("a", 20),
        punct('*', 21),
        paren(vec![ident("b", 23), punct('+', 24), ident("c", 25)], 22),
    ])
    else {
        panic!("the root is a binary operation");
    };
    assert_eq!(operator, Operator::Multiply);
    assert!(matches!(
        *right,
        Expression::Binary {
            operator: Operator::Add,
            ..
        }
    ));
}

/// An operator with no registered operation is refused at the operator.
#[test]
fn an_unsupported_operator_is_refused_at_the_operator() {
    let with_operator = |trees: Vec<Tree<At>>| {
        let mut region = vec![
            ident("in", 1),
            ident("a", 2),
            punct(':', 3),
            ident("f32", 4),
            bracket(Vec::new(), 5),
            punct(',', 6),
            ident("b", 7),
            punct(':', 8),
            ident("f32", 9),
            bracket(Vec::new(), 10),
            punct(';', 11),
            ident("out", 12),
            ident("a", 13),
        ];
        region.extend(trees);
        region.push(ident("b", 99));
        region
    };

    for (spelling, tokens) in [
        ("-", vec![punct('-', 20)]),
        ("/", vec![punct('/', 20)]),
        ("%", vec![punct('%', 20)]),
        ("|", vec![punct('|', 20)]),
    ] {
        assert_eq!(
            refused(&with_operator(tokens)),
            SyntaxError::UnsupportedOperator {
                operator: spelling.to_owned(),
                span: At(20),
            },
            "`{spelling}` has no registered operation",
        );
    }

    // A multi-character operator is refused as the whole operator rather than
    // read as its first character, so `a += b` cannot parse as `a + (= b)`.
    assert_eq!(
        refused(&with_operator(vec![joint('+', 20), punct('=', 21)])),
        SyntaxError::UnsupportedOperator {
            operator: "+=".to_owned(),
            span: At(20),
        },
    );
    assert_eq!(
        refused(&with_operator(vec![joint('*', 20), punct('*', 21)])),
        SyntaxError::UnsupportedOperator {
            operator: "**".to_owned(),
            span: At(20),
        },
    );

    // The accepting neighbours differ only in the operator.
    for character in ['*', '+'] {
        parse(&with_operator(vec![punct(character, 20)]), REGION)
            .unwrap_or_else(|_| panic!("`{character}` is registered"));
    }
}

/// A named operation call is refused at its name, not at the argument list.
#[test]
fn a_named_operation_call_is_refused_at_its_name() {
    let trees = vec![
        ident("in", 1),
        ident("a", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(Vec::new(), 5),
        punct(';', 6),
        ident("out", 7),
        ident("relu", 8),
        paren(vec![ident("a", 10)], 9),
    ];
    assert_eq!(
        refused(&trees),
        SyntaxError::NamedOperationCall {
            name: "relu".to_owned(),
            span: At(8),
        },
    );
}

/// A raw identifier is refused wherever a region takes a name.
///
/// Not taste: `Ident::new` panics on a raw spelling, so carrying one would mean
/// a region that parsed and then aborted rustc during emission with no span.
#[test]
fn a_raw_identifier_is_refused_at_every_name_position() {
    // As a symbol declaration.
    assert_eq!(
        refused(&[
            ident("sym", 1),
            raw_ident("type", 2),
            punct(';', 3),
            ident("in", 4),
            ident("a", 5),
            punct(':', 6),
            ident("f32", 7),
            bracket(vec![ident("n", 8)], 9),
            punct(';', 10),
            ident("out", 11),
            ident("a", 12),
        ]),
        SyntaxError::RawIdentifier {
            name: "type".to_owned(),
            span: At(2),
        },
    );

    // As an axis.
    assert_eq!(
        refused(&[
            ident("in", 1),
            ident("a", 2),
            punct(':', 3),
            ident("f32", 4),
            bracket(vec![raw_ident("n", 5)], 6),
            punct(';', 7),
            ident("out", 8),
            ident("a", 9),
        ]),
        SyntaxError::RawIdentifier {
            name: "n".to_owned(),
            span: At(5),
        },
    );

    // As a body reference.
    assert_eq!(
        refused(&[
            ident("in", 1),
            ident("a", 2),
            punct(':', 3),
            ident("f32", 4),
            bracket(Vec::new(), 5),
            punct(';', 6),
            ident("out", 7),
            raw_ident("a", 8),
        ]),
        SyntaxError::RawIdentifier {
            name: "a".to_owned(),
            span: At(8),
        },
    );
}

/// A region with no `out` statement is refused at the invocation.
#[test]
fn a_region_without_a_body_is_refused() {
    let mut trees = sym_n();
    trees.push(ident("in", 10));
    trees.extend(operand("a", 11));
    trees.push(punct(';', 16));
    assert_eq!(refused(&trees), SyntaxError::MissingBody { span: REGION });
}

/// `out` with nothing after it is refused at the keyword.
#[test]
fn an_empty_body_is_refused_at_the_keyword() {
    let trees = vec![
        ident("in", 1),
        ident("a", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(Vec::new(), 5),
        punct(';', 6),
        ident("out", 7),
    ];
    assert_eq!(refused(&trees), SyntaxError::EmptyBody { span: At(7) });
}

/// `out` is terminal: anything after the result expression is refused there.
#[test]
fn tokens_after_the_result_are_refused() {
    let mut trees = approved_region();
    trees.push(punct(';', 90));
    assert_eq!(
        refused(&trees),
        SyntaxError::TrailingTokens {
            found: "`;`".to_owned(),
            span: At(90),
        },
    );

    // A second `out` is the same refusal, at the second keyword.
    let mut trees = approved_region();
    trees.push(ident("out", 91));
    trees.push(ident("a", 92));
    assert_eq!(
        refused(&trees),
        SyntaxError::TrailingTokens {
            found: "`out`".to_owned(),
            span: At(91),
        },
    );
}

/// A group this grammar does not admit in a body is refused at the group.
///
/// An invisible group is included deliberately: it is how another macro hands a
/// proc macro an already-parsed expression, and a region that cannot see the
/// operand names it was given must refuse rather than expand over an opaque
/// value.
#[test]
fn an_unadmitted_group_in_the_body_is_refused() {
    let with_group = |delimiter: Delimiter| {
        vec![
            ident("in", 1),
            ident("a", 2),
            punct(':', 3),
            ident("f32", 4),
            bracket(Vec::new(), 5),
            punct(';', 6),
            ident("out", 7),
            group(delimiter, vec![ident("a", 9)], 8),
        ]
    };
    for delimiter in [Delimiter::Brace, Delimiter::Bracket, Delimiter::Invisible] {
        assert_eq!(
            refused(&with_group(delimiter)),
            SyntaxError::UnsupportedGroup {
                delimiter: delimiter.as_str(),
                role: "in a region's result expression; group a subexpression with `( … )`",
                span: At(8),
            },
            "{delimiter:?} must be refused in a body",
        );
    }
    // The accepting neighbour differs only in the delimiter.
    assert_eq!(
        parsed(&with_group(Delimiter::Parenthesis)).body,
        Expression::Operand(super::Name {
            text: "a".to_owned(),
            span: At(9),
        }),
    );
}

/// An empty parenthesized subexpression is refused at the group.
#[test]
fn an_empty_parenthesized_subexpression_is_refused() {
    let trees = vec![
        ident("in", 1),
        ident("a", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(Vec::new(), 5),
        punct(';', 6),
        ident("out", 7),
        paren(Vec::new(), 8),
    ];
    assert_eq!(
        refused(&trees),
        SyntaxError::ExpectedOperandReference {
            found: "( … )".to_owned(),
            span: At(8),
        },
    );
}

/// A dangling operator is refused past the end of the region rather than
/// silently completing the expression.
#[test]
fn a_dangling_operator_is_refused() {
    let trees = vec![
        ident("in", 1),
        ident("a", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(Vec::new(), 5),
        punct(';', 6),
        ident("out", 7),
        ident("a", 8),
        punct('+', 9),
    ];
    assert_eq!(
        refused(&trees),
        SyntaxError::ExpectedOperandReference {
            found: "the end of the region".to_owned(),
            span: At(9),
        },
    );
}

/// Both declaration statements may repeat, and their entries accumulate in
/// written order.
#[test]
fn declaration_statements_may_repeat() {
    let mut trees = sym_n();
    trees.extend([ident("sym", 4), ident("m", 5), punct(';', 6)]);
    trees.push(ident("in", 10));
    trees.extend(operand("a", 11));
    trees.push(punct(';', 16));
    trees.push(ident("in", 17));
    trees.extend(operand("b", 18));
    trees.push(punct(';', 23));
    trees.push(ident("out", 24));
    trees.push(ident("a", 25));

    let region = parsed(&trees);
    assert_eq!(
        region
            .symbols
            .iter()
            .map(|symbol| symbol.text.as_str())
            .collect::<Vec<_>>(),
        ["n", "m"],
    );
    assert_eq!(
        region
            .operands
            .iter()
            .map(|operand| operand.name.text.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"],
    );
}

/// The minimal region `in a: f32[]; out a`, with one `deliver` statement
/// spliced into its declaration block.
///
/// The statement's own tokens are the caller's, so each case names the spans its
/// refusal must land on; everything around them is fixed.
fn delivering(statement: Vec<Tree<At>>) -> Vec<Tree<At>> {
    let mut trees = vec![
        ident("in", 1),
        ident("a", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(Vec::new(), 5),
        punct(';', 6),
    ];
    trees.extend(statement);
    trees.push(ident("out", 90));
    trees.push(ident("a", 91));
    trees
}

/// `deliver` at span 10, followed by the caller's tokens.
fn deliver(rest: Vec<Tree<At>>) -> Vec<Tree<At>> {
    let mut trees = vec![ident("deliver", 10)];
    trees.extend(rest);
    trees
}

fn delivery(trees: &[Tree<At>]) -> StatedDelivery<At> {
    parsed(trees)
        .delivery
        .expect("the region states a delivery policy")
        .stated
}

/// Both accepted productions parse to exactly what they state.
///
/// The hyphenated profile name is the case worth naming: Rust's lexer admits no
/// hyphen inside an identifier, so `macos-and-ios` arrives as five tokens and a
/// grammar that read one would refuse the spelling Tom accepted.
#[test]
fn the_two_delivery_productions_parse_to_what_they_state() {
    let profile = delivering(deliver(vec![
        ident("macos", 11),
        punct('-', 12),
        ident("and", 13),
        punct('-', 14),
        ident("ios", 15),
        punct(';', 16),
    ]));
    assert_eq!(
        delivery(&profile),
        StatedDelivery::Profile(super::Name {
            text: "macos-and-ios".to_owned(),
            // The first identifier: joining spans needs the unstable
            // `Span::join`, and this crate holds only stable proc-macro
            // contracts.
            span: At(11),
        }),
    );

    let list = delivering(deliver(vec![
        ident("macos", 11),
        literal("14.0", 12),
        punct(',', 13),
        ident("ios", 14),
        literal("17.0", 15),
        punct(';', 16),
    ]));
    assert_eq!(
        delivery(&list),
        StatedDelivery::Families(vec![
            FamilyMinimumSyntax {
                name: super::Name {
                    text: "macos".to_owned(),
                    span: At(11),
                },
                minimum: DeploymentMinimumSyntax {
                    major: 14,
                    minor: 0,
                    span: At(12),
                },
            },
            FamilyMinimumSyntax {
                name: super::Name {
                    text: "ios".to_owned(),
                    span: At(14),
                },
                minimum: DeploymentMinimumSyntax {
                    major: 17,
                    minor: 0,
                    span: At(15),
                },
            },
        ]),
    );

    // The keyword is retained, because a refusal about the statement as a whole
    // — a stated family nothing compiles yet — is reported there.
    assert_eq!(
        parsed(&list).delivery.expect("stated").keyword,
        At(10),
        "the statement carries its own keyword",
    );
}

/// A `deliver` statement may appear once, and a second is refused at the keyword
/// that repeats.
#[test]
fn a_second_delivery_statement_is_refused_at_its_keyword() {
    let mut statement = deliver(vec![ident("macos", 11), punct(';', 12)]);
    statement.extend([ident("deliver", 20), ident("ios", 21), punct(';', 22)]);
    assert_eq!(
        refused(&delivering(statement)),
        SyntaxError::RepeatedDeliveryStatement { span: At(20) },
    );

    // The accepting neighbour differs only in the second statement's absence.
    assert!(matches!(
        delivery(&delivering(deliver(vec![
            ident("macos", 11),
            punct(';', 12)
        ]))),
        StatedDelivery::Profile(_),
    ));
}

/// A name followed by neither `;` nor a deployment minimum is neither
/// production, and is refused at the token that is neither.
#[test]
fn a_delivery_statement_that_is_neither_production_is_refused() {
    // `deliver macos, ios;` — a name list with no minimums, which is the
    // plausible mistake between the two productions.
    assert_eq!(
        refused(&delivering(deliver(vec![
            ident("macos", 11),
            punct(',', 12),
            ident("ios", 13),
            punct(';', 14),
        ]))),
        SyntaxError::ExpectedDeliverySpecifier {
            found: "`,`".to_owned(),
            span: At(12),
        },
    );

    // `deliver macos` at the end of the declaration block.
    assert_eq!(
        refused(&delivering(deliver(vec![ident("macos", 11)]))),
        SyntaxError::ExpectedDeliverySpecifier {
            found: "`out`".to_owned(),
            span: At(90),
        },
    );

    // A hyphen with no continuation ends at the missing name rather than
    // quietly naming a profile the consumer did not write.
    assert_eq!(
        refused(&delivering(deliver(vec![
            ident("macos", 11),
            punct('-', 12),
            punct(';', 13),
        ]))),
        SyntaxError::ExpectedName {
            role: "a delivery profile or an artifact family",
            found: "`;`".to_owned(),
            span: At(13),
        },
    );

    // And an empty statement is refused where the name would have gone.
    assert_eq!(
        refused(&delivering(deliver(vec![punct(';', 11)]))),
        SyntaxError::ExpectedName {
            role: "a delivery profile or an artifact family",
            found: "`;`".to_owned(),
            span: At(11),
        },
    );
}

/// A family list states one deployment minimum per family, and its entries are
/// separated the way `sym` and `in` separate theirs.
#[test]
fn a_family_list_states_a_minimum_for_every_family() {
    // `deliver macos 14.0, ios;` — the second family states none.
    assert_eq!(
        refused(&delivering(deliver(vec![
            ident("macos", 11),
            literal("14.0", 12),
            punct(',', 13),
            ident("ios", 14),
            punct(';', 15),
        ]))),
        SyntaxError::ExpectedDeploymentMinimum {
            found: "`;`".to_owned(),
            span: At(15),
        },
    );

    // `deliver macos 14.0 ios 17.0;` — the separator is missing.
    assert_eq!(
        refused(&delivering(deliver(vec![
            ident("macos", 11),
            literal("14.0", 12),
            ident("ios", 13),
            literal("17.0", 14),
            punct(';', 15),
        ]))),
        SyntaxError::ExpectedPunct {
            expected: ';',
            role: "or `,` after an artifact family's deployment minimum",
            found: "`ios`".to_owned(),
            span: At(13),
        },
    );

    // A trailing comma closes no `deliver` list, matching `sym` and `in` rather
    // than an axis list: an entry here is a statement's declaration.
    assert_eq!(
        refused(&delivering(deliver(vec![
            ident("macos", 11),
            literal("14.0", 12),
            punct(',', 13),
            punct(';', 14),
        ]))),
        SyntaxError::ExpectedName {
            role: "an artifact family",
            found: "`;`".to_owned(),
            span: At(14),
        },
    );
}

/// A deployment minimum is a plain `<major>.<minor>`, and a near miss is refused
/// at the version rather than guessed at.
#[test]
fn a_deployment_minimum_must_be_a_plain_major_minor_version() {
    let with_minimum = |text: &str| {
        delivering(deliver(vec![
            ident("macos", 11),
            literal(text, 12),
            punct(';', 13),
        ]))
    };

    let rejected = [
        "14", "14.", ".0", "14.0f32", "1_4.0", "14.0.1", "\"14.0\"", "0x14.0", "1e4.0", "-14.0",
        "70000.0", "14.70000",
    ];
    assert_eq!(
        rejected.len(),
        12,
        "the population this test covers is every near-miss shape, counted",
    );
    for text in rejected {
        assert_eq!(
            refused(&with_minimum(text)),
            SyntaxError::MalformedDeploymentMinimum {
                text: text.to_owned(),
                span: At(12),
            },
            "`{text}` must be refused as a deployment minimum",
        );
    }

    // The accepting neighbours differ only in the literal. `14.10` is here
    // because it is a different minimum from `14.1` and the same `f64`, which is
    // why the components are read from the source text.
    for (text, major, minor) in [("14.0", 14_u16, 0_u16), ("14.1", 14, 1), ("14.10", 14, 10)] {
        let StatedDelivery::Families(families) = delivery(&with_minimum(text)) else {
            panic!("a literal minimum opens the family-list production");
        };
        assert_eq!(
            families[0].minimum,
            DeploymentMinimumSyntax {
                major,
                minor,
                span: At(12),
            },
        );
    }
}

/// Every refusal carries a span, and it is the one the fixture wrote.
///
/// The population is named and counted rather than swept, because a scan that
/// silently reached nothing would report exactly what a passing scan reports.
#[test]
fn every_refusal_carries_the_span_of_its_own_token() {
    let cases: Vec<(&str, Vec<Tree<At>>, At)> = vec![
        ("empty", Vec::new(), REGION),
        ("not-a-statement", vec![ident("let", 7)], At(7)),
        (
            "unterminated",
            vec![ident("sym", 1), ident("n", 2), ident("out", 7)],
            At(7),
        ),
        (
            "missing-shape",
            vec![
                ident("in", 1),
                ident("a", 2),
                punct(':', 3),
                ident("f32", 4),
                punct(';', 7),
            ],
            At(7),
        ),
    ];
    assert_eq!(cases.len(), 4, "the population this test covers, counted");
    for (name, trees, expected) in cases {
        assert_eq!(*refused(&trees).span(), expected, "case `{name}`");
    }
}

/// `in x: f32[rows: 2, cols: 2]; out strict_serial_sum(x * 2.0 + 1.0, [cols])`,
/// which is the reduction region every case below perturbs.
///
/// Written as one fixture rather than three so each refusal differs from an
/// acceptance in exactly one token, which is what makes an acceptance evidence
/// about the rule.
fn reduction_region(axes: Vec<Tree<At>>, body: Vec<Tree<At>>) -> Vec<Tree<At>> {
    let mut trees = vec![
        ident("in", 1),
        ident("x", 2),
        punct(':', 3),
        ident("f32", 4),
        bracket(axes, 5),
        punct(';', 6),
        ident("out", 7),
    ];
    trees.extend(body);
    trees
}

/// The two declared axis shapes: `rows: 2, cols: 2`.
fn named_axes() -> Vec<Tree<At>> {
    vec![
        ident("rows", 10),
        punct(':', 11),
        literal("2", 12),
        punct(',', 13),
        ident("cols", 14),
        punct(':', 15),
        literal("2", 16),
    ]
}

/// `strict_serial_sum(x * 2.0 + 1.0, [cols])`.
fn reduction_body() -> Vec<Tree<At>> {
    vec![
        ident("strict_serial_sum", 20),
        paren(
            vec![
                ident("x", 21),
                punct('*', 22),
                literal("2.0", 23),
                punct('+', 24),
                literal("1.0", 25),
                punct(',', 26),
                bracket(vec![ident("cols", 27)], 28),
            ],
            29,
        ),
    ]
}

/// A named axis, a scalar constant, and the one registered call all parse, and
/// each carries the token it was written at.
#[test]
fn a_reduction_over_a_named_axis_parses() {
    let region = parsed(&reduction_region(named_axes(), reduction_body()));

    // A name is the axis's, and the extent stays a literal beside it.
    assert_eq!(
        region.operands[0].axes,
        vec![
            AxisSyntax {
                label: Some(super::Name {
                    text: "rows".to_owned(),
                    span: At(10),
                }),
                extent: AxisExtentSyntax::Literal {
                    value: 2,
                    span: At(12),
                },
            },
            AxisSyntax {
                label: Some(super::Name {
                    text: "cols".to_owned(),
                    span: At(14),
                }),
                extent: AxisExtentSyntax::Literal {
                    value: 2,
                    span: At(16),
                },
            },
        ],
    );
    assert_eq!(
        region.operands[0].axes[1]
            .name()
            .map(|name| name.text.as_str()),
        Some("cols"),
    );

    let Expression::Reduction {
        span,
        operand,
        axes,
    } = &region.body
    else {
        panic!("the root is a reduction: {:?}", region.body);
    };
    assert_eq!(
        *span,
        At(20),
        "a refusal about the reduction names its call"
    );
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].text, "cols");
    assert_eq!(axes[0].span, At(27), "an axis name carries its own token");

    // The operand is the whole `x * 2.0 + 1.0`, with `,` ending it rather than
    // being refused as an operator.
    let Expression::Binary {
        operator: Operator::Add,
        left,
        right,
        ..
    } = operand.as_ref()
    else {
        panic!("the reduced expression is an addition: {operand:?}");
    };
    assert!(matches!(
        **left,
        Expression::Binary {
            operator: Operator::Multiply,
            ..
        }
    ));
    assert_eq!(
        **right,
        Expression::Scalar(ScalarSyntax {
            text: "1.0".to_owned(),
            span: At(25),
        }),
    );
}

/// A bare symbol axis names itself, so `f32[n]` needs no second spelling to be
/// reducible once symbolic extents reach the semantic layer.
#[test]
fn a_symbolic_axis_is_named_by_its_own_symbol() {
    let mut trees = vec![ident("sym", 1), ident("n", 2), punct(';', 3)];
    trees.extend(reduction_region(
        vec![ident("n", 10)],
        vec![
            ident("strict_serial_sum", 20),
            paren(
                vec![
                    ident("x", 21),
                    punct(',', 26),
                    bracket(vec![ident("n", 27)], 28),
                ],
                29,
            ),
        ],
    ));
    let region = parsed(&trees);
    assert_eq!(
        region.operands[0].axes[0]
            .name()
            .map(|name| name.text.as_str()),
        Some("n"),
    );
    assert!(region.operands[0].axes[0].label.is_none());
}

/// A scalar constant is a plain real number; a near miss is refused at the
/// literal rather than guessed at.
#[test]
fn a_scalar_constant_must_be_a_plain_real_number() {
    let with_constant = |body: Vec<Tree<At>>| reduction_region(named_axes(), body);
    let times = |trees: Vec<Tree<At>>| {
        let mut body = vec![ident("x", 20), punct('*', 21)];
        body.extend(trees);
        body
    };

    let rejected = ["2", "2.0f32", "2.0_f32", "0x10", "1.", "\"2.0\"", "2.0e"];
    assert_eq!(
        rejected.len(),
        7,
        "the population this test covers is every near-miss shape, counted",
    );
    for text in rejected {
        assert_eq!(
            refused(&with_constant(times(vec![literal(text, 22)]))),
            SyntaxError::MalformedScalarLiteral {
                text: text.to_owned(),
                span: At(22),
            },
            "`{text}` must be refused as a scalar constant",
        );
    }

    // The accepting neighbours, each differing from a rejected spelling in one
    // way, and each carrying the number as a `str::parse` reads it.
    for (text, number) in [
        ("2.0", "2.0"),
        ("1_000.5", "1000.5"),
        ("1e-6", "1e-6"),
        ("1.5E3", "1.5E3"),
    ] {
        let Expression::Binary { right, .. } =
            parsed(&with_constant(times(vec![literal(text, 22)]))).body
        else {
            panic!("`{text}` parses as the right operand of `*`");
        };
        assert_eq!(
            *right,
            Expression::Scalar(ScalarSyntax {
                text: number.to_owned(),
                span: At(22),
            }),
        );
    }

    // A leading `-` signs the literal rather than naming an operator.
    let Expression::Binary { right, .. } = parsed(&with_constant(times(vec![
        punct('-', 22),
        literal("1.5", 23),
    ])))
    .body
    else {
        panic!("a signed constant parses as the right operand of `*`");
    };
    assert_eq!(
        *right,
        Expression::Scalar(ScalarSyntax {
            text: "-1.5".to_owned(),
            span: At(23),
        }),
    );

    // And it signs *a literal*: negation of an expression is not an operation
    // this profile registers, so `-x` is refused at the sign.
    assert_eq!(
        refused(&with_constant(times(vec![punct('-', 22), ident("x", 23),]))),
        SyntaxError::UnsupportedOperator {
            operator: "-".to_owned(),
            span: At(22),
        },
    );
}

/// Every way a reduction can be malformed is refused at its own token.
#[test]
fn a_malformed_reduction_is_refused_at_its_own_token() {
    let call = |arguments: Vec<Tree<At>>| {
        reduction_region(
            named_axes(),
            vec![ident("strict_serial_sum", 20), paren(arguments, 29)],
        )
    };
    let cases: Vec<RefusalCase> = vec![
        (
            "no arguments at all",
            call(Vec::new()),
            SyntaxError::ExpectedOperandReference {
                found: "( … )".to_owned(),
                span: At(29),
            },
        ),
        (
            "an operand and no axes",
            call(vec![ident("x", 21)]),
            SyntaxError::ExpectedPunct {
                expected: ',',
                role: "between a reduction's operand and its axes",
                found: "the end of the region".to_owned(),
                span: At(29),
            },
        ),
        (
            "an axis argument that is not a list",
            call(vec![ident("x", 21), punct(',', 26), ident("cols", 27)]),
            SyntaxError::ExpectedReductionAxes {
                found: "`cols`".to_owned(),
                span: At(27),
            },
        ),
        (
            "an empty axis list",
            call(vec![
                ident("x", 21),
                punct(',', 26),
                bracket(Vec::new(), 28),
            ]),
            SyntaxError::EmptyReductionAxes { span: At(28) },
        ),
        (
            "an axis list holding something that is not a name",
            call(vec![
                ident("x", 21),
                punct(',', 26),
                bracket(vec![literal("1", 27)], 28),
            ]),
            SyntaxError::ExpectedName {
                role: "an axis name to reduce",
                found: "`1`".to_owned(),
                span: At(27),
            },
        ),
        (
            "a third argument",
            call(vec![
                ident("x", 21),
                punct(',', 26),
                bracket(vec![ident("cols", 27)], 28),
                punct(',', 30),
                ident("x", 31),
            ]),
            SyntaxError::TrailingTokens {
                found: "`,`".to_owned(),
                span: At(30),
            },
        ),
    ];
    assert_eq!(
        cases.len(),
        6,
        "the population this test covers is every malformed argument shape, counted",
    );
    for (label, trees, expected) in cases {
        assert_eq!(refused(&trees), expected, "case `{label}`");
    }

    // The accepting neighbour, differing from the last case by one token.
    let _parses = parsed(&call(vec![
        ident("x", 21),
        punct(',', 26),
        bracket(vec![ident("cols", 27)], 28),
    ]));
}

/// A call this profile does not register is still refused at its name, and the
/// refusal names the one it does.
#[test]
fn an_unregistered_call_is_refused_and_names_the_registered_one() {
    let refusal = refused(&reduction_region(
        named_axes(),
        vec![ident("relu", 20), paren(vec![ident("x", 21)], 29)],
    ));
    assert_eq!(
        refusal,
        SyntaxError::NamedOperationCall {
            name: "relu".to_owned(),
            span: At(20),
        },
    );
    assert!(
        refusal.to_string().contains("strict_serial_sum"),
        "the refusal must name the call that is registered: {refusal}",
    );
}

/// A trailing comma closes a reduction's axis list, as it does an operand's.
#[test]
fn a_trailing_comma_closes_a_reduction_axis_list() {
    let region = parsed(&reduction_region(
        named_axes(),
        vec![
            ident("strict_serial_sum", 20),
            paren(
                vec![
                    ident("x", 21),
                    punct(',', 26),
                    bracket(
                        vec![
                            ident("rows", 27),
                            punct(',', 30),
                            ident("cols", 31),
                            punct(',', 32),
                        ],
                        28,
                    ),
                ],
                29,
            ),
        ],
    ));
    let Expression::Reduction { axes, .. } = &region.body else {
        panic!("the root is a reduction");
    };
    assert_eq!(axes.len(), 2);
}
