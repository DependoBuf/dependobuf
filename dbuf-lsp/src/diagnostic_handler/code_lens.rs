use tower_lsp::lsp_types::*;

use crate::common::ast_access::{
    ElaboratedAst, File, Loc, LocStringHelper, LocationHelpers, ParsedAst, Str,
};
use crate::common::navigator::Navigator;
use crate::common::{ast_visitor::*, navigator};

pub struct CodeLensProvider<'a> {
    file: &'a File,
    parsed: &'a ParsedAst,
    elaborated: &'a ElaboratedAst,

    result: Vec<CodeLens>,
}

impl CodeLensProvider<'_> {
    pub fn new(file: &File) -> CodeLensProvider {
        CodeLensProvider {
            file,
            parsed: file.get_parsed(),
            elaborated: file.get_elaborated(),
            result: Vec::new(),
        }
    }

    pub fn provide(&mut self) -> Vec<CodeLens> {
        visit_ast(self.parsed, self, self.elaborated);

        std::mem::take(&mut self.result)
    }

    fn calc_reference_count(&self, type_name: &Str) -> u32 {
        let navigator = Navigator::new(self.file);

        let symbol = navigator::Symbol::Type(type_name.to_string());
        (navigator.find_symbols(&symbol).len() - 1) as u32
    }

    fn push_type(&mut self, type_name: &Str, _: &Loc) {
        let ref_count = self.calc_reference_count(type_name);
        let title = format!("{} references", ref_count).to_string();
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

impl<'a> Visitor<'a> for CodeLensProvider<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        match &visit {
            Visit::Type(type_name, loc) => {
                self.push_type(type_name, loc);
                VisitResult::Skip
            }
            _ => VisitResult::Continue,
        }
    }
}
