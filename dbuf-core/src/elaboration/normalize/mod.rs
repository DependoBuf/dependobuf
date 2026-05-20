pub mod arith;
pub mod boolean;
pub mod strings;

use crate::ast::elaborated::ValueExpression;
use crate::elaboration::builtins::{self, BuiltinType};

fn find_var<Str: Clone + PartialEq>(
    vars: &mut Vec<ValueExpression<Str>>,
    expr: ValueExpression<Str>,
) -> usize {
    if let Some(idx) = vars.iter().position(|a| a == &expr) {
        return idx;
    }
    let idx = vars.len();
    vars.push(expr);
    idx
}

pub fn simplify<Str>(expr: &ValueExpression<Str>) -> ValueExpression<Str>
where
    Str: Clone + PartialEq + From<BuiltinType>,
{
    let result_type = crate::elaboration::type_of(expr);
    if result_type == builtins::get_builtin(&BuiltinType::UInt) {
        let nf = arith::normalize::<Str, u64>(expr);
        arith::poly_to_expr(&nf, result_type)
    } else if result_type == builtins::get_builtin(&BuiltinType::Int) {
        let nf = arith::normalize::<Str, i64>(expr);
        arith::poly_to_expr(&nf, result_type)
    } else if result_type == builtins::get_builtin(&BuiltinType::Bool) {
        let nf = boolean::normalize(expr);
        boolean::poly_to_expr(&nf, result_type)
    } else if result_type == builtins::get_builtin(&BuiltinType::String) {
        let strings = strings::normalize(expr);
        strings::strings_to_expr(&strings, result_type)
    } else {
        expr.clone()
    }
}
