use indexmap::IndexMap;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionItemLabelDetails,
    CompletionResponse, CompletionTriggerKind, Position,
};
use tracing::span::EnteredSpan;
use tracing::{Level, info, span, trace};

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

    let mut alias_visitor = AliasCollectorVisitor::new(data);
    visit_ast(ast, &mut alias_visitor);
    let data = alias_visitor.take();

    let activation_kind = match ctx {
        Some(x) if x.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER => {
            ActivationKind::Dot
        }
        Some(_) => ActivationKind::Basic,
        None => ActivationKind::Basic,
    };

    let mut completion_visitor = CompletionVisitor::new(pos, data, activation_kind);
    let Some(ans_fields) = visit_ast(ast, &mut completion_visitor) else {
        return Some(CompletionResponse::Array(vec![]));
    };

    let ans = ans_fields.into_iter().map(|f| f.to_lsp()).collect();
    Some(CompletionResponse::Array(ans))
}

/// Definition represetation.
#[derive(Clone)]
struct Definition {
    /// Name of definition.
    name: Str,
    /// Type of definition without dependencies.
    ty: Str,
}

impl Definition {
    pub fn to_lsp(&self) -> CompletionItem {
        CompletionItem {
            label: self.name.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: Some(self.ty.to_string()),
            }),
            kind: Some(CompletionItemKind::FIELD),
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

/// Struct contains all accessible definitions (currently fields and dependencies)
/// for current constructor.
struct ConstructorAccessable {
    /// Fields of constructor.
    fields: Vec<Definition>,
    /// Dependencies of constructor.
    dependencies: Vec<Definition>,
    /// Aliases for constructor (enum constructors only).
    aliases: Vec<Definition>,
}

/// Struct contains all constructors and their fields.
struct ConstructorDefinitions {
    /// Map of constructor and its definitions.
    data: IndexMap<Str, ConstructorAccessable>,
}

impl ConstructorDefinitions {
    /// Return all fields of a constructor.
    pub fn fields(&self, ctr: &Str) -> Vec<Definition> {
        self.data
            .get(ctr)
            .map(|c| c.fields.clone())
            .unwrap_or_default()
    }

    /// Return all dependencies of a constructor.
    pub fn dependencies(&self, ctr: &Str) -> Vec<Definition> {
        self.data
            .get(ctr)
            .map(|c| c.dependencies.clone())
            .unwrap_or_default()
    }

    /// Return all aliases of a constructor.
    pub fn aliases(&self, ctr: &Str) -> Vec<Definition> {
        self.data
            .get(ctr)
            .map(|c| c.aliases.clone())
            .unwrap_or_default()
    }

    /// Return type of field of constructor if any.
    pub fn hop_field(&self, current_ctr: &Str, field: &Str) -> Option<Str> {
        self.data
            .get(current_ctr)?
            .fields
            .iter()
            .find(|f| f.name == *field)
            .map(|f| f.ty.clone())
    }

    /// Return type of dependency of constructor if any.
    pub fn hop_dependency(&self, current_ctr: &Str, field: &Str) -> Option<Str> {
        self.data
            .get(current_ctr)?
            .dependencies
            .iter()
            .find(|f| f.name == *field)
            .map(|f| f.ty.clone())
    }

    /// Return type of alias of constructor if any.
    pub fn hop_alias(&self, current_ctr: &Str, field: &Str) -> Option<Str> {
        self.data
            .get(current_ctr)?
            .aliases
            .iter()
            .find(|f| f.name == *field)
            .map(|f| f.ty.clone())
    }

    /// Return type of dependency or field of constructor
    pub fn hop(&self, current_ctr: &Str, field: &Str) -> Option<Str> {
        self.hop_field(current_ctr, field)
            .or_else(|| self.hop_alias(current_ctr, field))
            .or_else(|| self.hop_dependency(current_ctr, field))
    }
}

/// Collects information about definition.
struct DefinitionCollector {
    /// Current stored definition name.
    field_name: Option<Str>,
    /// Current stored definition type.
    field_type: Option<Str>,
}

impl DefinitionCollector {
    fn new() -> DefinitionCollector {
        DefinitionCollector {
            field_name: None,
            field_type: None,
        }
    }

    fn set_name(&mut self, name: Str) {
        assert!(self.field_name.is_none());
        self.field_name = name.into();
    }

    fn set_type(&mut self, ty: Str) {
        assert!(self.field_type.is_none());
        self.field_type = ty.into();
    }

    fn take(&mut self) -> Definition {
        assert!(self.field_name.is_some());
        assert!(self.field_type.is_some());

        Definition {
            name: self.field_name.take().unwrap(),
            ty: self.field_type.take().unwrap(),
        }
    }
}

/// Visitor that collects all constructors and their definitions.
struct DataCollectorVisitor {
    /// Collected constructors with fields.
    data: ConstructorDefinitions,
    /// Name of current constructor.
    constructor_name: Option<Str>,
    /// Current collector of field.
    collector: DefinitionCollector,
    /// Indicator whether collecting dependency or field.
    setting_dependency: Option<bool>,
    /// Dependencies of current type.
    dependencies: Vec<Definition>,
}

impl DataCollectorVisitor {
    pub fn new() -> DataCollectorVisitor {
        DataCollectorVisitor {
            data: ConstructorDefinitions {
                data: IndexMap::new(),
            },
            constructor_name: None,
            collector: DefinitionCollector::new(),
            setting_dependency: None,
            dependencies: vec![],
        }
    }

    fn on_constructor(&mut self, name: &Name) {
        self.constructor_name = name.content.clone().into();
    }

    fn on_type(&mut self) {
        self.dependencies = vec![];
    }

    fn on_dependency_name(&mut self, name: &Name) {
        self.collector.set_name(name.content.clone());
        self.setting_dependency = Some(true);
    }

    fn on_dependency_type(&mut self, ty: &Name) {
        self.collector.set_type(ty.content.clone());
        let dependency = self.collector.take();
        self.dependencies.push(dependency);
    }

    fn on_field_name(&mut self, name: &Name) {
        self.collector.set_name(name.content.clone());
        self.setting_dependency = Some(false);
    }

    fn on_field_type(&mut self, ty: &Name) {
        self.collector.set_type(ty.content.clone());
        let field = self.collector.take();

        if let Some(old) = self
            .data
            .data
            .get_mut(self.constructor_name.as_ref().unwrap())
        {
            old.fields.push(field);
        } else {
            let current = ConstructorAccessable {
                fields: vec![field],
                dependencies: self.dependencies.clone(),
                aliases: vec![],
            };
            let old = self
                .data
                .data
                .insert(self.constructor_name.as_ref().unwrap().clone(), current);
            assert!(old.is_none());
        }
    }

    fn on_type_expr(&mut self, ty: &Name) {
        if self
            .setting_dependency
            .expect("setted at on_dependency/on_field")
        {
            self.setting_dependency = None;
            self.on_dependency_type(ty);
        } else {
            self.setting_dependency = None;
            self.on_field_type(ty);
        }
    }

    /// Return collected definitions.
    pub fn take(self) -> ConstructorDefinitions {
        self.data
    }
}

impl<'a> Visitor<'a> for DataCollectorVisitor {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(_, _) => self.on_type(),
            Visit::Dependency(name, _) => self.on_dependency_name(name),
            Visit::Branch => {}
            Visit::PatternAlias(_) => {}
            Visit::PatternCall(_, _) => return Skip,
            Visit::PatternCallArgument(_) => return Skip,
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(constructor) => self.on_constructor(constructor.name),
            Visit::Field(name, _) => self.on_field_name(name),
            Visit::TypeExpression(ty, _) => {
                self.on_type_expr(ty);
                return Skip;
            }
            Visit::Expression(_) => return Skip,
            Visit::AccessChainStart => return Skip,
            Visit::AccessChain(_) => {}
            Visit::AccessDot(_) => {}
            Visit::AccessChainLast(_) => {}
            Visit::ConstructorExpr(_) => return Skip,
            Visit::ConstructorExprArgument(_) => return Skip,
            Visit::ConstructorExprStop => {}
            Visit::VarAccess(_) => {}
            Visit::Operator(_, _) => {}
            Visit::Literal(_, _) => {}
        }

        Continue
    }
}

/// Visitor that collects alias information for every enum constructor.
struct AliasCollectorVisitor {
    /// Collected constructors with fields to deduce alias type.
    data: ConstructorDefinitions,
    /// Stack of constructor calls in pattern.
    constructor_stack: Vec<Str>,
    /// Current pattern dependency id.
    param_id: i32,
    /// Current field name of pattern constructor.
    current_field: Option<Str>,
    /// aliases of current branch.
    aliases: Vec<Definition>,
    /// Collector of dependencies.
    collector: DefinitionCollector,
    /// Dependencies of current type.
    dependencies: Vec<Definition>,
}

impl AliasCollectorVisitor {
    pub fn new(data: ConstructorDefinitions) -> AliasCollectorVisitor {
        AliasCollectorVisitor {
            data,
            constructor_stack: vec![],
            param_id: -1,
            current_field: None,
            aliases: vec![],
            collector: DefinitionCollector::new(),
            dependencies: vec![],
        }
    }

    fn on_type(&mut self) {
        self.dependencies = vec![];
        self.param_id = -1;
    }

    fn on_branch(&mut self) {
        self.param_id = 0;
        self.constructor_stack = vec![];
        self.current_field = None;
        self.aliases = vec![];
    }

    fn on_next_pattern(&mut self) {
        self.current_field = None;
        if self.constructor_stack.is_empty() {
            self.param_id += 1;
        }
    }

    fn on_pattern_call(&mut self, ctr: &Name) {
        self.current_field = None;
        self.constructor_stack.push(ctr.content.clone());
    }

    fn on_pattern_call_arg(&mut self, name: &Name) {
        self.current_field = name.content.clone().into();
    }

    fn on_pattern_call_stop(&mut self) {
        self.constructor_stack
            .pop()
            .expect("Since call stop count matches call count");
        if self.constructor_stack.is_empty() {
            self.param_id += 1;
        }
    }

    fn on_alias(&mut self, name: &Name) {
        if let Some(ctr) = self.constructor_stack.last() {
            let f_name = self
                .current_field
                .take()
                .expect("Sice should be set on pattern call argument");
            let ty = self.data.hop_field(ctr, &f_name);

            let Some(ty) = ty else {
                info!("Unknown field of constuctor {ctr}: {f_name}");
                return;
            };

            self.aliases.push(Definition {
                name: name.content.clone(),
                ty,
            });

            return;
        }

        assert!(
            self.param_id >= 0,
            "Should be set to non negative on new branch"
        );

        #[allow(
            clippy::cast_sign_loss,
            reason = "checked in assert that is not negative"
        )]
        let i = self.param_id as usize;

        if !(0..self.dependencies.len()).contains(&i) {
            info!(
                "Too many patterns for a type. Dependencies: {}, Current param index: {}",
                self.dependencies.len(),
                i
            );
            return;
        }

        let d = self.dependencies.get(i).expect("index in range");

        self.aliases.push(Definition {
            name: name.content.clone(),
            ty: d.ty.clone(),
        });
    }

    fn on_dependency_name(&mut self, name: &Name) {
        self.collector.set_name(name.content.clone());
    }

    fn on_dependency_type(&mut self, ty: &Name) {
        self.collector.set_type(ty.content.clone());
        let dependency = self.collector.take();
        self.dependencies.push(dependency);
    }

    fn on_enum_constructor(&mut self, ctr: &Name) {
        if let Some(ctr) = self.data.data.get_mut(&ctr.content) {
            assert!(
                ctr.aliases.is_empty(),
                "Aliases sets not only once per constructor"
            );
            ctr.aliases.clone_from(&self.aliases);
        }
    }

    /// Return definitions with alias definitions.
    pub fn take(self) -> ConstructorDefinitions {
        self.data
    }
}

impl<'a> Visitor<'a> for AliasCollectorVisitor {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match visit {
            Visit::Type(_, _) => {
                self.on_type();
                Continue
            }
            Visit::Dependency(name, _) => {
                self.on_dependency_name(name);
                Continue
            }
            Visit::Branch => {
                self.on_branch();
                Continue
            }
            Visit::PatternAlias(name) => {
                self.on_alias(name);
                Continue
            }
            Visit::PatternCall(name, _) => {
                self.on_pattern_call(name);
                Continue
            }
            Visit::PatternCallArgument(name) => {
                self.on_pattern_call_arg(name);
                Continue
            }
            Visit::PatternCallStop => {
                self.on_pattern_call_stop();
                Continue
            }
            Visit::PatternLiteral(_, _) => {
                self.on_next_pattern();
                Continue
            }
            Visit::PatternUnderscore(_) => {
                self.on_next_pattern();
                Continue
            }
            Visit::Constructor(constructor) => {
                if !constructor.of_message {
                    self.on_enum_constructor(constructor.name);
                }
                Skip
            }
            Visit::TypeExpression(name, _) => {
                self.on_dependency_type(name);
                Skip
            }
            _ => Continue,
        }
    }
}

/// Kind of completion activation.
#[derive(Debug)]
enum ActivationKind {
    /// Simple writing - returning current struct data.
    Basic,
    /// On dot - returning next struct data.
    Dot,
}

/// Visitor that defines correct completion at location.
struct CompletionVisitor {
    /// Cursor position.
    pos: Position,
    /// Accessible definitions of constructors.
    constructor_fields: ConstructorDefinitions,
    /// Current constructor which fields should be an answer.
    current_constructor: Option<Str>,
    /// Indicates whether constructor is first (without hops), so
    /// dependencies are correct auto-fill.
    first_constructor: bool,
    /// Activation kind of completion action.
    activation_kind: ActivationKind,
    /// current object tracing span.
    #[allow(
        dead_code,
        reason = "Used for RAII span, that will be exited when completion visitor drops"
    )]
    span: EnteredSpan,
}

impl CompletionVisitor {
    pub fn new(
        mut pos: Position,
        fields: ConstructorDefinitions,
        activation_kind: ActivationKind,
    ) -> CompletionVisitor {
        let span = span!(Level::TRACE, "completion_visitor").entered();
        trace!("Create CompletionVisitor with activation_kind: {activation_kind:?}");

        // FIXME: currently looks strage, but that is
        // how hangind dot access is handled in current AST.
        if matches!(activation_kind, ActivationKind::Dot) {
            pos.character -= 1;
        }

        CompletionVisitor {
            pos,
            constructor_fields: fields,
            current_constructor: None,
            first_constructor: true,
            activation_kind,
            span,
        }
    }

    fn enter_constructor(&mut self, ctr: &Name) {
        trace!("Enter constructor: {ctr}");

        self.current_constructor = ctr.content.clone().into();
        self.first_constructor = true;
    }

    fn hop(&mut self, name: &Name) {
        trace!("Hop with name: {name} which starts at: {:?}", name.start);

        if self.first_constructor {
            self.current_constructor = self
                .current_constructor
                .as_ref()
                .and_then(|old| self.constructor_fields.hop(old, &name.content));
            self.first_constructor = false;
        } else {
            self.current_constructor = self
                .current_constructor
                .as_ref()
                .and_then(|old| self.constructor_fields.hop_field(old, &name.content));
        }
    }

    fn stopped_at(&mut self, name: &Name) -> Vec<Definition> {
        assert!(name.contains(self.pos));
        trace!("Stopping at name: {name} which starts at: {:?}", name.start);

        if matches!(self.activation_kind, ActivationKind::Dot) {
            self.hop(name);
        }

        let Some(ctr) = &self.current_constructor else {
            return vec![];
        };

        let mut ans = self.constructor_fields.fields(ctr);
        if self.first_constructor {
            ans.extend(self.constructor_fields.dependencies(ctr));
            ans.extend(self.constructor_fields.aliases(ctr));
        }
        ans
    }

    fn part_of_chain(&mut self, name: &Name) {
        assert!(!name.contains(self.pos));
        trace!("Part of chain: {name} which starts at: {:?}", name.start);

        self.hop(name);
    }
}

impl<'a> Visitor<'a> for CompletionVisitor {
    type StopResult = Vec<Definition>;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match visit {
            Visit::Type(_, location) if !location.contains(self.pos) => Skip,
            Visit::Dependency(_, location) if !location.contains(self.pos) => Skip,
            Visit::PatternCall(_, location) if !location.contains(self.pos) => Skip,
            Visit::Constructor(constructor) if !constructor.loc.contains(self.pos) => Skip,
            Visit::Constructor(constructor) => {
                self.enter_constructor(constructor.name);
                Continue
            }
            Visit::Field(_, location) if !location.contains(self.pos) => Skip,
            Visit::TypeExpression(_, location) if !location.contains(self.pos) => Skip,
            Visit::Expression(location) if !location.contains(self.pos) => Skip,
            Visit::AccessChain(name) if !name.contains(self.pos) => {
                self.part_of_chain(name);
                Continue
            }
            Visit::AccessChain(name) if name.contains(self.pos) => Stop(self.stopped_at(name)),
            Visit::AccessChainLast(name) if name.contains(self.pos) => Stop(self.stopped_at(name)),
            Visit::VarAccess(name) if name.contains(self.pos) => Stop(self.stopped_at(name)),
            _ => Continue,
        }
    }
}
