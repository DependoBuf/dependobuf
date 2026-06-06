use indexmap::IndexMap;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemLabelDetails, CompletionResponse,
    CompletionTriggerKind, Position,
};

use crate::core::ast_visitor::*;
use crate::core::workspace::{File, LocNameHelper, LocationHelper, Name, Str};

pub fn run_completion(
    pos: Position,
    file: &File,
    ctx: Option<CompletionContext>,
) -> Option<CompletionResponse> {
    let mut collector_visitor = DataCollectorVisitor::new();

    let ast = file.get_parsed().take()?;

    visit_ast(ast, &mut collector_visitor);
    let data = collector_visitor.take();

    let activation_kind = match ctx {
        Some(x) if x.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER => {
            ActivationKind::Dot
        }
        Some(_) => ActivationKind::Basic,
        None => ActivationKind::Basic,
    };

    let mut completion_visitor = CompletionVisitor::new(pos, data, activation_kind);
    let ans_fields = visit_ast(ast, &mut completion_visitor)?;

    let ans = ans_fields.into_iter().map(|f| f.to_lsp()).collect();
    Some(CompletionResponse::Array(ans))
}

/// Field represetation.
#[derive(Clone)]
struct Field {
    /// Name of field.
    name: Str,
    /// Type of field without dependencies.
    ty: Str,
}

impl Field {
    pub fn to_lsp(&self) -> CompletionItem {
        CompletionItem {
            label: self.name.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: Some(self.ty.to_string()),
            }),
            kind: None,
            detail: Some(self.ty.to_string()),
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: Some(vec![".".to_string()]),
            data: None,
            tags: None,
        }
    }
}

/// Struct contains all constructors and their fields.
struct ConstructorsFields {
    /// pairs of constructors and theirs fields.
    data: IndexMap<Str, Vec<Field>>,
}

impl ConstructorsFields {
    /// Return all fields of a type.
    pub fn fields(&self, ctr: &Str) -> Vec<Field> {
        self.data.get(ctr).cloned().unwrap_or(vec![])
    }

    /// Return type of field of costructor if any.
    pub fn hop(&self, current_ctr: &Str, field: &Str) -> Option<Str> {
        let fields = self.data.get(current_ctr)?;
        fields
            .iter()
            .find(|f| f.name == *field)
            .map(|f| f.ty.clone())
    }
}

/// Collects information about field.
struct FieldCollector {
    /// current stored field name.
    field_name: Option<Str>,
    /// current stored field type.
    field_type: Option<Str>,
}

impl FieldCollector {
    fn set_name(&mut self, name: Str) {
        assert!(self.field_name.is_none());
        self.field_name = name.into();
    }

    fn set_type(&mut self, ty: Str) {
        assert!(self.field_type.is_none());
        self.field_type = ty.into();
    }

    fn take(&mut self) -> Field {
        assert!(self.field_name.is_some());
        assert!(self.field_type.is_some());

        Field {
            name: self.field_name.take().unwrap(),
            ty: self.field_type.take().unwrap(),
        }
    }
}

/// Visitor that collects all constructors and their fields.
struct DataCollectorVisitor {
    /// collected constructors with fields.
    data: ConstructorsFields,
    /// name of current constructor.
    constructor_name: Option<Str>,
    /// current collector of field
    collector: FieldCollector,
}

impl DataCollectorVisitor {
    pub fn new() -> DataCollectorVisitor {
        DataCollectorVisitor {
            data: ConstructorsFields {
                data: IndexMap::new(),
            },
            constructor_name: None,
            collector: FieldCollector {
                field_name: None,
                field_type: None,
            },
        }
    }

    fn enter(&mut self, name: &Name) {
        self.constructor_name = name.content.clone().into();
    }

    fn on_field_name(&mut self, name: &Name) {
        self.collector.set_name(name.content.clone());
    }

    fn on_field_type(&mut self, ty: &Name) {
        self.collector.set_type(ty.content.clone());
        let field = self.collector.take();

        if let Some(old) = self
            .data
            .data
            .get_mut(self.constructor_name.as_ref().unwrap())
        {
            old.push(field);
        } else {
            let old = self
                .data
                .data
                .insert(self.constructor_name.as_ref().unwrap().clone(), vec![field]);
            assert!(old.is_none());
        }
    }

    pub fn take(self) -> ConstructorsFields {
        self.data
    }
}

impl<'a> Visitor<'a> for DataCollectorVisitor {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(name, _) => self.enter(name),
            Visit::Dependency(_, _) => return Skip,
            Visit::Branch => {}
            Visit::PatternAlias(_) => return Skip,
            Visit::PatternCall(_, _) => return Skip,
            Visit::PatternCallArgument(_) => return Skip,
            Visit::PatternCallStop => return Skip,
            Visit::PatternLiteral(_, _) => return Skip,
            Visit::PatternUnderscore(_) => return Skip,
            Visit::Constructor(constructor) => self.enter(constructor.name),
            Visit::Filed(name, _) => self.on_field_name(name),
            Visit::TypeExpression(ty, _) => {
                self.on_field_type(ty);
                return Skip;
            }
            Visit::Expression(_) => return Skip,
            Visit::AccessChainStart => return Skip,
            Visit::AccessChain(_) => return Skip,
            Visit::AccessDot(_) => return Skip,
            Visit::AccessChainLast(_) => return Skip,
            Visit::ConstructorExpr(_) => return Skip,
            Visit::ConstructorExprArgument(_) => return Skip,
            Visit::ConstructorExprStop => return Skip,
            Visit::VarAccess(_) => return Skip,
            Visit::Operator(_, _) => return Skip,
            Visit::Literal(_, _) => return Skip,
        }

        Continue
    }
}

/// Kind of completion activation.
enum ActivationKind {
    /// Simple writing --- returning current struct data.
    Basic,
    /// On dot --- returning next struct data.
    Dot,
}

struct CompletionVisitor {
    pos: Position,
    constructor_fields: ConstructorsFields,
    current_constructor: Option<Str>,
    activation_kind: ActivationKind,
}

impl CompletionVisitor {
    pub fn new(
        mut pos: Position,
        fields: ConstructorsFields,
        activation_kind: ActivationKind,
    ) -> CompletionVisitor {
        // FIXME: currently looks strage, but that is
        // how hangind dot access is handled in current AST.
        if matches!(activation_kind, ActivationKind::Dot) {
            pos.character -= 1;
        }

        CompletionVisitor {
            pos,
            constructor_fields: fields,
            current_constructor: None,
            activation_kind,
        }
    }

    fn enter_constructor(&mut self, ctr: &Name) {
        self.current_constructor = ctr.content.clone().into();
    }

    fn hop(&mut self, name: &Name) {
        self.current_constructor = self
            .current_constructor
            .as_ref()
            .and_then(|old| self.constructor_fields.hop(old, &name.content));
    }

    fn stopped_at(&mut self, name: &Name) -> Vec<Field> {
        assert!(name.contains(self.pos));

        if matches!(self.activation_kind, ActivationKind::Dot) {
            self.hop(name);
        }

        let Some(ctr) = &self.current_constructor else {
            return vec![];
        };

        self.constructor_fields.fields(ctr)
    }

    fn part_of_chain(&mut self, name: &Name) {
        assert!(!name.contains(self.pos));

        self.hop(name);
    }
}

impl<'a> Visitor<'a> for CompletionVisitor {
    type StopResult = Vec<Field>;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match visit {
            Visit::Type(_, location) if !location.contains(self.pos) => Skip,
            Visit::Type(name, _) => {
                self.enter_constructor(name);
                Continue
            }
            Visit::Dependency(_, location) if !location.contains(self.pos) => Skip,
            Visit::PatternCall(_, location) if !location.contains(self.pos) => Skip,
            Visit::Constructor(constructor) if !constructor.loc.contains(self.pos) => Skip,
            Visit::Constructor(constructor) => {
                self.enter_constructor(constructor.name);
                Continue
            }
            Visit::Filed(_, location) if !location.contains(self.pos) => Skip,
            Visit::TypeExpression(_, location) if !location.contains(self.pos) => Skip,
            Visit::Expression(location) if !location.contains(self.pos) => Skip,
            Visit::AccessChain(name) if !name.contains(self.pos) => {
                self.part_of_chain(name);
                Continue
            }
            Visit::AccessChain(name) => Stop(self.stopped_at(name)),
            Visit::AccessChainLast(name) => Stop(self.stopped_at(name)),
            Visit::VarAccess(name) => Stop(self.stopped_at(name)),
            _ => Continue,
        }
    }
}
