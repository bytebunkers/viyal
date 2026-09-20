//! # High-level Intermediate Representation (HIR)
//!
//! **Status: Planned (v0.2)**
//!
//! HIR will sit between the AST and MIR. Its primary job is **desugaring**:
//!
//! | Sugar | Lowered Form |
//! |---|---|
//! | `for x in list` | `while` loop + index counter |
//! | `obj?.field` (safe access) | explicit `if-null` branch |
//! | `expr or { ... }` (unwrap-or-else) | explicit `match` on Result/Option |
//! | `type Alias = T` | expanded inline everywhere it is used |
//! | `fn foo<T>(x: T)` | monomorphised copies per concrete `T` |
//!
//! Currently [`compiler/mir/builder.rs`] operates directly on the raw AST.
//! When HIR is implemented it will be inserted between:
//!
//! ```text
//! compiler/parser  →  compiler/hir  →  compiler/mir
//! ```
//!
//! The planned public API will be:
//! ```ignore
//! pub fn lower_ast(program: ast::Program) -> HirProgram { ... }
//! ```

/// Opaque placeholder — remove when HIR is implemented in v0.2.
pub struct HirProgram;
