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
