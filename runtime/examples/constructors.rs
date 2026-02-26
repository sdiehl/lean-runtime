//! Test constructor allocation and pattern matching
//! Simulates List and Option types

use lean_runtime::*;

// List constructors
// nil : List α     -> tag 0, 0 fields
// cons : α → List α → List α  -> tag 1, 2 fields

unsafe fn l_nil() -> *mut LeanObject {
    lean_alloc_ctor(0, 0, 0)
}

unsafe fn l_cons(head: *mut LeanObject, tail: *mut LeanObject) -> *mut LeanObject {
    let obj = lean_alloc_ctor(1, 2, 0);
    lean_ctor_set(obj, 0, head);
    lean_ctor_set(obj, 1, tail);
    obj
}

// Option constructors
// none : Option α    -> tag 0, 0 fields
// some : α → Option α -> tag 1, 1 field

unsafe fn l_none() -> *mut LeanObject {
    lean_box(0) // Optimized: none is scalar 0
}

unsafe fn l_some(val: *mut LeanObject) -> *mut LeanObject {
    let obj = lean_alloc_ctor(1, 1, 0);
    lean_ctor_set(obj, 0, val);
    obj
}

// List.length : List α → Nat
unsafe fn l_list_length(list: *mut LeanObject) -> *mut LeanObject {
    let mut acc: usize = 0;
    let mut curr = list;

    loop {
        let tag = lean_obj_tag(curr);
        if tag == 0 {
            // nil
            break;
        } else {
            // cons
            acc += 1;
            lean_inc(curr);
            let tail = lean_ctor_get(curr, 1);
            lean_inc(tail);
            lean_dec(curr);
            curr = tail;
        }
    }

    lean_box(acc)
}

// List.head? : List α → Option α
unsafe fn l_list_head(list: *mut LeanObject) -> *mut LeanObject {
    let tag = lean_obj_tag(list);
    if tag == 0 {
        l_none()
    } else {
        let head = lean_ctor_get(list, 0);
        lean_inc(head);
        l_some(head)
    }
}

// List.map (f : α → β) (xs : List α) : List β
unsafe fn l_list_map(f: *mut LeanObject, xs: *mut LeanObject) -> *mut LeanObject {
    let tag = lean_obj_tag(xs);
    if tag == 0 {
        lean_dec(f);
        l_nil()
    } else {
        let head = lean_ctor_get(xs, 0);
        lean_inc(head);
        let tail = lean_ctor_get(xs, 1);
        lean_inc(tail);

        lean_inc(f);
        let new_head = lean_apply_1(f, head);
        let new_tail = l_list_map(f, tail);

        l_cons(new_head, new_tail)
    }
}

unsafe fn print_list_nat(list: *mut LeanObject) {
    print!("[");
    let mut curr = list;
    let mut first = true;

    loop {
        let tag = lean_obj_tag(curr);
        if tag == 0 {
            break;
        } else {
            if !first {
                print!(", ");
            }
            first = false;

            let head = lean_ctor_get(curr, 0);
            print!("{}", lean_unbox(head));

            curr = lean_ctor_get(curr, 1);
        }
    }
    println!("]");
}

// Helper: create a closure for (· + 1)
unsafe fn add_one(x: *mut LeanObject) -> *mut LeanObject {
    lean_nat_add(x, lean_box(1))
}

fn main() {
    unsafe {
        println!("=== Constructor Tests ===");

        // Build list [1, 2, 3]
        let list = l_cons(
            lean_box(1),
            l_cons(lean_box(2), l_cons(lean_box(3), l_nil())),
        );

        print!("list = ");
        lean_inc(list);
        print_list_nat(list);

        // Test length
        lean_inc(list);
        let len = l_list_length(list);
        println!("length = {}", lean_unbox(len));

        // Test head
        lean_inc(list);
        let head_opt = l_list_head(list);
        let tag = lean_obj_tag(head_opt);
        if tag == 1 {
            let head = lean_ctor_get(head_opt, 0);
            println!("head = some({})", lean_unbox(head));
        } else {
            println!("head = none");
        }
        lean_dec(head_opt);

        // Test map with closure
        let closure = lean_alloc_closure(add_one as *const (), 1, 0);
        lean_inc(list);
        let mapped = l_list_map(closure, list);
        print!("map (· + 1) list = ");
        print_list_nat(mapped);

        // Cleanup
        lean_dec(list);
        lean_dec(mapped);

        println!("=== All tests passed ===");
    }
}
