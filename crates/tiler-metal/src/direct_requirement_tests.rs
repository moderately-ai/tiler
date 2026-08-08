//! Tests for the Apple-family comparison of derived index-arithmetic requirements.
//!
//! Every case is device-free, which is the point: the negative cases name
//! hardware no machine in this workspace can produce — both reachable devices
//! report `Apple9` — and they still run in the ordinary gate because the
//! comparison takes a normalized observation rather than a device.

use tiler_ir::schedule::IndexArithmetic;

use crate::applicability::{MetalGpuFamily, MetalGpuFamilySupport};
use crate::direct_requirement::{
    AppleFamilyFloor, MetalIndexArithmeticRefusal, evaluate_against, evaluate_index_arithmetic,
    minimum_gpu_family,
};

/// The one arithmetic the governed vocabulary has, named once here.
const REQUIRED: IndexArithmetic = IndexArithmetic::CompleteU64;

/// The floor is the family Apple's table names, and its enumerator is Apple's.
///
/// Pinned against the literals rather than against `MetalGpuFamily::ALL[0]`,
/// which is a different family entirely and would make this test agree with the
/// observation vocabulary instead of with the source. The family comes from the
/// Metal Feature Set Tables row `64-bit integer math` (`Metal3 | Apple3 | —`) by
/// way of the first macOS compile profile's authority ledger; the enumerator
/// comes from `MTLDevice.h`. Nothing else may supply either.
#[test]
fn the_floor_is_the_sourced_apple_family_and_enumerator() {
    assert_eq!(
        minimum_gpu_family(REQUIRED),
        AppleFamilyFloor::Apple3,
        "the sourced `64-bit integer math` row names Apple3 as the minimum Apple family",
    );
    assert_eq!(
        AppleFamilyFloor::Apple3.apple_constant_value(),
        1003,
        "MTLDevice.h declares MTLGPUFamilyApple3 = 1003",
    );
}

/// The floor sits strictly below every family the vocabulary can report.
///
/// This is the property the module's `const` assertion enforces and the reason
/// the named-family arm may admit without comparing. Restated at run time
/// against the *whole* population, and counted, so a vocabulary that stopped
/// producing families could not make this look green.
#[test]
fn the_floor_is_below_every_observable_family() {
    let mut above = 0;
    for family in MetalGpuFamily::ALL {
        assert!(
            family.apple_constant().value() > AppleFamilyFloor::Apple3.apple_constant_value(),
            "{family} must be strictly above the Apple3 floor",
        );
        above += 1;
    }
    assert_eq!(
        above,
        MetalGpuFamily::COUNT,
        "the checked population and MetalGpuFamily::ALL must not disagree in length",
    );
    assert!(above > 0, "a population of nothing proves nothing");
}

/// Every family the vocabulary names clears the sourced floor.
#[test]
fn every_named_family_clears_the_sourced_floor() {
    let mut cleared = 0;
    for observed in MetalGpuFamily::ALL {
        assert_eq!(
            evaluate_index_arithmetic(REQUIRED, Some(MetalGpuFamilySupport::Highest(observed))),
            Ok(observed),
            "{observed} is above the sourced Apple3 floor",
        );
        cleared += 1;
    }
    assert_eq!(cleared, MetalGpuFamily::COUNT);
}

/// A device naming no family is `Unknown` against an Apple3 floor, not false.
///
/// The distinction is the whole content of this outcome. Such a device supports
/// none of `Apple5`–`Apple9`, which is consistent both with an `Apple3` device
/// that satisfies the requirement and with an `Apple1` device that does not, so
/// reporting it as unsupported would claim a fact about hardware nobody
/// observed. It still refuses, under ADR 0043's disposal of `Unknown`.
#[test]
fn a_device_naming_no_family_is_undecidable_rather_than_unsupported() {
    let refusal = evaluate_index_arithmetic(REQUIRED, Some(MetalGpuFamilySupport::NoneNamed))
        .expect_err("an undecidable device does not satisfy the requirement");
    assert_eq!(
        refusal,
        MetalIndexArithmeticRefusal::UndecidableBelowVocabulary {
            required: REQUIRED,
            floor: AppleFamilyFloor::Apple3,
            lowest_observable: MetalGpuFamily::Apple5,
        },
    );
    assert!(
        refusal.to_string().contains("unknown rather than false"),
        "the rendered cause says which of the two it is: {refusal}",
    );
}

/// An unobserved family is a different refusal from an observed absence.
///
/// Told apart because a caller repairs them differently: the first is an adapter
/// that did not ask, the second is a vocabulary that cannot decide.
#[test]
fn an_unobserved_family_refuses_as_a_gap_in_the_adapter() {
    let unobserved = evaluate_index_arithmetic(REQUIRED, None)
        .expect_err("an unasked device does not satisfy a requirement");
    assert_eq!(
        unobserved,
        MetalIndexArithmeticRefusal::Unobserved {
            required: REQUIRED,
            floor: AppleFamilyFloor::Apple3,
        },
    );
    let undecidable = evaluate_index_arithmetic(REQUIRED, Some(MetalGpuFamilySupport::NoneNamed))
        .expect_err("an undecidable device does not satisfy the requirement");
    assert_ne!(
        unobserved.rule(),
        undecidable.rule(),
        "a device nobody asked and a device that answered are different rules",
    );
}

/// The floor is an input to the comparison, not a constant it reads for itself.
///
/// Drives [`evaluate_against`] with the floor supplied explicitly and confirms
/// the outcome is the same one [`evaluate_index_arithmetic`] reaches through the
/// governed map, so the split seam is not a second policy. This is the seam a
/// floor above `Apple5` would be tested through, which is why it exists before
/// such a floor does.
#[test]
fn the_supplied_floor_reaches_the_same_outcome_as_the_governed_map() {
    let observations = [
        None,
        Some(MetalGpuFamilySupport::NoneNamed),
        Some(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple5)),
        Some(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9)),
    ];
    let mut compared = 0;
    for observed in observations {
        assert_eq!(
            evaluate_against(REQUIRED, minimum_gpu_family(REQUIRED), observed),
            evaluate_index_arithmetic(REQUIRED, observed),
            "the supplied floor and the governed map decide {observed:?} the same way",
        );
        compared += 1;
    }
    assert_eq!(compared, observations.len());
}

/// Every refusal names the arithmetic and the floor it could not establish.
///
/// A refusal reporting only a family would leave a caller unable to say *what*
/// the device could not carry, which is the difference between a typed cause and
/// a diagnostic string.
#[test]
fn every_refusal_names_the_index_arithmetic_and_its_floor() {
    let refusals = [
        evaluate_index_arithmetic(REQUIRED, None),
        evaluate_index_arithmetic(REQUIRED, Some(MetalGpuFamilySupport::NoneNamed)),
    ];
    let mut named = 0;
    for refusal in refusals {
        let refusal = refusal.expect_err("neither of these satisfies the requirement");
        assert_eq!(refusal.required(), REQUIRED);
        assert_eq!(refusal.floor(), AppleFamilyFloor::Apple3);
        let rendered = refusal.to_string();
        assert!(
            rendered.contains("index arithmetic"),
            "the rendered cause names index arithmetic: {rendered}",
        );
        assert!(
            rendered.contains("Apple3"),
            "the rendered cause names the floor: {rendered}",
        );
        named += 1;
    }
    assert_eq!(named, 2, "both refusal shapes were exercised");
}
