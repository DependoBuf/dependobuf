//! Module exports `FileState` struct representing a dbuf file state. Contains methods
//! to read file and build asts.
use dbuf_core::arena::InternedString;
use dbuf_core::location::LocatedName;
use dbuf_core::location::Location;
use dbuf_core::location::Offset;

use dbuf_core::ast::elaborated as e;
use dbuf_core::ast::parsed as p;
use dbuf_core::cst as c;

use crate::file_content::FileContent;
use crate::reporter::Reporter;

type Cst = c::Tree;
type Ast = p::Module<Location<Offset>, LocatedName<InternedString, Offset>>;
type East = e::Module<String>;

/// Structure representing one file.
pub struct File<'a> {
    /// Content of file
    content: &'a FileContent,
    /// CST representation of file.
    cst: Option<Cst>,
    /// AST representation of file.
    ast: Option<Ast>,
    /// EAST representation of file.
    east: Option<East>,
}

impl<'a> File<'a> {
    /// Read file and create File struct.
    pub fn new(content: &'a FileContent) -> File<'a> {
        File {
            content,
            cst: None,
            ast: None,
            east: None,
        }
    }

    pub fn get_name(&self) -> &str {
        self.content.get_name()
    }

    pub fn get_cst(&self) -> Option<&Cst> {
        self.cst.as_ref()
    }

    pub fn get_ast(&self) -> Option<&Ast> {
        self.ast.as_ref()
    }

    pub fn get_east(&self) -> Option<&East> {
        self.east.as_ref()
    }

    /// Builds cst on file.
    pub fn process_cst(&mut self, reporter: &mut Reporter) {
        let (tree, errors) = c::parse_to_cst(self.content.get_content());

        for e in errors {
            reporter.report(&e.into());
        }

        if let Some(tree) = tree {
            self.cst = tree.into();
        }
    }

    /// Builds ast on file.
    pub fn process_ast(&mut self, _reporter: &mut Reporter) {
        if let Some(cst) = self.get_cst() {
            let ast = c::convert_to_ast(cst);
            self.ast = ast.into();
        }
    }

    /// Builds east on file.
    pub fn process_east(&mut self, _reporter: &mut Reporter) {
        if let Some(_ast) = self.get_ast() {
            eprintln!("UNIMPLEMENTED: convertation from parsed module to elaborated");
            // self.east = (...).into();
        }
    }
}
