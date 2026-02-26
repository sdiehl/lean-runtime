//! Test closures and partial application

use lean_runtime::*;

// add : Nat → Nat → Nat
unsafe fn l_add(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    lean_nat_add(a, b)
}

// mul : Nat → Nat → Nat
unsafe fn l_mul(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    lean_nat_mul(a, b)
}

// compose : (β → γ) → (α → β) → α → γ
unsafe fn l_compose(f: *mut LeanObject, g: *mut LeanObject, x: *mut LeanObject) -> *mut LeanObject {
    let gx = lean_apply_1(g, x);
    lean_apply_1(f, gx)
}

// twice : (α → α) → α → α
unsafe fn l_twice(f: *mut LeanObject, x: *mut LeanObject) -> *mut LeanObject {
    lean_inc(f);
    let fx = lean_apply_1(f, x);
    lean_apply_1(f, fx)
}

fn main() {
    unsafe {
        println!("=== Closure Tests ===");

        // Test partial application: add 5
        let add5 = lean_alloc_closure(l_add as *const (), 2, 1);
        lean_closure_set(add5, 0, lean_box(5));

        // Apply add5 to 10
        lean_inc(add5);
        let result = lean_apply_1(add5, lean_box(10));
        println!("(add 5) 10 = {}", lean_unbox(result));

        // Test double partial application
        let mul_closure = lean_alloc_closure(l_mul as *const (), 2, 0);
        lean_inc(mul_closure);
        let mul3 = lean_apply_1(mul_closure, lean_box(3));
        let result2 = lean_apply_1(mul3, lean_box(7));
        println!("(mul 3) 7 = {}", lean_unbox(result2));

        // Test compose: (add 1) ∘ (mul 2)
        let add1 = lean_alloc_closure(l_add as *const (), 2, 1);
        lean_closure_set(add1, 0, lean_box(1));

        let mul2 = lean_alloc_closure(l_mul as *const (), 2, 1);
        lean_closure_set(mul2, 0, lean_box(2));

        let composed = lean_alloc_closure(l_compose as *const (), 3, 2);
        lean_closure_set(composed, 0, add1);
        lean_closure_set(composed, 1, mul2);

        let result3 = lean_apply_1(composed, lean_box(5));
        println!("((add 1) ∘ (mul 2)) 5 = {}", lean_unbox(result3));
        // Should be (5 * 2) + 1 = 11

        // Test twice: twice (add 3) 0 = (add 3) ((add 3) 0) = 6
        let add3 = lean_alloc_closure(l_add as *const (), 2, 1);
        lean_closure_set(add3, 0, lean_box(3));

        let twice_add3 = lean_alloc_closure(l_twice as *const (), 2, 1);
        lean_closure_set(twice_add3, 0, add3);

        let result4 = lean_apply_1(twice_add3, lean_box(0));
        println!("twice (add 3) 0 = {}", lean_unbox(result4));

        // Cleanup
        lean_dec(add5);
        lean_dec(mul_closure);

        println!("=== All tests passed ===");
    }
}
