//! Lexical analysis for K.
//!
//! The lexer is intentionally byte-oriented. K source is UTF-8 text, but its
//! first syntax is ASCII and a byte-oriented scanner makes source positions
//! deterministic and keeps the eventual freestanding implementation simple.
//! Identifiers and numbers are accepted using ASCII rules for now; non-ASCII
//! bytes produce a clear error instead of being silently misinterpreted.

use std::fmt;

/// A half-open byte range in the original source: `[start, end)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..{}", self.start, self.end)
    }
}

/// One token and the source range from which it came.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpannedToken {
    pub token: TokenKind,
    pub span: Span,
}

/// The tokens currently recognized by K's surface syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Integer(String),
    Character(u8),
    StringLiteral(Vec<u8>),
    Int,
    Char,
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
    Bool,
    Struct,
    Extern,
    If,
    Else,
    While,
    Return,
    Let,
    True,
    False,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    LessLess,
    GreaterGreater,
    Semicolon,
    Comma,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Dot,
    Eof,
}

impl TokenKind {
    fn from_identifier(text: &str) -> Self {
        match text {
            "int" => Self::Int,
            "char" => Self::Char,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "bool" => Self::Bool,
            "struct" => Self::Struct,
            "extern" => Self::Extern,
            "if" => Self::If,
            "else" => Self::Else,
            "while" => Self::While,
            "return" => Self::Return,
            "let" => Self::Let,
            "true" => Self::True,
            "false" => Self::False,
            _ => Self::Identifier(text.to_owned()),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(value) => write!(formatter, "identifier({value})"),
            Self::Integer(value) => write!(formatter, "integer({value})"),
            Self::Character(value) => write!(formatter, "character({value})"),
            Self::StringLiteral(value) => write!(formatter, "string({value:?})"),
            Self::Int => write!(formatter, "int"),
            Self::Char => write!(formatter, "char"),
            Self::U8 => write!(formatter, "u8"),
            Self::U16 => write!(formatter, "u16"),
            Self::U32 => write!(formatter, "u32"),
            Self::U64 => write!(formatter, "u64"),
            Self::I32 => write!(formatter, "i32"),
            Self::I64 => write!(formatter, "i64"),
            Self::Bool => write!(formatter, "bool"),
            Self::Struct => write!(formatter, "struct"),
            Self::Extern => write!(formatter, "extern"),
            Self::If => write!(formatter, "if"),
            Self::Else => write!(formatter, "else"),
            Self::While => write!(formatter, "while"),
            Self::Return => write!(formatter, "return"),
            Self::Let => write!(formatter, "let"),
            Self::True => write!(formatter, "true"),
            Self::False => write!(formatter, "false"),
            Self::Plus => write!(formatter, "+"),
            Self::Minus => write!(formatter, "-"),
            Self::Star => write!(formatter, "*"),
            Self::Slash => write!(formatter, "/"),
            Self::Equal => write!(formatter, "="),
            Self::EqualEqual => write!(formatter, "=="),
            Self::NotEqual => write!(formatter, "!="),
            Self::Less => write!(formatter, "<"),
            Self::LessEqual => write!(formatter, "<="),
            Self::Greater => write!(formatter, ">"),
            Self::GreaterEqual => write!(formatter, ">="),
            Self::Ampersand => write!(formatter, "&"),
            Self::Pipe => write!(formatter, "|"),
            Self::Caret => write!(formatter, "^"),
            Self::Tilde => write!(formatter, "~"),
            Self::LessLess => write!(formatter, "<<"),
            Self::GreaterGreater => write!(formatter, ">>"),
            Self::Semicolon => write!(formatter, ";"),
            Self::Comma => write!(formatter, ","),
            Self::LeftParen => write!(formatter, "("),
            Self::RightParen => write!(formatter, ")"),
            Self::LeftBrace => write!(formatter, "{{"),
            Self::RightBrace => write!(formatter, "}}"),
            Self::LeftBracket => write!(formatter, "["),
            Self::RightBracket => write!(formatter, "]"),
            Self::Dot => write!(formatter, "."),
            Self::Eof => write!(formatter, "eof"),
        }
    }
}

/// A lexical error with a stable source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "lex error at {}: {}", self.span, self.message)
    }
}

/// A single-pass iterator over K tokens.
pub struct Lexer<'source> {
    source: &'source [u8],
    cursor: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source: source.as_bytes(),
            cursor: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.cursor).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.cursor += 1;
        Some(byte)
    }

    fn skip_trivia(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.cursor += 1;
            }
            if self.source.get(self.cursor..self.cursor + 2) == Some(b"//") {
                while !matches!(self.peek(), None | Some(b'\n')) {
                    self.cursor += 1;
                }
            } else {
                break;
            }
        }
    }

    fn token(&mut self) -> Result<SpannedToken, LexError> {
        self.skip_trivia();
        let start = self.cursor;
        let Some(byte) = self.advance() else {
            return Ok(SpannedToken {
                token: TokenKind::Eof,
                span: Span { start, end: start },
            });
        };
        let token = match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while matches!(
                    self.peek(),
                    Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
                ) {
                    self.cursor += 1;
                }
                let text = std::str::from_utf8(&self.source[start..self.cursor]).unwrap();
                TokenKind::from_identifier(text)
            }
            b'0'..=b'9' => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.cursor += 1;
                }
                TokenKind::Integer(
                    String::from_utf8(self.source[start..self.cursor].to_vec()).unwrap(),
                )
            }
            b'\'' => {
                let value = match self.advance() {
                    Some(b'\\') => match self.advance() {
                        Some(b'n') => b'\n',
                        Some(b'r') => b'\r',
                        Some(b't') => b'\t',
                        Some(b'\\') => b'\\',
                        Some(b'\'') => b'\'',
                        Some(escaped) => {
                            return Err(LexError {
                                span: Span {
                                    start,
                                    end: self.cursor,
                                },
                                message: format!("unknown character escape `\\{escaped}`"),
                            })
                        }
                        None => {
                            return Err(LexError {
                                span: Span {
                                    start,
                                    end: self.cursor,
                                },
                                message: "unterminated character literal".to_owned(),
                            })
                        }
                    },
                    Some(value @ 0x20..=0x7e) => value,
                    Some(value) => {
                        return Err(LexError {
                            span: Span {
                                start,
                                end: self.cursor,
                            },
                            message: format!("invalid character byte 0x{value:02x}"),
                        })
                    }
                    None => {
                        return Err(LexError {
                            span: Span {
                                start,
                                end: self.cursor,
                            },
                            message: "unterminated character literal".to_owned(),
                        })
                    }
                };
                if self.advance() != Some(b'\'') {
                    return Err(LexError {
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                        message: "character literal must contain one character".to_owned(),
                    });
                }
                TokenKind::Character(value)
            }
            b'"' => {
                let mut value = Vec::new();
                loop {
                    match self.advance() {
                        Some(b'"') => break,
                        Some(b'\\') => match self.advance() {
                            Some(b'n') => value.push(b'\n'),
                            Some(b'r') => value.push(b'\r'),
                            Some(b't') => value.push(b'\t'),
                            Some(b'\\') => value.push(b'\\'),
                            Some(b'"') => value.push(b'"'),
                            Some(escaped) => {
                                return Err(LexError {
                                    span: Span {
                                        start,
                                        end: self.cursor,
                                    },
                                    message: format!("unknown string escape `\\{escaped}`"),
                                })
                            }
                            None => {
                                return Err(LexError {
                                    span: Span {
                                        start,
                                        end: self.cursor,
                                    },
                                    message: "unterminated string literal".to_owned(),
                                })
                            }
                        },
                        Some(byte @ 0x20..=0x7e) => value.push(byte),
                        Some(byte) => {
                            return Err(LexError {
                                span: Span {
                                    start,
                                    end: self.cursor,
                                },
                                message: format!("invalid string byte 0x{byte:02x}"),
                            })
                        }
                        None => {
                            return Err(LexError {
                                span: Span {
                                    start,
                                    end: self.cursor,
                                },
                                message: "unterminated string literal".to_owned(),
                            })
                        }
                    }
                }
                TokenKind::StringLiteral(value)
            }
            b'+' => TokenKind::Plus,
            b'-' => TokenKind::Minus,
            b'*' => TokenKind::Star,
            b'/' => TokenKind::Slash,
            b'=' if self.peek() == Some(b'=') => {
                self.cursor += 1;
                TokenKind::EqualEqual
            }
            b'=' => TokenKind::Equal,
            b'!' if self.peek() == Some(b'=') => {
                self.cursor += 1;
                TokenKind::NotEqual
            }
            b'<' if self.peek() == Some(b'=') => {
                self.cursor += 1;
                TokenKind::LessEqual
            }
            b'<' if self.peek() == Some(b'<') => {
                self.cursor += 1;
                TokenKind::LessLess
            }
            b'<' => TokenKind::Less,
            b'>' if self.peek() == Some(b'=') => {
                self.cursor += 1;
                TokenKind::GreaterEqual
            }
            b'>' if self.peek() == Some(b'>') => {
                self.cursor += 1;
                TokenKind::GreaterGreater
            }
            b'>' => TokenKind::Greater,
            b'&' => TokenKind::Ampersand,
            b'|' => TokenKind::Pipe,
            b'^' => TokenKind::Caret,
            b'~' => TokenKind::Tilde,
            b';' => TokenKind::Semicolon,
            b',' => TokenKind::Comma,
            b'(' => TokenKind::LeftParen,
            b')' => TokenKind::RightParen,
            b'{' => TokenKind::LeftBrace,
            b'}' => TokenKind::RightBrace,
            b'[' => TokenKind::LeftBracket,
            b']' => TokenKind::RightBracket,
            b'.' => TokenKind::Dot,
            _ => {
                return Err(LexError {
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                    message: format!("unexpected byte 0x{byte:02x}"),
                })
            }
        };
        Ok(SpannedToken {
            token,
            span: Span {
                start,
                end: self.cursor,
            },
        })
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<SpannedToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.source.len() {
            return None;
        }
        let item = self.token();
        if matches!(
            &item,
            Ok(SpannedToken {
                token: TokenKind::Eof,
                ..
            })
        ) {
            self.cursor = self.source.len() + 1;
        }
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, TokenKind};

    #[test]
    fn lexes_keywords_operators_and_comments() {
        let tokens: Vec<_> = Lexer::new("let value = 42; // ignore\nif (value >= 1) return value;")
            .map(|item| item.unwrap().token)
            .collect();
        assert_eq!(
            tokens,
            vec![
                TokenKind::Let,
                TokenKind::Identifier("value".into()),
                TokenKind::Equal,
                TokenKind::Integer("42".into()),
                TokenKind::Semicolon,
                TokenKind::If,
                TokenKind::LeftParen,
                TokenKind::Identifier("value".into()),
                TokenKind::GreaterEqual,
                TokenKind::Integer("1".into()),
                TokenKind::RightParen,
                TokenKind::Return,
                TokenKind::Identifier("value".into()),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn reports_unknown_bytes() {
        let error = Lexer::new("@").next().unwrap().unwrap_err();
        assert_eq!(error.span.start, 0);
    }

    #[test]
    fn lexes_character_literals_and_escapes() {
        let tokens: Vec<_> = Lexer::new("'K' '\\n'")
            .map(|item| item.unwrap().token)
            .collect();
        assert_eq!(
            tokens,
            vec![
                TokenKind::Character(b'K'),
                TokenKind::Character(b'\n'),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn lexes_string_literals() {
        let tokens: Vec<_> = Lexer::new("\"K\\n\"")
            .map(|item| item.unwrap().token)
            .collect();
        assert_eq!(
            tokens,
            vec![TokenKind::StringLiteral(vec![b'K', b'\n']), TokenKind::Eof]
        );
    }

    #[test]
    fn lexes_shared_conformance_fixture() {
        let source = include_str!("../compiler/lexer_conformance.k");
        let tokens: Vec<_> = Lexer::new(source).map(|item| item.unwrap()).collect();
        assert_eq!(
            tokens.first().map(|item| &item.token),
            Some(&TokenKind::Struct)
        );
        assert_eq!(
            tokens.first().map(|item| item.span),
            Some(super::Span { start: 37, end: 43 })
        );
        assert_eq!(
            tokens.get(1).map(|item| &item.token),
            Some(&TokenKind::Identifier("Pair".into()))
        );
        assert_eq!(
            tokens.get(1).map(|item| item.span),
            Some(super::Span { start: 44, end: 48 })
        );
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::StringLiteral(_))));
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::Character(b'\''))));
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::NotEqual)));
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::LessEqual)));
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::GreaterEqual)));
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::Else)));
        assert!(tokens
            .iter()
            .any(|item| matches!(item.token, TokenKind::While)));
        assert_eq!(tokens.last().map(|item| &item.token), Some(&TokenKind::Eof));
        assert_eq!(
            tokens.last().map(|item| item.span),
            Some(super::Span {
                start: source.len(),
                end: source.len()
            })
        );
        assert!(tokens
            .windows(2)
            .all(|pair| pair[0].span.end <= pair[1].span.start));
    }
}
