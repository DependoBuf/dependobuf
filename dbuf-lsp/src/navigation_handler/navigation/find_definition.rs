//! Find definition of symbol
//!

use tower_lsp::lsp_types::Range;

use crate::core::ast_access::LocStringHelper;
use crate::core::ast_access::LocationHelpers;
use crate::core::ast_access::Str;
use crate::core::ast_visitor::VisitResult::*;
use crate::core::ast_visitor::safe_skip::safe_skip;
use crate::core::ast_visitor::*;
use crate::core::dbuf_language::get_builtin_types;

use crate::core::navigator::Navigator;
use crate::core::navigator::Symbol;

pub fn find_definition_impl(navigator: &Navigator, symbol: &Symbol) -> Option<Range> {
    match symbol {
        Symbol::Type(t) => {
            if get_builtin_types().contains(t) {
                return None;
            }
            let mut visitor = FindTypeVisitor { t };
            visit_ast(navigator.parsed, &mut visitor, navigator.elaborated)
        }
        Symbol::Dependency { t, dependency } => {
            let mut visitor = FindDependencyVisitor { t, dependency };
            visit_ast(navigator.parsed, &mut visitor, navigator.elaborated)
        }
        Symbol::Field {
            t,
            constructor,
            field,
        } => {
            let mut visitor = FindFieldVisitor {
                t,
                constructor,
                field,
            };
            visit_ast(navigator.parsed, &mut visitor, navigator.elaborated)
        }
        Symbol::Alias {
            t,
            branch_id,
            alias,
        } => {
            let mut visitor = FindAliasVisitor {
                t,
                branch_id: *branch_id,
                alias,
            };
            visit_ast(navigator.parsed, &mut visitor, navigator.elaborated)
        }
        Symbol::Constructor { t, constructor } => {
            let mut visitor = FindConstructorVisitor { t, constructor };
            visit_ast(navigator.parsed, &mut visitor, navigator.elaborated)
        }
        Symbol::None => None,
    }
}
struct FindTypeVisitor<'a> {
    t: &'a String,
}

impl FindTypeVisitor<'_> {
    fn check_type(&self, t: &Str) -> VisitResult<Range> {
        if t.as_ref() == self.t {
            Stop(t.get_location().to_lsp())
        } else {
            Skip
        }
    }
}

impl<'a> Visitor<'a> for FindTypeVisitor<'a> {
    type StopResult = Range;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Keyword(_, _) => Continue,
            Visit::Type(t, _) => self.check_type(t),
            _ => safe_skip(&visit),
        }
    }
}

struct FindDependencyVisitor<'a> {
    t: &'a String,
    dependency: &'a String,
}

impl FindDependencyVisitor<'_> {
    fn check_type(&self, t: &Str) -> VisitResult<Range> {
        if t.as_ref() == self.t { Continue } else { Skip }
    }

    fn check_dependency(&self, d: &Str) -> VisitResult<Range> {
        if d.as_ref() == self.dependency {
            Stop(d.get_location().to_lsp())
        } else {
            Skip
        }
    }
}

impl<'a> Visitor<'a> for FindDependencyVisitor<'a> {
    type StopResult = Range;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Keyword(_, _) => Continue,
            Visit::Type(t, _) => self.check_type(t),
            Visit::Dependency(d, _) => self.check_dependency(d),
            _ => safe_skip(&visit),
        }
    }
}

struct FindFieldVisitor<'a> {
    t: &'a String,
    constructor: &'a String,
    field: &'a String,
}

impl FindFieldVisitor<'_> {
    fn check_type(&self, t: &Str) -> VisitResult<Range> {
        if t.as_ref() == self.t { Continue } else { Skip }
    }

    fn check_constructor(&self, c: &Str) -> VisitResult<Range> {
        if c.as_ref() == self.constructor {
            Continue
        } else {
            Skip
        }
    }

    fn check_field(&self, f: &Str) -> VisitResult<Range> {
        if f.as_ref() == self.field {
            Stop(f.get_location().to_lsp())
        } else {
            Skip
        }
    }
}

impl<'a> Visitor<'a> for FindFieldVisitor<'a> {
    type StopResult = Range;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Keyword(_, _) => Continue,
            Visit::Type(t, _) => self.check_type(t),
            Visit::Branch => Continue,
            Visit::Constructor(c) => self.check_constructor(c.name),
            Visit::Filed(f, _) => self.check_field(f),
            _ => safe_skip(&visit),
        }
    }
}

struct FindAliasVisitor<'a> {
    t: &'a String,
    branch_id: usize,
    alias: &'a String,
}

impl FindAliasVisitor<'_> {
    fn check_type(&self, t: &Str) -> VisitResult<Range> {
        if t.as_ref() == self.t { Continue } else { Skip }
    }

    fn check_branch(&mut self) -> VisitResult<Range> {
        if self.branch_id == 0 {
            Continue
        } else {
            self.branch_id -= 1;
            Skip
        }
    }

    fn check_alias(&self, a: &Str) -> VisitResult<Range> {
        if a.as_ref() == self.alias {
            Stop(a.get_location().to_lsp())
        } else {
            Skip
        }
    }
}

impl<'a> Visitor<'a> for FindAliasVisitor<'a> {
    type StopResult = Range;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Keyword(_, _) => Continue,
            Visit::Type(t, _) => self.check_type(t),
            Visit::Branch => self.check_branch(),
            Visit::PatternAlias(a) => self.check_alias(a),
            Visit::PatternCall(_, _) => Continue,
            Visit::PatternCallArgument(_) => Continue,
            _ => safe_skip(&visit),
        }
    }
}

struct FindConstructorVisitor<'a> {
    t: &'a String,
    constructor: &'a String,
}

impl FindConstructorVisitor<'_> {
    fn check_type(&self, t: &Str) -> VisitResult<Range> {
        if t.as_ref() == self.t { Continue } else { Skip }
    }

    fn check_constructor(&self, c: &Str) -> VisitResult<Range> {
        if c.as_ref() == self.constructor {
            Stop(c.get_location().to_lsp())
        } else {
            Skip
        }
    }
}

impl<'a> Visitor<'a> for FindConstructorVisitor<'a> {
    type StopResult = Range;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Keyword(_, _) => Continue,
            Visit::Type(t, _) => self.check_type(t),
            Visit::Branch => Continue,
            Visit::Constructor(c) => self.check_constructor(c.name),
            _ => safe_skip(&visit),
        }
    }
}
