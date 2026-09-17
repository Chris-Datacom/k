//! Recursive-descent parser and abstract syntax tree for K.
//!
//! The parser intentionally accepts only the small grammar documented in
//! `docs/language.md`. It is useful to keep this boundary narrow: syntax can
//! grow later without hiding unresolved language design decisions in a large
//! parser. Every AST node keeps the byte span that produced it so later
//! semantic diagnostics can point back to source.

use std::fmt;

use crate::lexer::{LexError, Lexer, Span, SpannedToken, TokenKind};

/// A complete K source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub structs: Vec<StructDefinition>,
    pub functions: Vec<Function>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

/// A function definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub return_type: Type,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Block,
    pub span: Span,
}

/// A named function parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub ty: Type,
    pub name: String,
    pub span: Span,
}

/// The types currently accepted by the parser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Void,
    Int,
    Char,
    Pointer(Box<Type>),
    Struct(String),
}

/// A braced sequence of statements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Statements in the initial K grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
        span: Span,
    },
    Assign {
        target: Expression,
        value: Expression,
        span: Span,
    },
    Return {
        value: Option<Expression>,
        span: Span,
    },
    If {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
        span: Span,
    },
    While {
        condition: Expression,
        body: Block,
        span: Span,
    },
    Block(Block),
    Expression {
        expression: Expression,
        span: Span,
    },
}

/// Expressions before name resolution and type checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Integer {
        value: String,
        span: Span,
    },
    Character {
        value: u8,
        span: Span,
    },
    String {
        value: Vec<u8>,
        span: Span,
    },
    Boolean {
        value: bool,
        span: Span,
    },
    Name {
        value: String,
        span: Span,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
    Index {
        base: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    Field {
        base: Box<Expression>,
        field: String,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Negate,
    AddressOf,
    Dereference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// A syntax error with the source position where parsing stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "parse error at {}: {}", self.span, self.message)
    }
}

/// Parse one complete K source file.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens: Vec<SpannedToken> = Lexer::new(source)
        .map(|item| item.map_err(ParseError::from))
        .collect::<Result<_, _>>()?;
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn parse_program(mut self) -> Result<Program, ParseError> {
        let start = self.current_span().start;
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        while !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Struct) {
                structs.push(self.parse_struct()?);
            } else {
                functions.push(self.parse_function()?);
            }
        }
        let end = self.current_span().end;
        Ok(Program {
            structs,
            functions,
            span: Span { start, end },
        })
    }

    fn parse_struct(&mut self) -> Result<StructDefinition, ParseError> {
        let start = self.expect(TokenKind::Struct)?.start;
        let (name, _) = self.expect_identifier("struct name")?;
        self.expect(TokenKind::LeftBrace)?;
        let mut fields = Vec::new();
        while !self.at(&TokenKind::RightBrace) && !self.at(&TokenKind::Eof) {
            let field_start = self.current_span().start;
            let ty = self.parse_type()?;
            let (field_name, field_span) = self.expect_identifier("field name")?;
            self.expect(TokenKind::Semicolon)?;
            fields.push(StructField {
                name: field_name,
                ty,
                span: Span { start: field_start, end: field_span.end },
            });
        }
        let end = self.expect(TokenKind::RightBrace)?.end;
        Ok(StructDefinition {
            name,
            fields,
            span: Span { start, end },
        })
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let start = self.current_span().start;
        let return_type = self.parse_type()?;
        let (name, _) = self.expect_identifier("function name")?;
        self.expect(TokenKind::LeftParen)?;
        let mut parameters = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            loop {
                let parameter_start = self.current_span().start;
                let ty = self.parse_type()?;
                let (name, name_span) = self.expect_identifier("parameter name")?;
                parameters.push(Parameter {
                    ty,
                    name,
                    span: Span {
                        start: parameter_start,
                        end: name_span.end,
                    },
                });
                if !self.consume(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen)?;
        let body = self.parse_block()?;
        Ok(Function {
            return_type,
            name,
            parameters,
            span: Span {
                start,
                end: body.span.end,
            },
            body,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let mut ty = match self.take().token {
            TokenKind::Identifier(value) if value == "void" => Ok(Type::Void),
            TokenKind::Struct => {
                let (name, _) = self.expect_identifier("struct name")?;
                Ok(Type::Struct(name))
            }
            TokenKind::Int => Ok(Type::Int),
            TokenKind::Char => Ok(Type::Char),
            token => Err(self.error_expected("type", token)),
        }?;
        while self.consume(TokenKind::Star) {
            ty = Type::Pointer(Box::new(ty));
        }
        Ok(ty)
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.expect(TokenKind::LeftBrace)?.start;
        let mut statements = Vec::new();
        while !self.at(&TokenKind::RightBrace) && !self.at(&TokenKind::Eof) {
            statements.push(self.parse_statement()?);
        }
        let end = self.expect(TokenKind::RightBrace)?.end;
        Ok(Block {
            statements,
            span: Span { start, end },
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().token.clone() {
            TokenKind::Let => {
                let start = self.take().span.start;
                let (name, _) = self.expect_identifier("variable name")?;
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expression()?;
                let end = self.expect(TokenKind::Semicolon)?.end;
                Ok(Statement::Let {
                    name,
                    value,
                    span: Span { start, end },
                })
            }
            TokenKind::Return => {
                let start = self.take().span.start;
                let value = if self.at(&TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                let end = self.expect(TokenKind::Semicolon)?.end;
                Ok(Statement::Return {
                    value,
                    span: Span { start, end },
                })
            }
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::LeftBrace => self.parse_block().map(Statement::Block),
            _ => {
                let expression = self.parse_expression()?;
                if self.consume(TokenKind::Equal) {
                    let start = expression.span().start;
                    let value = self.parse_expression()?;
                    let end = self.expect(TokenKind::Semicolon)?.end;
                    return Ok(Statement::Assign {
                        target: expression,
                        value,
                        span: Span {
                            start,
                            end,
                        },
                    });
                }
                let span = self.expect(TokenKind::Semicolon)?.end;
                Ok(Statement::Expression {
                    span: Span {
                        start: expression.span().start,
                        end: span,
                    },
                    expression,
                })
            }
        }
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        let start = self.take().span.start;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.consume(TokenKind::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };
        let end = else_branch
            .as_ref()
            .map_or(then_branch.span.end, |block| block.span.end);
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            span: Span { start, end },
        })
    }

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        let start = self.take().span.start;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        let body = self.parse_block()?;
        Ok(Statement::While {
            condition,
            span: Span {
                start,
                end: body.span.end,
            },
            body,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(
        &mut self,
        minimum_precedence: u8,
    ) -> Result<Expression, ParseError> {
        let mut left = self.parse_prefix_expression()?;
        while let Some((operator, precedence)) = self.binary_operator() {
            if precedence < minimum_precedence {
                break;
            }
            self.take();
            let right = self.parse_binary_expression(precedence + 1)?;
            let span = Span {
                start: left.span().start,
                end: right.span().end,
            };
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, ParseError> {
        let expression = match self.peek().token.clone() {
            TokenKind::Minus => {
                let start = self.take().span.start;
                let operand = self.parse_prefix_expression()?;
                Expression::Unary {
                    operator: UnaryOperator::Negate,
                    span: Span {
                        start,
                        end: operand.span().end,
                    },
                    operand: Box::new(operand),
                }
            }
            TokenKind::Ampersand => {
                let start = self.take().span.start;
                let operand = self.parse_prefix_expression()?;
                Expression::Unary {
                    operator: UnaryOperator::AddressOf,
                    span: Span {
                        start,
                        end: operand.span().end,
                    },
                    operand: Box::new(operand),
                }
            }
            TokenKind::Star => {
                let start = self.take().span.start;
                let operand = self.parse_prefix_expression()?;
                Expression::Unary {
                    operator: UnaryOperator::Dereference,
                    span: Span { start, end: operand.span().end },
                    operand: Box::new(operand),
                }
            }
            TokenKind::LeftParen => {
                self.take();
                let expression = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                expression
            }
            _ => self.parse_primary_expression()?,
        };
        self.parse_postfix_expression(expression)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, ParseError> {
        let token = self.take();
        let span = token.span;
        match token.token {
            TokenKind::Integer(value) => Ok(Expression::Integer { value, span }),
            TokenKind::Character(value) => Ok(Expression::Character { value, span }),
            TokenKind::StringLiteral(value) => Ok(Expression::String { value, span }),
            TokenKind::True => Ok(Expression::Boolean { value: true, span }),
            TokenKind::False => Ok(Expression::Boolean { value: false, span }),
            TokenKind::Identifier(value) => Ok(Expression::Name { value, span }),
            token => Err(self.error_expected("expression", token)),
        }
    }

    fn parse_postfix_expression(&mut self, mut expression: Expression) -> Result<Expression, ParseError> {
        loop {
            if self.consume(TokenKind::LeftBracket) {
                let index = self.parse_expression()?;
                let end = self.expect(TokenKind::RightBracket)?.end;
                expression = Expression::Index {
                    span: Span { start: expression.span().start, end },
                    base: Box::new(expression),
                    index: Box::new(index),
                };
                continue;
            }
            if self.consume(TokenKind::Dot) {
                let (field, field_span) = self.expect_identifier("field name")?;
                let start = expression.span().start;
                expression = Expression::Field {
                    base: Box::new(expression),
                    field,
                    span: Span { start, end: field_span.end },
                };
                continue;
            }
            if !self.consume(TokenKind::LeftParen) {
                break;
            }
            let mut arguments = Vec::new();
            if !self.at(&TokenKind::RightParen) {
                loop {
                    arguments.push(self.parse_expression()?);
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }
            }
            let end = self.expect(TokenKind::RightParen)?.end;
            expression = Expression::Call {
                span: Span {
                    start: expression.span().start,
                    end,
                },
                callee: Box::new(expression),
                arguments,
            };
        }
        Ok(expression)
    }

    fn binary_operator(&self) -> Option<(BinaryOperator, u8)> {
        Some(match self.peek().token {
            TokenKind::EqualEqual => (BinaryOperator::Equal, 1),
            TokenKind::NotEqual => (BinaryOperator::NotEqual, 1),
            TokenKind::Less => (BinaryOperator::Less, 1),
            TokenKind::LessEqual => (BinaryOperator::LessEqual, 1),
            TokenKind::Greater => (BinaryOperator::Greater, 1),
            TokenKind::GreaterEqual => (BinaryOperator::GreaterEqual, 1),
            TokenKind::Plus => (BinaryOperator::Add, 2),
            TokenKind::Minus => (BinaryOperator::Subtract, 2),
            TokenKind::Star => (BinaryOperator::Multiply, 3),
            TokenKind::Slash => (BinaryOperator::Divide, 3),
            _ => return None,
        })
    }

    fn peek(&self) -> &SpannedToken {
        &self.tokens[self.cursor]
    }

    fn current_span(&self) -> Span {
        self.peek().span
    }

    fn take(&mut self) -> SpannedToken {
        let token = self.tokens[self.cursor].clone();
        if !matches!(token.token, TokenKind::Eof) {
            self.cursor += 1;
        }
        token
    }

    fn at(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().token) == std::mem::discriminant(expected)
    }

    fn consume(&mut self, expected: TokenKind) -> bool {
        if self.at(&expected) {
            self.take();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Span, ParseError> {
        if self.at(&expected) {
            Ok(self.take().span)
        } else {
            Err(self.error_expected(&expected.to_string(), self.peek().token.clone()))
        }
    }

    fn expect_identifier(&mut self, context: &str) -> Result<(String, Span), ParseError> {
        let token = self.take();
        match token.token {
            TokenKind::Identifier(value) => Ok((value, token.span)),
            other => Err(self.error_expected(context, other)),
        }
    }

    fn error_expected(&self, expected: &str, found: TokenKind) -> ParseError {
        ParseError {
            span: self.current_span(),
            message: format!("expected {expected}, found {found}"),
        }
    }
}

impl Expression {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::Integer { span, .. }
            | Self::Character { span, .. }
            | Self::String { span, .. }
            | Self::Boolean { span, .. }
            | Self::Name { span, .. }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Call { span, .. } => *span,
            Self::Index { span, .. } | Self::Field { span, .. } => *span,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self {
            span: error.span,
            message: error.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse, BinaryOperator, Expression, Statement, Type};

    #[test]
    fn parses_function_and_expression_precedence() {
        let program = parse("int main() { let answer = 2 + 3 * 4; return answer; }").unwrap();
        assert_eq!(program.functions.len(), 1);
        let function = &program.functions[0];
        assert_eq!(function.name, "main");
        assert_eq!(function.return_type, Type::Int);
        let Statement::Let { value, .. } = &function.body.statements[0] else {
            panic!("expected let statement");
        };
        let Expression::Binary {
            operator, right, ..
        } = value
        else {
            panic!("expected addition");
        };
        assert_eq!(*operator, BinaryOperator::Add);
        assert!(matches!(
            right.as_ref(),
            Expression::Binary {
                operator: BinaryOperator::Multiply,
                ..
            }
        ));
    }

    #[test]
    fn reports_missing_semicolon_at_the_stopping_span() {
        let error = parse("int main() { return 1 }").unwrap_err();
        assert!(error.message.contains("expected ;"));
        assert_eq!(error.span.start, 22);
    }

    #[test]
    fn parses_assignment_and_character_values() {
        let program = parse("char main() { let value = 'K'; value = 'L'; return value; }").unwrap();
        assert!(matches!(program.functions[0].body.statements[1], Statement::Assign { .. }));
        assert!(matches!(
            program.functions[0].body.statements[0],
            Statement::Let {
                value: Expression::Character { value: b'K', .. },
                ..
            }
        ));
    }

    #[test]
    fn parses_pointer_types_and_indexing() {
        let program = parse("int read(int* ptr) { return ptr[1]; }").unwrap();
        assert!(matches!(program.functions[0].parameters[0].ty, Type::Pointer(_)));
        assert!(matches!(
            program.functions[0].body.statements[0],
            Statement::Return {
                value: Some(Expression::Index { .. }),
                ..
            }
        ));
    }

    #[test]
    fn parses_void_functions_and_strings() {
        let program = parse("void print() { return; } char* text() { return \"hi\"; }").unwrap();
        assert_eq!(program.functions[0].return_type, Type::Void);
        assert!(matches!(
            program.functions[1].body.statements[0],
            Statement::Return {
                value: Some(Expression::String { .. }),
                ..
            }
        ));
    }

    #[test]
    fn parses_structs_and_field_access() {
        let program = parse(
            "struct Token { int kind; int start; } int read(struct Token* token) { return token.kind; }",
        )
        .unwrap();
        assert_eq!(program.structs[0].name, "Token");
        assert_eq!(program.structs[0].fields.len(), 2);
        assert!(matches!(
            program.functions[0].body.statements[0],
            Statement::Return {
                value: Some(Expression::Field { .. }),
                ..
            }
        ));
    }
}
