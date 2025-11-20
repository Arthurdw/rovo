//! Rovo LSP - Language Server Protocol implementation for Rovo annotations
//!
//! This crate provides LSP support for Rovo, including:
//! - Syntax highlighting and validation of Rovo annotations
//! - Auto-completion for annotations, status codes, and security schemes
//! - Hover information for types, status codes, and annotations
//! - Code actions for adding annotations and derives
//! - Go-to-definition for response types
//! - Find references for tags

pub mod backend;
pub mod code_actions;
pub mod completion;
pub mod diagnostics;
pub mod docs;
pub mod handlers;
pub mod parser;
pub mod type_resolver;
