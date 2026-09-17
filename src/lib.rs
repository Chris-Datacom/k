//! The K compiler library.
//!
//! K is a small, low-level language inspired by C. This crate is deliberately
//! split into compiler stages so that the eventual self-hosted compiler can
//! reproduce the same boundaries:
//!
//! 1. [`lexer`] turns bytes into source-spanned tokens.
//! 2. [`parser`] turns tokens into an abstract syntax tree.
//! 3. [`sema`] resolves names and enforces K's initial type rules.
//! 4. [`ir`] lowers checked programs into a typed intermediate boundary.
//! 5. [`codegen`] emits a small native executable.
//!
//! The initial milestones keep each stage useful and testable, giving the
//! language a concrete foundation before syntax and code generation grow.

pub mod lexer;
pub mod parser;
pub mod sema;
pub mod codegen;
pub mod ir;
