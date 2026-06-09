#![cfg(any(feature = "rust", feature = "kotlin", feature = "swift"))]
#![cfg_attr(
    not(feature = "rust"),
    allow(
        dead_code,
        reason = "some code using only in rust feature (FIXME: separate crates by code usage)"
    )
)]
#![cfg_attr(
    not(feature = "rust"),
    allow(
        unused_imports,
        reason = "some code using only in rust feature (FIXME: separate crates by cde usage)"
    )
)]

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
