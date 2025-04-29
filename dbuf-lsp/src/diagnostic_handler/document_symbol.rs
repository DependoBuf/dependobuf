use tower_lsp::lsp_types::*;

use crate::core::ast_access::{
    ElaboratedAst, ElaboratedHelper, File, Loc, LocStringHelper, LocationHelpers, ParsedAst, Str,
};
use crate::core::ast_visitor::*;

pub struct DocumentSymbolProvider<'a> {
    parsed: &'a ParsedAst,
    elaborated: &'a ElaboratedAst,

    result: Vec<DocumentSymbol>,
    last_enum: bool,
}

#[allow(deprecated)] // Init DocumentSymbol field deprecated
impl DocumentSymbolProvider<'_> {
    pub fn new(file: &File) -> DocumentSymbolProvider<'_> {
        DocumentSymbolProvider {
            parsed: file.get_parsed(),
            elaborated: file.get_elaborated(),
            result: Vec::new(),
            last_enum: false,
        }
    }

    pub fn provide(&mut self) -> DocumentSymbolResponse {
        visit_ast(self.parsed, self, self.elaborated);

        std::mem::take(&mut self.result).into()
    }

    fn push_type_symbol(&mut self, type_name: &Str, loc: &Loc) {
        let kind = if self.elaborated.is_message(type_name.as_ref()) {
            self.last_enum = false;
            SymbolKind::STRUCT
        } else {
            self.last_enum = true;
            SymbolKind::ENUM
        };

        let s = DocumentSymbol {
            name: type_name.to_string(),
            detail: None,
            kind,
            tags: None,
            deprecated: None,
            range: loc.to_lsp(),
            selection_range: type_name.get_location().to_lsp(),
            children: Some(Vec::new()),
        };
        self.result.push(s);
    }

    fn push_dependency(&mut self, dep_name: &Str, loc: &Loc) {
        let s = DocumentSymbol {
            name: dep_name.to_string(),
            detail: None,
            kind: SymbolKind::VARIABLE,
            tags: None,
            deprecated: None,
            range: loc.to_lsp(),
            selection_range: dep_name.get_location().to_lsp(),
            children: None,
        };

        self.result
            .last_mut()
            .unwrap()
            .children
            .as_mut()
            .unwrap()
            .push(s);
    }

    fn push_field(&mut self, field_name: &Str, loc: &Loc) {
        let s = DocumentSymbol {
            name: field_name.to_string(),
            detail: None,
            kind: SymbolKind::FIELD,
            tags: None,
            deprecated: None,
            range: loc.to_lsp(),
            selection_range: field_name.get_location().to_lsp(),
            children: None,
        };

        if !self.last_enum {
            self.result
                .last_mut()
                .unwrap()
                .children
                .as_mut()
                .unwrap()
                .push(s);
        } else {
            self.result
                .last_mut()
                .unwrap()
                .children
                .as_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .children
                .as_mut()
                .unwrap()
                .push(s);
        }
    }

    fn push_constructor(&mut self, cons_name: &Str, loc: &Loc) {
        if !self.last_enum {
            return;
        }

        let s = DocumentSymbol {
            name: cons_name.to_string(),
            detail: None,
            kind: SymbolKind::CONSTRUCTOR,
            tags: None,
            deprecated: None,
            range: loc.to_lsp(),
            selection_range: cons_name.get_location().to_lsp(),
            children: Some(Vec::new()),
        };

        self.result
            .last_mut()
            .unwrap()
            .children
            .as_mut()
            .unwrap()
            .push(s);
    }
}

impl<'a> Visitor<'a> for DocumentSymbolProvider<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        match &visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, location) => self.push_type_symbol(type_name, location),
            Visit::Dependency(dep_name, location) => self.push_dependency(dep_name, location),
            Visit::Branch => {}
            Visit::PatternAlias(_) => {}
            Visit::PatternCall(_, _) => return VisitResult::Skip,
            Visit::PatternCallArgument(_) => {}
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(c) => self.push_constructor(c.name, c.loc),
            Visit::Filed(field_name, location) => self.push_field(field_name, location),
            Visit::TypeExpression(_, _) => return VisitResult::Skip,
            Visit::Expression(_) => return VisitResult::Skip,
            Visit::AccessChainStart => return VisitResult::Skip,
            Visit::AccessChain(_) => {}
            Visit::AccessDot(_) => {}
            Visit::AccessChainLast(_) => {}
            Visit::ConstructorExpr(_) => return VisitResult::Skip,
            Visit::ConstructorExprArgument(_) => {}
            Visit::ConstructorExprStop => {}
            Visit::VarAccess(_) => {}
            Visit::Operator(_, _) => {}
            Visit::Literal(_, _) => {}
        }
        VisitResult::Continue
    }
}
