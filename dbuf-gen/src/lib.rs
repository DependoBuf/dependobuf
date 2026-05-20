#![cfg(any(feature = "rust", feature = "kotlin", feature = "swift"))]

mod ast;
mod format;
mod scope;

#[cfg(feature = "rust")]
pub mod codegen;
#[cfg(feature = "rust")]
mod generate;
#[cfg(feature = "rust")]
mod rust_gen;

#[cfg(feature = "kotlin")]
pub mod kotlin_gen;
#[cfg(feature = "swift")]
pub mod swift_gen;
