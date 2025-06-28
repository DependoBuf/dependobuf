mod tests {
    mod basic {
        include!("./canon/basic.rs");
    }

    #[test]
    fn basic_counting() {
        let four1 = {
            let zero = basic::Nat::zero().expect("couldn't construct zero");
            let one = basic::Nat::suc(Box::new(zero)).expect("couldn't construct One");

            let mut following = one;
            for _ in 2..5 {
                following =
                    basic::Nat::suc(Box::new(following)).expect("couldn't construct Following");
            }
            following
        };

        let four2 = {
            let zero = basic::Nat::zero().expect("couldn't construct zero");
            let one = basic::Nat::suc(Box::new(zero)).expect("couldn't construct One");
            let two = basic::Nat::suc(Box::new(one)).expect("couldn't construct One");
            let three = basic::Nat::suc(Box::new(two)).expect("couldn't construct One");
            let four = basic::Nat::suc(Box::new(three)).expect("couldn't construct One");
            four
        };

        assert_eq!(four1, four2);
    }

    mod nat_vec {
        include!("./canon/nat_vec.rs");
    }

    #[test]
    fn nat_vec_extending() {
        let nil = nat_vec::Vec::nil().expect("couldn't construct nil");
        let cons = nat_vec::Vec::cons(
            nil.dependencies.n.clone(),
            Box::new(nat_vec::Nat::zero().expect("couldn't generate zero")),
            Box::new(nil),
        )
        .expect("couldn't construct cons");

        let mut following = cons;
        for _ in 2..5 {
            following = nat_vec::Vec::cons(
                following.dependencies.n.clone(),
                Box::new(nat_vec::Nat::zero().expect("couldn't generate zero")),
                Box::new(following),
            )
            .expect("couldn't construct Following")
        }
    }
}
