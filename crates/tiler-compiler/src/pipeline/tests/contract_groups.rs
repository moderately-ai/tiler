use super::*;

#[test]
fn contract_groups_fall_back_after_infeasibility_and_do_not_plan_later_groups() {
    let stated = StrictF32NumericalContract::named_profile();
    let groups = vec![
        (stated[0].key, vec![("preferred", false)]),
        (stated[1].key, vec![("fallback", true)]),
        (stated[2].key, vec![("later", true)]),
    ];
    let mut evaluated = Vec::new();
    let outcome = evaluate_preferred_groups(
        &stated,
        groups,
        |item| {
            evaluated.push(item.0);
            Ok::<_, ()>(item)
        },
        |item| item.1,
        |_| (),
    )
    .unwrap();

    assert_eq!(outcome.selected_contract, Some(stated[1].key));
    assert_eq!(evaluated, ["preferred", "fallback"]);
    assert_eq!(
        outcome
            .evaluated
            .iter()
            .map(|item| item.0)
            .collect::<Vec<_>>(),
        ["preferred", "fallback"]
    );
    assert_eq!(outcome.pruned, [(("later", true), stated[1].key)]);
}

#[test]
fn contract_group_evaluation_rejects_an_unstated_contract_key() {
    let stated = StrictF32NumericalContract::named_profile();
    let error = evaluate_preferred_groups(
        &stated,
        vec![("test.unstated-contract", vec![("candidate", true)])],
        Ok::<_, CompileError>,
        |item| item.1,
        |_| {
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "semantic-portfolio-unstated-contract",
                },
            ))
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
            ProgramError::Structure {
                rule: "semantic-portfolio-unstated-contract"
            }
        ))
    ));
}
