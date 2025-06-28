use dbuf_rust_runtime::{serde, Box, ConstructorError, DeserializeError, Serialize, Deserialize, to_vec, from_slice};

pub mod nat {
    use super::serde::{self, Serialize, Deserialize};
    
    mod deps {
        pub(super) use super::super::{Box, ConstructorError, DeserializeError, Serialize, Deserialize, to_vec, from_slice};
        // pub(super) use super::super::{};
    }
    
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(crate = "self::serde")]
    pub enum Body {
        Suc {
            pred: deps::Box<Nat>
        },
        Zero {
        
        }
    }
    
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(crate = "self::serde")]
    pub struct Dependencies {
    
    }
    
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(crate = "self::serde")]
    pub struct Nat {
        pub body: Body,
        pub dependencies: Dependencies
    }
    
    impl Nat {
        pub fn suc(pred: deps::Box<Nat>) -> Result<Self, deps::ConstructorError> {
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
        pub fn zero() -> Result<Self, deps::ConstructorError> {
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
    
    impl deps::Serialize for Nat {
        fn serialize(self) -> Box<[u8]> {
            deps::to_vec(&self).unwrap().into_boxed_slice()
        }
    }
    
    impl deps::Deserialize for Nat {
        fn deserialize<'a>(slice: &'a [u8]) -> Result<Self, deps::DeserializeError> {
            deps::from_slice::<Self>(slice).map_err(|err| err.into())
        }
    }
}

pub use nat::Nat as Nat;
