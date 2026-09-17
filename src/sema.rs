//! Semantic analysis for K.
//!
//! This stage checks relationships that syntax alone cannot know: names must
//! resolve, declarations must be unique in their scope, expressions must have
//! compatible types, calls must match known function signatures, and control
//! flow constructs must receive boolean conditions. The checker does not yet
//! lower or mutate the parser AST; that separation keeps diagnostics easy to
//! test and leaves room for a typed intermediate representation later.

use std::collections::HashMap;
use std::fmt;

use crate::lexer::Span;
use crate::parser::{
    BinaryOperator, Block, Expression, Function, Program, Statement, Type, UnaryOperator,
};

/// A semantic diagnostic with the source location that caused it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "type error at {}: {}", self.span, self.message)
    }
}

/// Check a parsed K program, returning every error found during the pass.
pub fn check(program: &Program) -> Result<(), Vec<SemanticError>> {
    Checker::new(program).run()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValueType {
    Void,
    Int,
    Char,
    Bool,
    Pointer(Box<ValueType>),
    Struct(String),
    Function {
        return_type: Type,
        parameters: Vec<Type>,
    },
    Invalid,
}

impl ValueType {
    fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Void => Self::Void,
            Type::Int => Self::Int,
            Type::Char => Self::Char,
            Type::Pointer(inner) => Self::Pointer(Box::new(Self::from_type(inner))),
            Type::Struct(name) => Self::Struct(name.clone()),
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Void => "void",
            Self::Char => "char",
            Self::Bool => "bool",
            Self::Pointer(_) => "pointer",
            Self::Struct(_) => "struct",
            Self::Function { .. } => "function",
            Self::Invalid => "invalid expression",
        }
    }
}

struct Checker<'program> {
    program: &'program Program,
    functions: HashMap<String, ValueType>,
    structs: HashMap<String, HashMap<String, Type>>,
    scopes: Vec<HashMap<String, ValueType>>,
    errors: Vec<SemanticError>,
}

impl<'program> Checker<'program> {
    fn new(program: &'program Program) -> Self {
        Self {
            program,
            functions: HashMap::new(),
            structs: HashMap::new(),
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn run(mut self) -> Result<(), Vec<SemanticError>> {
        self.collect_functions();
        self.collect_structs();
        for function in &self.program.functions {
            self.check_function(function);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    fn collect_functions(&mut self) {
        for function in &self.program.functions {
            if self.functions.contains_key(&function.name) {
                self.error(function.span, format!("duplicate function `{}`", function.name));
                continue;
            }

            self.functions.insert(
                function.name.clone(),
                ValueType::Function {
                    return_type: function.return_type.clone(),
                    parameters: function
                        .parameters
                        .iter()
                        .map(|parameter| parameter.ty.clone())
                        .collect(),
                },
            );
        }
    }

    fn collect_structs(&mut self) {
        for definition in &self.program.structs {
            if self.structs.contains_key(&definition.name) {
                self.error(definition.span, format!("duplicate struct `{}`", definition.name));
                continue;
            }
            let mut fields = HashMap::new();
            for field in &definition.fields {
                if fields.insert(field.name.clone(), field.ty.clone()).is_some() {
                    self.error(field.span, format!("duplicate field `{}`", field.name));
                }
            }
            self.structs.insert(definition.name.clone(), fields);
        }
    }

    fn check_function(&mut self, function: &Function) {
        self.push_scope();
        for parameter in &function.parameters {
            self.declare(parameter.name.clone(), ValueType::from_type(&parameter.ty), parameter.span);
        }
        self.check_block(&function.body, &function.return_type);
        self.pop_scope();
    }

    fn check_block(&mut self, block: &Block, return_type: &Type) {
        for statement in &block.statements {
            self.check_statement(statement, return_type);
        }
    }

    fn check_statement(&mut self, statement: &Statement, return_type: &Type) {
        match statement {
            Statement::Declare { ty, name, span } => {
                self.declare(name.clone(), ValueType::from_type(ty), *span);
            }
            Statement::Let { name, value, span } => {
                let value_type = self.check_expression(value);
                self.declare(name.clone(), value_type, *span);
            }
            Statement::Assign { target, value, span } => {
                let expected = self.check_lvalue(target);
                let actual = self.check_expression(value);
                self.require_same(&expected, &actual, *span, "assignment");
            }
            Statement::Return { value, span } => match value {
                Some(value) => {
                    let actual = self.check_expression(value);
                    let expected = ValueType::from_type(return_type);
                    self.require_same(&expected, &actual, *span, "return value");
                }
                None if *return_type != Type::Void => self.error(*span, format!("expected a {} return value", type_name(return_type))),
                None => {}
            },
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition_type = self.check_expression(condition);
                self.require_bool(condition_type, condition.span());
                self.check_nested_block(then_branch, return_type);
                if let Some(else_branch) = else_branch {
                    self.check_nested_block(else_branch, return_type);
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                let condition_type = self.check_expression(condition);
                self.require_bool(condition_type, condition.span());
                self.check_nested_block(body, return_type);
            }
            Statement::Block(block) => self.check_nested_block(block, return_type),
            Statement::Expression { expression, .. } => {
                self.check_expression(expression);
            }
        }
    }

    fn check_nested_block(&mut self, block: &Block, return_type: &Type) {
        self.push_scope();
        self.check_block(block, return_type);
        self.pop_scope();
    }

    fn check_expression(&mut self, expression: &Expression) -> ValueType {
        match expression {
            Expression::Integer { .. } => ValueType::Int,
            Expression::Character { .. } => ValueType::Char,
            Expression::String { .. } => ValueType::Pointer(Box::new(ValueType::Char)),
            Expression::Boolean { .. } => ValueType::Bool,
            Expression::Name { value, span } => self.resolve_name(value, *span),
            Expression::Unary {
                operator,
                operand,
                span,
            } => {
                let operand_type = self.check_expression(operand);
                match operator {
                    UnaryOperator::Negate => {
                        if matches!(operand_type, ValueType::Int | ValueType::Char | ValueType::Invalid) {
                            operand_type
                        } else {
                            self.error(*span, format!("cannot negate {}", operand_type.display_name()));
                            ValueType::Invalid
                        }
                    }
                    UnaryOperator::AddressOf => {
                        if matches!(operand_type, ValueType::Invalid) {
                            ValueType::Invalid
                        } else {
                            ValueType::Pointer(Box::new(operand_type))
                        }
                    }
                    UnaryOperator::Dereference => match operand_type {
                        ValueType::Pointer(inner) => *inner,
                        ValueType::Invalid => ValueType::Invalid,
                        other => {
                            self.error(*span, format!("cannot dereference {}", other.display_name()));
                            ValueType::Invalid
                        }
                    }
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_type = self.check_expression(left);
                let right_type = self.check_expression(right);
                self.check_binary(*operator, left_type, right_type, *span)
            }
            Expression::Call {
                callee,
                arguments,
                span,
            } => self.check_call(callee, arguments, *span),
            Expression::Index { base, index, span } => {
                let base_type = self.check_expression(base);
                let index_type = self.check_expression(index);
                self.require_same(&ValueType::Int, &index_type, index.span(), "index");
                match base_type {
                    ValueType::Pointer(inner) => *inner,
                    ValueType::Invalid => ValueType::Invalid,
                    other => {
                        self.error(*span, format!("cannot index {}", other.display_name()));
                        ValueType::Invalid
                    }
                }
            }
            Expression::Field { base, field, span } => {
                let base_type = self.check_expression(base);
                let struct_name = match base_type {
                    ValueType::Struct(name) => Some(name),
                    ValueType::Pointer(inner) => match *inner {
                        ValueType::Struct(name) => Some(name),
                        _ => None,
                    },
                    ValueType::Invalid => None,
                    _ => {
                        self.error(*span, "field access requires a struct".to_owned());
                        None
                    }
                };
                if let Some(name) = struct_name {
                    if let Some(fields) = self.structs.get(&name) {
                        if let Some(ty) = fields.get(field) {
                            return ValueType::from_type(ty);
                        }
                    }
                    self.error(*span, format!("unknown field `{field}` on struct `{name}`"));
                }
                ValueType::Invalid
            }
        }
    }

    fn check_lvalue(&mut self, expression: &Expression) -> ValueType {
        match expression {
            Expression::Name { value, span } => self.resolve_name(value, *span),
            Expression::Unary { operator: UnaryOperator::Dereference, .. }
            | Expression::Index { .. } => self.check_expression(expression),
            Expression::Field { .. } => self.check_expression(expression),
            _ => {
                self.error(expression.span(), "assignment target must be a variable or memory location".to_owned());
                ValueType::Invalid
            }
        }
    }

    fn check_binary(
        &mut self,
        operator: BinaryOperator,
        left: ValueType,
        right: ValueType,
        span: Span,
    ) -> ValueType {
        if left == ValueType::Invalid || right == ValueType::Invalid {
            return ValueType::Invalid;
        }
        match operator {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide => {
                if left != right || !matches!(left, ValueType::Int | ValueType::Char) {
                    self.error(
                        span,
                        format!(
                            "arithmetic requires matching integer types, found {} and {}",
                            left.display_name(),
                            right.display_name()
                        ),
                    );
                    ValueType::Invalid
                } else {
                    left
                }
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                if left != right {
                    self.error(
                        span,
                        format!(
                            "comparison requires matching types, found {} and {}",
                            left.display_name(),
                            right.display_name()
                        ),
                    );
                    ValueType::Invalid
                } else {
                    ValueType::Bool
                }
            }
        }
    }

    fn check_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        span: Span,
    ) -> ValueType {
        let callee_type = self.check_expression(callee);
        let ValueType::Function {
            return_type,
            parameters,
        } = callee_type
        else {
            if callee_type != ValueType::Invalid {
                self.error(span, format!("cannot call {}", callee_type.display_name()));
            }
            for argument in arguments {
                self.check_expression(argument);
            }
            return ValueType::Invalid;
        };

        if arguments.len() != parameters.len() {
            self.error(
                span,
                format!("expected {} arguments, found {}", parameters.len(), arguments.len()),
            );
        }
        for (argument, expected) in arguments.iter().zip(parameters.iter()) {
            let actual = self.check_expression(argument);
            self.require_same(
                &ValueType::from_type(expected),
                &actual,
                argument.span(),
                "argument",
            );
        }
        ValueType::from_type(&return_type)
    }

    fn resolve_name(&mut self, name: &str, span: Span) -> ValueType {
        for scope in self.scopes.iter().rev() {
            if let Some(value_type) = scope.get(name) {
                return value_type.clone();
            }
        }
        if let Some(function_type) = self.functions.get(name) {
            return function_type.clone();
        }
        if name == "print" {
            return ValueType::Function {
                return_type: Type::Void,
                parameters: vec![Type::Pointer(Box::new(Type::Char))],
            };
        }
        self.error(span, format!("undefined name `{name}`"));
        ValueType::Invalid
    }

    fn declare(&mut self, name: String, value_type: ValueType, span: Span) {
        let scope = self.scopes.last_mut().expect("checker always has a scope");
        if scope.contains_key(&name) {
            self.error(span, format!("duplicate declaration `{name}`"));
        } else {
            scope.insert(name, value_type);
        }
    }

    fn require_bool(&mut self, actual: ValueType, span: Span) {
        self.require_same(&ValueType::Bool, &actual, span, "condition");
    }

    fn require_same(&mut self, expected: &ValueType, actual: &ValueType, span: Span, context: &str) {
        if *actual != ValueType::Invalid && expected != actual {
            self.error(
                span,
                format!(
                    "{context} requires {}, found {}",
                    expected.display_name(),
                    actual.display_name()
                ),
            );
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn error(&mut self, span: Span, message: String) {
        self.errors.push(SemanticError { span, message });
    }
}

fn type_name(ty: &Type) -> &'static str {
    match ty {
        Type::Void => "void",
        Type::Int => "int",
        Type::Char => "char",
        Type::Pointer(_) => "pointer",
        Type::Struct(_) => "struct",
    }
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::parser::parse;

    #[test]
    fn accepts_known_names_and_matching_calls() {
        let program = parse("int add(int left, int right) { return left + right; } int main() { let value = add(2, 3); return value; }").unwrap();
        assert!(check(&program).is_ok());
    }

    #[test]
    fn reports_undefined_names_and_bad_conditions() {
        let program = parse("int main() { if (1) { return missing; } return 0; }").unwrap();
        let errors = check(&program).unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].message.contains("condition requires bool"));
        assert!(errors[1].message.contains("undefined name `missing`"));
    }

    #[test]
    fn reports_call_arity_and_return_type_errors() {
        let program = parse("int get() { return true; } int main() { return get(1); }").unwrap();
        let errors = check(&program).unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].message.contains("return value requires int"));
        assert!(errors[1].message.contains("expected 0 arguments, found 1"));
    }

    #[test]
    fn checks_assignment_types() {
        let program = parse("int main() { let value = 1; value = true; return value; }").unwrap();
        let errors = check(&program).unwrap_err();
        assert!(errors.iter().any(|error| error.message.contains("assignment requires int")));
    }

    #[test]
    fn accepts_pointer_dereference_and_indexing() {
        let program = parse("int read(int* ptr) { ptr[1] = 7; return *ptr; }").unwrap();
        assert!(check(&program).is_ok());
    }

    #[test]
    fn accepts_void_returns_and_string_pointers() {
        let program = parse("void print() { return; } char* text() { return \"hi\"; }").unwrap();
        assert!(check(&program).is_ok());
    }

    #[test]
    fn rejects_value_return_from_void_function() {
        let program = parse("void bad() { return 1; }").unwrap();
        let errors = check(&program).unwrap_err();
        assert!(errors[0].message.contains("return value requires void"));
    }

    #[test]
    fn checks_struct_fields() {
        let program = parse(
            "struct Token { int kind; } int read(struct Token* token) { return token.kind; }",
        )
        .unwrap();
        assert!(check(&program).is_ok());
        let bad = parse(
            "struct Token { int kind; } int read(struct Token* token) { return token.missing; }",
        )
        .unwrap();
        assert!(check(&bad).unwrap_err()[0].message.contains("unknown field"));
    }
}
