//! Inlay hint provider.
//!

use crate::core::ast_access::{
    ElaboratedAst, ElaboratedHelper, File, LocStringHelper, LocationHelpers, PositionHelpers, Str,
};
use crate::core::ast_visitor::VisitResult::*;
use crate::core::ast_visitor::scope_visitor::ScopeVisitor;
use crate::core::ast_visitor::*;

use dbuf_core::ast::elaborated::TypeExpression;
use tower_lsp::lsp_types::*;

pub fn get_inlay_hint(range: Range, file: &File) -> Vec<InlayHint> {
    let mut visitor = InlayVisitor {
        range,
        elaborated: file.get_elaborated(),
        scope: ScopeVisitor::new(file.get_elaborated()),
        ans: vec![],
    };

    visit_ast(file.get_parsed(), &mut visitor, file.get_elaborated());

    visitor.collect()
}

struct InlayVisitor<'a> {
    range: Range,
    elaborated: &'a ElaboratedAst,
    scope: ScopeVisitor<'a>,
    ans: Vec<InlayHint>,
}

impl InlayVisitor<'_> {
    fn collect(self) -> Vec<InlayHint> {
        self.ans
    }

    fn get_type_of_field(&self, name: &Str) -> String {
        let constructor = self.scope.get_constructor_expr();
        let type_name = self
            .elaborated
            .get_constructor(constructor)
            .expect("valid ast")
            .fields
            .iter()
            .find(|f| f.0 == name.as_ref())
            .map(
                |(
                    _,
                    TypeExpression::TypeExpression {
                        name,
                        dependencies: _,
                    },
                )| name,
            )
            .expect("valid ast");
        type_name.to_owned()
    }

    fn save_hint(&mut self, name: &Str) {
        assert!(self.scope.has_constructor_expr());

        let type_name = self.get_type_of_field(name);

        let mut pos = name.get_location().end();
        pos.columns += 1; // ':' character 

        self.ans.push(InlayHint {
            position: pos.to_lsp(),
            label: InlayHintLabel::String(type_name),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: None,
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }
}

impl<'a> Visitor<'a> for InlayVisitor<'a> {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Type(_, location) if !location.intersects(self.range) => return Skip,
            Visit::Dependency(_, location) if !location.intersects(self.range) => return Skip,
            Visit::PatternCall(_, location) if !location.intersects(self.range) => return Skip,
            Visit::Constructor(constructor) if !constructor.loc.intersects(self.range) => {
                return Skip;
            }
            Visit::Filed(_, location) if !location.intersects(self.range) => return Skip,
            Visit::TypeExpression(_, location) if !location.intersects(self.range) => return Skip,
            Visit::Expression(location) if !location.intersects(self.range) => return Skip,
            _ => {}
        };

        match &visit {
            Visit::PatternCallArgument(arg) => self.save_hint(arg),
            Visit::ConstructorExprArgument(arg) => self.save_hint(arg),
            _ => {}
        }

        assert!(matches!(self.scope.visit(visit), Continue));

        Continue
    }
}
