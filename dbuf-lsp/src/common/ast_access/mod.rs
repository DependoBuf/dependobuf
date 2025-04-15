mod elaborated_ast;
mod file;
mod location;
mod parsers;
mod string;

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;

use dbuf_core::ast::parsed;

use parsers::*;

pub use elaborated_ast::ElaboratedHelper;
pub use file::*;
pub use location::*;
pub use string::*;

pub type Str = LocString;
pub type Loc = Location;
pub type ParsedAst = parsed::Module<Loc, Str>;
pub use elaborated_ast::ElaboratedAst;

#[derive(Debug)]
pub struct WorkspaceAccess {
    files: DashMap<Url, File>,
}

impl WorkspaceAccess {
    pub fn new() -> WorkspaceAccess {
        WorkspaceAccess {
            files: DashMap::new(),
        }
    }

    pub fn open(&self, url: Url, version: i32, text: &String) {
        let parsed: ParsedAst = get_parsed(text);
        let elaborated: ElaboratedAst = get_elaborated(text);

        let mut file = File::new();
        file.set_ast(version, parsed, elaborated);

        self.files.insert(url, file);
    }

    pub fn change(&self, url: &Url, version: i32, text: &String) {
        let parsed: ParsedAst = get_parsed(text);
        let elaborated: ElaboratedAst = get_elaborated(text);

        let mut file = self.files.get_mut(&url).expect("file should be opened");

        file.set_ast(version, parsed, elaborated);
    }

    pub fn read(&self, url: &Url) -> dashmap::mapref::one::Ref<'_, Url, file::File> {
        self.files.get(url).expect("file should be opened")
    }

    pub fn close(&self, url: &Url) {
        self.files.remove(url);
    }
}

unsafe impl Send for WorkspaceAccess {}
unsafe impl Sync for WorkspaceAccess {}
