use super::Visit;
use super::VisitResult;

/// Skips only allowed visits.
pub fn safe_skip<T>(visit: &Visit<'_>) -> VisitResult<T> {
    match visit {
        Visit::Keyword(_, _location) => VisitResult::Skip,
        Visit::Type(_loc_string, _locationn) => VisitResult::Skip,
        Visit::Dependency(_loc_string, _location) => VisitResult::Skip,
        Visit::Branch => VisitResult::Skip,
        Visit::PatternAlias(_loc_string) => VisitResult::Continue,
        Visit::PatternCall(_loc_string, _location) => VisitResult::Skip,
        Visit::PatternCallArgument(_loc_string) => VisitResult::Skip,
        Visit::PatternCallStop => VisitResult::Continue,
        Visit::PatternLiteral(_literal, _locationn) => VisitResult::Continue,
        Visit::PatternUnderscore(_location) => VisitResult::Continue,
        Visit::Constructor(_constructor) => VisitResult::Skip,
        Visit::Filed(_loc_string, _locationn) => VisitResult::Skip,
        Visit::TypeExpression(_loc_string, _locationn) => VisitResult::Skip,
        Visit::Expression(_location) => VisitResult::Skip,
        Visit::AccessChainStart => VisitResult::Skip,
        Visit::AccessChain(_loc_string) => VisitResult::Continue,
        Visit::AccessDot(_location) => VisitResult::Continue,
        Visit::AccessChainLast(_loc_string) => VisitResult::Continue,
        Visit::ConstructorExpr(_loc_string) => VisitResult::Skip,
        Visit::ConstructorExprArgument(_loc_string) => VisitResult::Skip,
        Visit::ConstructorExprStop => VisitResult::Continue,
        Visit::VarAccess(_loc_string) => VisitResult::Continue,
        Visit::Operator(_, _location) => VisitResult::Continue,
        Visit::Literal(_literal, _locationn) => VisitResult::Continue,
    }
}
