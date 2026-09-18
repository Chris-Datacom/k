//! Typed, stack-based intermediate representation.
//!
//! Expressions lower to values on a virtual stack. This keeps the first IR
//! compact while making loads, stores, calls, and control flow explicit.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use crate::parser::{
    BinaryOperator, Block, Expression, Function, Program, Statement, Type, UnaryOperator,
};
use crate::sema;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrType {
    Void,
    Int,
    Char,
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
    Bool,
    /// A pointer to `IrType`. The `bool` marks a `volatile`-qualified
    /// pointee: `Load`/`Store`/dereference instructions through it are
    /// tagged `volatile` and must never be reordered, merged, or elided.
    Pointer(Box<IrType>, bool),
    Struct(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Local {
    pub name: String,
    pub ty: IrType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    Constant(i64),
    StringLiteral(Vec<u8>),
    LoadLocal {
        name: String,
        ty: IrType,
    },
    StoreLocal {
        name: String,
        ty: IrType,
    },
    AddressLocal {
        name: String,
        ty: IrType,
    },
    FieldAddress {
        offset: i64,
        ty: IrType,
    },
    Load {
        ty: IrType,
        volatile: bool,
    },
    Store {
        ty: IrType,
        volatile: bool,
    },
    Scale {
        bytes: i64,
    },
    Unary {
        operator: UnaryOperator,
        ty: IrType,
        volatile: bool,
    },
    Binary {
        operator: BinaryOperator,
        ty: IrType,
    },
    Call {
        name: String,
        arguments: usize,
        ty: IrType,
    },
    PrintString(Vec<u8>),
    OutB,
    InB,
    Cli,
    Sti,
    Hlt,
    Pause,
    Cast(IrType),
    Pop,
    Branch {
        then_block: usize,
        else_block: usize,
    },
    Jump {
        target: usize,
    },
    Return {
        has_value: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub struct TypedProgram {
    pub structs: BTreeMap<String, StructLayout>,
    pub functions: BTreeMap<String, IrFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructLayout {
    pub size: i64,
    pub fields: BTreeMap<String, StructFieldLayout>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructFieldLayout {
    pub offset: i64,
    pub ty: IrType,
}

#[derive(Clone, Debug)]
pub struct IrFunction {
    pub return_type: IrType,
    pub parameters: Vec<IrType>,
    pub locals: Vec<Local>,
    pub blocks: Vec<BasicBlock>,
}

pub fn lower(program: &Program) -> Result<TypedProgram, Vec<sema::SemanticError>> {
    sema::check(program)?;
    let source = fold_constants(program);
    let structs = build_struct_layouts(&source);
    let functions = source
        .functions
        .iter()
        .map(|function| lower_function(function, &structs))
        .map(|(name, mut function)| {
            optimize_function(&mut function);
            (name, function)
        })
        .collect();
    Ok(TypedProgram { structs, functions })
}

fn build_struct_layouts(program: &Program) -> BTreeMap<String, StructLayout> {
    program
        .structs
        .iter()
        .map(|definition| {
            let mut offset = 0;
            let fields = definition
                .fields
                .iter()
                .map(|field| {
                    let layout = StructFieldLayout {
                        offset,
                        ty: ir_type(&field.ty),
                    };
                    offset += ir_size(&layout.ty);
                    (field.name.clone(), layout)
                })
                .collect();
            (
                definition.name.clone(),
                StructLayout {
                    size: offset,
                    fields,
                },
            )
        })
        .collect()
}

fn fold_constants(program: &Program) -> Program {
    let mut folded = program.clone();
    for function in &mut folded.functions {
        fold_block(&mut function.body);
    }
    folded
}

fn fold_block(block: &mut Block) {
    for statement in &mut block.statements {
        match statement {
            Statement::Declare { .. } => {}
            Statement::Let { value, .. }
            | Statement::Assign { value, .. }
            | Statement::Expression {
                expression: value, ..
            } => fold_expression(value),
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    fold_expression(value);
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                fold_expression(condition);
                fold_block(then_branch);
                if let Some(else_branch) = else_branch {
                    fold_block(else_branch);
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                fold_expression(condition);
                fold_block(body);
            }
            Statement::Block(body) => fold_block(body),
        }
    }
}

fn fold_expression(expression: &mut Expression) {
    match expression {
        Expression::Binary {
            left,
            operator,
            right,
            span,
        } => {
            fold_expression(left);
            fold_expression(right);
            let (
                Expression::Integer {
                    value: left_value, ..
                },
                Expression::Integer {
                    value: right_value, ..
                },
            ) = (left.as_ref(), right.as_ref())
            else {
                return;
            };
            let (Ok(left_value), Ok(right_value)) =
                (left_value.parse::<i64>(), right_value.parse::<i64>())
            else {
                return;
            };
            let value = match operator {
                BinaryOperator::Add => Some(left_value + right_value),
                BinaryOperator::Subtract => Some(left_value - right_value),
                BinaryOperator::Multiply => Some(left_value * right_value),
                BinaryOperator::Divide if right_value != 0 => Some(left_value / right_value),
                _ => None,
            };
            if let Some(value) = value {
                *expression = Expression::Integer {
                    value: value.to_string(),
                    span: *span,
                };
            }
        }
        Expression::Unary { operand, .. } => fold_expression(operand),
        Expression::Call {
            callee, arguments, ..
        } => {
            fold_expression(callee);
            for argument in arguments {
                fold_expression(argument);
            }
        }
        Expression::Index { base, index, .. } => {
            fold_expression(base);
            fold_expression(index);
        }
        Expression::Cast { operand, .. } => fold_expression(operand),
        Expression::Integer { .. }
        | Expression::Character { .. }
        | Expression::String { .. }
        | Expression::Boolean { .. }
        | Expression::Name { .. }
        | Expression::Field { .. } => {}
    }
}

fn lower_function(
    function: &Function,
    structs: &BTreeMap<String, StructLayout>,
) -> (String, IrFunction) {
    let mut locals = function
        .parameters
        .iter()
        .map(|parameter| Local {
            name: parameter.name.clone(),
            ty: ir_type(&parameter.ty),
        })
        .collect::<Vec<_>>();
    collect_locals(&function.body, &mut locals);

    let mut builder = FunctionBuilder {
        blocks: vec![BasicBlock {
            id: 0,
            instructions: Vec::new(),
        }],
        locals: &locals,
        structs,
    };
    builder.block(0, &function.body);
    let blocks = builder.blocks;

    (
        function.name.clone(),
        IrFunction {
            return_type: ir_type(&function.return_type),
            parameters: function.parameters.iter().map(|p| ir_type(&p.ty)).collect(),
            locals,
            blocks,
        },
    )
}

fn optimize_function(function: &mut IrFunction) {
    for block in &mut function.blocks {
        fold_instruction_constants(&mut block.instructions);
    }
    prune_unreachable_blocks(function);
}

fn fold_instruction_constants(instructions: &mut Vec<Instruction>) {
    let mut folded = Vec::with_capacity(instructions.len());
    for instruction in instructions.drain(..) {
        let replacement = match instruction {
            Instruction::Binary { operator, ty }
                if folded.len() >= 2
                    && matches!(folded[folded.len() - 1], Instruction::Constant(_))
                    && matches!(folded[folded.len() - 2], Instruction::Constant(_)) =>
            {
                let right = match folded.pop().expect("constant checked") {
                    Instruction::Constant(value) => value,
                    _ => unreachable!(),
                };
                let left = match folded.pop().expect("constant checked") {
                    Instruction::Constant(value) => value,
                    _ => unreachable!(),
                };
                match constant_binary(operator, left, right) {
                    Some(value) => Some(vec![Instruction::Constant(value)]),
                    None => Some(vec![
                        Instruction::Constant(left),
                        Instruction::Constant(right),
                        Instruction::Binary { operator, ty },
                    ]),
                }
            }
            other => Some(vec![other]),
        };
        if let Some(instructions) = replacement {
            folded.extend(instructions);
        }
    }
    *instructions = folded;
}

fn constant_binary(operator: BinaryOperator, left: i64, right: i64) -> Option<i64> {
    match operator {
        BinaryOperator::Add => Some(left + right),
        BinaryOperator::Subtract => Some(left - right),
        BinaryOperator::Multiply => Some(left * right),
        BinaryOperator::Divide if right != 0 => Some(left / right),
        BinaryOperator::Equal => Some(i64::from(left == right)),
        BinaryOperator::NotEqual => Some(i64::from(left != right)),
        BinaryOperator::Less => Some(i64::from(left < right)),
        BinaryOperator::LessEqual => Some(i64::from(left <= right)),
        BinaryOperator::Greater => Some(i64::from(left > right)),
        BinaryOperator::GreaterEqual => Some(i64::from(left >= right)),
        BinaryOperator::Divide => None,
    }
}

fn prune_unreachable_blocks(function: &mut IrFunction) {
    let mut reachable = vec![false; function.blocks.len()];
    let mut pending = VecDeque::from([0]);
    while let Some(id) = pending.pop_front() {
        if id >= function.blocks.len() || reachable[id] {
            continue;
        }
        reachable[id] = true;
        for instruction in &function.blocks[id].instructions {
            match instruction {
                Instruction::Branch {
                    then_block,
                    else_block,
                } => {
                    pending.push_back(*then_block);
                    pending.push_back(*else_block);
                }
                Instruction::Jump { target } => pending.push_back(*target),
                _ => {}
            }
        }
        if function.blocks[id]
            .instructions
            .last()
            .is_some_and(|instruction| {
                !matches!(
                    instruction,
                    Instruction::Branch { .. }
                        | Instruction::Jump { .. }
                        | Instruction::Return { .. }
                )
            })
            && id + 1 < function.blocks.len()
        {
            pending.push_back(id + 1);
        }
    }

    let mut remap = vec![usize::MAX; function.blocks.len()];
    let mut blocks = Vec::new();
    for (old_id, block) in function.blocks.iter().enumerate() {
        if reachable[old_id] {
            remap[old_id] = blocks.len();
            blocks.push(block.clone());
        }
    }
    for (id, block) in blocks.iter_mut().enumerate() {
        block.id = id;
        for instruction in &mut block.instructions {
            match instruction {
                Instruction::Branch {
                    then_block,
                    else_block,
                } => {
                    *then_block = remap[*then_block];
                    *else_block = remap[*else_block];
                }
                Instruction::Jump { target } => *target = remap[*target],
                _ => {}
            }
        }
    }
    function.blocks = blocks;
}

fn collect_locals(block: &Block, locals: &mut Vec<Local>) {
    for statement in &block.statements {
        match statement {
            Statement::Declare { ty, name, .. } => locals.push(Local {
                name: name.clone(),
                ty: ir_type(ty),
            }),
            Statement::Let { name, value, .. } => locals.push(Local {
                name: name.clone(),
                ty: expression_type_with_locals(value, locals),
            }),
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_locals(then_branch, locals);
                if let Some(else_branch) = else_branch {
                    collect_locals(else_branch, locals);
                }
            }
            Statement::While { body, .. } | Statement::Block(body) => collect_locals(body, locals),
            Statement::Assign { .. } | Statement::Return { .. } | Statement::Expression { .. } => {}
        }
    }
}

fn expression_type_with_locals(expression: &Expression, locals: &[Local]) -> IrType {
    match expression {
        Expression::Name { value, .. } => locals
            .iter()
            .find(|local| local.name == *value)
            .map(|local| local.ty.clone())
            .unwrap_or(IrType::Int),
        Expression::Unary {
            operator: UnaryOperator::Dereference,
            operand,
            ..
        } => match expression_type_with_locals(operand, locals) {
            IrType::Pointer(inner, _) => *inner,
            _ => IrType::Int,
        },
        Expression::Index { base, .. } => match expression_type_with_locals(base, locals) {
            IrType::Pointer(inner, _) => *inner,
            _ => IrType::Int,
        },
        Expression::Unary { operand, .. } => expression_type_with_locals(operand, locals),
        Expression::Binary { left, .. } => expression_type_with_locals(left, locals),
        Expression::Cast { ty, .. } => ir_type(ty),
        _ => expression_type(expression),
    }
}

struct FunctionBuilder<'locals> {
    blocks: Vec<BasicBlock>,
    locals: &'locals [Local],
    structs: &'locals BTreeMap<String, StructLayout>,
}

impl FunctionBuilder<'_> {
    fn block(&mut self, mut id: usize, block: &Block) -> usize {
        for statement in &block.statements {
            id = self.statement(id, statement);
        }
        id
    }

    fn statement(&mut self, id: usize, statement: &Statement) -> usize {
        match statement {
            Statement::Let { name, value, .. } => {
                self.expression(id, value);
                if let Some(local) = self.local(name) {
                    self.push(
                        id,
                        Instruction::StoreLocal {
                            name: name.clone(),
                            ty: local.ty.clone(),
                        },
                    );
                }
            }
            Statement::Declare { .. } => {}
            Statement::Assign { target, value, .. } => {
                self.lvalue(id, target);
                self.expression(id, value);
                self.push(
                    id,
                    Instruction::Store {
                        ty: self.expression_type(value),
                        volatile: self.lvalue_volatility(target),
                    },
                );
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    self.expression(id, value);
                }
                self.push(
                    id,
                    Instruction::Return {
                        has_value: value.is_some(),
                    },
                );
            }
            Statement::Expression { expression, .. } => {
                self.expression(id, expression);
                if !is_void_expression(expression) {
                    self.push(id, Instruction::Pop);
                }
            }
            Statement::Block(block) => return self.block(id, block),
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expression(id, condition);
                let then_id = self.new_block();
                let else_id = self.new_block();
                let end_id = self.new_block();
                self.push(
                    id,
                    Instruction::Branch {
                        then_block: then_id,
                        else_block: else_id,
                    },
                );
                let then_end = self.block(then_id, then_branch);
                if !self.terminated(then_end) {
                    self.push(then_end, Instruction::Jump { target: end_id });
                }
                if let Some(else_branch) = else_branch {
                    let else_end = self.block(else_id, else_branch);
                    if !self.terminated(else_end) {
                        self.push(else_end, Instruction::Jump { target: end_id });
                    }
                } else {
                    self.push(else_id, Instruction::Jump { target: end_id });
                }
                return end_id;
            }
            Statement::While {
                condition, body, ..
            } => {
                let condition_id = self.new_block();
                let body_id = self.new_block();
                let end_id = self.new_block();
                self.push(
                    id,
                    Instruction::Jump {
                        target: condition_id,
                    },
                );
                self.expression(condition_id, condition);
                self.push(
                    condition_id,
                    Instruction::Branch {
                        then_block: body_id,
                        else_block: end_id,
                    },
                );
                self.block(body_id, body);
                self.push(
                    body_id,
                    Instruction::Jump {
                        target: condition_id,
                    },
                );
                return end_id;
            }
        }
        id
    }

    fn expression(&mut self, id: usize, expression: &Expression) {
        match expression {
            Expression::Integer { value, .. } => {
                if let Ok(value) = value.parse() {
                    self.push(id, Instruction::Constant(value));
                }
            }
            Expression::Character { value, .. } => {
                self.push(id, Instruction::Constant(i64::from(*value)));
            }
            Expression::String { value, .. } => {
                self.push(id, Instruction::StringLiteral(value.clone()));
            }
            Expression::Boolean { value, .. } => {
                self.push(id, Instruction::Constant(i64::from(*value)));
            }
            Expression::Name { value, .. } => {
                if let Some(local) = self.local(value) {
                    self.push(
                        id,
                        Instruction::LoadLocal {
                            name: value.clone(),
                            ty: local.ty.clone(),
                        },
                    );
                }
            }
            Expression::Unary {
                operator, operand, ..
            } => {
                if matches!(operator, UnaryOperator::AddressOf) {
                    self.lvalue(id, operand);
                } else {
                    self.expression(id, operand);
                    let volatile = matches!(operator, UnaryOperator::Dereference)
                        && self.is_volatile_pointer(operand);
                    self.push(
                        id,
                        Instruction::Unary {
                            operator: *operator,
                            ty: self.expression_type(expression),
                            volatile,
                        },
                    );
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.expression(id, left);
                self.expression(id, right);
                self.push(
                    id,
                    Instruction::Binary {
                        operator: *operator,
                        ty: self.expression_type(expression),
                    },
                );
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                if let Expression::Name { value, .. } = callee.as_ref() {
                    if value == "print" && arguments.len() == 1 {
                        if let Expression::String { value, .. } = &arguments[0] {
                            self.push(id, Instruction::PrintString(value.clone()));
                            return;
                        }
                    }
                    if value == "outb" && arguments.len() == 2 {
                        self.expression(id, &arguments[0]);
                        self.expression(id, &arguments[1]);
                        self.push(id, Instruction::OutB);
                        return;
                    }
                    if value == "inb" && arguments.len() == 1 {
                        self.expression(id, &arguments[0]);
                        self.push(id, Instruction::InB);
                        return;
                    }
                    if arguments.is_empty() {
                        let instruction = match value.as_str() {
                            "cli" => Some(Instruction::Cli),
                            "sti" => Some(Instruction::Sti),
                            "hlt" => Some(Instruction::Hlt),
                            "pause" => Some(Instruction::Pause),
                            _ => None,
                        };
                        if let Some(instruction) = instruction {
                            self.push(id, instruction);
                            return;
                        }
                    }
                }
                for argument in arguments {
                    self.expression(id, argument);
                }
                if let Expression::Name { value, .. } = callee.as_ref() {
                    self.push(
                        id,
                        Instruction::Call {
                            name: value.clone(),
                            arguments: arguments.len(),
                            ty: self.expression_type(expression),
                        },
                    );
                }
            }
            Expression::Index { base, index, .. } => {
                self.expression(id, base);
                self.expression(id, index);
                self.push(
                    id,
                    Instruction::Scale {
                        bytes: pointee_size(&self.expression_type(base)),
                    },
                );
                self.push(
                    id,
                    Instruction::Binary {
                        operator: BinaryOperator::Add,
                        ty: IrType::Pointer(Box::new(self.expression_type(expression)), false),
                    },
                );
                self.push(
                    id,
                    Instruction::Load {
                        ty: self.expression_type(expression),
                        volatile: self.is_volatile_pointer(base),
                    },
                );
            }
            Expression::Field { base, .. } => {
                self.field_address(id, base, expression);
                self.push(
                    id,
                    Instruction::Load {
                        ty: self.expression_type(expression),
                        volatile: false,
                    },
                );
            }
            Expression::Cast { ty, operand, .. } => {
                self.expression(id, operand);
                self.push(id, Instruction::Cast(ir_type(ty)));
            }
        }
    }

    fn lvalue(&mut self, id: usize, expression: &Expression) {
        match expression {
            Expression::Name { value, .. } => {
                if let Some(local) = self.local(value) {
                    self.push(
                        id,
                        Instruction::AddressLocal {
                            name: value.clone(),
                            ty: IrType::Pointer(Box::new(local.ty.clone()), false),
                        },
                    );
                }
            }
            Expression::Unary {
                operator: UnaryOperator::Dereference,
                operand,
                ..
            } => self.expression(id, operand),
            Expression::Index { base, index, .. } => {
                self.expression(id, base);
                self.expression(id, index);
                self.push(
                    id,
                    Instruction::Scale {
                        bytes: pointee_size(&self.expression_type(base)),
                    },
                );
                self.push(
                    id,
                    Instruction::Binary {
                        operator: BinaryOperator::Add,
                        ty: IrType::Pointer(Box::new(self.expression_type(expression)), false),
                    },
                );
            }
            Expression::Field { base, .. } => self.field_address(id, base, expression),
            _ => {}
        }
    }

    fn field_address(&mut self, id: usize, base: &Expression, field: &Expression) {
        let (struct_name, field_name) = match field {
            Expression::Field { base, field, .. } => {
                let base_type = self.expression_type(base);
                let name = match base_type {
                    IrType::Struct(name) => name,
                    IrType::Pointer(inner, _) => match *inner {
                        IrType::Struct(name) => name,
                        _ => return,
                    },
                    _ => return,
                };
                (name, field)
            }
            _ => return,
        };
        let Some(field_layout) = self
            .structs
            .get(&struct_name)
            .and_then(|layout| layout.fields.get(field_name))
            .cloned()
        else {
            return;
        };
        match self.expression_type(base) {
            IrType::Struct(_) => self.lvalue(id, base),
            _ => self.expression(id, base),
        }
        self.push(
            id,
            Instruction::FieldAddress {
                offset: field_layout.offset,
                ty: IrType::Pointer(Box::new(field_layout.ty.clone()), false),
            },
        );
    }

    fn local(&self, name: &str) -> Option<&Local> {
        self.locals.iter().find(|local| local.name == name)
    }

    /// Returns true when `pointer_expression` statically has a
    /// volatile-qualified pointer type, meaning a load or store through it
    /// must never be reordered, merged, or elided.
    fn is_volatile_pointer(&self, pointer_expression: &Expression) -> bool {
        matches!(
            self.expression_type(pointer_expression),
            IrType::Pointer(_, true)
        )
    }

    /// Returns true when assigning through `target` accesses memory behind
    /// a volatile-qualified pointer (`*ptr = ...` or `ptr[i] = ...`).
    fn lvalue_volatility(&self, target: &Expression) -> bool {
        match target {
            Expression::Unary {
                operator: UnaryOperator::Dereference,
                operand,
                ..
            } => self.is_volatile_pointer(operand),
            Expression::Index { base, .. } => self.is_volatile_pointer(base),
            _ => false,
        }
    }

    fn expression_type(&self, expression: &Expression) -> IrType {
        match expression {
            Expression::Name { value, .. } => self
                .local(value)
                .map(|local| local.ty.clone())
                .unwrap_or(IrType::Int),
            Expression::Field { base, field, .. } => {
                let base_type = self.expression_type(base);
                let name = match base_type {
                    IrType::Struct(name) => name,
                    IrType::Pointer(inner, _) => match *inner {
                        IrType::Struct(name) => name,
                        _ => return IrType::Int,
                    },
                    _ => return IrType::Int,
                };
                self.structs
                    .get(&name)
                    .and_then(|layout| layout.fields.get(field))
                    .map(|field| field.ty.clone())
                    .unwrap_or(IrType::Int)
            }
            Expression::Index { base, .. } => match self.expression_type(base) {
                IrType::Pointer(inner, _) => *inner,
                _ => IrType::Int,
            },
            Expression::Unary {
                operator: UnaryOperator::Dereference,
                operand,
                ..
            } => match self.expression_type(operand) {
                IrType::Pointer(inner, _) => *inner,
                _ => IrType::Int,
            },
            Expression::Unary { operand, .. } => self.expression_type(operand),
            Expression::Binary { left, .. } => self.expression_type(left),
            Expression::Cast { ty, .. } => ir_type(ty),
            _ => expression_type(expression),
        }
    }

    fn push(&mut self, id: usize, instruction: Instruction) {
        self.blocks[id].instructions.push(instruction);
    }

    fn new_block(&mut self) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            instructions: Vec::new(),
        });
        id
    }

    fn terminated(&self, id: usize) -> bool {
        matches!(
            self.blocks[id].instructions.last(),
            Some(Instruction::Return { .. })
                | Some(Instruction::Jump { .. })
                | Some(Instruction::Branch { .. })
        )
    }
}

fn expression_type(expression: &Expression) -> IrType {
    match expression {
        Expression::Integer { .. } => IrType::Int,
        Expression::Character { .. } => IrType::Char,
        Expression::String { .. } => IrType::Pointer(Box::new(IrType::Char), false),
        Expression::Boolean { .. } => IrType::Bool,
        Expression::Name { .. } => IrType::Int,
        Expression::Unary {
            operator: UnaryOperator::AddressOf,
            operand,
            ..
        } => IrType::Pointer(Box::new(expression_type(operand)), false),
        Expression::Unary { operand, .. } => expression_type(operand),
        Expression::Binary { left, .. } => expression_type(left),
        Expression::Cast { ty, .. } => ir_type(ty),
        Expression::Call { .. } | Expression::Index { .. } | Expression::Field { .. } => {
            IrType::Int
        }
    }
}

fn is_void_expression(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Call { callee, .. }
            if matches!(callee.as_ref(), Expression::Name { value, .. }
                if matches!(value.as_str(), "print" | "outb" | "cli" | "sti" | "hlt" | "pause"))
    )
}

fn ir_type(ty: &Type) -> IrType {
    match ty {
        Type::Void => IrType::Void,
        Type::Int => IrType::Int,
        Type::Char => IrType::Char,
        Type::U8 => IrType::U8,
        Type::U16 => IrType::U16,
        Type::U32 => IrType::U32,
        Type::U64 => IrType::U64,
        Type::I32 => IrType::I32,
        Type::I64 => IrType::I64,
        Type::Bool => IrType::Bool,
        Type::Pointer(inner, volatile) => IrType::Pointer(Box::new(ir_type(inner)), *volatile),
        Type::Struct(name) => IrType::Struct(name.clone()),
    }
}

fn pointee_size(ty: &IrType) -> i64 {
    match ty {
        IrType::Pointer(inner, _) => pointee_size(inner),
        IrType::Char => 1,
        IrType::U8 => 1,
        IrType::U16 => 2,
        IrType::U32 | IrType::I32 => 4,
        IrType::U64 | IrType::I64 => 8,
        IrType::Bool => 1,
        IrType::Int | IrType::Void | IrType::Struct(_) => 8,
    }
}

fn ir_size(ty: &IrType) -> i64 {
    match ty {
        IrType::Struct(_) => 8,
        IrType::Void | IrType::Int | IrType::Pointer(_, _) => 8,
        IrType::Char | IrType::Bool => 1,
        IrType::U8 => 1,
        IrType::U16 => 2,
        IrType::U32 | IrType::I32 => 4,
        IrType::U64 | IrType::I64 => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::{lower, Instruction, IrType};
    use crate::parser::{parse, BinaryOperator};

    #[test]
    fn lowers_expressions_to_explicit_instructions() {
        let program = parse("int main() { let value = 2 + 3; return value; }").unwrap();
        let ir = lower(&program).unwrap();
        let instructions = &ir.functions["main"].blocks[0].instructions;
        assert!(instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Constant(5))));
        assert_eq!(ir.functions["main"].locals[0].ty, IrType::Int);
    }

    #[test]
    fn folds_ir_constants_after_lowering() {
        let program = parse("int main() { return 2 + 3 * 4; }").unwrap();
        let ir = lower(&program).unwrap();
        let instructions = &ir.functions["main"].blocks[0].instructions;
        assert!(instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Constant(14))));
        assert!(!instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::Binary {
                operator: BinaryOperator::Multiply,
                ..
            }
        )));
    }

    #[test]
    fn krumpyos_intrinsics_lower_without_stack_pops() {
        let program = parse("void idle() { cli(); sti(); hlt(); pause(); }").unwrap();
        let ir = lower(&program).unwrap();
        let instructions = &ir.functions["idle"].blocks[0].instructions;
        assert_eq!(
            instructions,
            &[
                Instruction::Cli,
                Instruction::Sti,
                Instruction::Hlt,
                Instruction::Pause
            ]
        );
    }

    #[test]
    fn marks_volatile_pointer_dereference_and_store_instructions() {
        let program = parse(
            "void poke(volatile u16* port) { *port = (u16)1; } u16 peek(volatile u16* port) { return *port; }",
        )
        .unwrap();
        let ir = lower(&program).unwrap();
        let poke = &ir.functions["poke"].blocks[0].instructions;
        assert!(poke
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Store { volatile: true, .. })));
        let peek = &ir.functions["peek"].blocks[0].instructions;
        assert!(peek
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Unary { volatile: true, .. })));
    }

    #[test]
    fn marks_volatile_indexed_load_and_store_instructions() {
        let program = parse(
            "void write(volatile u16* buffer, int index) { buffer[index] = (u16)1; } u16 read(volatile u16* buffer, int index) { return buffer[index]; }",
        )
        .unwrap();
        let ir = lower(&program).unwrap();
        let write = &ir.functions["write"].blocks[0].instructions;
        assert!(write
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Store { volatile: true, .. })));
        let read = &ir.functions["read"].blocks[0].instructions;
        assert!(read
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Load { volatile: true, .. })));
    }

    #[test]
    fn does_not_mark_plain_pointer_accesses_as_volatile() {
        let program =
            parse("void write(u16* buffer, int index) { buffer[index] = (u16)1; }").unwrap();
        let ir = lower(&program).unwrap();
        let write = &ir.functions["write"].blocks[0].instructions;
        assert!(write.iter().any(|instruction| matches!(
            instruction,
            Instruction::Store {
                volatile: false,
                ..
            }
        )));
        assert!(!write
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Store { volatile: true, .. })));
    }

    #[test]
    fn records_ordered_struct_field_offsets() {
        let program = parse(
            "struct Token { char kind; int start; int end; } int read(struct Token* token) { return token.start; }",
        )
        .unwrap();
        let ir = lower(&program).unwrap();
        let layout = &ir.structs["Token"];
        assert_eq!(layout.size, 17);
        assert_eq!(layout.fields["kind"].offset, 0);
        assert_eq!(layout.fields["start"].offset, 1);
        assert_eq!(layout.fields["end"].offset, 9);
    }

    #[test]
    fn records_fixed_width_struct_field_sizes() {
        let program = parse(
            "struct Header { u8 kind; u16 length; u32 checksum; u64 address; } int main() { return 0; }",
        )
        .unwrap();
        let ir = lower(&program).unwrap();
        let layout = &ir.structs["Header"];
        assert_eq!(layout.size, 15);
        assert_eq!(layout.fields["kind"].offset, 0);
        assert_eq!(layout.fields["length"].offset, 1);
        assert_eq!(layout.fields["checksum"].offset, 3);
        assert_eq!(layout.fields["address"].offset, 7);
    }
}
