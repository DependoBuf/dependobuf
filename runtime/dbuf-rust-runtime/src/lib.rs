#[derive(Debug)]
pub enum ConstructorError {
    MismatchedDependencies,
}

pub type Box<T> = std::boxed::Box<T>;
