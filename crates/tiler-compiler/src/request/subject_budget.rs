//! The measured byte budget of one canonical request subject.
//!
//! A reporting control rather than a claim: it prints the preimage's size and
//! the share each component contributes, so a field added to the subject shows
//! up as a measured cost instead of being noticed only when a pinned qualifier
//! moves.

use super::*;
use crate::target::honourability::encode_declared_behaviours;

/// Reports what the canonical explain subject is made of, byte by byte.
///
/// **The decomposition is the point, not the total.** The subject is hashed
/// once per compilation to derive the explain writer's request qualifier,
/// byte at a time, and it is compared whenever a record's evidence is bound
/// to its compilation — so its size is paid on the compile path rather than
/// only when a trace is rendered. A component that is large because it
/// *expands* something already identified by a shorter injective identity is
/// redundant work; one that is large because it carries irreducible content
/// is not. Only the breakdown distinguishes those two.
#[test]
fn the_explain_subject_byte_budget() {
    let program = super::tests::program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let target = verified.for_target(0).unwrap();
    let subject = target.subject();
    let identity = &subject.semantic_identity;

    let components: [(&str, usize); 4] = [
        ("semantic graph", identity.graph().as_bytes().len()),
        (
            "reached definitions",
            identity.reached_definitions().as_bytes().len(),
        ),
        (
            "admission provenance",
            identity.admission_provenance().as_bytes().len(),
        ),
        (
            "registry snapshot",
            identity.registry_snapshot().as_bytes().len(),
        ),
    ];
    let lowering = subject.lowering_registry.as_bytes().len();
    let declared = TargetProfile::governed_declared_behaviours();
    let mut numerical_bytes = Vec::new();
    encode_declared_behaviours(&mut numerical_bytes, &declared);
    let numerical = numerical_bytes.len();
    let declaration_lines = declared.len();
    let total = subject.canonical_explain_subject_bytes().len();
    let embedded: usize = components.iter().map(|(_, size)| size).sum();

    println!("MEASURE explain subject total: {total} bytes");
    let tenths = |size: usize| size.saturating_mul(1000) / total;
    for (name, size) in components {
        let share = tenths(size);
        println!(
            "MEASURE   {name}: {size} bytes ({}.{}%)",
            share / 10,
            share % 10
        );
    }
    println!(
        "MEASURE   lowering registry identity: {lowering} bytes ({}.{}%)",
        tenths(lowering) / 10,
        tenths(lowering) % 10
    );
    {
        // Counted in the encoded bytes rather than through the registry API,
        // because the question is exactly how many times a shared value was
        // *written*, and the written form is the only place that shows.
        let registry = subject.lowering_registry.as_bytes();
        for (name, needle) in [
            ("registry snapshot", identity.registry_snapshot().as_bytes()),
            (
                "reached definitions",
                identity.reached_definitions().as_bytes(),
            ),
            (
                "admission provenance",
                identity.admission_provenance().as_bytes(),
            ),
        ] {
            let mut occurrences = 0_usize;
            let mut at = 0_usize;
            while at + needle.len() <= registry.len() {
                if &registry[at..at + needle.len()] == needle {
                    occurrences += 1;
                    at += needle.len();
                } else {
                    at += 1;
                }
            }
            println!(
                "MEASURE     {name} appears {occurrences}x in the registry identity = {} bytes",
                occurrences * needle.len(),
            );
        }
    }
    println!(
        "MEASURE   target honourability declarations: {numerical} bytes ({}.{}%) over \
         {declaration_lines} lines",
        tenths(numerical) / 10,
        tenths(numerical) % 10
    );
    println!(
        "MEASURE   everything else (keys, shapes, budgets, contracts, framing): {} bytes",
        total - embedded - lowering - numerical,
    );
}
