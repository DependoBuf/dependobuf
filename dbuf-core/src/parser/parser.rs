use super::lexer::*;
use crate::ast::operators::*;
use crate::ast::parsed::definition::*;
use crate::ast::parsed::*;
use chumsky::{pratt, prelude::*, span};

pub fn create_parser<'src>(
) -> impl Parser<'src, &'src [Token], Module<Span, String>, extra::Err<Rich<'src, Token>>> {
    parser_type_declaration()
        .repeated()
        .collect()
        .then_ignore(end())
}

fn parser_type_declaration<'src>() -> impl Parser<
    'src,
    &'src [Token],
    Definition<Span, String, TypeDeclaration<Span, String>>,
    extra::Err<Rich<'src, Token>>,
> {
    end().map(|_| {
        (Definition {
            loc: todo!(),
            name: todo!(),
            data: todo!(),
        })
    })
}

// Module<Loc, Str> = Definitions<Span, String, TypeDeclaration<Span, String>>;
// Definitions<Loc, Name, Data> = Vec<Definition<Span, String, TypeDeclaration<Span, String>>>;
