use dbuf_storage::Database;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub enum ConstructorError {
    MismatchedDependencies,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Message<Body, Dependencies> {
    pub body: Body,
    pub dependencies: Dependencies,
}

pub type Box<T> = std::boxed::Box<T>;

pub mod sum {
    use serde::{Deserialize, Serialize};

    mod deps {
        pub use super::super::{ConstructorError, Message};
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Body {}

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Dependencies {
        pub a: i32,
    }

    // alias for the generated type
    pub type Sum = deps::Message<Body, Dependencies>;

    // inherit implementation with all constructors
    impl Sum {
        pub fn new(dependencies: Dependencies) -> Result<Self, deps::ConstructorError> {
            let body = Body {};
            Ok(deps::Message { body, dependencies })
        }
    }
}

pub use sum::Sum;

pub mod foo {
    use deps::sum;
    use serde::{Deserialize, Serialize};

    mod deps {
        pub use super::super::{sum, Sum};
        pub use super::super::{ConstructorError, Message};
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Body {
        pub sum: deps::Sum,
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Dependencies {
        pub a: i32,
        pub b: i32,
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

pub mod user {
    use serde::{Deserialize, Serialize};

    mod deps {
        pub use super::super::{ConstructorError, Message};
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Body {
        pub a: i32,
        pub b: i32,
        pub c: i32,
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Dependencies {}

    // alias for the generated type
    pub type User = deps::Message<Body, Dependencies>;

    // inherit implementation with all constructors
    impl User {
        pub fn new(body: Body) -> Result<Self, deps::ConstructorError> {
            let dependencies = Dependencies {};
            Ok(deps::Message { body, dependencies })
        }
    }
}

pub mod nat {
    // general prelude
    use super::{Box, ConstructorError, Message};
    use serde::{Deserialize, Serialize};

    // optional part where used types are imported

    // body definition
    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub enum Body {
        Zero,
        Suc { pred: Box<Self> },
    }

    // dependencies definition
    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct Dependencies {}

    // alias for the generated type
    pub type Nat = Message<Body, Dependencies>;

    // inherit implementation with all constructors
    impl Nat {
        pub fn zero() -> Result<Self, ConstructorError> {
            let body = Body::Zero;
            let dependencies = Dependencies {};
            Ok(Message { body, dependencies })
        }

        pub fn suc(pred: Nat) -> Result<Self, ConstructorError> {
            let body = Body::Suc {
                pred: Box::new(pred.body),
            };
            let dependencies = Dependencies {};
            Ok(Message { body, dependencies })
        }
    }
}

const TEST_DATA: &str = "./tests/test_data";

struct CleanupDir(String);
impl Drop for CleanupDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_sum_db_operations() {
    let db_path = format!("{}/test_sum_db_operations", TEST_DATA);
    let db = Database::new(&db_path).expect("Failed to open database");

    let _cleanup = CleanupDir(db_path.clone());

    let sum_deps = sum::Dependencies { a: 42 };
    let sum = sum::Sum::new(sum_deps).unwrap();

    let id = db.insert(&sum).unwrap();

    let retrieved: sum::Sum = db.get(&id).unwrap();

    assert_eq!(retrieved.dependencies.a, 42);
    assert_eq!(retrieved, sum);

    let updated_deps = sum::Dependencies { a: 100 };
    let updated_sum = sum::Sum::new(updated_deps).unwrap();

    db.update(&id, &updated_sum).unwrap();

    let retrieved_updated: sum::Sum = db.get(&id).unwrap();
    assert_eq!(retrieved_updated.dependencies.a, 100);
    assert_eq!(retrieved_updated, updated_sum);
}

#[test]
fn test_foo_db_operations() {
    let db_path = format!("{}/test_foo_db_operations", TEST_DATA);
    let db = Database::new(&db_path).expect("Failed to open database");

    let _cleanup = CleanupDir(db_path.clone());

    let foo_deps = foo::Dependencies { a: 10, b: 20 };
    let foo = foo::Foo::new(foo_deps).unwrap();

    let id = db.insert(&foo).unwrap();

    let retrieved: foo::Foo = db.get(&id).unwrap();

    assert_eq!(retrieved.dependencies.a, 10);
    assert_eq!(retrieved.dependencies.b, 20);
    assert_eq!(retrieved.body.sum.dependencies.a, 10); // -a + b = -10 + 20 = 10
    assert_eq!(retrieved, foo);

    let updated_deps = foo::Dependencies { a: 30, b: 50 };
    let updated_foo = foo::Foo::new(updated_deps).unwrap();

    db.update(&id, &updated_foo).unwrap();

    let retrieved_updated: foo::Foo = db.get(&id).unwrap();
    assert_eq!(retrieved_updated.dependencies.a, 30);
    assert_eq!(retrieved_updated.dependencies.b, 50);
    assert_eq!(retrieved_updated.body.sum.dependencies.a, 20); // -a + b = -30 + 50 = 20
    assert_eq!(retrieved_updated, updated_foo);
}

#[test]
fn test_user_db_operations() {
    let db_path = format!("{}/test_user_db_operations", TEST_DATA);
    let db = Database::new(&db_path).expect("Failed to open database");

    let _cleanup = CleanupDir(db_path.clone());

    let user_body = user::Body { a: 1, b: 2, c: 3 };
    let user = user::User::new(user_body).unwrap();

    let id = db.insert(&user).unwrap();

    let retrieved: user::User = db.get(&id).unwrap();

    assert_eq!(retrieved.body.a, 1);
    assert_eq!(retrieved.body.b, 2);
    assert_eq!(retrieved.body.c, 3);
    assert_eq!(retrieved, user);

    let updated_body = user::Body { a: 4, b: 5, c: 6 };
    let updated_user = user::User::new(updated_body).unwrap();

    db.update(&id, &updated_user).unwrap();

    let retrieved_updated: user::User = db.get(&id).unwrap();
    assert_eq!(retrieved_updated.body.a, 4);
    assert_eq!(retrieved_updated.body.b, 5);
    assert_eq!(retrieved_updated.body.c, 6);
    assert_eq!(retrieved_updated, updated_user);
}

#[test]
fn test_nat_db_operations() {
    let db_path = format!("{}/test_nat_db_operations", TEST_DATA);
    let db = Database::new(&db_path).expect("Failed to open database");

    let _cleanup = CleanupDir(db_path.clone());

    let zero = nat::Nat::zero().unwrap();
    let one = nat::Nat::suc(nat::Nat::zero().unwrap()).unwrap();
    let two = nat::Nat::suc(nat::Nat::suc(nat::Nat::zero().unwrap()).unwrap()).unwrap();

    let zero_id = db.insert(&zero).unwrap();
    let one_id = db.insert(&one).unwrap();
    let two_id = db.insert(&two).unwrap();

    let retrieved_zero: nat::Nat = db.get(&zero_id).unwrap();
    let retrieved_one: nat::Nat = db.get(&one_id).unwrap();
    let retrieved_two: nat::Nat = db.get(&two_id).unwrap();

    match &retrieved_zero.body {
        nat::Body::Zero => {}
        _ => panic!("Expected Zero"),
    }

    match &retrieved_one.body {
        nat::Body::Suc { pred } => match &**pred {
            nat::Body::Zero => {}
            _ => panic!("Expected Suc(Zero)"),
        },
        _ => panic!("Expected Suc"),
    }

    match &retrieved_two.body {
        nat::Body::Suc { pred } => match &**pred {
            nat::Body::Suc { pred: inner_pred } => match &**inner_pred {
                nat::Body::Zero => {}
                _ => panic!("Expected Suc(Suc(Zero))"),
            },
            _ => panic!("Expected Suc(Suc(_))"),
        },
        _ => panic!("Expected Suc"),
    }

    db.update(&one_id, &two).unwrap();
    let updated_one: nat::Nat = db.get(&one_id).unwrap();

    match &updated_one.body {
        nat::Body::Suc { pred } => match &**pred {
            nat::Body::Suc { pred: inner_pred } => match &**inner_pred {
                nat::Body::Zero => {}
                _ => panic!("Expected Suc(Suc(Zero)) after update"),
            },
            _ => panic!("Expected Suc(Suc(_)) after update"),
        },
        _ => panic!("Expected Suc after update"),
    }

    db.delete(&zero_id).unwrap();
    let result = db.get::<nat::Nat>(&zero_id);
    assert!(result.is_err());
}
