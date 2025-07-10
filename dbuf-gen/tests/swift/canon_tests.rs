use dbuf_gen::swift_gen;
use pretty_assertions::assert_eq;

use crate::common;

#[test]
fn basic() {
    let module = common::get_basic_module();

    let code = swift_gen::generate_module(module);
    let expected = include_str!("./canon/basic.swift");

    assert_eq!(code, expected);
}

#[test]
fn nat_vec() {
    let module = common::get_nat_vec_module();

    let code = swift_gen::generate_module(module);
    let expected = include_str!("./canon/nat_vec.swift");

    assert_eq!(code, expected);
}
