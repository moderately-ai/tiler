//! Executing one decoded scalar entry over a launch grid on the host CPU.
//!
//! # The execution model, stated rather than borrowed
//!
//! A Metal dispatch hands a grid to hardware that decides how invocations are
//! distributed across threadgroups, SIMD groups, and cores. This backend's
//! execution model is the whole of what it is: **one invocation at a time, in
//! ascending grid index, on the calling thread**. There is no workgroup, no
//! subgroup, no concurrency, and therefore nothing for a barrier to synchronize
//! — which is why [`crate::image::translate`] refuses a kernel containing one
//! rather than treating it as a no-op.
//!
//! Ascending order is a property of this bounded backend and not of the
//! contract. The kernel's own numerical realization forbids reassociation, so
//! the *arithmetic* does not depend on invocation order; what does depend on it
//! is only which store lands last where two invocations write the same element,
//! and the schedule's write-ownership witness is what forbids that. Stating the
//! order anyway is honest: this implementation has one, and a future threaded
//! CPU backend would not, so anything that turns out to depend on it is a defect
//! the two backends would disagree about.
//!
//! # What a bound buffer is here
//!
//! Not a device allocation. A [`Placement`] names a host allocation, the byte
//! offset the routed binding said the value starts at, and the number of bytes
//! the route said must be reachable through it. Element offsets in the image are
//! *element* indices, so the addressed byte range is derived rather than assumed
//! to start at zero — the same fact `RoutedBinding::accessible_offset` exists to
//! carry, realized without an argument table.

use crate::image::{Block, F32_BYTES, ImageBinaryOp, ImageCompareOp, Instruction, ScalarEntry};

/// One routed ABI slot resolved to host storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Placement {
    /// Index of the host allocation backing this slot.
    pub allocation: usize,
    /// First addressed byte of the bound value within that allocation.
    pub offset: u64,
    /// Number of bytes the route requires be reachable from `offset`.
    pub bytes: u64,
}

/// A value in one invocation's SSA environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Cell {
    Bool(bool),
    Index(u64),
    /// An `f32` held as its exact bit pattern.
    ///
    /// Bits rather than `f32` throughout the environment, so a signed zero, a
    /// subnormal, and a non-canonical NaN payload survive being carried between
    /// instructions. Arithmetic converts at the operation and converts straight
    /// back, which is the only point where the host's floating-point unit is
    /// involved at all.
    F32(u32),
}

/// Why one dispatch could not be carried out.
///
/// Reached **after** the routing commit, so none of these is a fallback: they
/// are reported. Every one of them is a condition the decode and the host
/// preflight could not decide — a bound allocation shorter than the route said,
/// an element offset outside the buffer the entry declared, a value read on a
/// path its definition did not run on — and each fails closed rather than
/// returning whatever the output storage happened to hold.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionError {
    /// A placement names a host allocation that was not supplied.
    UnboundSlot {
        /// ABI slot of the entry's buffer parameter.
        slot: usize,
        /// Allocation index the placement names.
        allocation: usize,
        /// How many allocations were supplied.
        supplied: usize,
    },
    /// A bound allocation is shorter than the routed accessible range.
    UndersizedAllocation {
        /// ABI slot of the entry's buffer parameter.
        slot: usize,
        /// Bytes the route requires be reachable.
        needed: u64,
        /// Bytes the allocation holds beyond the binding's offset.
        held: u64,
    },
    /// An access addresses an element outside the entry's declared parameter.
    ElementOutOfRange {
        /// ABI slot of the entry's buffer parameter.
        slot: usize,
        /// Element index the instruction computed.
        element: u64,
        /// Elements the parameter declares.
        declared: u64,
    },
    /// An access addresses bytes outside the routed accessible range.
    RangeOutOfBounds {
        /// ABI slot of the entry's buffer parameter.
        slot: usize,
        /// First byte of the access, relative to the binding's offset.
        at: u64,
        /// Bytes reachable through the binding.
        reachable: u64,
    },
    /// A slot is read on a path where its defining instruction did not run.
    UndefinedSlot {
        /// The slot read.
        slot: u32,
    },
    /// An index computation left the representable range.
    IndexOverflow {
        /// The operation that overflowed.
        operation: &'static str,
    },
    /// An index division or remainder by zero was reached.
    DivisionByZero {
        /// The operation that divided.
        operation: &'static str,
    },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnboundSlot {
                slot,
                allocation,
                supplied,
            } => write!(
                formatter,
                "cpu.execute.unbound: ABI slot {slot} names allocation {allocation} and {supplied} \
                 were supplied",
            ),
            Self::UndersizedAllocation { slot, needed, held } => write!(
                formatter,
                "cpu.execute.undersized: ABI slot {slot} must reach {needed} byte(s) and its \
                 allocation holds {held}",
            ),
            Self::ElementOutOfRange {
                slot,
                element,
                declared,
            } => write!(
                formatter,
                "cpu.execute.element-range: ABI slot {slot} addresses element {element} and its \
                 parameter declares {declared}",
            ),
            Self::RangeOutOfBounds {
                slot,
                at,
                reachable,
            } => write!(
                formatter,
                "cpu.execute.byte-range: ABI slot {slot} addresses byte {at} and {reachable} are \
                 reachable",
            ),
            Self::UndefinedSlot { slot } => write!(
                formatter,
                "cpu.execute.undefined: slot {slot} is read on a path its definition did not run \
                 on",
            ),
            Self::IndexOverflow { operation } => write!(
                formatter,
                "cpu.execute.overflow: {operation} left the representable index range",
            ),
            Self::DivisionByZero { operation } => {
                write!(formatter, "cpu.execute.divide-by-zero: {operation} by zero")
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

/// Executes one entry over a launch grid against host allocations.
///
/// `placements[slot]` resolves the entry's ABI slot `slot` to host storage, in
/// the entry's own buffer-parameter order.
///
/// # Errors
///
/// Returns the first [`ExecutionError`]. A failure leaves the allocations in
/// whatever state the invocations before it produced, which is why a caller
/// must not read an output back after one — the same discipline
/// `prototypes/serial-sum-run` applies to a command buffer that did not reach
/// `Completed`.
pub fn execute(
    entry: &ScalarEntry,
    placements: &[Placement],
    allocations: &mut [Vec<u8>],
    grid_threads: u64,
) -> Result<(), ExecutionError> {
    // Every placement is checked once, before any invocation runs. A refusal
    // discovered on invocation 4,000 would have already written 4,000 results.
    for (slot, placement) in placements.iter().enumerate() {
        let held = allocations
            .get(placement.allocation)
            .ok_or(ExecutionError::UnboundSlot {
                slot,
                allocation: placement.allocation,
                supplied: allocations.len(),
            })?
            .len();
        let held = u64::try_from(held).expect("a host allocation length fits a u64");
        let reachable = held.saturating_sub(placement.offset);
        if reachable < placement.bytes {
            return Err(ExecutionError::UndersizedAllocation {
                slot,
                needed: placement.bytes,
                held: reachable,
            });
        }
    }

    for invocation in 0..grid_threads {
        let mut environment = Environment {
            cells: vec![None; entry.slot_types.len()],
            invocation,
        };
        run_block(
            entry,
            &entry.body,
            &mut environment,
            placements,
            allocations,
        )?;
    }
    Ok(())
}

/// One invocation's SSA environment.
struct Environment {
    cells: Vec<Option<Cell>>,
    invocation: u64,
}

impl Environment {
    fn read(&self, slot: u32) -> Result<Cell, ExecutionError> {
        let at = usize::try_from(slot).expect("u32 fits every supported host usize");
        self.cells
            .get(at)
            .copied()
            .flatten()
            .ok_or(ExecutionError::UndefinedSlot { slot })
    }

    fn index(&self, slot: u32) -> Result<u64, ExecutionError> {
        match self.read(slot)? {
            Cell::Index(value) => Ok(value),
            // A decode proved every operand's declared type, so this is
            // unreachable for a decoded image and is reported rather than
            // asserted: the alternative is a panic inside a dispatch.
            Cell::Bool(_) | Cell::F32(_) => Err(ExecutionError::UndefinedSlot { slot }),
        }
    }

    fn bits(&self, slot: u32) -> Result<u32, ExecutionError> {
        match self.read(slot)? {
            Cell::F32(value) => Ok(value),
            Cell::Bool(_) | Cell::Index(_) => Err(ExecutionError::UndefinedSlot { slot }),
        }
    }

    fn boolean(&self, slot: u32) -> Result<bool, ExecutionError> {
        match self.read(slot)? {
            Cell::Bool(value) => Ok(value),
            Cell::Index(_) | Cell::F32(_) => Err(ExecutionError::UndefinedSlot { slot }),
        }
    }

    fn write(&mut self, slot: u32, cell: Cell) {
        let at = usize::try_from(slot).expect("u32 fits every supported host usize");
        // A decode proved the slot is in range; writing out of range would be a
        // decoder defect and is checked rather than indexed blindly.
        if let Some(target) = self.cells.get_mut(at) {
            *target = Some(cell);
        }
    }
}

/// Resolves one element access to a byte range inside a host allocation.
fn element_range(
    entry: &ScalarEntry,
    placements: &[Placement],
    slot: usize,
    element: u64,
) -> Result<(usize, usize), ExecutionError> {
    let declared = entry.buffers[slot].element_count;
    if element >= declared {
        return Err(ExecutionError::ElementOutOfRange {
            slot,
            element,
            declared,
        });
    }
    let placement = placements[slot];
    let at = element
        .checked_mul(F32_BYTES)
        .ok_or(ExecutionError::IndexOverflow {
            operation: "an element byte offset",
        })?;
    let end = at
        .checked_add(F32_BYTES)
        .ok_or(ExecutionError::IndexOverflow {
            operation: "an element byte offset",
        })?;
    // Both the *declared parameter extent* and the *routed accessible range*
    // are checked, and they are different claims: the first is what the kernel
    // says it may address, the second is what the artifact told this host to
    // make reachable. A backend trusting either alone reads storage the other
    // never authorized.
    if end > placement.bytes {
        return Err(ExecutionError::RangeOutOfBounds {
            slot,
            at,
            reachable: placement.bytes,
        });
    }
    let base = placement
        .offset
        .checked_add(at)
        .ok_or(ExecutionError::IndexOverflow {
            operation: "a bound byte address",
        })?;
    let base = usize::try_from(base).map_err(|_| ExecutionError::IndexOverflow {
        operation: "a bound byte address",
    })?;
    Ok((base, base + 4))
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm per instruction, and the exhaustive match is what makes a widened image vocabulary a build error here"
)]
fn run_block(
    entry: &ScalarEntry,
    block: &Block,
    environment: &mut Environment,
    placements: &[Placement],
    allocations: &mut [Vec<u8>],
) -> Result<(), ExecutionError> {
    for instruction in &block.instructions {
        match instruction {
            Instruction::GlobalInvocationIndex { result } => {
                environment.write(*result, Cell::Index(environment.invocation));
            }
            Instruction::ConstBool { result, value } => {
                environment.write(*result, Cell::Bool(*value));
            }
            Instruction::ConstIndex { result, value } => {
                environment.write(*result, Cell::Index(*value));
            }
            Instruction::ConstF32 { result, bits } => {
                environment.write(*result, Cell::F32(*bits));
            }
            Instruction::Binary {
                result,
                op,
                lhs,
                rhs,
            } => {
                let cell = match op {
                    ImageBinaryOp::IndexAdd => Cell::Index(
                        environment
                            .index(*lhs)?
                            .checked_add(environment.index(*rhs)?)
                            .ok_or(ExecutionError::IndexOverflow {
                                operation: "an index addition",
                            })?,
                    ),
                    ImageBinaryOp::IndexMultiply => Cell::Index(
                        environment
                            .index(*lhs)?
                            .checked_mul(environment.index(*rhs)?)
                            .ok_or(ExecutionError::IndexOverflow {
                                operation: "an index multiplication",
                            })?,
                    ),
                    ImageBinaryOp::IndexDivide => Cell::Index(
                        environment
                            .index(*lhs)?
                            .checked_div(environment.index(*rhs)?)
                            .ok_or(ExecutionError::DivisionByZero {
                                operation: "an index division",
                            })?,
                    ),
                    ImageBinaryOp::IndexModulo => Cell::Index(
                        environment
                            .index(*lhs)?
                            .checked_rem(environment.index(*rhs)?)
                            .ok_or(ExecutionError::DivisionByZero {
                                operation: "an index remainder",
                            })?,
                    ),
                    // The host's own binary32 addition and multiplication, with
                    // no reassociation, no contraction, and no reordering: the
                    // realization forbids all three and this backend has no
                    // transformation to disable.
                    ImageBinaryOp::F32Add => Cell::F32(
                        (f32::from_bits(environment.bits(*lhs)?)
                            + f32::from_bits(environment.bits(*rhs)?))
                        .to_bits(),
                    ),
                    ImageBinaryOp::F32Multiply => Cell::F32(
                        (f32::from_bits(environment.bits(*lhs)?)
                            * f32::from_bits(environment.bits(*rhs)?))
                        .to_bits(),
                    ),
                };
                environment.write(*result, cell);
            }
            Instruction::Compare {
                result,
                op,
                lhs,
                rhs,
            } => {
                let value = match op {
                    ImageCompareOp::IndexLessThan => {
                        environment.index(*lhs)? < environment.index(*rhs)?
                    }
                };
                environment.write(*result, Cell::Bool(value));
            }
            Instruction::CanonicalizeF32Nan { result, source } => {
                let bits = environment.bits(*source)?;
                // The realization's own pattern, read from the image rather
                // than from this file. A backend carrying its own constant
                // would agree with the reference until the contract changed.
                let canonical = if f32::from_bits(bits).is_nan() {
                    entry.numerics.canonical_nan_bits
                } else {
                    bits
                };
                environment.write(*result, Cell::F32(canonical));
            }
            Instruction::Load {
                result,
                buffer,
                offset,
            } => {
                let slot = usize::try_from(*buffer).expect("u32 fits every supported host usize");
                let element = environment.index(*offset)?;
                let (from, to) = element_range(entry, placements, slot, element)?;
                let bytes = &allocations[placements[slot].allocation][from..to];
                environment.write(
                    *result,
                    Cell::F32(u32::from_le_bytes(
                        <[u8; 4]>::try_from(bytes).expect("a four-byte window was taken"),
                    )),
                );
            }
            Instruction::Store {
                buffer,
                offset,
                value,
            } => {
                let slot = usize::try_from(*buffer).expect("u32 fits every supported host usize");
                let element = environment.index(*offset)?;
                let bits = environment.bits(*value)?;
                let (from, to) = element_range(entry, placements, slot, element)?;
                allocations[placements[slot].allocation][from..to]
                    .copy_from_slice(&bits.to_le_bytes());
            }
            Instruction::Predicated { predicate, body } => {
                if environment.boolean(*predicate)? {
                    run_block(entry, body, environment, placements, allocations)?;
                }
            }
            Instruction::SerialLoop {
                start,
                end,
                induction,
                accumulators,
                initial,
                yields,
                results,
                body,
            } => {
                let mut carried: Vec<Cell> = initial
                    .iter()
                    .map(|slot| environment.read(*slot))
                    .collect::<Result<_, _>>()?;
                for step in *start..*end {
                    if let Some(slot) = induction {
                        environment.write(*slot, Cell::Index(step));
                    }
                    for (slot, cell) in accumulators.iter().zip(carried.iter()) {
                        environment.write(*slot, *cell);
                    }
                    run_block(entry, body, environment, placements, allocations)?;
                    carried = yields
                        .iter()
                        .map(|slot| environment.read(*slot))
                        .collect::<Result<_, _>>()?;
                }
                for (slot, cell) in results.iter().zip(carried.iter()) {
                    environment.write(*slot, *cell);
                }
            }
        }
    }
    Ok(())
}
