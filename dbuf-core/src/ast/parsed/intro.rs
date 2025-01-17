use std::ops::{Deref, DerefMut};

/// Type alias for a sequence of introductions.
pub type Intros<Loc, Name, Data> = Vec<Intro<Loc, Name, Data>>;

/// Introduction of a named, located data.
#[derive(Clone, Copy, Debug)]
pub struct Intro<Loc, Name, Data> {
    /// Introduction location.
    pub loc: Loc,
    /// Name for data.
    pub name: Name,
    /// Introduction content.
    pub data: Data,
}

impl<Loc, Str, Data> Deref for Intro<Loc, Str, Data> {
    type Target = Data;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<Loc, Str, Data> DerefMut for Intro<Loc, Str, Data> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
