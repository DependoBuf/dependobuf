pub mod lookup;
pub mod namespace;
pub mod node;

use crate::format::BoxAllocator;

#[derive(Clone, Copy)]
pub struct GlobalContext<'a> {
    pub alloc: &'a BoxAllocator,
}

mod tests {
    use super::{lookup::Cursor, namespace::NamespaceTree};

    #[test]
    fn basic() {
        type Namespace<'a> = NamespaceTree<'a, i32, String>;
        let mut s0 = Namespace::root("a".to_owned());
        let mut s1 = s0.insert(0, "b".to_owned());
        {
            let s11 = s1.insert(0, "c".to_owned());
            s11.finish();
        }

        {
            let s12 = s1.insert(1, "d".to_owned());

            {
                let c = s12.cursor();
                assert_eq!(c.value(), "d");
                let c = c.go_back().unwrap();
                assert_eq!(c.value(), "b");
                assert_eq!(c.clone().next(&0).unwrap().value(), "c");
                let c = c.go_back().unwrap();
                assert_eq!(c.value(), "a");
            }

            s12.finish();
        }
    }
}
