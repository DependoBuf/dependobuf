//! Module provides SemanticTokenProvider
//!

mod modifier;
mod token;

use dbuf_core::ast::operators::Literal;
use modifier::Modifier;
use token::Token;

use tower_lsp::lsp_types::SemanticToken;
use tower_lsp::lsp_types::SemanticTokenModifier;
use tower_lsp::lsp_types::SemanticTokenType;
use tower_lsp::lsp_types::SemanticTokens;

use crate::common::ast_access::{
    ElaboratedAst, ElaboratedHelper, File, Loc, LocStringHelper, ParsedAst, Str,
};

use crate::common::ast_visitor::scope_visitor::ScopeVisitor;
use crate::common::ast_visitor::*;

struct TokenBuilder {
    location: Loc,
    token: Token,
    modifiers: Vec<Modifier>,
}

impl TokenBuilder {
    fn new(token: Token) -> TokenBuilder {
        TokenBuilder {
            location: Loc::default(),
            token,
            modifiers: vec![],
        }
    }
    fn at(mut self, location: Loc) -> TokenBuilder {
        self.location = location;
        self
    }
    fn with_modifier(mut self, modifier: Modifier) -> TokenBuilder {
        self.modifiers.push(modifier);
        self
    }
    fn collect(self) -> SemanticToken {
        assert!(self.location != Loc::default());
        assert!(self.location.start.line == self.location.end.line);

        let len = self.location.end.character - self.location.start.character;
        let mut modifier = 0;

        for modify in self.modifiers.iter() {
            modifier |= 1 << modify.to_index();
        }

        SemanticToken {
            delta_line: self.location.start.line,
            delta_start: self.location.start.character,
            length: len,
            token_type: self.token.to_index(),
            token_modifiers_bitset: modifier,
        }
    }
}

/// Semantic token provider.
pub struct SemanticTokenProvider<'a> {
    parsed: &'a ParsedAst,
    elaborated: &'a ElaboratedAst,

    last_line: u32,
    last_char: u32,
    result: Vec<SemanticToken>,

    scope: ScopeVisitor<'a>,
}

impl<'a> SemanticTokenProvider<'a> {
    pub fn new(file: &File) -> SemanticTokenProvider {
        SemanticTokenProvider {
            parsed: file.get_parsed(),
            elaborated: file.get_elaborated(),
            last_line: 0,
            last_char: 0,
            result: vec![],
            scope: ScopeVisitor::new(file.get_elaborated()),
        }
    }
    /// Returns all semantic tokens types in correct order.
    pub fn get_token_types() -> Vec<SemanticTokenType> {
        token::get_all_tokens()
    }
    /// Returns all semantic token modifiers in correct order.
    pub fn get_token_modifiers() -> Vec<SemanticTokenModifier> {
        modifier::get_all_modifiers()
    }
    /// Returns all semantic tokens of file
    pub fn provide(&mut self) -> SemanticTokens {
        self.last_line = 0;
        self.last_char = 0;
        self.result = vec![];

        visit_ast(self.parsed, self, self.elaborated);

        SemanticTokens {
            result_id: None,
            data: std::mem::take(&mut self.result),
        }
    }

    fn push_string(&mut self, str: &Str, token: Token, is_declaration: bool) {
        let mut builder = TokenBuilder::new(token).at(str.get_location());
        if is_declaration {
            builder = builder.with_modifier(Modifier::Declaration);
        }
        let mut token = builder.collect();

        if self.last_line != token.delta_line {
            assert!(token.delta_line > self.last_line || self.result.is_empty());
            token.delta_line -= self.last_line;

            self.last_line += token.delta_line;
            self.last_char = token.delta_start;
        } else {
            assert!(token.delta_start > self.last_char || self.result.is_empty());
            token.delta_line = 0;
            token.delta_start -= self.last_char;

            self.last_char += token.delta_start;
        }

        self.result.push(token);
    }

    fn push_str(&mut self, text: &str, location: Loc, token: Token) {
        let mut str = Str::new(text);
        str.set_location(location);

        self.push_string(&str, token, false);
    }

    fn push_keyword(&mut self, text: &str, location: &Loc) {
        self.push_str(text, *location, Token::Keyword);
    }
    fn push_operator(&mut self, text: &str, location: &Loc) {
        self.push_str(text, *location, Token::Operator);
    }
    fn push_literal(&mut self, literal: &Literal, location: &Loc) {
        match literal {
            Literal::Bool(b) => {
                let str = if *b { "true" } else { "false" };
                self.push_str(str, *location, Token::Keyword);
            }
            Literal::Double(d) => {
                let str = d.to_string();
                self.push_str(&str, *location, Token::Number);
            }
            Literal::Int(i) => {
                let str = i.to_string();
                self.push_str(&str, *location, Token::Number);
            }
            Literal::UInt(ui) => {
                let str = ui.to_string() + "u";
                self.push_str(&str, *location, Token::Number);
            }
            Literal::Str(s) => {
                let str = "\"".to_owned() + s + "\"";
                self.push_str(&str, *location, Token::String);
            }
        }
    }

    fn get_type_token(&self, type_name: &Str) -> Token {
        if self.elaborated.is_buildin_type(type_name.as_ref()) {
            Token::Type
        } else if self.elaborated.is_message(type_name.as_ref()) {
            Token::Message
        } else {
            Token::Enum
        }
    }

    fn get_constructor_token(&self, constructor_name: &Str) -> Token {
        if self.elaborated.is_message(constructor_name.as_ref()) {
            Token::Message
        } else {
            Token::EnumConstructor
        }
    }

    fn get_access_token(&self, access_name: &Str) -> Token {
        let type_name = self.scope.get_type();
        if self
            .elaborated
            .is_type_dependency(type_name, access_name.as_ref())
        {
            return Token::Parameter;
        }
        let cons = self.scope.get_constructor();
        if self
            .elaborated
            .is_constructor_field(cons, access_name.as_ref())
        {
            return Token::Property;
        }

        // Assuming: constructor alias
        Token::Parameter
    }
}

impl<'a> Visitor<'a> for SemanticTokenProvider<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        match &visit {
            Visit::Keyword(keyword, location) => self.push_keyword(keyword, location),
            Visit::Type(type_name, _) => {
                let token = self.get_type_token(type_name);
                self.push_string(type_name, token, true);
            }
            Visit::Dependency(dep_name, _) => self.push_string(dep_name, Token::Parameter, true),
            Visit::Branch => {}
            Visit::PatternAlias(alias) => {
                self.push_string(alias, Token::Property, false);
            }
            Visit::PatternCall(call_name, _) => {
                let token = self.get_constructor_token(call_name);
                self.push_string(call_name, token, false);
            }
            Visit::PatternCallArgument(arg_name) => {
                assert!(
                    arg_name.is_none(),
                    "constructor call argument name is not implemented"
                );
            }
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(literal, location) => self.push_literal(literal, location),
            Visit::PatternUnderscore(location) => self.push_keyword("*", location),
            Visit::Constructor(ctr) => {
                if !ctr.of_message {
                    self.push_string(ctr.name, Token::EnumConstructor, true);
                }
            }
            Visit::Filed(field_name, _) => self.push_string(field_name, Token::Property, true),
            Visit::TypeExpression(type_name, _) => {
                let token = self.get_type_token(type_name);
                self.push_string(type_name, token, false);
            }
            Visit::Expression(_) => {}
            Visit::AccessChainStart => {}
            Visit::AccessChain(access) => {
                let token = self.get_access_token(access);
                self.push_string(access, token, false);
            }
            Visit::AccessDot(loc) => self.push_operator(".", loc),
            Visit::AccessChainLast(access) => {
                let token = self.get_access_token(access);
                self.push_string(access, token, false);
            }
            Visit::ConstructorExpr(cons_name) => {
                let token = self.get_constructor_token(cons_name);
                self.push_string(cons_name, token, false);
            }
            Visit::ConstructorExprArgument(arg_name) => {
                assert!(
                    arg_name.is_none(),
                    "constructor call argument name is not implemented"
                )
            }
            Visit::ConstructorExprStop => {}
            Visit::VarAccess(access) => {
                let token = self.get_access_token(access);
                self.push_string(access, token, false);
            }
            Visit::Operator(op, location) => self.push_operator(op, location),
            Visit::Literal(literal, location) => self.push_literal(literal, location),
        };

        assert!(matches!(self.scope.visit(visit), VisitResult::Continue));

        VisitResult::Continue
    }
}
