use tower_lsp::lsp_types::*;

use crate::core::ast_access::{
    ElaboratedAst, ElaboratedHelper, File, Loc, LocStringHelper, LocationHelpers, Str,
};
use crate::core::ast_visitor::*;

/// Returns all document symbols of file.
pub fn provide_document_symbols(file: &File) -> Vec<DocumentSymbol> {
    let mut visitor = SymbolVisitor::new(file);
    visit_ast(file.get_parsed(), &mut visitor, file.get_elaborated());
    visitor.collect()
}

#[allow(
    deprecated,
    reason = "Field `deprecated` of structure DocumentSymbol is deprecated, but need to init it with None"
)]
fn get_symbol(
    name: String,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    DocumentSymbol {
        name,
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children,
    }
}

trait BuilderState {}

struct Empty {}
struct Message {
    type_symbol: DocumentSymbol,
}
struct Enum {
    type_symbol: DocumentSymbol,
}
struct Constructor {
    type_symbol: DocumentSymbol,
    constructor_symbol: DocumentSymbol,
}

impl BuilderState for Empty {}
impl BuilderState for Message {}
impl BuilderState for Enum {}
impl BuilderState for Constructor {}

struct Builder<S: BuilderState> {
    response: Vec<DocumentSymbol>,
    extra: S,
}

impl Builder<Empty> {
    fn push_message(self, type_name: &Str, loc: &Loc) -> Builder<Message> {
        let s = get_symbol(
            type_name.to_string(),
            SymbolKind::STRUCT,
            loc.to_lsp(),
            type_name.get_location().to_lsp(),
            Some(Vec::new()),
        );
        Builder {
            response: self.response,
            extra: Message { type_symbol: s },
        }
    }
    fn push_enum(self, type_name: &Str, loc: &Loc) -> Builder<Enum> {
        let s = get_symbol(
            type_name.to_string(),
            SymbolKind::ENUM,
            loc.to_lsp(),
            type_name.get_location().to_lsp(),
            Some(Vec::new()),
        );
        Builder {
            response: self.response,
            extra: Enum { type_symbol: s },
        }
    }
    fn collect(self) -> Vec<DocumentSymbol> {
        self.response
    }
}
impl Builder<Message> {
    fn push_field(mut self, field_name: &Str, loc: &Loc) -> Builder<Message> {
        let s = get_symbol(
            field_name.to_string(),
            SymbolKind::FIELD,
            loc.to_lsp(),
            field_name.get_location().to_lsp(),
            None,
        );
        self.extra
            .type_symbol
            .children
            .as_mut()
            .expect("message symbol have children")
            .push(s);
        self
    }

    fn stop(mut self) -> Builder<Empty> {
        self.response.push(self.extra.type_symbol);
        Builder {
            response: self.response,
            extra: Empty {},
        }
    }
}

impl Builder<Enum> {
    fn push_constructor(self, cons_name: &Str, loc: &Loc) -> Builder<Constructor> {
        let s = get_symbol(
            cons_name.to_string(),
            SymbolKind::ENUM_MEMBER,
            loc.to_lsp(),
            cons_name.get_location().to_lsp(),
            Some(Vec::new()),
        );
        Builder {
            response: self.response,
            extra: Constructor {
                type_symbol: self.extra.type_symbol,
                constructor_symbol: s,
            },
        }
    }

    fn stop(mut self) -> Builder<Empty> {
        self.response.push(self.extra.type_symbol);
        Builder {
            response: self.response,
            extra: Empty {},
        }
    }
}

impl Builder<Constructor> {
    fn push_field(mut self, field_name: &Str, loc: &Loc) -> Builder<Constructor> {
        let s = get_symbol(
            field_name.to_string(),
            SymbolKind::FIELD,
            loc.to_lsp(),
            field_name.get_location().to_lsp(),
            None,
        );
        self.extra
            .constructor_symbol
            .children
            .as_mut()
            .expect("constructor symbol have children")
            .push(s);
        self
    }

    fn stop(mut self) -> Builder<Enum> {
        self.extra
            .type_symbol
            .children
            .as_mut()
            .expect("enum symbol have children")
            .push(self.extra.constructor_symbol);
        Builder {
            response: self.response,
            extra: Enum {
                type_symbol: self.extra.type_symbol,
            },
        }
    }
}
enum SymbolBuilder {
    EmptyBuilder(Builder<Empty>),
    MessageBuilder(Builder<Message>),
    EnumBuilder(Builder<Enum>),
    ConstructorBuilder(Builder<Constructor>),
    Invalid,
}

impl From<Builder<Empty>> for SymbolBuilder {
    fn from(value: Builder<Empty>) -> Self {
        SymbolBuilder::EmptyBuilder(value)
    }
}

impl From<Builder<Message>> for SymbolBuilder {
    fn from(value: Builder<Message>) -> Self {
        SymbolBuilder::MessageBuilder(value)
    }
}

impl From<Builder<Enum>> for SymbolBuilder {
    fn from(value: Builder<Enum>) -> Self {
        SymbolBuilder::EnumBuilder(value)
    }
}

impl From<Builder<Constructor>> for SymbolBuilder {
    fn from(value: Builder<Constructor>) -> Self {
        SymbolBuilder::ConstructorBuilder(value)
    }
}

impl SymbolBuilder {
    fn new() -> SymbolBuilder {
        Builder {
            response: Vec::new(),
            extra: Empty {},
        }
        .into()
    }
}

impl Default for SymbolBuilder {
    fn default() -> Self {
        Self::Invalid
    }
}

struct SymbolVisitor<'a> {
    elaborated: &'a ElaboratedAst,

    builder: SymbolBuilder,
}

impl SymbolVisitor<'_> {
    fn new(file: &File) -> SymbolVisitor<'_> {
        SymbolVisitor {
            elaborated: file.get_elaborated(),
            builder: SymbolBuilder::new(),
        }
    }
    fn collect(self) -> Vec<DocumentSymbol> {
        let empty_builder = match self.builder {
            SymbolBuilder::EmptyBuilder(builder) => builder,
            SymbolBuilder::MessageBuilder(builder) => builder.stop(),
            SymbolBuilder::EnumBuilder(builder) => builder.stop(),
            SymbolBuilder::ConstructorBuilder(builder) => builder.stop().stop(),
            SymbolBuilder::Invalid => panic!("invalid state of Symbol builder"),
        };

        empty_builder.collect()
    }

    fn push_type_symbol(&mut self, type_name: &Str, loc: &Loc) {
        let builder = std::mem::take(&mut self.builder);

        let empty_builder = match builder {
            SymbolBuilder::EmptyBuilder(builder) => builder,
            SymbolBuilder::MessageBuilder(builder) => builder.stop(),
            SymbolBuilder::EnumBuilder(builder) => builder.stop(),
            SymbolBuilder::ConstructorBuilder(builder) => builder.stop().stop(),
            SymbolBuilder::Invalid => panic!("invalid state of Symbol builder"),
        };

        if self.elaborated.is_message(type_name.as_ref()) {
            self.builder = empty_builder.push_message(type_name, loc).into();
        } else {
            self.builder = empty_builder.push_enum(type_name, loc).into();
        }
    }

    fn push_field(&mut self, field_name: &Str, loc: &Loc) {
        let builder = std::mem::take(&mut self.builder);

        self.builder = match builder {
            SymbolBuilder::MessageBuilder(builder) => builder.push_field(field_name, loc).into(),
            SymbolBuilder::ConstructorBuilder(builder) => {
                builder.push_field(field_name, loc).into()
            }
            _ => panic!("bad builder state"),
        };
    }

    fn push_constructor(&mut self, cons_name: &Str, loc: &Loc) {
        let builder = std::mem::take(&mut self.builder);

        let enum_builder = match builder {
            SymbolBuilder::EnumBuilder(builder) => builder,
            SymbolBuilder::ConstructorBuilder(builder) => builder.stop(),
            _ => panic!("bad builder state"),
        };

        self.builder = enum_builder.push_constructor(cons_name, loc).into()
    }
}

impl<'a> Visitor<'a> for SymbolVisitor<'a> {
    type StopResult = ();
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, location) => self.push_type_symbol(type_name, location),
            Visit::Dependency(_, _) => return VisitResult::Skip,
            Visit::Branch => {}
            Visit::PatternAlias(_) => {}
            Visit::PatternCall(_, _) => return VisitResult::Skip,
            Visit::PatternCallArgument(_) => {}
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(c) => {
                if !c.of_message {
                    self.push_constructor(c.name, c.loc)
                }
            }
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
