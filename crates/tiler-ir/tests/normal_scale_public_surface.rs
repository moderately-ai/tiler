//! Exact public-surface and authority census for normal-scale discharge.

use tiler_ir::kernel::VerifiedKernel;
use tiler_ir::schedule::{ArithmeticType, SubnormalFreedom, VerifiedScheduledRegion};
use tiler_ir::semantic::{
    AttributeFieldId, ENCODED_NUMERIC_SCALE_DOMAIN, SemanticPredicateIdentity,
    positive_normal_scalar_predicate,
};

const DRAFT_STATEMENT: &str = "/// **Draft surface, not yet accepted.**";
const NORMAL_SCALE_PUBLIC_SURFACE_COUNT: usize = 6;

struct PublicSubject {
    name: &'static str,
    source: &'static str,
    declaration: &'static str,
}

const PUBLIC_SUBJECTS: [PublicSubject; NORMAL_SCALE_PUBLIC_SURFACE_COUNT] = [
    PublicSubject {
        name: "positive_normal_scalar_predicate",
        source: include_str!("../src/semantic/precondition.rs"),
        declaration: "pub fn positive_normal_scalar_predicate()",
    },
    PublicSubject {
        name: "ENCODED_NUMERIC_SCALE_DOMAIN",
        source: include_str!("../src/semantic/quantization.rs"),
        declaration: "pub const ENCODED_NUMERIC_SCALE_DOMAIN:",
    },
    PublicSubject {
        name: "SubnormalFreedom",
        source: include_str!("../src/schedule/numerics.rs"),
        declaration: "pub enum SubnormalFreedom",
    },
    PublicSubject {
        name: "SubnormalFreedom::discharges",
        source: include_str!("../src/schedule/numerics.rs"),
        declaration: "pub const fn discharges(",
    },
    PublicSubject {
        name: "VerifiedScheduledRegion::subnormal_freedom",
        source: include_str!("../src/schedule/model.rs"),
        declaration: "pub const fn subnormal_freedom(&self)",
    },
    PublicSubject {
        name: "VerifiedKernel::subnormal_freedom",
        source: include_str!("../src/kernel/model.rs"),
        declaration: "pub const fn subnormal_freedom(&self)",
    },
];

fn declaration_position(subject: &PublicSubject) -> usize {
    let positions = subject
        .source
        .match_indices(subject.declaration)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(
        positions.len(),
        1,
        "normal-scale public symbol `{}` must have exactly one source declaration matching `{}`",
        subject.name,
        subject.declaration,
    );
    positions[0]
}

fn rustdoc_block_before(subject: &PublicSubject, position: usize) -> Vec<&str> {
    let declaration_line_start = subject.source[..position]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    assert!(
        subject.source[declaration_line_start..position]
            .trim()
            .is_empty(),
        "normal-scale public symbol `{}` declaration anchor must start after indentation only",
        subject.name,
    );
    let mut lines = subject.source[..declaration_line_start].lines().rev();

    loop {
        let line = lines.next().unwrap_or_else(|| {
            panic!(
                "normal-scale public symbol `{}` has no preceding source lines",
                subject.name
            )
        });
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            continue;
        }
        assert!(
            trimmed.starts_with("///"),
            "normal-scale public symbol `{}` must have its own rustdoc block",
            subject.name,
        );

        let mut block = vec![trimmed];
        block.extend(
            lines
                .by_ref()
                .map(str::trim)
                .take_while(|line| line.starts_with("///")),
        );
        return block;
    }
}

#[test]
fn all_six_normal_scale_exports_are_public_with_one_truthful_draft_statement() {
    let _: fn() -> SemanticPredicateIdentity = positive_normal_scalar_predicate;
    let _: AttributeFieldId = ENCODED_NUMERIC_SCALE_DOMAIN;
    let _ = SubnormalFreedom::Unproven;
    let _: fn(SubnormalFreedom, ArithmeticType) -> bool = SubnormalFreedom::discharges;
    let _: for<'a> fn(&'a VerifiedScheduledRegion) -> SubnormalFreedom =
        VerifiedScheduledRegion::subnormal_freedom;
    let _: for<'a> fn(&'a VerifiedKernel) -> SubnormalFreedom = VerifiedKernel::subnormal_freedom;

    let mut declaration_count = 0;
    let mut draft_statement_count = 0;
    for subject in &PUBLIC_SUBJECTS {
        let position = declaration_position(subject);
        declaration_count += 1;
        let rustdoc = rustdoc_block_before(subject, position);
        let subject_statement_count = rustdoc
            .iter()
            .filter(|line| **line == DRAFT_STATEMENT)
            .count();
        assert_eq!(
            subject_statement_count, 1,
            "normal-scale public symbol `{}` must carry exactly one authority statement `{DRAFT_STATEMENT}` in its own rustdoc block",
            subject.name,
        );
        draft_statement_count += subject_statement_count;
    }

    assert_eq!(declaration_count, NORMAL_SCALE_PUBLIC_SURFACE_COUNT);
    assert_eq!(draft_statement_count, NORMAL_SCALE_PUBLIC_SURFACE_COUNT);
}
