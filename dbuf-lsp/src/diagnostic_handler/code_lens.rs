use tower_lsp::lsp_types::*;

use crate::core::ast_access::{File, Loc, LocStringHelper, LocationHelpers, Str};
use crate::core::navigator::Navigator;
use crate::core::{ast_visitor::*, navigator};

/// Returns all code lens of file.
pub fn provide_code_lens(file: &File) -> Vec<CodeLens> {
    let mut visitor = CodeLensVisitor::new(file);
    visit_ast(file.get_parsed(), &mut visitor, file.get_elaborated());
    visitor.result
}

struct CodeLensVisitor<'a> {
    file: &'a File,
    result: Vec<CodeLens>,
}

impl CodeLensVisitor<'_> {
    fn new(file: &File) -> CodeLensVisitor {
        CodeLensVisitor {
            file,
            result: Vec::new(),
        }
    }

    fn calc_reference_count(&self, type_name: &Str) -> u32 {
        let navigator = Navigator::new(self.file);

        let symbol = navigator::Symbol::Type(type_name.to_string());
        (navigator.find_symbols(&symbol).len() - 1) as u32
    }

    fn push_type(&mut self, type_name: &Str, _: &Loc) {
        let ref_count = self.calc_reference_count(type_name);
        let title = format!("{} references", ref_count);
        let command = Command {
            title,
            command: "".to_owned(),
            arguments: None,
        };

        let lens = CodeLens {
            range: type_name.get_location().to_lsp(),
            command: Some(command),
            data: None,
        };

        self.result.push(lens);
    }
}

impl<'a> Visitor<'a> for CodeLensVisitor<'a> {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Type(type_name, loc) => {
                self.push_type(type_name, loc);
                VisitResult::Skip
            }
            _ => VisitResult::Continue,
        }
    }
}
