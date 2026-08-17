//! What `sym n;` binds, decided from both sides.
//!
//! Every refusal below has an accepting neighbour differing in one declaration,
//! so an acceptance is evidence about the rule rather than about the fixture.

use tiler_ir::program::StorageScalar;
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{InputKey, OutputKey};
use tiler_ir::shape::{Axis, BindingSource, FactProvenance, ShapeSymbol, SymbolScope};

use super::{
    BoundRegion, BoundResultAxis, DeclaredAxis, REGION_SCOPE, RegionBindError, RegionDeclarations,
    storage_scalar_path,
};

/// A span a test can construct and assert on.
///
/// `proc_macro::Span` panics outside an expanding macro, which is exactly why
/// [`RegionDeclarations`] is generic over the span: without this the module's
/// diagnostics could be compiled but never observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct At(u32);

const REGION: At = At(0);

fn input(key: &str) -> InputKey {
    InputKey::new(key).expect("the fixture keys are valid")
}

fn output(key: &str) -> OutputKey {
    OutputKey::new(key).expect("the fixture keys are valid")
}

fn symbol_axis(name: &str, span: u32) -> DeclaredAxis<At> {
    DeclaredAxis::Symbol {
        name: name.to_owned(),
        span: At(span),
    }
}

fn scoped(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(
        SymbolScope::new(REGION_SCOPE).expect("the region scope is nonempty"),
        name,
    )
    .expect("the fixture symbol names are valid")
}

/// `sym n; in a: f32[n], b: f32[n]; out d: f32[n]`, operands in the given order.
///
/// The `order` parameter is what makes the declaration-order property testable:
/// the same region, written two ways, must reach one environment.
fn two_operand_region(reversed: bool) -> Result<BoundRegion, RegionBindError<At>> {
    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1))?;
    let mut operands = vec![("a", 2_u32), ("b", 3)];
    if reversed {
        operands.reverse();
    }
    for (key, span) in operands {
        region.operand(
            input(key),
            StorageScalar::F32,
            vec![symbol_axis("n", span)],
            At(span),
        )?;
    }
    region.result(
        output("d"),
        StorageScalar::F32,
        vec![symbol_axis("n", 4)],
        At(4),
    )?;
    region.bind()
}

/// One symbol, one canonical source, and an equality owed by every other axis.
///
/// The environment holds exactly one root binding — ADR 0008's rule, which
/// `ShapeEnv` enforces by rejecting a second `bind` — so the second occurrence
/// cannot be a binding at all. That is why it is carried as an obligation, and
/// this test asserts both halves rather than only the one that is visible in the
/// environment.
#[test]
fn one_symbol_is_sourced_once_and_obliges_every_other_occurrence() {
    let region = two_operand_region(false).expect("the region is bindable");

    assert_eq!(region.symbols().len(), 1);
    let bound = &region.symbols()[0];
    assert_eq!(bound.symbol, scoped("n"));
    assert_eq!(bound.source.operand, 0);
    assert_eq!(bound.source.axis, Axis::new(0));
    assert_eq!(bound.obligations.len(), 1);
    assert_eq!(bound.obligations[0].operand, 1);
    assert_eq!(bound.obligations[0].axis, Axis::new(0));

    let binding = region
        .environment()
        .binding(&scoped("n"))
        .expect("the symbol is declared and bound");
    assert_eq!(
        binding.source(),
        &BindingSource::InputDimension {
            input: input("a"),
            axis: Axis::new(0),
        },
        "the environment names the interface key, not the operand's position",
    );
    // `InputDimension` floors at `LiveDevicePreflight`, and the index layer's
    // `EXTENT_PHASE_CEILING` is that same phase, so this is the one admissible
    // phase rather than a preference.
    assert_eq!(binding.phase(), AvailabilityPhase::LiveDevicePreflight);
    assert_eq!(binding.provenance(), FactProvenance::RuntimeValidated);

    assert_eq!(region.result.axes, vec![BoundResultAxis::Symbol(0)]);
}

/// Declaration order changes no part of the environment the graph identifies.
///
/// The interface key is what a binding names, so writing `in b, a` moves the
/// operand positions in the emitted table and moves nothing the environment
/// records. Without this the canonical-source rule would be "whichever occurrence
/// was written first", which is the thing the ratified decision refused.
#[test]
fn declaration_order_does_not_change_the_environment() {
    let forward = two_operand_region(false).expect("the region is bindable");
    let reversed = two_operand_region(true).expect("the region is bindable");

    assert_eq!(
        forward.environment_identity(),
        reversed.environment_identity(),
        "one interface is one environment however its `in` list was ordered",
    );
    assert_eq!(
        reversed
            .environment()
            .binding(&scoped("n"))
            .expect("the symbol is bound")
            .source(),
        &BindingSource::InputDimension {
            input: input("a"),
            axis: Axis::new(0),
        },
        "`a` sources the symbol in both orders because canonical order is over keys",
    );

    // The emitted table indexes operands, so it does move — and it must, because
    // the interface order is what a call site supplies. The environment is the
    // part identity is a function of, and it did not.
    assert_ne!(forward.facts_source(), reversed.facts_source());
}

/// A symbol nothing sources is refused at its own declaration.
#[test]
fn a_symbol_no_operand_axis_names_is_refused() {
    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1)).unwrap();
    region.declare_symbol("m", At(2)).unwrap();
    region
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 3)],
            At(3),
        )
        .unwrap();
    region
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("n", 4)],
            At(4),
        )
        .unwrap();
    assert_eq!(
        region.bind().expect_err("`m` has no source"),
        RegionBindError::UnboundSymbol {
            name: "m".to_owned(),
            span: At(2),
        },
    );

    // A result axis is not a source: the result's extent is computed from the
    // inputs, so reading it back out of a value that does not exist yet would be
    // circular. The neighbour differs only in that `m` sizes the result.
    let mut only_on_the_result = RegionDeclarations::new(REGION);
    only_on_the_result.declare_symbol("n", At(1)).unwrap();
    only_on_the_result.declare_symbol("m", At(2)).unwrap();
    only_on_the_result
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 3)],
            At(3),
        )
        .unwrap();
    only_on_the_result
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("m", 5)],
            At(4),
        )
        .unwrap();
    assert_eq!(
        only_on_the_result
            .bind()
            .expect_err("a result axis sources nothing"),
        RegionBindError::UnboundSymbol {
            name: "m".to_owned(),
            span: At(2),
        },
    );

    // And the accepting neighbour: one operand axis is enough.
    two_operand_region(false).expect("an operand axis sources the symbol");
}

/// An axis naming a symbol no `sym` declared is refused at that axis.
#[test]
fn an_undeclared_symbol_is_refused_at_the_axis_that_names_it() {
    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1)).unwrap();
    region
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 2), symbol_axis("k", 7)],
            At(2),
        )
        .unwrap();
    region
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("n", 4)],
            At(4),
        )
        .unwrap();
    assert_eq!(
        region.bind().expect_err("`k` was never declared"),
        RegionBindError::UndeclaredSymbol {
            name: "k".to_owned(),
            span: At(7),
        },
    );

    // The result is checked by the same rule, at its own span.
    let mut on_the_result = RegionDeclarations::new(REGION);
    on_the_result.declare_symbol("n", At(1)).unwrap();
    on_the_result
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 2)],
            At(2),
        )
        .unwrap();
    on_the_result
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("k", 9)],
            At(4),
        )
        .unwrap();
    assert_eq!(
        on_the_result.bind().expect_err("`k` was never declared"),
        RegionBindError::UndeclaredSymbol {
            name: "k".to_owned(),
            span: At(9),
        },
    );
}

/// A second result is refused at the declaration that crosses the bound.
///
/// The bound is this frontend's runtime-value profile, not the model's: the
/// semantic graph carries ordered named outputs, and what is missing is a
/// decided call-site shape for more than one returned value.
#[test]
fn a_second_result_is_refused_by_the_bounded_profile() {
    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1)).unwrap();
    region
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 2)],
            At(2),
        )
        .unwrap();
    region
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("n", 4)],
            At(4),
        )
        .expect("the first result is admitted");
    assert_eq!(
        region
            .result(
                output("e"),
                StorageScalar::F32,
                vec![symbol_axis("n", 5)],
                At(5),
            )
            .expect_err("a second result is outside the profile"),
        RegionBindError::UnsupportedResultCardinality {
            declared: 2,
            limit: 1,
            span: At(5),
        },
    );
    // The rejected declaration left the draft usable, so the region still binds
    // with the result it did accept.
    region.bind().expect("the first result survived");

    // No result at all is refused too, at the region rather than at a token.
    let mut resultless = RegionDeclarations::new(REGION);
    resultless.declare_symbol("n", At(1)).unwrap();
    resultless
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 2)],
            At(2),
        )
        .unwrap();
    assert_eq!(
        resultless.bind().expect_err("a region returns a value"),
        RegionBindError::UnsupportedResultCardinality {
            declared: 0,
            limit: 1,
            span: REGION,
        },
    );
}

/// A repeated declaration is rejected naming both sites, and changes nothing.
#[test]
fn a_repeated_declaration_is_rejected_naming_both_sites() {
    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1)).unwrap();
    assert_eq!(
        region
            .declare_symbol("n", At(2))
            .expect_err("one `sym` declares one variable"),
        RegionBindError::DuplicateSymbol {
            name: "n".to_owned(),
            span: At(2),
            first: At(1),
        },
    );

    region
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 3)],
            At(3),
        )
        .unwrap();
    assert_eq!(
        region
            .operand(
                input("a"),
                StorageScalar::F32,
                vec![symbol_axis("n", 4)],
                At(4),
            )
            .expect_err("an interface key names one operand"),
        RegionBindError::DuplicateOperand {
            key: input("a"),
            span: At(4),
            first: At(3),
        },
    );

    region
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("n", 5)],
            At(5),
        )
        .unwrap();
    // Both rejections left the draft exactly as it was.
    let bound = region.bind().expect("the accepted declarations survived");
    assert_eq!(bound.symbols().len(), 1);
    assert_eq!(bound.operands.len(), 1);
}

/// A region with no operands binds nothing and can construct nothing.
#[test]
fn a_region_without_operands_is_refused() {
    let mut region = RegionDeclarations::new(REGION);
    region
        .result(
            output("d"),
            StorageScalar::F32,
            vec![DeclaredAxis::Literal(4)],
            At(1),
        )
        .unwrap();
    assert_eq!(
        region.bind().expect_err("nothing supplies a value"),
        RegionBindError::NoOperands { span: REGION },
    );
}

/// Nothing an expansion emits leaves the invocation.
///
/// The accepted inline developer experience forbids a source scan, a consumer
/// `build.rs`, a registry, a prepare step, and runtime source JIT, and requires
/// generated tokens to resolve through the one crate the consumer declared. This
/// checks the emitted text against that list rather than trusting the sentence
/// above, and it names its population — every `::`-rooted path — instead of
/// reporting "no offending token found" for text it failed to scan.
#[test]
fn generated_facts_name_only_the_facade() {
    let region = two_operand_region(false).expect("the region is bindable");
    let facts = region.facts_source();

    for forbidden in [
        "include_bytes!",
        "include_str!",
        "include!",
        "std::fs",
        "std::process",
        "std::env",
        "extern crate",
        "::tiler_",
        "compile",
    ] {
        assert!(
            !facts.contains(forbidden),
            "the emitted facts contain `{forbidden}`: {facts}",
        );
    }

    // Every crate root the text names. A `::` is a path *root* exactly when the
    // character before it does not continue an identifier, which is what
    // distinguishes `::tiler` from the `::` inside `tiler::__private`.
    let roots: Vec<&str> = facts
        .match_indices("::")
        .filter(|(at, _)| {
            facts[..*at]
                .chars()
                .next_back()
                .is_none_or(|before| !(before.is_alphanumeric() || before == '_'))
        })
        .map(|(at, _)| {
            let rest = &facts[at.saturating_add(2)..];
            let end = rest
                .find(|character: char| !(character.is_alphanumeric() || character == '_'))
                .unwrap_or(rest.len());
            &rest[..end]
        })
        .collect();
    // A floor rather than an exact pin: the exact shape is pinned byte for byte
    // by the fixture comparisons below, and what this needs is evidence that the
    // scan reached past the first path instead of reporting "all clear" for text
    // it never walked.
    assert!(
        roots.len() >= 8,
        "the scan found {} rooted paths, too few to have walked these facts: {facts}",
        roots.len(),
    );
    for root in roots {
        assert_eq!(
            root, "tiler",
            "an absolute path in the emitted facts names `{root}` rather than the one crate the \
             consumer declared: {facts}",
        );
    }
}

/// The emitted text is byte-identical to what the facade's fixtures compile.
///
/// Without this the two ends are only related by having been written on the same
/// day: the fixtures would prove that *some* facts compile and bind, and the
/// emitter would prove that *some* text is produced, with nothing connecting
/// them. Reading the fixture is what makes the compile-pass evidence evidence
/// about this emitter.
#[test]
fn the_emitted_facts_are_what_the_facade_fixtures_compile() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tiler/tests/facade/pass/bind_one_symbol_from_multiple_operands.rs"
    ))
    .expect("the facade's compile-pass fixture is readable from the macro crate");
    let facts = two_operand_region(false)
        .expect("the region is bindable")
        .facts_source();
    assert!(
        source.contains(&facts),
        "the fixture no longer contains the text this emitter produces.\n\nemitted:\n{facts}\n",
    );
}

/// The single-operand region, and its fixture.
#[test]
fn the_single_operand_facts_are_what_the_facade_fixture_compiles() {
    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1)).unwrap();
    region
        .operand(
            input("a"),
            StorageScalar::F32,
            vec![symbol_axis("n", 2)],
            At(2),
        )
        .unwrap();
    region
        .result(
            output("d"),
            StorageScalar::F32,
            vec![symbol_axis("n", 3), DeclaredAxis::Literal(2)],
            At(3),
        )
        .unwrap();
    let facts = region
        .bind()
        .expect("the region is bindable")
        .facts_source();

    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tiler/tests/facade/pass/bind_one_symbol_from_one_operand.rs"
    ))
    .expect("the facade's compile-pass fixture is readable from the macro crate");
    assert!(
        source.contains(&facts),
        "the fixture no longer contains the text this emitter produces.\n\nemitted:\n{facts}\n",
    );
}

/// Every storage carrier has one exact facade path, including U32.
#[test]
fn every_storage_scalar_has_an_exact_frontend_spelling() {
    const SCALARS: [StorageScalar; 4] = [
        StorageScalar::U8,
        StorageScalar::F32,
        StorageScalar::Bf16,
        StorageScalar::U32,
    ];
    for scalar in SCALARS {
        let expected = match scalar {
            StorageScalar::U8 => "::tiler::value::StorageScalar::U8",
            StorageScalar::F32 => "::tiler::value::StorageScalar::F32",
            StorageScalar::Bf16 => "::tiler::value::StorageScalar::Bf16",
            StorageScalar::U32 => "::tiler::value::StorageScalar::U32",
        };
        assert_eq!(storage_scalar_path(scalar), expected);
    }

    let mut region = RegionDeclarations::new(REGION);
    region.declare_symbol("n", At(1)).unwrap();
    region
        .operand(
            input("token_ids"),
            StorageScalar::U32,
            vec![symbol_axis("n", 2)],
            At(2),
        )
        .unwrap();
    region
        .result(
            output("copied_ids"),
            StorageScalar::U32,
            vec![symbol_axis("n", 3)],
            At(3),
        )
        .unwrap();
    let facts = region.bind().expect("the U32 region binds").facts_source();
    assert!(
        facts.contains("::tiler::value::StorageScalar::U32"),
        "the generated facts must carry the exact U32 facade path: {facts}",
    );
}
