sealed class Nat {
    private constructor() {
        
        // constructor body
    }
    class Suc: Nat {
        constructor(): super() {
        // inner class constructor
        }
    }
    class Zero: Nat {
        constructor(): super() {
        // inner class constructor
        }
    }
}
sealed class Vec {
    val n: Nat;
    private constructor(n: Nat) {
        this.n = n;
        // constructor body
    }
    class Cons: Vec {
        constructor(): super() {
        // inner class constructor
        }
    }
    class Nil: Vec {
        constructor(): super() {
        // inner class constructor
        }
    }
}
