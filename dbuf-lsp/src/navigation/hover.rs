/// Hover provider.
///
use tower_lsp::lsp_types::{LanguageString, MarkedString};

use crate::core::ast_access::{ElaboratedHelper, File};
use crate::core::dbuf_language::get_builtin_types;
use crate::core::navigator::Symbol;
use crate::core::pretty_printer::PrettyPrinter;

pub fn get_hover(symbol: Symbol, file: &File) -> Vec<MarkedString> {
    let elaborated = file.get_elaborated();

    match symbol {
        Symbol::Type { type_name } => vec![MarkedString::LanguageString(get_explicit_type(
            type_name, file,
        ))],
        Symbol::Dependency {
            type_name,
            dependency,
        } => vec![
            MarkedString::LanguageString(get_type_header(&type_name, file)),
            MarkedString::LanguageString(get_dependency_declaration(&type_name, &dependency, file)),
            MarkedString::String(format!("dependency of {}", type_name)),
        ],
        Symbol::Field {
            type_name,
            constructor,
            field,
        } => {
            let mut strings = Vec::with_capacity(4);

            strings.push(MarkedString::LanguageString(get_type_header(
                &type_name, file,
            )));
            if !elaborated.is_message(&type_name) {
                strings.push(MarkedString::LanguageString(get_constructor_header(
                    &constructor,
                )));
            }
            strings.push(MarkedString::LanguageString(get_field_declaration(
                &type_name,
                &constructor,
                &field,
                file,
            )));
            strings.push(MarkedString::String(format!("field of {}", constructor)));
            strings
        }
        Symbol::Alias {
            type_name,
            branch_id,
            alias,
        } => vec![
            MarkedString::LanguageString(get_type_header(&type_name, file)),
            MarkedString::LanguageString(get_explicit_branch(&type_name, branch_id, file)),
            MarkedString::String(format!("alias {}", alias)),
        ],
        Symbol::Constructor {
            type_name,
            constructor,
        } => {
            vec![
                MarkedString::LanguageString(get_type_header(&type_name, file)),
                MarkedString::LanguageString(get_explicit_constructor(
                    &type_name,
                    &constructor,
                    file,
                )),
                MarkedString::String(format!("constructor of {}", type_name)),
            ]
        }
        Symbol::None => Vec::new(),
    }
}

fn get_explicit_type(type_name: String, file: &File) -> LanguageString {
    if get_builtin_types().contains(&type_name) {
        LanguageString {
            language: "dbuf".to_owned(),
            value: type_name,
        }
    } else {
        let mut code = String::new();
        let mut printer = PrettyPrinter::new(&mut code);
        printer.print_type(file.get_parsed(), type_name.as_ref());

        LanguageString {
            language: "dbuf".to_owned(),
            value: code,
        }
    }
}

fn get_explicit_constructor(type_name: &str, constructor: &str, file: &File) -> LanguageString {
    let mut code = String::new();

    let mut printer = PrettyPrinter::new(&mut code);
    printer.print_selected_constructor(file.get_parsed(), type_name, constructor);

    LanguageString {
        language: "dbuf".to_owned(),
        value: code,
    }
}

fn get_type_header(type_name: &str, file: &File) -> LanguageString {
    let elaborated = file.get_elaborated();

    let header = if elaborated.is_message(type_name) {
        "message ".to_owned() + type_name
    } else {
        "enum ".to_owned() + type_name
    };

    LanguageString {
        language: "dbuf".to_owned(),
        value: header,
    }
}

fn get_constructor_header(constructor: &str) -> LanguageString {
    let header = constructor.to_owned();

    LanguageString {
        language: "dbuf".to_owned(),
        value: header,
    }
}

fn get_explicit_branch(type_name: &str, branch_id: usize, file: &File) -> LanguageString {
    let mut branch = String::new();

    let mut printer = PrettyPrinter::new(&mut branch);
    printer.print_selected_branch(file.get_parsed(), type_name, branch_id);

    LanguageString {
        language: "dbuf".to_owned(),
        value: branch,
    }
}

fn get_dependency_declaration(type_name: &str, dependency: &str, file: &File) -> LanguageString {
    let mut dependency_declaration = String::new();

    let mut printer = PrettyPrinter::new(&mut dependency_declaration);
    printer.print_selected_dependency(file.get_parsed(), type_name, dependency);

    LanguageString {
        language: "dbuf".to_owned(),
        value: dependency_declaration,
    }
}

fn get_field_declaration(
    type_name: &str,
    constructor: &str,
    field: &str,
    file: &File,
) -> LanguageString {
    let mut field_declaration = String::new();

    let mut printer = PrettyPrinter::new(&mut field_declaration);
    printer.print_selected_field(file.get_parsed(), type_name, constructor, field);

    LanguageString {
        language: "dbuf".to_owned(),
        value: field_declaration,
    }
}
