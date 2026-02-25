//! Closure support

use crate::object::*;

#[repr(C)]
pub struct LeanClosure {
    pub header: LeanObject,
    pub fun: *const (),
    pub arity: u16,
    pub num_fixed: u16,
}

impl LeanClosure {
    #[inline(always)]
    pub unsafe fn fixed_args_ptr(&self) -> *mut *mut LeanObject {
        (self as *const Self as *mut u8).add(std::mem::size_of::<Self>()) as *mut *mut LeanObject
    }
}

#[inline(always)]
pub const fn lean_closure_object_size(num_fixed: u32) -> usize {
    std::mem::size_of::<LeanClosure>() + (num_fixed as usize) * 8
}

#[inline]
pub unsafe fn lean_alloc_closure(fun: *const (), arity: u32, num_fixed: u32) -> *mut LeanObject {
    let size = lean_closure_object_size(num_fixed);
    let obj = lean_alloc_object(size);
    let closure = obj as *mut LeanClosure;
    (*closure).header.rc = 1;
    (*closure).header.tag = LEAN_CLOSURE_TAG;
    (*closure).header.other = 0;
    (*closure).header.cs_sz = 0;
    (*closure).fun = fun;
    (*closure).arity = arity as u16;
    (*closure).num_fixed = num_fixed as u16;
    obj
}

#[inline(always)]
pub unsafe fn lean_closure_get(o: *mut LeanObject, i: u32) -> *mut LeanObject {
    *(*(o as *mut LeanClosure)).fixed_args_ptr().add(i as usize)
}

#[inline(always)]
pub unsafe fn lean_closure_set(o: *mut LeanObject, i: u32, v: *mut LeanObject) {
    *(*(o as *mut LeanClosure)).fixed_args_ptr().add(i as usize) = v;
}

macro_rules! lean_fn_type {
    ($name:ident, $($param:ident),+) => {
        pub type $name = unsafe fn($($param: *mut LeanObject),+) -> *mut LeanObject;
    };
}

lean_fn_type!(LeanFn1, a1);
lean_fn_type!(LeanFn2, a1, a2);
lean_fn_type!(LeanFn3, a1, a2, a3);
lean_fn_type!(LeanFn4, a1, a2, a3, a4);
lean_fn_type!(LeanFn5, a1, a2, a3, a4, a5);
lean_fn_type!(LeanFn6, a1, a2, a3, a4, a5, a6);
lean_fn_type!(LeanFn7, a1, a2, a3, a4, a5, a6, a7);
lean_fn_type!(LeanFn8, a1, a2, a3, a4, a5, a6, a7, a8);

/// Collect fixed args from a closure into a buffer, incrementing their refcounts.
unsafe fn collect_fixed(f: *mut LeanObject, num_fixed: u32, buf: &mut [*mut LeanObject]) {
    for i in 0..num_fixed {
        let arg = lean_closure_get(f, i);
        crate::lean_inc(arg);
        buf[i as usize] = arg;
    }
}

/// Create a new closure with `num_fixed` existing fixed args + `n` new args appended.
unsafe fn fix_args(f: *mut LeanObject, new_args: &[*mut LeanObject]) -> *mut LeanObject {
    let closure = f as *mut LeanClosure;
    let arity = (*closure).arity as u32;
    let num_fixed = (*closure).num_fixed as u32;
    let total = num_fixed + new_args.len() as u32;
    let new_c = lean_alloc_closure((*closure).fun, arity, total);
    for i in 0..num_fixed {
        let arg = lean_closure_get(f, i);
        crate::lean_inc(arg);
        lean_closure_set(new_c, i, arg);
    }
    for (j, &arg) in new_args.iter().enumerate() {
        lean_closure_set(new_c, num_fixed + j as u32, arg);
    }
    crate::lean_dec(f);
    new_c
}

/// Call a fully-saturated closure. `args` must contain exactly `arity` arguments
/// (fixed args already prepended by caller).
unsafe fn curry(fun: *const (), arity: u32, args: &[*mut LeanObject]) -> *mut LeanObject {
    macro_rules! curry_call {
        ($fn_type:ty, [$($i:literal),+]) => {{
            let fp: $fn_type = std::mem::transmute(fun);
            fp($(args[$i]),+)
        }};
    }
    match arity {
        1 => curry_call!(LeanFn1, [0]),
        2 => curry_call!(LeanFn2, [0, 1]),
        3 => curry_call!(LeanFn3, [0, 1, 2]),
        4 => curry_call!(LeanFn4, [0, 1, 2, 3]),
        5 => curry_call!(LeanFn5, [0, 1, 2, 3, 4]),
        6 => curry_call!(LeanFn6, [0, 1, 2, 3, 4, 5]),
        7 => curry_call!(LeanFn7, [0, 1, 2, 3, 4, 5, 6]),
        8 => curry_call!(LeanFn8, [0, 1, 2, 3, 4, 5, 6, 7]),
        9..=16 => {
            type LeanFnM =
                unsafe fn(*mut LeanObject, u32, *const *mut LeanObject) -> *mut LeanObject;
            let fp: LeanFnM = std::mem::transmute(fun);
            fp(args[0], arity - 1, args[1..].as_ptr())
        }
        _ => panic!("curry: arity {arity} not supported"),
    }
}

/// Apply `n` arguments to a closure. Handles partial application, exact saturation,
/// and over-saturation (chaining).
unsafe fn apply_n(mut f: *mut LeanObject, new_args: &[*mut LeanObject]) -> *mut LeanObject {
    let mut args_remaining = new_args;
    loop {
        let closure = f as *mut LeanClosure;
        let arity = (*closure).arity as u32;
        let num_fixed = (*closure).num_fixed as u32;
        let n = args_remaining.len() as u32;
        let total = num_fixed + n;

        if total < arity {
            // Partial application: not enough args yet
            return fix_args(f, args_remaining);
        } else if total == arity {
            // Exact saturation
            let fun = (*closure).fun;
            let mut all_args = [std::ptr::null_mut::<LeanObject>(); 16];
            collect_fixed(f, num_fixed, &mut all_args);
            for (j, &arg) in args_remaining.iter().enumerate() {
                all_args[num_fixed as usize + j] = arg;
            }
            let result = curry(fun, arity, &all_args[..arity as usize]);
            crate::lean_dec(f);
            return result;
        } else {
            // Over-saturation: saturate first, then loop with remaining args
            let needed = (arity - num_fixed) as usize;
            let (first, rest) = args_remaining.split_at(needed);
            let fun = (*closure).fun;
            let mut all_args = [std::ptr::null_mut::<LeanObject>(); 16];
            collect_fixed(f, num_fixed, &mut all_args);
            for (j, &arg) in first.iter().enumerate() {
                all_args[num_fixed as usize + j] = arg;
            }
            let result = curry(fun, arity, &all_args[..arity as usize]);
            crate::lean_dec(f);
            f = result;
            args_remaining = rest;
        }
    }
}

macro_rules! lean_apply {
    ($name:ident, $($arg:ident),+) => {
        #[inline]
        #[allow(clippy::too_many_arguments)]
        pub unsafe fn $name(f: *mut LeanObject, $($arg: *mut LeanObject),+) -> *mut LeanObject {
            apply_n(f, &[$($arg),+])
        }
    };
}

lean_apply!(lean_apply_1, a1);
lean_apply!(lean_apply_2, a1, a2);
lean_apply!(lean_apply_3, a1, a2, a3);
lean_apply!(lean_apply_4, a1, a2, a3, a4);
lean_apply!(lean_apply_5, a1, a2, a3, a4, a5);
lean_apply!(lean_apply_6, a1, a2, a3, a4, a5, a6);
lean_apply!(lean_apply_7, a1, a2, a3, a4, a5, a6, a7);
lean_apply!(lean_apply_8, a1, a2, a3, a4, a5, a6, a7, a8);

/// Apply `n` arguments passed as a raw pointer array.
#[inline]
pub unsafe fn lean_apply_m(
    f: *mut LeanObject,
    n: u32,
    args: *const *mut LeanObject,
) -> *mut LeanObject {
    let args_slice = std::slice::from_raw_parts(args, n as usize);
    apply_n(f, args_slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn test_fn_1(a: *mut LeanObject) -> *mut LeanObject {
        // identity
        a
    }

    unsafe fn test_fn_2(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
        // add two scalars
        let va = crate::lean_unbox(a);
        let vb = crate::lean_unbox(b);
        crate::lean_box(va + vb)
    }

    unsafe fn test_fn_3(
        a: *mut LeanObject,
        b: *mut LeanObject,
        c: *mut LeanObject,
    ) -> *mut LeanObject {
        let va = crate::lean_unbox(a);
        let vb = crate::lean_unbox(b);
        let vc = crate::lean_unbox(c);
        crate::lean_box(va + vb + vc)
    }

    #[test]
    fn apply_1_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_1 as *const (), 1, 0);
            let arg = crate::lean_box(42);
            let result = lean_apply_1(f, arg);
            assert_eq!(crate::lean_unbox(result), 42);
        }
    }

    #[test]
    fn apply_1_partial() {
        unsafe {
            // Create arity-2 closure, apply one arg -> get partial, then apply second
            let f = lean_alloc_closure(test_fn_2 as *const (), 2, 0);
            let partial = lean_apply_1(f, crate::lean_box(10));
            assert!(!crate::lean_is_scalar(partial));
            let result = lean_apply_1(partial, crate::lean_box(20));
            assert_eq!(crate::lean_unbox(result), 30);
        }
    }

    #[test]
    fn apply_2_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_2 as *const (), 2, 0);
            let result = lean_apply_2(f, crate::lean_box(5), crate::lean_box(7));
            assert_eq!(crate::lean_unbox(result), 12);
        }
    }

    #[test]
    fn apply_2_with_fixed() {
        unsafe {
            // arity-3 closure with 1 fixed arg, apply 2 more
            let f = lean_alloc_closure(test_fn_3 as *const (), 3, 1);
            lean_closure_set(f, 0, crate::lean_box(100));
            let result = lean_apply_2(f, crate::lean_box(10), crate::lean_box(1));
            assert_eq!(crate::lean_unbox(result), 111);
        }
    }

    #[test]
    fn apply_3_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_3 as *const (), 3, 0);
            let result = lean_apply_3(
                f,
                crate::lean_box(1),
                crate::lean_box(2),
                crate::lean_box(3),
            );
            assert_eq!(crate::lean_unbox(result), 6);
        }
    }

    #[test]
    fn partial_chain() {
        unsafe {
            // arity-3, apply one at a time
            let f = lean_alloc_closure(test_fn_3 as *const (), 3, 0);
            let p1 = lean_apply_1(f, crate::lean_box(10));
            let p2 = lean_apply_1(p1, crate::lean_box(20));
            let result = lean_apply_1(p2, crate::lean_box(30));
            assert_eq!(crate::lean_unbox(result), 60);
        }
    }

    #[test]
    fn apply_m() {
        unsafe {
            let f = lean_alloc_closure(test_fn_3 as *const (), 3, 0);
            let args: [*mut LeanObject; 3] =
                [crate::lean_box(2), crate::lean_box(3), crate::lean_box(4)];
            let result = lean_apply_m(f, 3, args.as_ptr());
            assert_eq!(crate::lean_unbox(result), 9);
        }
    }

    // -- higher arity test functions --

    unsafe fn test_fn_4(
        a: *mut LeanObject,
        b: *mut LeanObject,
        c: *mut LeanObject,
        d: *mut LeanObject,
    ) -> *mut LeanObject {
        let r = crate::lean_unbox(a)
            + crate::lean_unbox(b)
            + crate::lean_unbox(c)
            + crate::lean_unbox(d);
        crate::lean_box(r)
    }

    unsafe fn test_fn_5(
        a: *mut LeanObject,
        b: *mut LeanObject,
        c: *mut LeanObject,
        d: *mut LeanObject,
        e: *mut LeanObject,
    ) -> *mut LeanObject {
        let r = crate::lean_unbox(a)
            + crate::lean_unbox(b)
            + crate::lean_unbox(c)
            + crate::lean_unbox(d)
            + crate::lean_unbox(e);
        crate::lean_box(r)
    }

    unsafe fn test_fn_6(
        a: *mut LeanObject,
        b: *mut LeanObject,
        c: *mut LeanObject,
        d: *mut LeanObject,
        e: *mut LeanObject,
        f: *mut LeanObject,
    ) -> *mut LeanObject {
        let r = crate::lean_unbox(a)
            + crate::lean_unbox(b)
            + crate::lean_unbox(c)
            + crate::lean_unbox(d)
            + crate::lean_unbox(e)
            + crate::lean_unbox(f);
        crate::lean_box(r)
    }

    unsafe fn test_fn_7(
        a: *mut LeanObject,
        b: *mut LeanObject,
        c: *mut LeanObject,
        d: *mut LeanObject,
        e: *mut LeanObject,
        f: *mut LeanObject,
        g: *mut LeanObject,
    ) -> *mut LeanObject {
        let r = crate::lean_unbox(a)
            + crate::lean_unbox(b)
            + crate::lean_unbox(c)
            + crate::lean_unbox(d)
            + crate::lean_unbox(e)
            + crate::lean_unbox(f)
            + crate::lean_unbox(g);
        crate::lean_box(r)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn test_fn_8(
        a: *mut LeanObject,
        b: *mut LeanObject,
        c: *mut LeanObject,
        d: *mut LeanObject,
        e: *mut LeanObject,
        f: *mut LeanObject,
        g: *mut LeanObject,
        h: *mut LeanObject,
    ) -> *mut LeanObject {
        let r = crate::lean_unbox(a)
            + crate::lean_unbox(b)
            + crate::lean_unbox(c)
            + crate::lean_unbox(d)
            + crate::lean_unbox(e)
            + crate::lean_unbox(f)
            + crate::lean_unbox(g)
            + crate::lean_unbox(h);
        crate::lean_box(r)
    }

    #[test]
    fn apply_4_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_4 as *const (), 4, 0);
            let r = lean_apply_4(
                f,
                crate::lean_box(1),
                crate::lean_box(2),
                crate::lean_box(3),
                crate::lean_box(4),
            );
            assert_eq!(crate::lean_unbox(r), 10);
        }
    }

    #[test]
    fn apply_5_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_5 as *const (), 5, 0);
            let r = lean_apply_5(
                f,
                crate::lean_box(1),
                crate::lean_box(2),
                crate::lean_box(3),
                crate::lean_box(4),
                crate::lean_box(5),
            );
            assert_eq!(crate::lean_unbox(r), 15);
        }
    }

    #[test]
    fn apply_6_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_6 as *const (), 6, 0);
            let r = lean_apply_6(
                f,
                crate::lean_box(1),
                crate::lean_box(2),
                crate::lean_box(3),
                crate::lean_box(4),
                crate::lean_box(5),
                crate::lean_box(6),
            );
            assert_eq!(crate::lean_unbox(r), 21);
        }
    }

    #[test]
    fn apply_7_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_7 as *const (), 7, 0);
            let r = lean_apply_7(
                f,
                crate::lean_box(1),
                crate::lean_box(2),
                crate::lean_box(3),
                crate::lean_box(4),
                crate::lean_box(5),
                crate::lean_box(6),
                crate::lean_box(7),
            );
            assert_eq!(crate::lean_unbox(r), 28);
        }
    }

    #[test]
    fn apply_8_direct() {
        unsafe {
            let f = lean_alloc_closure(test_fn_8 as *const (), 8, 0);
            let r = lean_apply_8(
                f,
                crate::lean_box(1),
                crate::lean_box(2),
                crate::lean_box(3),
                crate::lean_box(4),
                crate::lean_box(5),
                crate::lean_box(6),
                crate::lean_box(7),
                crate::lean_box(8),
            );
            assert_eq!(crate::lean_unbox(r), 36);
        }
    }

    /// Over-saturation: apply 2 args to an arity-1 closure.
    /// First arg saturates, producing a result closure; second arg is applied to that.
    #[test]
    fn over_saturate_apply_2_on_arity_1() {
        unsafe {
            // test_fn_1 is identity, arity 1.
            // We'll make it return a closure that adds.
            // Simpler: make a fn that returns a closure.
            // Let's use: apply_2(arity-1-returns-closure, a, b)
            //   -> step 1: call fn(a) -> new closure
            //   -> step 2: apply(new_closure, b)
            unsafe fn make_adder(x: *mut LeanObject) -> *mut LeanObject {
                // returns a closure that adds x to its argument
                let f = lean_alloc_closure(test_fn_2 as *const (), 2, 1);
                lean_closure_set(f, 0, x);
                f
            }
            let f = lean_alloc_closure(make_adder as *const (), 1, 0);
            let r = lean_apply_2(f, crate::lean_box(100), crate::lean_box(23));
            assert_eq!(crate::lean_unbox(r), 123);
        }
    }

    /// Over-saturation via apply_3 on an arity-2 closure.
    #[test]
    fn over_saturate_apply_3_on_arity_2() {
        unsafe {
            // fn(a,b) -> closure that adds a+b to its arg
            unsafe fn make_adder2(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
                let sum = crate::lean_unbox(a) + crate::lean_unbox(b);
                let f = lean_alloc_closure(test_fn_2 as *const (), 2, 1);
                lean_closure_set(f, 0, crate::lean_box(sum));
                f
            }
            let f = lean_alloc_closure(make_adder2 as *const (), 2, 0);
            let r = lean_apply_3(
                f,
                crate::lean_box(10),
                crate::lean_box(20),
                crate::lean_box(5),
            );
            // make_adder2(10,20) = closure(30 + _), then apply(_, 5) = 35
            assert_eq!(crate::lean_unbox(r), 35);
        }
    }

    /// Deep partial chain: arity-8 applied one arg at a time.
    #[test]
    fn deep_partial_chain_arity_8() {
        unsafe {
            let f = lean_alloc_closure(test_fn_8 as *const (), 8, 0);
            let p1 = lean_apply_1(f, crate::lean_box(1));
            let p2 = lean_apply_1(p1, crate::lean_box(2));
            let p3 = lean_apply_1(p2, crate::lean_box(3));
            let p4 = lean_apply_1(p3, crate::lean_box(4));
            let p5 = lean_apply_1(p4, crate::lean_box(5));
            let p6 = lean_apply_1(p5, crate::lean_box(6));
            let p7 = lean_apply_1(p6, crate::lean_box(7));
            let r = lean_apply_1(p7, crate::lean_box(8));
            assert_eq!(crate::lean_unbox(r), 36);
        }
    }

    /// apply_m for partial application (fewer args than arity).
    #[test]
    fn apply_m_partial() {
        unsafe {
            let f = lean_alloc_closure(test_fn_5 as *const (), 5, 0);
            let args: [*mut LeanObject; 3] = [
                crate::lean_box(10),
                crate::lean_box(20),
                crate::lean_box(30),
            ];
            let partial = lean_apply_m(f, 3, args.as_ptr());
            // Should be a closure with 3 fixed args, arity 5
            assert!(!crate::lean_is_scalar(partial));
            let r = lean_apply_2(partial, crate::lean_box(40), crate::lean_box(50));
            assert_eq!(crate::lean_unbox(r), 150);
        }
    }

    /// Mixed: pre-fixed args + apply_m for exact saturation.
    #[test]
    fn apply_m_with_prefixed() {
        unsafe {
            let f = lean_alloc_closure(test_fn_4 as *const (), 4, 2);
            lean_closure_set(f, 0, crate::lean_box(100));
            lean_closure_set(f, 1, crate::lean_box(200));
            let args: [*mut LeanObject; 2] = [crate::lean_box(30), crate::lean_box(40)];
            let r = lean_apply_m(f, 2, args.as_ptr());
            assert_eq!(crate::lean_unbox(r), 370);
        }
    }

    /// apply_2 in chunks on arity-8.
    #[test]
    fn chunked_apply_2_on_arity_8() {
        unsafe {
            let f = lean_alloc_closure(test_fn_8 as *const (), 8, 0);
            let p = lean_apply_2(f, crate::lean_box(1), crate::lean_box(2));
            let p = lean_apply_2(p, crate::lean_box(3), crate::lean_box(4));
            let p = lean_apply_2(p, crate::lean_box(5), crate::lean_box(6));
            let r = lean_apply_2(p, crate::lean_box(7), crate::lean_box(8));
            assert_eq!(crate::lean_unbox(r), 36);
        }
    }
}
