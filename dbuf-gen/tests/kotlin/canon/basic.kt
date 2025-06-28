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
