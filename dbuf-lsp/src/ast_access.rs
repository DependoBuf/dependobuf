use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use dbuf_core::ast::parsed::Module;

pub type Ast = Module<(), String>;

#[derive(Debug)]
pub struct AstAccess {
    ast: RwLock<Ast>,
}

impl AstAccess {
    pub fn new() -> AstAccess {
        AstAccess {
            ast: RwLock::new(vec![]),
        }
    }
    pub fn read(&self) -> RwLockReadGuard<'_, Ast> {
        self.ast.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<'_, Ast> {
        self.ast.write().unwrap()
    }
}

unsafe impl Send for AstAccess {}
unsafe impl Sync for AstAccess {}
