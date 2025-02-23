mod ast;
mod codegen;
mod scope;

mod sample;

use scope::{Scope, TaggerScope};
use pretty::{BoxDoc, BoxAllocator, DocAllocator};

const NEST_UNIT: isize = 4;
