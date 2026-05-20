use super::parse_file;
use dbuf_core::elaboration::elaborate;

#[test]
fn test_incorrect_elaboration() {
    insta::glob!("incorrect_dbufs/*.dbuf", |path| {
        let ast = parse_file(path);
        let result = elaborate(&ast);
        assert!(
            result.is_err(),
            "Expected elaboration to fail for '{}'",
            path.display()
        );
    });
}
