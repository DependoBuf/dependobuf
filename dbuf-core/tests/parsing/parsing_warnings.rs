use insta::Settings;

use std::fs;
use std::path::Path;

use dbuf_core::arena::InternedString;
use dbuf_core::ast::parsed::Module;
use dbuf_core::cst::Tree;
use dbuf_core::cst::convert_to_ast;
use dbuf_core::cst::parse_to_cst;
use dbuf_core::error::ParsingError;
use dbuf_core::location::LocatedName;
use dbuf_core::location::Location;
use dbuf_core::location::Offset;

type DataParts = (
    Tree,
    Vec<ParsingError>,
    Module<Location<Offset>, LocatedName<InternedString, Offset>>,
);

fn get(path: &Path) -> DataParts {
    let input = fs::read_to_string(path).unwrap();

    let (tree, errors) = parse_to_cst(&input);

    assert!(
        tree.is_some(),
        "Some errors while parsing file '{}'",
        path.display()
    );
    assert!(
        !errors.is_empty(),
        "Expected errors while parsing file '{}'",
        path.display()
    );

    let tree = tree.unwrap();
    let ast = convert_to_ast(&tree);

    (tree, errors, ast)
}

fn get_setting(path: &Path) -> Settings {
    let mut settings = Settings::new();
    settings.set_snapshot_path(format!(
        "snapshots/warning/{}",
        path.file_stem().unwrap().display()
    ));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_suffix("");
    settings
}

#[test]
fn test_warning_error() {
    insta::glob!("partially_correct_dbufs/*.dbuf", |path| {
        get_setting(path).bind(|| {
            let (_, errors, _) = get(path);
            insta::assert_debug_snapshot!("correct_errors", errors);
        });
    });
}

#[test]
fn test_warning_cst() {
    insta::glob!("partially_correct_dbufs/*.dbuf", |path| {
        get_setting(path).bind(|| {
            let (tree, _, _) = get(path);
            insta::assert_debug_snapshot!("correct_cst", tree);
        });
    });
}

#[test]
fn test_warning_ast() {
    insta::glob!("partially_correct_dbufs/*.dbuf", |path| {
        get_setting(path).bind(|| {
            let (_, _, ast) = get(path);
            insta::assert_debug_snapshot!("correct_ast", ast);
        });
    });
}
