use num_traits::{One, Zero};
use std::collections::BTreeMap;

use crate::ast::elaborated::ValueExpression;
use crate::ast::operators::Literal;

pub trait ArithCoeff:
    Clone
    + Copy
    + PartialEq
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + Zero
    + One
{
    #[must_use]
    fn coeff_neg(self) -> Self;
    fn is_neg(self) -> bool;
    #[must_use]
    fn coeff_abs(self) -> Self {
        if self.is_neg() {
            self.coeff_neg()
        } else {
            self
        }
    }
    fn try_from_literal(lit: &Literal) -> Option<Self>;
    fn to_literal(self) -> Literal;
}

impl ArithCoeff for i64 {
    fn coeff_neg(self) -> Self {
        -self
    }
    fn is_neg(self) -> bool {
        self < 0
    }
    fn try_from_literal(lit: &Literal) -> Option<Self> {
        match lit {
            Literal::Int(n) => Some(*n),
            _ => None,
        }
    }
    fn to_literal(self) -> Literal {
        Literal::Int(self)
    }
}

impl ArithCoeff for u64 {
    fn coeff_neg(self) -> Self {
        unreachable!("negation is not defined for UInt")
    }
    fn is_neg(self) -> bool {
        false
    }
    fn try_from_literal(lit: &Literal) -> Option<Self> {
        match lit {
            Literal::UInt(n) => Some(*n),
            _ => None,
        }
    }
    fn to_literal(self) -> Literal {
        Literal::UInt(self)
    }
}

pub type Mono = BTreeMap<usize, u32>;
pub type Poly<C> = BTreeMap<Mono, C>;

#[derive(Debug, Clone)]
pub struct NormalForm<Str, C> {
    pub vars: Vec<ValueExpression<Str>>,
    pub poly: Poly<C>,
}
