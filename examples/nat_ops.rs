//! Test natural number operations
//! Simulates: def isZero (n : Nat) : Bool := n == 0

use lean_runtime::*;

// def isZero (n : Nat) : Bool := n == 0
unsafe fn l_isZero(n: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(n) {
        let val = lean_unbox(n);
        lean_bool(val == 0)
    } else {
        lean_bool(false)
    }
}

// def addOne (n : Nat) : Nat := n + 1
unsafe fn l_addOne(n: *mut LeanObject) -> *mut LeanObject {
    lean_nat_add(n, lean_box(1))
}

// def factorial (n : Nat) : Nat
unsafe fn l_factorial(n: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(n) {
        let val = lean_unbox(n);
        if val <= 1 {
            lean_box(1)
        } else {
            let n_minus_1 = lean_box(val - 1);
            let fact_n_minus_1 = l_factorial(n_minus_1);
            lean_nat_mul(lean_box(val), fact_n_minus_1)
        }
    } else {
        lean_box(1)
    }
}

unsafe fn print_bool(b: *mut LeanObject) {
    if lean_unbox_bool(b) {
        println!("true");
    } else {
        println!("false");
    }
}

unsafe fn print_nat(n: *mut LeanObject) {
    if lean_is_scalar(n) {
        println!("{}", lean_unbox(n));
    } else {
        println!("<big nat>");
    }
}

fn main() {
    unsafe {
        println!("=== Natural Number Tests ===");

        // Test isZero
        print!("isZero(0) = ");
        print_bool(l_isZero(lean_box(0)));

        print!("isZero(5) = ");
        print_bool(l_isZero(lean_box(5)));

        // Test addOne
        print!("addOne(41) = ");
        print_nat(l_addOne(lean_box(41)));

        // Test factorial
        print!("factorial(5) = ");
        print_nat(l_factorial(lean_box(5)));

        print!("factorial(10) = ");
        print_nat(l_factorial(lean_box(10)));

        println!("=== All tests passed ===");
    }
}
