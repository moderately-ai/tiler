//! The numerical contract one region states, and the vocabulary it states it in.
//!
//! **Tom decided on 2026-08-01, at the live session, that there is no default.**
//! Presented with keeping the flush-only contract, moving every expansion to the
//! flushing-and-reassociating one, and letting the grammar state it, he chose the
//! third with a sharper rule than the option offered: *"avoid assuming defaults
//! and be specific — we can always simplify and add sane defaults later when we
//! know the shape of the project better."* So a region states its contract in its
//! own text, and a region that states none is refused at expansion with a
//! diagnostic naming what to write. `decide-the-inline-frontend-numerical-contract`
//! records the decision with its eliminations; this module executes it.
//!
//! What that rules out is the thing a default would have done quietly. The two
//! contracts this frontend's one bound compile declaration admits are *different
//! meanings* rather than two settings — under the flushing-and-reassociating one
//! a reduction may be split or folded as a workgroup tree, so its result may
//! differ from the flush-only reading in the last bits — and neither is stricter
//! than the other in every respect. A frontend that picked one would be choosing
//! what a consumer's program computes.
//!
//! # This module names contracts; it does not define them
//!
//! [`CONTRACTS`] is a table from a region's spelling to a `NumericalContract`
//! constant the compiler already publishes, and nothing here composes one. A
//! composed contract would be a meaning this frontend invented, statable by a
//! consumer and unknown to every other statement of the same subject — the
//! artifact identity, the explain trace, and the cache key all name the
//! compiler's own contract key, and a frontend-local point would have no entry
//! among them.
//!
//! # Whether a target can honour it is a different question, answered later
//!
//! Every name here resolves, and resolving is not admitting: the measured Apple
//! `f32` row flushes subnormals in every math mode, so a region stating
//! `strict_f32` parses, means exactly what it says, and is refused by the
//! compiler's own target feasibility check with a typed reason naming the
//! dimension. Pre-answering that here would put a target fact in the grammar,
//! where a second measured declaration would then have to contradict it.

use core::fmt;

use tiler_compiler::session::NumericalContract;

use crate::grammar::ContractSyntax;

/// Every numerical contract a region may state, under the name it states it by.
///
/// The five constants `tiler_compiler::session` publishes, each under the
/// lowercase spelling of the constant's own name. A table rather than an enum,
/// for the reason [`crate::region`]'s `ELEMENT_TYPES` is one: the compiler owns
/// what these contracts *are*, and this module owns only what a region calls
/// them, so a second type restating the vocabulary would be a second thing to
/// keep in agreement with the first.
///
/// It widens exactly when the compiler publishes another constant, and that is a
/// deliberate coupling rather than an omission: a name here with no constant
/// behind it could not be compiled under, and a constant with no name here is
/// simply not yet statable from a region.
///
/// The spelling is lowercase `snake_case` rather than the constant's own
/// `SCREAMING_SNAKE_CASE`, following `strict_serial_sum` — the ratified region
/// call whose Rust facade is `StrictSerialF32Sum`. The accepted grammar already
/// declines to mirror an item's Rust casing, and every other name a region
/// writes (`f32`, `macos`, `out`) is lowercase.
const CONTRACTS: [(&str, NumericalContract); 5] = [
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

/// One numerical contract a region stated, and the name it stated it by.
///
/// The name is carried beside the contract because a `NumericalContract` has no
/// consumer-facing spelling: its `key` is a canonical identity string derived
/// from the whole dimension vector, and its `Debug` rendering is that vector. A
/// refusal about a stated contract has to quote the word the consumer wrote, and
/// this is where that word survives past resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StatedContract {
    name: &'static str,
    contract: NumericalContract,
}

impl StatedContract {
    /// Returns the name the region stated this contract by.
    pub(crate) const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the contract the compiler compiles under.
    pub(crate) const fn contract(self) -> NumericalContract {
        self.contract
    }
}

/// Why a region names no numerical contract this frontend can state.
///
/// Two refusals rather than one, because they are different mistakes with
/// different fixes: a name that is not a contract is reported at the name, and a
/// region that states nothing has no token to report at and is reported at the
/// invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ContractRefusal<S> {
    /// The region states no `contract` statement at all.
    Unstated {
        /// The invocation, because no token is responsible for an absence.
        span: S,
    },
    /// The statement names a contract this frontend does not publish.
    UnknownContract {
        /// The name as written.
        name: String,
        /// The token.
        span: S,
    },
}

impl<S> fmt::Display for ContractRefusal<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unstated { .. } => write!(
                formatter,
                "this region states no numerical contract, so what its arithmetic means is \
                 undecided; add a `contract` statement to the declaration block, as in `contract \
                 flush_subnormals_to_zero_f32;`, naming one of {}. There is no default: a contract \
                 decides which results this program may return — whether subnormal operands flush \
                 to zero, and whether a sum may be regrouped — and two of these are different \
                 meanings rather than two settings, so choosing one here would choose what your \
                 program computes",
                rendered_contracts(),
            ),
            Self::UnknownContract { name, .. } => write!(
                formatter,
                "`{name}` is not a numerical contract a region may state; this frontend names {}. \
                 Whether the target you deliver for can honour the contract you state is a \
                 separate question, answered by the compiler when it plans the region",
                rendered_contracts(),
            ),
        }
    }
}

impl<S> ContractRefusal<S> {
    /// Returns the span this refusal must be reported at.
    pub(crate) const fn span(&self) -> &S {
        match self {
            Self::Unstated { span } | Self::UnknownContract { span, .. } => span,
        }
    }
}

/// Renders the contract vocabulary a diagnostic offers.
fn rendered_contracts() -> String {
    CONTRACTS
        .iter()
        .map(|(name, _)| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Resolves one stated contract name, or nothing.
///
/// Matched exactly, with no case folding and no prefixes, for
/// `NamedProfile::parse`'s reason and one stronger: a name that is nearly a
/// contract decides which results a program may return, so
/// `FLUSH_SUBNORMALS_TO_ZERO_F32` is refused rather than folded to the name
/// beside it.
pub(crate) fn resolve(name: &str) -> Option<StatedContract> {
    CONTRACTS
        .iter()
        .find(|(stated, _)| *stated == name)
        .map(|(name, contract)| StatedContract {
            name,
            contract: *contract,
        })
}

/// Reports whether any region spelling names this contract.
///
/// The reverse of [`resolve`], and it exists for one claim a test in
/// [`crate::aot`] makes: every contract the bound compile declaration honours
/// must be nameable from a region, or this frontend would be measuring a meaning
/// no consumer can ask for. Nothing on the expansion path needs the direction,
/// so it is not compiled into one.
#[cfg(test)]
pub(crate) fn statable(contract: NumericalContract) -> bool {
    CONTRACTS.iter().any(|(_, named)| *named == contract)
}

/// The numerical contract one region's tokens resolve to.
///
/// `region` is the span an absence is reported at, because no token is
/// responsible for a statement that was not written.
///
/// # Errors
///
/// Returns [`ContractRefusal::Unstated`] when the region states no `contract`
/// statement, and [`ContractRefusal::UnknownContract`], carrying the name's own
/// token, when it states one this frontend does not publish.
pub(crate) fn stated_contract<S: Copy>(
    stated: Option<&ContractSyntax<S>>,
    region: S,
) -> Result<StatedContract, ContractRefusal<S>> {
    let Some(statement) = stated else {
        return Err(ContractRefusal::Unstated { span: region });
    };
    resolve(&statement.name.text).ok_or_else(|| ContractRefusal::UnknownContract {
        name: statement.name.text.clone(),
        span: statement.name.span,
    })
}

#[cfg(test)]
mod tests;
