use std::ops::{Deref, DerefMut};

/// Type alias for a sequence of definitions.
pub type Definitions<Loc, Name, Data> = Vec<Definition<Loc, Name, Data>>;

/// Located definition of a named entity.
#[derive(Clone, Copy, Debug)]
pub struct Definition<Loc, Name, Data> {
    /// Definition location.
    pub loc: Loc,
    /// Name for new entity.
    pub name: Name,
    /// Definition content.
    pub data: Data,
}

impl<Loc, Str, Data> Deref for Definition<Loc, Str, Data> {
    type Target = Data;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<Loc, Str, Data> DerefMut for Definition<Loc, Str, Data> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
