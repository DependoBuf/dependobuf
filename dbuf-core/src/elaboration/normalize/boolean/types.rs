use crate::ast::elaborated::ValueExpression;
use std::collections::{BTreeMap, BTreeSet};

pub type Mono = BTreeMap<usize, bool>;
pub type Poly = BTreeSet<Mono>;

pub struct NormalForm<Str> {
    pub vars: Vec<ValueExpression<Str>>,
    pub poly: Poly,
}
