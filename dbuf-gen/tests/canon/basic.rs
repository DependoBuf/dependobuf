use dbuf_rust_runtime::{Box, ConstructorError};

pub mod nat {
    mod deps {
        pub(super) use super::super::{Box, ConstructorError};
        // pub(super) use super::super::{};
    }
    
    #[derive(PartialEq, Eq)]
    pub enum Body {
        Suc {
            pred: deps::Box<Nat>
        },
        Zero {
        
        }
    }
    
    #[derive(PartialEq, Eq)]
    struct Dependencies {
    
    }
    
    #[derive(PartialEq, Eq)]
    pub struct Nat {
        body: Body,
        dependencies: Dependencies
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
