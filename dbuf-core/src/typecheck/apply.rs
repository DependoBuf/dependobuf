use crate::ast::elaborated as e;
use crate::error::elaborating::Error::{self, ElaboratingError};
use crate::typecheck::{subst, unify};

pub type Bindings<Str> = Vec<(Str, e::ValueExpression<Str>)>;

fn type_of<Str: Clone>(expr: &e::ValueExpression<Str>) -> e::TypeExpression<Str> {
    match expr {
        e::ValueExpression::Variable { ty, .. } => ty.clone(),
        e::ValueExpression::Constructor { result_type, .. } => result_type.clone(),
        e::ValueExpression::OpCall { result_type, .. } => result_type.clone(),
    }
}

/// # Panics
/// # Errors
pub fn application<Str>(
    constructor: &e::Constructor<Str>,
    arg: &e::ValueExpression<Str>,
    module: &e::Module<Str>,
) -> Result<(e::Constructor<Str>, Bindings<Str>), Error>
where
    Str: Clone + Eq + Ord,
{
    let e::Constructor {
        implicits,
        fields,
        result_type,
    } = constructor;

    let ((_var_name, field_type), rest_fields) = fields.split_first().ok_or(ElaboratingError)?;

    let arg_type = type_of(arg);
    let (arg_bindings, implicit_bindings) =
        unify::unify_type(&arg_type, field_type, module).map_err(|_| ElaboratingError)?;

    let new_fields = rest_fields
        .iter()
        .map(|(name, ty)| {
            let ty = subst::apply_bindings_to_type(ty.clone(), &implicit_bindings);
            (name.clone(), ty)
        })
        .collect();

    let new_result_type = subst::apply_bindings_to_type(result_type.clone(), &implicit_bindings);

    Ok((
        e::Constructor {
            implicits: implicits.clone(),
            fields: new_fields,
            result_type: new_result_type,
        },
        arg_bindings,
    ))
}
