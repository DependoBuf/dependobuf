sealed class Nat {
    private constructor() {
        // constructor asserts
    }
    class Suc: Nat {
        val pred: Nat;
        constructor(pred: Nat): super() {
            // inner class asserts
            this.pred = pred;
        }
    }
    class Zero: Nat {
        constructor(): super() {
            // inner class asserts
        }
    }
}
sealed class Vec {
    val n: Nat;
    private constructor(n: Nat) {
        // constructor asserts
        this.n = n;
    }
    class Cons: Vec {
        val value: Nat;
        val tail: Vec;
        constructor(p: Nat, value: Nat, tail: Vec): super(Nat.Suc(p)) {
            // inner class asserts
            this.value = value;
            this.tail = tail;
        }
    }
    class Nil: Vec {
        constructor(): super(Nat.Zero()) {
            // inner class asserts
        }
    }
}
