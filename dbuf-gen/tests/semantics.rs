mod tests {
    mod basic {
        include!("./canon/basic.rs");
    }

    #[test]
    fn basic_counting() {
        let four1 = {
            let zero = basic::Nat::Zero().expect("couldn't construct Zero");
            let one = basic::Nat::Suc(Box::new(zero)).expect("couldn't construct One");

            let mut following = one;
            for _ in 2..5 {
                following =
                    basic::Nat::Suc(Box::new(following)).expect("couldn't construct Following");
            }
            following
        };

        let four2 = {
            let zero = basic::Nat::Zero().expect("couldn't construct Zero");
            let one = basic::Nat::Suc(Box::new(zero)).expect("couldn't construct One");
            let two = basic::Nat::Suc(Box::new(one)).expect("couldn't construct One");
            let three = basic::Nat::Suc(Box::new(two)).expect("couldn't construct One");
            let four = basic::Nat::Suc(Box::new(three)).expect("couldn't construct One");
            four
        };

        assert_eq!(four1, four2);
    }

    mod nat_vec {
        include!("./canon/nat_vec.rs");
    }

    #[test]
    fn nat_vec_extending() {
        let nil = nat_vec::Vec::Nil().expect("couldn't construct Nil");
        let cons = nat_vec::Vec::Cons(
            nil.dependencies.n.clone(),
            Box::new(nat_vec::Nat::Zero().expect("couldn't generate Zero")),
            Box::new(nil),
        )
        .expect("couldn't construct Cons");

        let mut following = cons;
        for _ in 2..5 {
            following = nat_vec::Vec::Cons(
                following.dependencies.n.clone(),
                Box::new(nat_vec::Nat::Zero().expect("couldn't generate Zero")),
                Box::new(following),
            )
            .expect("couldn't construct Following")
        }
    }
}
