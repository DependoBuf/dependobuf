pub mod advanced;
pub mod builtins;
pub mod checker;
pub mod context;
pub mod graph;
mod integration_tests;
pub mod interning;
pub mod scope_checks;
pub mod simple;
pub mod strategy;

// Re-export key types for convenience
pub use advanced::AdvancedTyper;
pub use builtins::BuiltinTypes;
pub use checker::{TypeChecker, TypeCheckError};
pub use context::Context;
pub use graph::TopSortBuilder;
pub use interning::{InternedString, StringInterner, ModuleInterner};
pub use scope_checks::{ScopeChecker, ScopeCheckerError};
pub use simple::{SimpleTyper, SimpleTyperError, SimpleType};
pub use strategy::{StrategyBuilder, StrategyError, CheckerTask};