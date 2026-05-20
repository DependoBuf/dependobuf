use crate::ast::elaborated as e;
use crate::elaboration::builtins::BuiltinType;
use crate::elaboration::{subst, type_of, unify};
use crate::error::elaborating::Error::{self, ArityMismatch};

/// # Errors
pub fn application<Str>(
    constructor: &e::Constructor<Str>,
    arg: &e::ValueExpression<Str>,
    module: &e::Module<Str>,
) -> Result<e::Constructor<Str>, Error>
where
    Str: Clone + Eq + Ord + From<BuiltinType> + ToString,
{
    let e::Constructor {
        implicits,
        fields,
        result_type,
    } = constructor;

    let ((_var_name, field_type), rest_fields) = fields.split_first().ok_or(ArityMismatch {
        expected: 1,
        found: 0,
    })?;

    let arg_type = type_of(arg);
    let implicit_bindings = unify::unify_type(&arg_type, field_type, module)?;

    let new_fields = rest_fields
        .iter()
        .map(|(name, ty)| {
            let ty = subst::apply_bindings_to_type(ty.clone(), &implicit_bindings);
            (name.clone(), ty)
        })
        .collect();

    let new_result_type = subst::apply_bindings_to_type(result_type.clone(), &implicit_bindings);

    Ok(e::Constructor {
        implicits: implicits.clone(),
        fields: new_fields,
        result_type: new_result_type,
    })
}
