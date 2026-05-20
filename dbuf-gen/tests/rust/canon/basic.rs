use dbuf_rust_runtime::{Box, ConstructorError, DeserializeError, DbufPrimitive};
use std::io::{Write, Read, Error};
use std::slice;
pub mod nat {
    mod deps {
        // pub(super) use super::super::{};
    }
    mod descriptor {
        pub(super) const Suc: u8 = 0;
        pub(super) const Zero: u8 = 1;
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Body {
        Suc {
            pred: super::Box<Nat>
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
        pub fn suc(pred: super::Box<Nat>) -> Result<Self, super::ConstructorError> {
            let body = if (()) == (()) {
                Ok(Body::Suc {
                    pred: pred
                })
            } else {
                Err(super::ConstructorError::MismatchedDependencies)
            }?;
            let dependencies = Dependencies {
            
            };
            Ok(Self { body: body, dependencies: dependencies })
        }
        pub fn zero() -> Result<Self, super::ConstructorError> {
            let body = if () == () {
                Ok(Body::Zero {
                
                })
            } else {
                Err(super::ConstructorError::MismatchedDependencies)
            }?;
            let dependencies = Dependencies {
            
            };
            Ok(Self { body: body, dependencies: dependencies })
        }
        pub fn serialize<W: super::Write>(self, writer: &mut W) -> Result<(), super::Error> {
            match self.body {
                Body::Suc { pred } => {
                    writer.write_all(&[descriptor::Suc])?;
                    pred.serialize(writer)?;
                },
                Body::Zero {  } => {
                    writer.write_all(&[descriptor::Zero])?;
                },
            }
            Ok(())
        }
        pub fn deserialize<R: super::Read>(dependencies: Dependencies, reader: &mut R) -> Result<Self, super::DeserializeError> {
            let mut descriptor = 0;
            reader.read(super::slice::from_mut(&mut descriptor)).map_err(|e| super::DeserializeError::IoError(e))?;
            match descriptor {
                descriptor::Suc => {
                    if let () = () {
                        let pred = Self::deserialize(Dependencies {
                        
                        }, reader)?;
                        Self::suc(Box::new(pred)).map_err(|e| super::DeserializeError::ConstructorError(e))
                    } else {
                        Err(super::DeserializeError::DependenciesDescriptorMismatch)
                    }},
                descriptor::Zero => {
                    if let () = () {
                        Self::zero().map_err(|e| super::DeserializeError::ConstructorError(e))
                    } else {
                        Err(super::DeserializeError::DependenciesDescriptorMismatch)
                    }},
                _ => Err(super::DeserializeError::UnknownDescriptor),
            }
        }
    }
}

pub use nat::Nat as Nat;
