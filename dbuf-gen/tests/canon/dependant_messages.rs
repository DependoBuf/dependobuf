// TODO
// outdated for now

pub mod foo {
    use deps::sum;

    mod deps {
        pub use super::super::{Box, ConstructorError, Message};
        pub use super::super::{Sum, sum};
        pub use crate::try_block;
    }

    #[derive(PartialEq, Eq)]
    pub struct Body {
        pub sum: deps::Sum,
    }

    #[derive(PartialEq, Eq)]
    pub struct Dependencies {
        a: i32,
        b: i32,
    }

    // alias for the generated type
    pub type Foo = deps::Message<Body, Dependencies>;

    // inherit implementation with all constructors
    impl Foo {
        pub fn new(dependencies: Dependencies) -> Result<Self, deps::ConstructorError> {
            let body = Body {
                sum: deps::Sum::new(sum::Dependencies {
                    a: -dependencies.a + dependencies.b,
                })
                .expect("..."),
            };
            Ok(deps::Message { body, dependencies })
        }
    }
}
