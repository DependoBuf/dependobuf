mod common;

mod tests {
    use crate::common;
    use dbuf_core::ast::elaborated as e;
    use dbuf_gen::{codegen, kotlin_gen};
    use pretty_assertions::assert_eq;

    #[test]
    fn basic() {
        let module = common::get_basic_module();
        let code = kotlin_gen::generate_module(module);

        println!("{}", code);
        let expected = include_str!("./canon-kotlin/basic.kt");
        assert_eq!(code, expected);
    }

    #[test]
    fn nat_vec() {
        let module = common::get_nat_vec_module();
        let code = kotlin_gen::generate_module(module);

        println!("{}", code);
        let expected = include_str!("./canon-kotlin/nat_vec.kt");
        assert_eq!(code, expected);
    }

    #[test]
    fn dependent_messages() {
        // message Sum (a Int) {}

        // message Foo (a Int) (b Int) {
        //     sum Sum (-a + b);
        // }

        // message Bar (c Int) (s String) {
        //     e Int;
        //     d Int;
        //     f Foo e d;
        //     g Foo c (e + d);
        // }

        // message Kek (a Int) (b Int) (f Foo a b) {
        //     bar Bar a "qwe";
        // }

        // message Sum (a Int) {}
        // -- elaborated ast
        // Its Type
        // [
        //   Type {
        //       dependencies: vec![("a".to_owned(), Expression::Type {
        //           name: "Int".to_owned(),
        //           dependencies: Rec::new([]),
        //       })],
        //       constructor_names: ConstructorNames::OfMessage("Sum".to_owned()),
        //   },
        //   Type {
        //       dependencies: vec![("a".to_owned(), Expression::Type {
        //           name: "Int".to_owned(),
        //           dependencies: Rec::new([]),
        //       })],
        //       constructor_names: ConstructorNames::OfMessage("Sum".to_owned()),
        //   },
        // ]
        // Corresponding part of constructors
        // [(
        //     "Sum".to_owned(),
        //     Constructor {
        //         implicits: vec![(
        //             "a".to_owned(),
        //             Expression::Type {
        //                 name: "Int".to_owned(),
        //                 dependencies: Rec::new([]),
        //             },
        //         )],
        //         fields: Vec::new(),
        //         result_type: Expression::Type {
        //             name: "Sum".to_owned(),
        //             dependencies: Rec::new([Expression::Variable {
        //                 name: "a".to_owned(),
        //             }]),
        //         },
        //     },
        // ),
        // (
        //     "Foo".to_owned(),
        //     Constructor {
        //         implicits: vec![
        //             (
        //                 "a".to_owned(),
        //                 Expression::Type {
        //                     name: "Int".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //             ),
        //             (
        //                 "b".to_owned(),
        //                 Expression::Type {
        //                     name: "Int".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //             ),
        //         ],
        //         fields: vec![(
        //             "sum".to_owned(),
        //             Expression::Constructor {
        //                 name: "Sum".to_owned(),
        //                 implicits: Rec::new([Expression::OpCall(OpCall::Binary(
        //                     super::operators::BinaryOp::Plus,
        //                     Rec::new(Expression::OpCall(OpCall::Unary(
        //                         super::operators::UnaryOp::Minus,
        //                         Rec::new(Expression::Variable {
        //                             name: "a".to_owned(),
        //                         }),
        //                     ))),
        //                     Rec::new(Expression::Variable {
        //                         name: "b".to_owned(),
        //                     }),
        //                 ))]),
        //                 arguments: Rec::new([]),
        //             },
        //         )],
        //         result_type: Expression::Type {
        //             name: "Foo".to_owned(),
        //             dependencies: Rec::new([
        //                 Expression::Variable {
        //                     name: "a".to_owned(),
        //                 },
        //                 Expression::Variable {
        //                     name: "b".to_owned(),
        //                 },
        //             ]),
        //         },
        //     },
        // )]
    }

    #[test]
    fn complex_dependencies_substitution() {
        // TODO

        // message Checker (n Nat) (l List n) {}

        // message Kek (n Nat) (l List n) {
        //     c1 Checker n l
        //     c2 Checker (Suc n) (Cons "a" l)
        //     c3 Checker (Suc Suc n) (Cons "b" (Cons "a" l))
        // }

        // broad ast snippet
        // (
        //     "Checker".to_owned(),
        //     Constructor {
        //         implicits: vec![
        //             (
        //                 "n".to_owned(),
        //                 Expression::Type {
        //                     name: "Nat".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //             ),
        //             (
        //                 "l".to_owned(),
        //                 Expression::Type {
        //                     name: "List".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //             ),
        //         ],
        //         fields: Vec::new(),
        //         result_type: Expression::Type {
        //             name: "Checker".to_owned(),
        //             dependencies: Rec::new([
        //                 Expression::Type {
        //                     name: "Nat".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //                 Expression::Type {
        //                     name: "List".to_owned(),
        //                     dependencies: Rec::new([Expression::Variable {
        //                         name: "n".to_owned(),
        //                     }]),
        //                 },
        //             ]),
        //         },
        //     },
        // )

        // (
        //     "Kek".to_owned(),
        //     Constructor {
        //         implicits: vec![
        //             (
        //                 "n".to_owned(),
        //                 Expression::Type {
        //                     name: "Nat".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //             ),
        //             (
        //                 "l".to_owned(),
        //                 Expression::Type {
        //                     name: "List".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //             ),
        //         ],
        //         fields: vec![
        //             (
        //                 "c1".to_owned(),
        //                 Expression::Type {
        //                     name: "Checker".to_owned(),
        //                     dependencies: Rec::new([
        //                         Expression::Variable {
        //                             name: "n".to_owned(),
        //                         },
        //                         Expression::Variable {
        //                             name: "l".to_owned(),
        //                         },
        //                     ]),
        //                 },
        //             ),
        //             (
        //                 "c2".to_owned(),
        //                 Expression::Type {
        //                     name: "Checker".to_owned(),
        //                     dependencies: Rec::new([
        //                         Expression::Constructor {
        //                             name: "Suc".to_owned(),
        //                             implicits: Rec::new([]),
        //                             arguments: Rec::new([Expression::Variable {
        //                                 name: "n".to_owned(),
        //                             }]),
        //                         },
        //                         Expression::Constructor {
        //                             name: "Cons".to_owned(),
        //                             implicits: Rec::new([Expression::Variable {
        //                                 name: "n".to_owned(),
        //                             }]),
        //                             arguments: Rec::new([
        //                                 Expression::OpCall(OpCall::Literal(
        //                                     super::operators::Literal::Str("a".to_owned()),
        //                                 )),
        //                                 Expression::Variable {
        //                                     name: "l".to_owned(),
        //                                 },
        //                             ]),
        //                         },
        //                     ]),
        //                 },
        //             ),
        //             (
        //                 "c3".to_owned(),
        //                 Expression::Type {
        //                     name: "Checker".to_owned(),
        //                     dependencies: Rec::new([
        //                         Expression::Constructor {
        //                             name: "Suc".to_owned(),
        //                             implicits: Rec::new([]),
        //                             arguments: Rec::new([Expression::Constructor {
        //                                 name: "Suc".to_owned(),
        //                                 implicits: Rec::new([]),
        //                                 arguments: Rec::new([Expression::Variable {
        //                                     name: "n".to_owned(),
        //                                 }]),
        //                             }]),
        //                         },
        //                         Expression::Constructor {
        //                             name: "Cons".to_owned(),
        //                             implicits: Rec::new([Expression::Constructor {
        //                                 name: "Suc".to_owned(),
        //                                 implicits: Rec::new([]),
        //                                 arguments: Rec::new([Expression::Variable {
        //                                     name: "n".to_owned(),
        //                                 }]),
        //                             }]),
        //                             arguments: Rec::new([
        //                                 Expression::OpCall(OpCall::Literal(
        //                                     super::operators::Literal::Str("b".to_owned()),
        //                                 )),
        //                                 Expression::Constructor {
        //                                     name: "Cons".to_owned(),
        //                                     implicits: Rec::new([Expression::Variable {
        //                                         name: "n".to_owned(),
        //                                     }]),
        //                                     arguments: Rec::new([
        //                                         Expression::OpCall(OpCall::Literal(
        //                                             super::operators::Literal::Str("a".to_owned()),
        //                                         )),
        //                                         Expression::Variable {
        //                                             name: "l".to_owned(),
        //                                         },
        //                                     ]),
        //                                 },
        //                             ]),
        //                         },
        //                     ]),
        //                 },
        //             ),
        //         ],
        //         result_type: Expression::Type {
        //             name: "Kek".to_owned(),
        //             dependencies: Rec::new([
        //                 Expression::Type {
        //                     name: "Nat".to_owned(),
        //                     dependencies: Rec::new([]),
        //                 },
        //                 Expression::Type {
        //                     name: "List".to_owned(),
        //                     dependencies: Rec::new([Expression::Variable {
        //                         name: "n".to_owned(),
        //                     }]),
        //                 },
        //             ]),
        //         },
        //     },
        // )
    }
}
