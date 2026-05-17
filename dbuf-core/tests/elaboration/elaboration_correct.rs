use super::parse_file;
use dbuf_core::elaboration::typecheck::elaborate;
use insta::Settings;
use std::path::Path;

fn get_setting(path: &Path) -> Settings {
    let mut settings = Settings::new();
    settings.set_snapshot_path(format!(
        "snapshots/correct/{}",
        path.file_stem().unwrap().display()
    ));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_suffix("");
    settings
}

fn test_file(path: &Path) {
    get_setting(path).bind(|| {
        let ast = parse_file(path);
        let elaborated = elaborate(&ast);
        insta::assert_debug_snapshot!("elaborated", elaborated);
    });
}

#[test]
fn test_builtin_types() {
    insta::glob!("correct_dbufs/simple_messages.dbuf", test_file);
    insta::glob!("correct_dbufs/builtin_deps.dbuf", test_file);
}

#[test]
fn test_literals() {
    insta::glob!("correct_dbufs/literals.dbuf", test_file);
}

#[test]
fn test_unary_ops() {
    insta::glob!("correct_dbufs/unary_ops.dbuf", test_file);
}

#[test]
fn test_binary_ops() {
    insta::glob!("correct_dbufs/binary_ops.dbuf", test_file);
}

#[test]
fn test_dot_access() {
    insta::glob!("correct_dbufs/dot_access.dbuf", test_file);
}

#[test]
fn test_dep_on_dep() {
    insta::glob!("correct_dbufs/dep_on_dep.dbuf", test_file);
}

#[test]
fn test_field_on_field() {
    insta::glob!("correct_dbufs/field_on_field.dbuf", test_file);
}

#[test]
fn test_dependent_messages() {
    insta::glob!("correct_dbufs/dependent_messages.dbuf", test_file);
    insta::glob!("correct_dbufs/expressions.dbuf", test_file);
}

#[test]
fn test_correct_elaboration() {
    insta::glob!("correct_dbufs/*.dbuf", test_file);
}
