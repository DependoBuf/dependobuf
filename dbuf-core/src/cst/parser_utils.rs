//! Module contains utils functions for parsing.
//!
//! Contains:
//!   * `MapToken` trait, that maps `Token` to `Child::Token`.
//!   * `MapTree` trait, that maps `Vec<Child>` like to `Child::Tree`.
//!   * `ChildFlatten` trait that maps `Vec<Child>` like to `Vec<Child>`.

use chumsky::Parser;
use chumsky::extra::ParserExtra;
use chumsky::input::Input;

use super::Location;
use super::{Child, Token, Tree, TreeKind};

/// Trait implemented for every `Token` parser.
///
/// Contains method that convert `Token` to `Child::Token`.
pub trait MapToken<'src, I, E>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    /// From parser with output `Token` generate parser
    /// with output `Child::Token`.
    fn map_token(self) -> impl Parser<'src, I, Child, E> + Clone;
}

impl<'src, I, E, P> MapToken<'src, I, E> for P
where
    I: Input<'src, Span = Location, Token = Token>,
    E: ParserExtra<'src, I>,
    P: Parser<'src, I, Token, E> + Clone,
{
    fn map_token(self) -> impl Parser<'src, I, Child, E> + Clone {
        self.map_with(|t, extra| Child::Token(t, extra.span()))
    }
}

/// Trait implemented for every `Child` type parser.
///
/// Contains method that convert output to `Child::Tree`.
///
/// Generic parameters:
///   * `'src` - lifetime of src.
///   * `I` - Input for parser.
///   * `O` - `ChildFlatten` output of parser.
///   * `E` - Extra for parser.
pub trait MapTree<'src, I, O, E>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    /// From parser with `Vec<Child>` like output generate parser
    /// with output `Child::Tree`.
    fn map_tree(self, kind: TreeKind) -> impl Parser<'src, I, Child, E> + Clone;
}

impl<'src, I, O, E, P> MapTree<'src, I, O, E> for P
where
    I: Input<'src, Span = Location, Token = Token>,
    E: ParserExtra<'src, I>,
    P: Parser<'src, I, O, E> + Clone,
    O: ChildFlatten,
{
    fn map_tree(self, kind: TreeKind) -> impl Parser<'src, I, Child, E> + Clone {
        self.map_with(move |ch, extra| {
            Child::Tree(Tree {
                kind: kind.clone(),
                location: extra.span(),
                children: ch.flatten(),
            })
        })
    }
}

/// Trait that flattens `T = Child, Option<Child>, Vec<Child>, (T, T)` to
/// `Vec<Child>`.
trait ChildFlatten {
    /// Flattens complex `Child` struct to simple `Vec<Child>`.
    fn flatten(self) -> Vec<Child>;
}

impl ChildFlatten for Child {
    fn flatten(self) -> Vec<Child> {
        vec![self]
    }
}

impl ChildFlatten for Option<Child> {
    fn flatten(self) -> Vec<Child> {
        self.map_or(vec![], |ch| vec![ch])
    }
}

impl ChildFlatten for Vec<Child> {
    fn flatten(self) -> Vec<Child> {
        self
    }
}

impl<T, U> ChildFlatten for (T, U)
where
    T: ChildFlatten,
    U: ChildFlatten,
{
    fn flatten(self) -> Vec<Child> {
        let mut lhs = self.0.flatten();
        let mut rhs = self.1.flatten();
        lhs.append(&mut rhs);
        lhs
    }
}
