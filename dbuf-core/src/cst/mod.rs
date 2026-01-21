use std::fs;

use logos::Logos;

mod token;

pub enum TreeKind {
    ErrorTree,
    File,
    Message,
    DependencyList,
    Dependency,
    Body,
    Field,
}

#[allow(clippy::missing_panics_doc)]
pub fn cst_main() {
    let str = fs::read_to_string("dbuf_file.dbuf").unwrap();

    let lexer = token::Token::lexer(&str);
    for x in lexer {
        match x {
            Ok(x) => println!("{x:#?}"),
            Err(e) => println!("!! {e}"),
        }
    }
}
