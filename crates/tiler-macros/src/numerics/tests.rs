//! The numerical contract vocabulary, decided from both sides.
//!
//! No toolchain and no compilation: this module resolves a name into a contract
//! the compiler defines, and whether a *target* can honour the result is
//! [`crate::aot`]'s question, exercised there against a real declaration. The
//! split is what lets these cases be exhaustive without running the driver five
//! times.
//!
//! The spans are integers a test can assert on, for [`crate::grammar`]'s reason:
//! `proc_macro::Span` cannot be constructed outside an expanding macro, so a
//! refusal written against it would carry a span no test could observe.

use tiler_compiler::session::NumericalContract;

use super::{CONTRACTS, ContractRefusal, resolve, statable, stated_contract};
use crate::grammar::{ContractSyntax, Name};

/// A span a test can construct and assert on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct At(u32);

/// The span a refusal naming no single token is reported at.
const REGION: At = At(0);

/// `contract <name>;` with its keyword at span 10 and its name at span 11.
fn statement(name: &str) -> ContractSyntax<At> {
    ContractSyntax {
        keyword: At(10),
        name: Name {
            text: name.to_owned(),
            span: At(11),
        },
    }
}

/// The vocabulary is exactly the five contracts the compiler publishes, each
/// under one name, and every name resolves to the constant it spells.
///
/// The population is named rather than counted: a sixth entry, a renamed one, or
/// two names for one contract each fail here, and a check that only counted five
/// would pass on all three.
#[test]
fn every_published_contract_has_exactly_one_region_spelling() {
    let expected: [(&str, NumericalContract); 5] = [
        ("strict_f32", NumericalContract::STRICT_F32),
        (
            "flush_subnormals_to_zero_f32",
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        ),
        ("relaxed_f32", NumericalContract::RELAXED_F32),
        ("reassociate_f32", NumericalContract::REASSOCIATE_F32),
        (
            "flush_and_reassociate_f32",
            NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        ),
    ];
    assert_eq!(
        CONTRACTS, expected,
        "the vocabulary moved; a region's contract names are a public surface",
    );

    for (name, contract) in expected {
        let stated = resolve(name).unwrap_or_else(|| panic!("`{name}` must resolve"));
        assert_eq!(stated.contract(), contract, "`{name}` resolves elsewhere");
        assert_eq!(stated.name(), name, "`{name}` loses its own spelling");
        assert!(statable(contract), "`{name}` names an unstatable contract");
    }

    // Distinctness in both directions, which the table above does not itself
    // assert: two names for one contract would let a consumer state one meaning
    // two ways, and one name for two contracts is unreachable but would make
    // `resolve` order-dependent.
    let mut names: Vec<&str> = CONTRACTS.iter().map(|(name, _)| *name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), CONTRACTS.len(), "two entries share a name");
    let mut contracts: Vec<&'static str> = CONTRACTS
        .iter()
        .map(|(_, contract)| contract.key())
        .collect();
    contracts.sort_unstable();
    contracts.dedup();
    assert_eq!(
        contracts.len(),
        CONTRACTS.len(),
        "two names spell one contract",
    );
}

/// A region stating a contract resolves to it, carrying the name it was stated
/// by.
#[test]
fn a_stated_contract_resolves_to_what_it_names() {
    let stated = stated_contract(Some(&statement("flush_and_reassociate_f32")), REGION)
        .expect("the name is one this frontend publishes");
    assert_eq!(
        stated.contract(),
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
    );
    assert_eq!(stated.name(), "flush_and_reassociate_f32");
}

/// A region stating no contract is refused at the invocation, and the refusal
/// names the statement to add and every name it may take.
///
/// The absence is the whole point of Tom's 2026-08-01 decision, so the assertion
/// is on what the diagnostic *says*: a refusal that only reported "no contract"
/// would satisfy a check that the region was refused while leaving a consumer
/// with nothing to write.
#[test]
fn a_region_stating_no_contract_is_refused_at_the_invocation() {
    let refusal =
        stated_contract::<At>(None, REGION).expect_err("there is no default contract to fall to");
    assert_eq!(refusal, ContractRefusal::Unstated { span: REGION });
    assert_eq!(
        *refusal.span(),
        REGION,
        "an absence has no token of its own"
    );

    let message = refusal.to_string();
    assert!(
        message.contains("`contract` statement"),
        "the refusal must name the statement to add: {message}",
    );
    assert!(
        message.contains("contract flush_subnormals_to_zero_f32;"),
        "the refusal must show the statement written out: {message}",
    );
    for (name, _) in CONTRACTS {
        assert!(
            message.contains(&format!("`{name}`")),
            "the refusal must offer `{name}`: {message}",
        );
    }

    // The accepting neighbour, differing only in the statement's presence.
    let _stated = stated_contract(Some(&statement("strict_f32")), REGION)
        .expect("a stated contract is not refused");
}

/// A name this frontend does not publish is refused at the name, and the refusal
/// lists the admissible ones.
///
/// The near misses are chosen deliberately. `FLUSH_SUBNORMALS_TO_ZERO_F32` is
/// the constant's own Rust casing — the runner-up spelling, refused rather than
/// case-folded, so a region admits one spelling of each contract instead of two.
/// `flush-subnormals-to-zero-f32` is the `deliver` vocabulary's hyphenated style,
/// which this statement does not use. The rest are plausible inventions.
#[test]
fn an_unpublished_contract_name_is_refused_at_the_name() {
    let rejected = [
        "FLUSH_SUBNORMALS_TO_ZERO_F32",
        "flush-subnormals-to-zero-f32",
        "flush_subnormals_to_zero",
        "strict",
        "fast_math",
        "f32",
    ];
    assert_eq!(
        rejected.len(),
        6,
        "the population this test covers is every near-miss shape, counted",
    );
    for name in rejected {
        let refusal = stated_contract(Some(&statement(name)), REGION)
            .expect_err("`{name}` is not a published contract");
        assert_eq!(
            refusal,
            ContractRefusal::UnknownContract {
                name: name.to_owned(),
                span: At(11),
            },
            "`{name}` must be refused at the name it was written at",
        );
        let message = refusal.to_string();
        for (published, _) in CONTRACTS {
            assert!(
                message.contains(&format!("`{published}`")),
                "the refusal must offer `{published}`: {message}",
            );
        }
        assert!(
            message.contains("answered by the compiler when it plans the region"),
            "the refusal must not pre-answer whether a target honours a contract: {message}",
        );
        assert!(resolve(name).is_none(), "`{name}` must resolve to nothing");
    }

    // The accepting neighbour differs from the first rejected spelling only in
    // case, which is the whole of what makes the refusal above a rule about
    // spelling rather than about the contract.
    assert!(resolve("flush_subnormals_to_zero_f32").is_some());
}
