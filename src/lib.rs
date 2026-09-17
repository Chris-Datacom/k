//! The K compiler library.
//!
//! K is a small, low-level language inspired by C. This crate is deliberately
//! split into compiler stages so that the eventual self-hosted compiler can
//! reproduce the same boundaries:
//!
//! 1. [`lexer`] turns bytes into source-spanned tokens.
//! 2. A parser will turn tokens into an abstract syntax tree.
//! 3. Semantic analysis will resolve names and enforce K's type rules.
//! 4. Code generation will eventually emit a small native executable.
//!
//! The first milestone only implements stage one. Keeping it useful and
//! testable gives the language a concrete foundation before syntax grows.

pub mod lexer;
