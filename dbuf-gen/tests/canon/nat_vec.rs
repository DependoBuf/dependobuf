use dbuf_rust_runtime::{Box, ConstructorError};

pub mod nat {
    mod deps {
        pub(super) use super::super::{Box, ConstructorError};
        // pub(super) use super::super::{};
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Body {
        Suc {
            pred: deps::Box<Nat>
        },
        Zero {
        
        }
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Dependencies {
    
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Nat {
        pub body: Body,
        pub dependencies: Dependencies
    }
    
    impl Nat {
        pub fn Suc(pred: deps::Box<Nat>) -> Result<Self, deps::ConstructorError> {
            let body = if (()) == (()) {
                Ok(Body::Suc {
                    pred: pred
                })
            } else {
                Err(deps::ConstructorError::MismatchedDependencies)
            }?;
            let dependencies = Dependencies {
            
            };
            Ok(Self { body: body, dependencies: dependencies })
        }
        pub fn Zero() -> Result<Self, deps::ConstructorError> {
            let body = if () == () {
                Ok(Body::Zero {
                
                })
            } else {
                Err(deps::ConstructorError::MismatchedDependencies)
            }?;
            let dependencies = Dependencies {
            
            };
            Ok(Self { body: body, dependencies: dependencies })
        }
    }
}

pub use nat::Nat as Nat;

pub mod vec {
    mod deps {
        pub(super) use super::super::{Box, ConstructorError};
        pub(super) use super::super::{{nat, Nat}};
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Body {
        Cons {
            val: deps::Box<deps::nat::Nat>,
            tail: deps::Box<Vec>
        },
        Nil {
        
        }
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Dependencies {
        pub n: deps::Box<deps::nat::Nat>
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Vec {
        pub body: Body,
        pub dependencies: Dependencies
    }
    
    impl Vec {
        pub fn Cons(p: deps::Box<deps::nat::Nat>, val: deps::Box<deps::nat::Nat>, tail: deps::Box<Vec>) -> Result<Self, deps::ConstructorError> {
            let body = if ((),
            (&p)) == ((),
            (&tail.dependencies.n)) {
                Ok(Body::Cons {
                    val: val,
                    tail: tail
                })
            } else {
                Err(deps::ConstructorError::MismatchedDependencies)
            }?;
            let dependencies = Dependencies {
                n: Box::new(deps::nat::Nat::Suc(p).expect("constructor 'Nat::Suc' failed"))
            };
            Ok(Self { body: body, dependencies: dependencies })
        }
        pub fn Nil() -> Result<Self, deps::ConstructorError> {
            let body = if () == () {
                Ok(Body::Nil {
                
                })
            } else {
                Err(deps::ConstructorError::MismatchedDependencies)
            }?;
            let dependencies = Dependencies {
                n: Box::new(deps::nat::Nat::Zero().expect("constructor 'Nat::Zero' failed"))
            };
            Ok(Self { body: body, dependencies: dependencies })
        }
    }
}

pub use vec::Vec as Vec;
