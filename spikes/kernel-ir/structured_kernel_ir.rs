//! Compile-checking spike for Tiler's structured-kernel verifier boundary.
//! This is deliberately dependency-free and is not an implementation API.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ValueId(u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BufferId(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Type {
    Bool,
    U32,
    F32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Builtin {
    WorkgroupX,
    LocalInvocationX,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Uniformity {
    WorkgroupUniform,
    InvocationVarying,
}

impl Uniformity {
    fn join(self, rhs: Self) -> Self {
        if self == Self::InvocationVarying || rhs == Self::InvocationVarying {
            Self::InvocationVarying
        } else {
            Self::WorkgroupUniform
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum MemorySpace {
    Device,
    Workgroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Access {
    Read,
    Write,
    ReadWrite,
}

impl Access {
    fn readable(self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite)
    }

    fn writable(self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite)
    }
}

#[derive(Clone, Debug)]
struct Buffer {
    id: BufferId,
    element: Type,
    space: MemorySpace,
    access: Access,
}

#[derive(Clone, Debug)]
enum ValueOp {
    ConstantU32,
    ConstantF32,
    Parameter {
        ty: Type,
        uniformity: Uniformity,
    },
    Builtin(Builtin),
    Add(ValueId, ValueId),
    Mul(ValueId, ValueId),
    Less(ValueId, ValueId),
    Maximum(ValueId, ValueId),
    Convert {
        input: ValueId,
        contract: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ExecutionScope {
    Subgroup,
    Workgroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Ordering {
    AcquireRelease,
}

#[derive(Clone, Debug)]
enum Statement {
    Let {
        id: ValueId,
        ty: Type,
        op: ValueOp,
    },
    If {
        condition: ValueId,
        schedule_predicate: Option<&'static str>,
        body: Vec<Statement>,
    },
    For {
        induction: ValueId,
        upper: ValueId,
        body: Vec<Statement>,
    },
    Load {
        id: ValueId,
        ty: Type,
        buffer: BufferId,
        index: ValueId,
        bounds_witness: Option<&'static str>,
    },
    Store {
        buffer: BufferId,
        index: ValueId,
        value: ValueId,
        bounds_witness: Option<&'static str>,
        ownership_witness: Option<&'static str>,
    },
    Barrier {
        schedule_sync: &'static str,
        execution_scope: ExecutionScope,
        memory_scope: ExecutionScope,
        fenced_space: MemorySpace,
        ordering: Ordering,
    },
    Collective {
        id: ValueId,
        ty: Type,
        input: ValueId,
        reduction_plan: &'static str,
        execution_scope: ExecutionScope,
        combine_order: &'static str,
    },
}

#[derive(Clone, Debug)]
struct SyncContract {
    execution_scope: ExecutionScope,
    memory_scope: ExecutionScope,
    fenced_space: MemorySpace,
    ordering: Ordering,
}

#[derive(Clone, Debug)]
struct ReductionContract {
    execution_scope: ExecutionScope,
    combine_order: &'static str,
    ty: Type,
}

#[derive(Clone, Debug)]
struct BoundsContract {
    buffer: BufferId,
    required_predicate: Option<&'static str>,
}

#[derive(Clone, Debug, Default)]
struct ScheduleContract {
    identity: &'static str,
    builtins: BTreeSet<Builtin>,
    bounds: BTreeMap<&'static str, BoundsContract>,
    predicates: BTreeSet<&'static str>,
    ownership: BTreeMap<&'static str, BufferId>,
    conversions: BTreeSet<&'static str>,
    sync: BTreeMap<&'static str, SyncContract>,
    reductions: BTreeMap<&'static str, ReductionContract>,
}

#[derive(Clone, Debug)]
struct Kernel {
    scheduled_region_identity: &'static str,
    buffers: Vec<Buffer>,
    body: Vec<Statement>,
}

#[derive(Clone, Debug, Default)]
struct BackendSupport {
    spaces: BTreeSet<MemorySpace>,
    builtins: BTreeSet<Builtin>,
    barrier_scopes: BTreeSet<ExecutionScope>,
    collective_scopes: BTreeSet<ExecutionScope>,
}

#[derive(Clone, Copy, Debug)]
struct ValueFact {
    ty: Type,
    uniformity: Uniformity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Error {
    WrongSchedule,
    DuplicateValue(ValueId),
    Undefined(ValueId),
    TypeMismatch(&'static str),
    UnknownBuffer(BufferId),
    IllegalRead(BufferId),
    IllegalWrite(BufferId),
    MissingBounds,
    WrongBounds,
    MissingOwnership,
    WrongOwnership,
    UndeclaredBuiltin(Builtin),
    UnknownConversion,
    DivergentBarrier,
    NonuniformBarrierLoop,
    SynchronizationMismatch,
    ReductionMismatch,
    UnsupportedBackendFeature(&'static str),
}

struct Verifier<'a> {
    schedule: &'a ScheduleContract,
    buffers: BTreeMap<BufferId, &'a Buffer>,
}

impl<'a> Verifier<'a> {
    fn verify(kernel: &'a Kernel, schedule: &'a ScheduleContract) -> Result<(), Error> {
        if kernel.scheduled_region_identity != schedule.identity {
            return Err(Error::WrongSchedule);
        }
        let buffers = kernel.buffers.iter().map(|b| (b.id, b)).collect();
        Self { schedule, buffers }.region(
            &kernel.body,
            &mut BTreeMap::new(),
            &BTreeSet::new(),
            false,
            false,
        )
    }

    fn region(
        &self,
        statements: &[Statement],
        values: &mut BTreeMap<ValueId, ValueFact>,
        active_predicates: &BTreeSet<&'static str>,
        divergent: bool,
        nonuniform_loop: bool,
    ) -> Result<(), Error> {
        for statement in statements {
            match statement {
                Statement::Let { id, ty, op } => {
                    let fact = self.value_op(op, values)?;
                    if fact.ty != *ty {
                        return Err(Error::TypeMismatch("let result"));
                    }
                    define(values, *id, fact)?;
                }
                Statement::If {
                    condition,
                    schedule_predicate,
                    body,
                } => {
                    let condition = get(values, *condition)?;
                    if condition.ty != Type::Bool {
                        return Err(Error::TypeMismatch("if condition"));
                    }
                    let mut nested = values.clone();
                    let mut nested_predicates = active_predicates.clone();
                    if let Some(predicate) = schedule_predicate {
                        if !self.schedule.predicates.contains(predicate) {
                            return Err(Error::WrongBounds);
                        }
                        nested_predicates.insert(predicate);
                    }
                    self.region(
                        body,
                        &mut nested,
                        &nested_predicates,
                        divergent || condition.uniformity == Uniformity::InvocationVarying,
                        nonuniform_loop,
                    )?;
                }
                Statement::For {
                    induction,
                    upper,
                    body,
                } => {
                    let upper = get(values, *upper)?;
                    if upper.ty != Type::U32 {
                        return Err(Error::TypeMismatch("loop upper bound"));
                    }
                    let mut nested = values.clone();
                    define(
                        &mut nested,
                        *induction,
                        ValueFact {
                            ty: Type::U32,
                            uniformity: upper.uniformity,
                        },
                    )?;
                    self.region(
                        body,
                        &mut nested,
                        active_predicates,
                        divergent,
                        nonuniform_loop || upper.uniformity == Uniformity::InvocationVarying,
                    )?;
                }
                Statement::Load {
                    id,
                    ty,
                    buffer,
                    index,
                    bounds_witness,
                } => {
                    let binding = self.buffer(*buffer)?;
                    if !binding.access.readable() {
                        return Err(Error::IllegalRead(*buffer));
                    }
                    if binding.element != *ty || get(values, *index)?.ty != Type::U32 {
                        return Err(Error::TypeMismatch("load"));
                    }
                    self.bounds(*buffer, *bounds_witness, active_predicates)?;
                    define(
                        values,
                        *id,
                        ValueFact {
                            ty: *ty,
                            uniformity: Uniformity::InvocationVarying,
                        },
                    )?;
                }
                Statement::Store {
                    buffer,
                    index,
                    value,
                    bounds_witness,
                    ownership_witness,
                } => {
                    let binding = self.buffer(*buffer)?;
                    if !binding.access.writable() {
                        return Err(Error::IllegalWrite(*buffer));
                    }
                    if get(values, *index)?.ty != Type::U32
                        || get(values, *value)?.ty != binding.element
                    {
                        return Err(Error::TypeMismatch("store"));
                    }
                    self.bounds(*buffer, *bounds_witness, active_predicates)?;
                    match ownership_witness {
                        None => return Err(Error::MissingOwnership),
                        Some(witness) if self.schedule.ownership.get(witness) != Some(buffer) => {
                            return Err(Error::WrongOwnership)
                        }
                        Some(_) => {}
                    }
                }
                Statement::Barrier {
                    schedule_sync,
                    execution_scope,
                    memory_scope,
                    fenced_space,
                    ordering,
                } => {
                    if divergent {
                        return Err(Error::DivergentBarrier);
                    }
                    if nonuniform_loop {
                        return Err(Error::NonuniformBarrierLoop);
                    }
                    let expected = self
                        .schedule
                        .sync
                        .get(schedule_sync)
                        .ok_or(Error::SynchronizationMismatch)?;
                    if expected.execution_scope != *execution_scope
                        || expected.memory_scope != *memory_scope
                        || expected.fenced_space != *fenced_space
                        || expected.ordering != *ordering
                    {
                        return Err(Error::SynchronizationMismatch);
                    }
                }
                Statement::Collective {
                    id,
                    ty,
                    input,
                    reduction_plan,
                    execution_scope,
                    combine_order,
                } => {
                    let input = get(values, *input)?;
                    let expected = self
                        .schedule
                        .reductions
                        .get(reduction_plan)
                        .ok_or(Error::ReductionMismatch)?;
                    if input.ty != *ty
                        || expected.ty != *ty
                        || expected.execution_scope != *execution_scope
                        || expected.combine_order != *combine_order
                    {
                        return Err(Error::ReductionMismatch);
                    }
                    define(
                        values,
                        *id,
                        ValueFact {
                            ty: *ty,
                            uniformity: Uniformity::InvocationVarying,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    fn value_op(
        &self,
        op: &ValueOp,
        values: &BTreeMap<ValueId, ValueFact>,
    ) -> Result<ValueFact, Error> {
        match op {
            ValueOp::ConstantU32 => Ok(ValueFact {
                ty: Type::U32,
                uniformity: Uniformity::WorkgroupUniform,
            }),
            ValueOp::ConstantF32 => Ok(ValueFact {
                ty: Type::F32,
                uniformity: Uniformity::WorkgroupUniform,
            }),
            ValueOp::Parameter { ty, uniformity } => Ok(ValueFact {
                ty: *ty,
                uniformity: *uniformity,
            }),
            ValueOp::Builtin(builtin) => {
                if !self.schedule.builtins.contains(builtin) {
                    return Err(Error::UndeclaredBuiltin(*builtin));
                }
                Ok(ValueFact {
                    ty: Type::U32,
                    uniformity: match builtin {
                        Builtin::WorkgroupX => Uniformity::WorkgroupUniform,
                        Builtin::LocalInvocationX => Uniformity::InvocationVarying,
                    },
                })
            }
            ValueOp::Add(a, b) | ValueOp::Mul(a, b) => same_numeric(values, *a, *b),
            ValueOp::Maximum(a, b) => {
                let fact = same_numeric(values, *a, *b)?;
                if fact.ty != Type::F32 {
                    return Err(Error::TypeMismatch("maximum"));
                }
                Ok(fact)
            }
            ValueOp::Less(a, b) => {
                let fact = same_numeric(values, *a, *b)?;
                Ok(ValueFact {
                    ty: Type::Bool,
                    uniformity: fact.uniformity,
                })
            }
            ValueOp::Convert { input, contract } => {
                if !self.schedule.conversions.contains(contract) {
                    return Err(Error::UnknownConversion);
                }
                let input = get(values, *input)?;
                Ok(ValueFact {
                    ty: Type::F32,
                    uniformity: input.uniformity,
                })
            }
        }
    }

    fn buffer(&self, id: BufferId) -> Result<&Buffer, Error> {
        self.buffers
            .get(&id)
            .copied()
            .ok_or(Error::UnknownBuffer(id))
    }

    fn bounds(
        &self,
        buffer: BufferId,
        witness: Option<&'static str>,
        active_predicates: &BTreeSet<&'static str>,
    ) -> Result<(), Error> {
        match witness {
            None => Err(Error::MissingBounds),
            Some(witness) => match self.schedule.bounds.get(witness) {
                Some(contract)
                    if contract.buffer == buffer
                        && contract
                            .required_predicate
                            .is_none_or(|predicate| active_predicates.contains(predicate)) =>
                {
                    Ok(())
                }
                _ => Err(Error::WrongBounds),
            },
        }
    }
}

fn define(
    values: &mut BTreeMap<ValueId, ValueFact>,
    id: ValueId,
    fact: ValueFact,
) -> Result<(), Error> {
    if values.insert(id, fact).is_some() {
        Err(Error::DuplicateValue(id))
    } else {
        Ok(())
    }
}

fn get(values: &BTreeMap<ValueId, ValueFact>, id: ValueId) -> Result<ValueFact, Error> {
    values.get(&id).copied().ok_or(Error::Undefined(id))
}

fn same_numeric(
    values: &BTreeMap<ValueId, ValueFact>,
    a: ValueId,
    b: ValueId,
) -> Result<ValueFact, Error> {
    let a = get(values, a)?;
    let b = get(values, b)?;
    if a.ty != b.ty || a.ty == Type::Bool {
        return Err(Error::TypeMismatch("binary operands"));
    }
    Ok(ValueFact {
        ty: a.ty,
        uniformity: a.uniformity.join(b.uniformity),
    })
}

fn verify_backend(kernel: &Kernel, support: &BackendSupport) -> Result<(), Error> {
    for buffer in &kernel.buffers {
        if !support.spaces.contains(&buffer.space) {
            return Err(Error::UnsupportedBackendFeature("memory space"));
        }
    }
    fn walk(body: &[Statement], support: &BackendSupport) -> Result<(), Error> {
        for statement in body {
            match statement {
                Statement::Let {
                    op: ValueOp::Builtin(builtin),
                    ..
                } if !support.builtins.contains(builtin) => {
                    return Err(Error::UnsupportedBackendFeature("builtin"));
                }
                Statement::If { body, .. } | Statement::For { body, .. } => walk(body, support)?,
                Statement::Barrier {
                    execution_scope, ..
                } if !support.barrier_scopes.contains(execution_scope) => {
                    return Err(Error::UnsupportedBackendFeature("barrier scope"));
                }
                Statement::Collective {
                    execution_scope, ..
                } if !support.collective_scopes.contains(execution_scope) => {
                    return Err(Error::UnsupportedBackendFeature("collective scope"));
                }
                _ => {}
            }
        }
        Ok(())
    }
    walk(&kernel.body, support)
}

fn base_schedule() -> ScheduleContract {
    ScheduleContract {
        identity: "scheduled-region-v1",
        builtins: [Builtin::WorkgroupX, Builtin::LocalInvocationX].into(),
        bounds: [
            (
                "x-tail",
                BoundsContract {
                    buffer: BufferId(0),
                    required_predicate: Some("elementwise-active"),
                },
            ),
            (
                "x-reduction-range",
                BoundsContract {
                    buffer: BufferId(0),
                    required_predicate: None,
                },
            ),
            (
                "z-tail",
                BoundsContract {
                    buffer: BufferId(1),
                    required_predicate: Some("elementwise-active"),
                },
            ),
            (
                "partials-by-lane",
                BoundsContract {
                    buffer: BufferId(2),
                    required_predicate: None,
                },
            ),
        ]
        .into(),
        predicates: ["elementwise-active"].into(),
        ownership: [("z-by-i", BufferId(1)), ("partials-by-lane", BufferId(2))].into(),
        conversions: ["semantic-f16-to-f32-rne"].into(),
        sync: [(
            "partials-ready",
            SyncContract {
                execution_scope: ExecutionScope::Workgroup,
                memory_scope: ExecutionScope::Workgroup,
                fenced_space: MemorySpace::Workgroup,
                ordering: Ordering::AcquireRelease,
            },
        )]
        .into(),
        reductions: [(
            "tree-128-1",
            ReductionContract {
                execution_scope: ExecutionScope::Workgroup,
                combine_order: "128,64,32,16,8,4,2,1",
                ty: Type::F32,
            },
        )]
        .into(),
    }
}

fn buffers() -> Vec<Buffer> {
    vec![
        Buffer {
            id: BufferId(0),
            element: Type::F32,
            space: MemorySpace::Device,
            access: Access::Read,
        },
        Buffer {
            id: BufferId(1),
            element: Type::F32,
            space: MemorySpace::Device,
            access: Access::Write,
        },
        Buffer {
            id: BufferId(2),
            element: Type::F32,
            space: MemorySpace::Workgroup,
            access: Access::ReadWrite,
        },
    ]
}

fn elementwise_kernel() -> Kernel {
    Kernel {
        scheduled_region_identity: "scheduled-region-v1",
        buffers: buffers(),
        body: vec![
            Statement::Let {
                id: ValueId(0),
                ty: Type::U32,
                op: ValueOp::Builtin(Builtin::WorkgroupX),
            },
            Statement::Let {
                id: ValueId(1),
                ty: Type::U32,
                op: ValueOp::Builtin(Builtin::LocalInvocationX),
            },
            Statement::Let {
                id: ValueId(2),
                ty: Type::U32,
                op: ValueOp::ConstantU32,
            },
            Statement::Let {
                id: ValueId(3),
                ty: Type::U32,
                op: ValueOp::Mul(ValueId(0), ValueId(2)),
            },
            Statement::Let {
                id: ValueId(4),
                ty: Type::U32,
                op: ValueOp::Add(ValueId(3), ValueId(1)),
            },
            Statement::Let {
                id: ValueId(5),
                ty: Type::U32,
                op: ValueOp::Parameter {
                    ty: Type::U32,
                    uniformity: Uniformity::WorkgroupUniform,
                },
            },
            Statement::Let {
                id: ValueId(6),
                ty: Type::Bool,
                op: ValueOp::Less(ValueId(4), ValueId(5)),
            },
            Statement::If {
                condition: ValueId(6),
                schedule_predicate: Some("elementwise-active"),
                body: vec![
                    Statement::Load {
                        id: ValueId(7),
                        ty: Type::F32,
                        buffer: BufferId(0),
                        index: ValueId(4),
                        bounds_witness: Some("x-tail"),
                    },
                    Statement::Let {
                        id: ValueId(8),
                        ty: Type::F32,
                        op: ValueOp::ConstantF32,
                    },
                    Statement::Let {
                        id: ValueId(9),
                        ty: Type::F32,
                        op: ValueOp::Maximum(ValueId(7), ValueId(8)),
                    },
                    Statement::Store {
                        buffer: BufferId(1),
                        index: ValueId(4),
                        value: ValueId(9),
                        bounds_witness: Some("z-tail"),
                        ownership_witness: Some("z-by-i"),
                    },
                ],
            },
        ],
    }
}

fn reduction_kernel() -> Kernel {
    Kernel {
        scheduled_region_identity: "scheduled-region-v1",
        buffers: buffers(),
        body: vec![
            Statement::Let {
                id: ValueId(0),
                ty: Type::U32,
                op: ValueOp::Builtin(Builtin::LocalInvocationX),
            },
            Statement::Load {
                id: ValueId(1),
                ty: Type::F32,
                buffer: BufferId(0),
                index: ValueId(0),
                bounds_witness: Some("x-reduction-range"),
            },
            Statement::Store {
                buffer: BufferId(2),
                index: ValueId(0),
                value: ValueId(1),
                bounds_witness: Some("partials-by-lane"),
                ownership_witness: Some("partials-by-lane"),
            },
            Statement::Barrier {
                schedule_sync: "partials-ready",
                execution_scope: ExecutionScope::Workgroup,
                memory_scope: ExecutionScope::Workgroup,
                fenced_space: MemorySpace::Workgroup,
                ordering: Ordering::AcquireRelease,
            },
            Statement::Collective {
                id: ValueId(2),
                ty: Type::F32,
                input: ValueId(1),
                reduction_plan: "tree-128-1",
                execution_scope: ExecutionScope::Workgroup,
                combine_order: "128,64,32,16,8,4,2,1",
            },
        ],
    }
}

fn backend_support() -> BackendSupport {
    BackendSupport {
        spaces: [MemorySpace::Device, MemorySpace::Workgroup].into(),
        builtins: [Builtin::WorkgroupX, Builtin::LocalInvocationX].into(),
        barrier_scopes: [ExecutionScope::Workgroup].into(),
        collective_scopes: [ExecutionScope::Workgroup].into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_elementwise_and_reduction_refine_schedule() {
        let schedule = base_schedule();
        Verifier::verify(&elementwise_kernel(), &schedule).unwrap();
        Verifier::verify(&reduction_kernel(), &schedule).unwrap();
        verify_backend(&elementwise_kernel(), &backend_support()).unwrap();
    }

    #[test]
    fn rejects_use_before_definition() {
        let mut kernel = elementwise_kernel();
        kernel.body.insert(
            0,
            Statement::Let {
                id: ValueId(20),
                ty: Type::U32,
                op: ValueOp::Add(ValueId(99), ValueId(99)),
            },
        );
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::Undefined(ValueId(99)))
        );
    }

    #[test]
    fn rejects_wrong_operand_type() {
        let mut kernel = elementwise_kernel();
        kernel.body.insert(
            1,
            Statement::Let {
                id: ValueId(20),
                ty: Type::F32,
                op: ValueOp::ConstantF32,
            },
        );
        kernel.body.insert(
            2,
            Statement::Let {
                id: ValueId(21),
                ty: Type::F32,
                op: ValueOp::Add(ValueId(0), ValueId(20)),
            },
        );
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::TypeMismatch("binary operands"))
        );
    }

    #[test]
    fn rejects_illegal_buffer_access_mode() {
        let mut kernel = elementwise_kernel();
        kernel.body.push(Statement::Load {
            id: ValueId(20),
            ty: Type::F32,
            buffer: BufferId(1),
            index: ValueId(0),
            bounds_witness: Some("z-tail"),
        });
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::IllegalRead(BufferId(1)))
        );
    }

    #[test]
    fn rejects_missing_bounds_witness() {
        let mut kernel = elementwise_kernel();
        if let Statement::If { body, .. } = &mut kernel.body[7] {
            if let Statement::Load { bounds_witness, .. } = &mut body[0] {
                *bounds_witness = None;
            }
        }
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::MissingBounds)
        );
    }

    #[test]
    fn rejects_bounds_witness_without_dominating_schedule_predicate() {
        let mut kernel = elementwise_kernel();
        if let Statement::If {
            schedule_predicate, ..
        } = &mut kernel.body[7]
        {
            *schedule_predicate = None;
        }
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::WrongBounds)
        );
    }

    #[test]
    fn rejects_wrong_ownership_witness() {
        let mut kernel = elementwise_kernel();
        if let Statement::If { body, .. } = &mut kernel.body[7] {
            if let Statement::Store {
                ownership_witness, ..
            } = &mut body[3]
            {
                *ownership_witness = Some("unknown");
            }
        }
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::WrongOwnership)
        );
    }

    #[test]
    fn rejects_undeclared_builtin() {
        let mut schedule = base_schedule();
        schedule.builtins.remove(&Builtin::WorkgroupX);
        assert_eq!(
            Verifier::verify(&elementwise_kernel(), &schedule),
            Err(Error::UndeclaredBuiltin(Builtin::WorkgroupX))
        );
    }

    #[test]
    fn rejects_barrier_in_lane_varying_if() {
        let mut kernel = reduction_kernel();
        kernel.body.push(Statement::Let {
            id: ValueId(3),
            ty: Type::U32,
            op: ValueOp::ConstantU32,
        });
        kernel.body.push(Statement::Let {
            id: ValueId(4),
            ty: Type::Bool,
            op: ValueOp::Less(ValueId(0), ValueId(3)),
        });
        kernel.body.push(Statement::If {
            condition: ValueId(4),
            schedule_predicate: None,
            body: vec![Statement::Barrier {
                schedule_sync: "partials-ready",
                execution_scope: ExecutionScope::Workgroup,
                memory_scope: ExecutionScope::Workgroup,
                fenced_space: MemorySpace::Workgroup,
                ordering: Ordering::AcquireRelease,
            }],
        });
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::DivergentBarrier)
        );
    }

    #[test]
    fn rejects_barrier_in_nonuniform_loop() {
        let mut kernel = reduction_kernel();
        kernel.body.push(Statement::For {
            induction: ValueId(4),
            upper: ValueId(0),
            body: vec![Statement::Barrier {
                schedule_sync: "partials-ready",
                execution_scope: ExecutionScope::Workgroup,
                memory_scope: ExecutionScope::Workgroup,
                fenced_space: MemorySpace::Workgroup,
                ordering: Ordering::AcquireRelease,
            }],
        });
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::NonuniformBarrierLoop)
        );
    }

    #[test]
    fn rejects_barrier_scope_or_fence_change() {
        let mut kernel = reduction_kernel();
        if let Statement::Barrier { memory_scope, .. } = &mut kernel.body[3] {
            *memory_scope = ExecutionScope::Subgroup;
        }
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::SynchronizationMismatch)
        );
    }

    #[test]
    fn rejects_reduction_order_change() {
        let mut kernel = reduction_kernel();
        if let Statement::Collective { combine_order, .. } = &mut kernel.body[4] {
            *combine_order = "lane-unspecified";
        }
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::ReductionMismatch)
        );
    }

    #[test]
    fn rejects_uncontracted_conversion() {
        let mut kernel = elementwise_kernel();
        kernel.body.insert(
            7,
            Statement::Let {
                id: ValueId(30),
                ty: Type::F32,
                op: ValueOp::Convert {
                    input: ValueId(5),
                    contract: "backend-cast",
                },
            },
        );
        assert_eq!(
            Verifier::verify(&kernel, &base_schedule()),
            Err(Error::UnknownConversion)
        );
    }

    #[test]
    fn backend_support_is_separate_from_kernel_correctness() {
        let kernel = reduction_kernel();
        let schedule = base_schedule();
        Verifier::verify(&kernel, &schedule).unwrap();
        let mut support = backend_support();
        support.collective_scopes.clear();
        assert_eq!(
            verify_backend(&kernel, &support),
            Err(Error::UnsupportedBackendFeature("collective scope"))
        );
    }
}
