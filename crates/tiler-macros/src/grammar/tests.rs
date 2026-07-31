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

use super::{AxisSyntax, Expression, Operator, RegionSyntax, SyntaxError, parse};

/// A span a test can construct and assert on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct At(u32);

/// The span a refusal naming no single token is reported at.
const REGION: At = At(0);

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
        vec![AxisSyntax::Symbol(super::Name {
            text: "n".to_owned(),
            span: At(14),
        })],
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
            vec![AxisSyntax::Literal { value, span: At(5) }],
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
