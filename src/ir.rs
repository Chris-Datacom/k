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
    Int,
    Char,
    Bool,
    Pointer(Box<IrType>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Local {
    pub name: String,
    pub ty: IrType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    Constant(i64),
    LoadLocal { name: String, ty: IrType },
    StoreLocal { name: String, ty: IrType },
    AddressLocal { name: String, ty: IrType },
    Load { ty: IrType },
    Store { ty: IrType },
    Unary { operator: UnaryOperator, ty: IrType },
    Binary { operator: BinaryOperator, ty: IrType },
    Call { name: String, arguments: usize, ty: IrType },
    Pop,
    Branch { then_block: usize, else_block: usize },
    Jump { target: usize },
    Return { has_value: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub struct TypedProgram {
    pub functions: BTreeMap<String, IrFunction>,
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
    let functions = source
        .functions
        .iter()
        .map(lower_function)
        .map(|(name, mut function)| {
            optimize_function(&mut function);
            (name, function)
        })
        .collect();
    Ok(TypedProgram { functions })
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
            Statement::While { condition, body, .. } => {
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
            let (Expression::Integer { value: left_value, .. },
                Expression::Integer { value: right_value, .. }) =
                (left.as_ref(), right.as_ref()) else {
                    return;
                };
            let (Ok(left_value), Ok(right_value)) =
                (left_value.parse::<i64>(), right_value.parse::<i64>()) else {
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
        Expression::Integer { .. }
        | Expression::Character { .. }
        | Expression::Boolean { .. }
        | Expression::Name { .. } => {}
    }
}

fn lower_function(function: &Function) -> (String, IrFunction) {
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
        if let Some(last) = function.blocks[id].instructions.last() {
            match last {
                Instruction::Branch {
                    then_block,
                    else_block,
                } => {
                    pending.push_back(*then_block);
                    pending.push_back(*else_block);
                }
                Instruction::Jump { target } => pending.push_back(*target),
                Instruction::Return { .. } => {}
                _ => {
                    if id + 1 < function.blocks.len() {
                        pending.push_back(id + 1);
                    }
                }
            }
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
            Statement::Let { name, value, .. } => locals.push(Local {
                name: name.clone(),
                ty: expression_type(value),
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
            Statement::While { body, .. } | Statement::Block(body) => {
                collect_locals(body, locals)
            }
            Statement::Assign { .. }
            | Statement::Return { .. }
            | Statement::Expression { .. } => {}
        }
    }
}

struct FunctionBuilder<'locals> {
    blocks: Vec<BasicBlock>,
    locals: &'locals [Local],
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
            Statement::Assign { target, value, .. } => {
                self.lvalue(id, target);
                self.expression(id, value);
                self.push(
                    id,
                    Instruction::Store {
                        ty: expression_type(value),
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
                self.push(id, Instruction::Pop);
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
                self.push(id, Instruction::Jump { target: condition_id });
                self.expression(condition_id, condition);
                self.push(
                    condition_id,
                    Instruction::Branch {
                        then_block: body_id,
                        else_block: end_id,
                    },
                );
                self.block(body_id, body);
                self.push(body_id, Instruction::Jump { target: condition_id });
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
                    self.push(
                        id,
                        Instruction::Unary {
                            operator: *operator,
                            ty: expression_type(expression),
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
                        ty: expression_type(expression),
                    },
                );
            }
            Expression::Call {
                callee,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.expression(id, argument);
                }
                if let Expression::Name { value, .. } = callee.as_ref() {
                    self.push(
                        id,
                        Instruction::Call {
                            name: value.clone(),
                            arguments: arguments.len(),
                            ty: expression_type(expression),
                        },
                    );
                }
            }
            Expression::Index { base, index, .. } => {
                self.expression(id, base);
                self.expression(id, index);
                self.push(id, Instruction::Constant(8));
                self.push(
                    id,
                    Instruction::Binary {
                        operator: BinaryOperator::Multiply,
                        ty: IrType::Int,
                    },
                );
                self.push(
                    id,
                    Instruction::Binary {
                        operator: BinaryOperator::Add,
                        ty: IrType::Pointer(Box::new(expression_type(expression))),
                    },
                );
                self.push(
                    id,
                    Instruction::Load {
                        ty: expression_type(expression),
                    },
                );
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
                            ty: IrType::Pointer(Box::new(local.ty.clone())),
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
                self.push(id, Instruction::Constant(8));
                self.push(
                    id,
                    Instruction::Binary {
                        operator: BinaryOperator::Multiply,
                        ty: IrType::Int,
                    },
                );
                self.push(
                    id,
                    Instruction::Binary {
                        operator: BinaryOperator::Add,
                        ty: IrType::Pointer(Box::new(expression_type(expression))),
                    },
                );
            }
            _ => {}
        }
    }

    fn local(&self, name: &str) -> Option<&Local> {
        self.locals.iter().find(|local| local.name == name)
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
        Expression::Boolean { .. } => IrType::Bool,
        Expression::Name { .. } => IrType::Int,
        Expression::Unary {
            operator: UnaryOperator::AddressOf,
            operand,
            ..
        } => IrType::Pointer(Box::new(expression_type(operand))),
        Expression::Unary { operand, .. } => expression_type(operand),
        Expression::Binary { left, .. } => expression_type(left),
        Expression::Call { .. } | Expression::Index { .. } => IrType::Int,
    }
}

fn ir_type(ty: &Type) -> IrType {
    match ty {
        Type::Int => IrType::Int,
        Type::Char => IrType::Char,
        Type::Pointer(inner) => IrType::Pointer(Box::new(ir_type(inner))),
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
        assert!(instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::Constant(14)
        )));
        assert!(!instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::Binary {
                operator: BinaryOperator::Multiply,
                ..
            }
        )));
    }
}
