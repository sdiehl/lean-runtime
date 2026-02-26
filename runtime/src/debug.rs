//! Debug instrumentation for the Lean runtime.
//!
//! All functions here are gated behind `cfg(debug_assertions)` at call sites,
//! so they impose zero cost in release builds.

use crate::object::*;
use crate::r#box::lean_is_scalar;

/// Sentinel written into `rc` when an object is freed. Detects use-after-free.
pub const LEAN_FREED_SENTINEL: i32 = i32::MIN; // 0x80000000

/// Minimum plausible heap address. Pointers below this are invalid.
const MIN_HEAP_ADDR: usize = 0x10000;

/// Validate that a pointer looks like a valid heap object.
/// Call this from `lean_inc_ref`, `lean_dec_ref`, `lean_ctor_get`, etc.
#[cold]
#[inline(never)]
pub unsafe fn lean_debug_validate_ptr(o: *mut LeanObject, caller: &str) {
    let addr = o as usize;
    if addr == 0 {
        panic!("[lean-debug] {caller}: null pointer");
    }
    if lean_is_scalar(o) {
        panic!(
            "[lean-debug] {caller}: scalar {:#x} (unboxed={}) passed to non-scalar function",
            addr,
            addr >> 1
        );
    }
    if addr < MIN_HEAP_ADDR {
        panic!(
            "[lean-debug] {caller}: suspiciously low address {:#x}",
            addr
        );
    }
    if !addr.is_multiple_of(4) {
        panic!(
            "[lean-debug] {caller}: misaligned pointer {:#x} (not 4-byte aligned)",
            addr
        );
    }
}

/// Check whether an object has been freed (sentinel in rc field).
#[cold]
#[inline(never)]
pub unsafe fn lean_debug_check_freed(o: *mut LeanObject, caller: &str) {
    if (*o).rc == LEAN_FREED_SENTINEL {
        panic!(
            "[lean-debug] {caller}: use-after-free on {:#x} (tag was {}, other was {})",
            o as usize,
            (*o).tag,
            (*o).other
        );
    }
}

/// Full validation: pointer validity + freed check.
#[inline(always)]
pub unsafe fn lean_debug_assert_valid(o: *mut LeanObject, caller: &str) {
    lean_debug_validate_ptr(o, caller);
    lean_debug_check_freed(o, caller);
}

/// Poison an object's rc field after freeing. Called from `lean_free_object`.
#[inline(always)]
pub unsafe fn lean_debug_poison(o: *mut LeanObject) {
    (*o).rc = LEAN_FREED_SENTINEL;
    // Also poison the tag to make freed objects more recognizable
    (*o).tag = 0xFF;
}

/// Check that a ctor field index is within bounds.
#[cold]
#[inline(never)]
pub unsafe fn lean_debug_check_ctor_bounds(o: *mut LeanObject, i: u32, caller: &str) {
    let num_objs = (*o).other as u32;
    if i >= num_objs {
        panic!(
            "[lean-debug] {caller}: field index {} >= num_objs {} on {:#x} (tag={})",
            i,
            num_objs,
            o as usize,
            (*o).tag
        );
    }
}

/// Check whether an object is persistent (rc == 0).
#[inline(always)]
pub unsafe fn lean_debug_is_persistent(o: *mut LeanObject) -> bool {
    !lean_is_scalar(o) && (*o).rc == 0
}

/// Dump an object's metadata to stderr. Useful from lldb:
///   `expr lean_runtime::debug::lean_debug_dump(ptr)`
#[no_mangle]
pub unsafe extern "C" fn lean_debug_dump(o: *mut LeanObject) {
    let addr = o as usize;
    if o.is_null() {
        eprintln!("[lean-debug] dump: null pointer");
        return;
    }
    if lean_is_scalar(o) {
        eprintln!(
            "[lean-debug] dump: scalar {:#x} (value={})",
            addr,
            addr >> 1
        );
        return;
    }
    if addr < MIN_HEAP_ADDR || !addr.is_multiple_of(4) {
        eprintln!("[lean-debug] dump: invalid pointer {:#x}", addr);
        return;
    }
    let rc = (*o).rc;
    let tag = (*o).tag;
    let other = (*o).other;
    let cs_sz = (*o).cs_sz;

    if rc == LEAN_FREED_SENTINEL {
        eprintln!(
            "[lean-debug] dump {:#x}: FREED OBJECT (tag was {}, other was {})",
            addr, tag, other
        );
        return;
    }

    let kind = match tag {
        0..=244 => "ctor",
        245 => "closure",
        246 => "array",
        247 => "thunk",
        248 => "sarray",
        249 => "string",
        250 => "mpz",
        251 => "bigint",
        253 => "ref",
        254 => "external",
        _ => "unknown",
    };
    eprintln!(
        "[lean-debug] dump {:#x}: {kind} rc={rc} tag={tag} other={other} cs_sz={cs_sz}",
        addr
    );

    match tag {
        0..=244 => {
            let num_objs = (*o).num_objs();
            let objs = (*o).obj_fields_ptr();
            for i in 0..num_objs {
                let child = *objs.add(i as usize);
                let child_addr = child as usize;
                if lean_is_scalar(child) {
                    eprintln!(
                        "  field[{i}]: scalar {child_addr:#x} (value={})",
                        child_addr >> 1
                    );
                } else if child.is_null() {
                    eprintln!("  field[{i}]: null");
                } else if child_addr < MIN_HEAP_ADDR {
                    eprintln!("  field[{i}]: INVALID {child_addr:#x}");
                } else {
                    eprintln!(
                        "  field[{i}]: {child_addr:#x} rc={} tag={}",
                        (*child).rc,
                        (*child).tag
                    );
                }
            }
        }
        245 => {
            let closure = o as *mut crate::closure::LeanClosure;
            let arity = (*closure).arity;
            let num_fixed = (*closure).num_fixed;
            eprintln!("  arity={arity} num_fixed={num_fixed}");
        }
        249 => {
            let s = crate::lean_string_to_str(o);
            let display: String = s.chars().take(80).collect();
            eprintln!("  content=\"{display}\"");
        }
        246 => {
            let arr = o as *mut crate::array::LeanArray;
            eprintln!("  size={} capacity={}", (*arr).size, (*arr).capacity);
        }
        248 => {
            let sarr = o as *mut crate::sarray::LeanSArray;
            eprintln!(
                "  elem_size={} size={} capacity={}",
                other,
                (*sarr).size,
                (*sarr).capacity
            );
        }
        247 => {
            let thunk = o as *mut crate::thunk::LeanThunk;
            let value = (*thunk).value;
            let closure = (*thunk).closure;
            if value.is_null() {
                eprintln!("  value=null (unevaluated)");
            } else if lean_is_scalar(value) {
                eprintln!(
                    "  value=scalar {:#x} (value={})",
                    value as usize,
                    (value as usize) >> 1
                );
            } else {
                eprintln!(
                    "  value={:#x} rc={} tag={}",
                    value as usize,
                    (*value).rc,
                    (*value).tag
                );
            }
            if closure.is_null() {
                eprintln!("  closure=null");
            } else if lean_is_scalar(closure) {
                eprintln!("  closure=scalar {:#x}", closure as usize);
            } else {
                eprintln!(
                    "  closure={:#x} rc={} tag={}",
                    closure as usize,
                    (*closure).rc,
                    (*closure).tag
                );
            }
        }
        250 => {
            let val = crate::bignat::lean_bignat_value(o);
            let display: String = val.to_string().chars().take(80).collect();
            eprintln!("  value={display}");
        }
        253 => {
            let r = o as *mut crate::stref::LeanRef;
            let value = (*r).value;
            if value.is_null() {
                eprintln!("  value=null");
            } else if lean_is_scalar(value) {
                eprintln!(
                    "  value=scalar {:#x} (value={})",
                    value as usize,
                    (value as usize) >> 1
                );
            } else {
                eprintln!(
                    "  value={:#x} rc={} tag={}",
                    value as usize,
                    (*value).rc,
                    (*value).tag
                );
            }
        }
        254 => {
            let ext = o as *mut crate::external::LeanExternal;
            let data = (*ext).data;
            let cls = (*ext).class;
            eprintln!("  data={:#x} class={:#x}", data as usize, cls as usize);
            if !cls.is_null() {
                eprintln!(
                    "  finalize={} foreach={}",
                    (*cls).finalize.is_some(),
                    (*cls).foreach.is_some()
                );
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_ptr_catches_null() {
        unsafe {
            let result = std::panic::catch_unwind(|| {
                lean_debug_validate_ptr(std::ptr::null_mut(), "test");
            });
            assert!(result.is_err());
            let msg = result.unwrap_err().downcast::<String>().unwrap();
            assert!(msg.contains("null pointer"), "got: {msg}");
        }
    }

    #[test]
    fn validate_ptr_catches_scalar() {
        unsafe {
            let scalar = crate::lean_box(42);
            let result = std::panic::catch_unwind(|| {
                lean_debug_validate_ptr(scalar, "test");
            });
            assert!(result.is_err());
            let msg = result.unwrap_err().downcast::<String>().unwrap();
            assert!(msg.contains("scalar"), "got: {msg}");
        }
    }

    #[test]
    fn validate_ptr_accepts_valid_heap_object() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            lean_debug_validate_ptr(obj, "test");
            lean_debug_check_freed(obj, "test");
            lean_debug_assert_valid(obj, "test");
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn check_freed_catches_poisoned_object() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            let _ptr = obj; // save pointer
            crate::lean_dec(obj); // frees and poisons
                                  // The memory is freed, but in debug mode the sentinel was written.
                                  // We can't safely read freed memory in all cases, so just test
                                  // the poison function directly.
            let fake = std::alloc::alloc(std::alloc::Layout::from_size_align(16, 8).unwrap())
                as *mut LeanObject;
            lean_debug_poison(fake);
            assert_eq!((*fake).rc, LEAN_FREED_SENTINEL);
            assert_eq!((*fake).tag, 0xFF);
            let result = std::panic::catch_unwind(|| {
                lean_debug_check_freed(fake, "test");
            });
            assert!(result.is_err());
            let msg = result.unwrap_err().downcast::<String>().unwrap();
            assert!(msg.contains("use-after-free"), "got: {msg}");
            std::alloc::dealloc(
                fake as *mut u8,
                std::alloc::Layout::from_size_align(16, 8).unwrap(),
            );
        }
    }

    #[test]
    fn ctor_bounds_catches_out_of_range() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 2, 0);
            crate::lean_ctor_set(obj, 0, crate::lean_box(0));
            crate::lean_ctor_set(obj, 1, crate::lean_box(0));
            // Valid access
            lean_debug_check_ctor_bounds(obj, 0, "test");
            lean_debug_check_ctor_bounds(obj, 1, "test");
            // Out of bounds
            let result = std::panic::catch_unwind(|| {
                lean_debug_check_ctor_bounds(obj, 2, "test");
            });
            assert!(result.is_err());
            let msg = result.unwrap_err().downcast::<String>().unwrap();
            assert!(msg.contains("field index 2 >= num_objs 2"), "got: {msg}");
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn persistent_object_check() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            assert!(!lean_debug_is_persistent(obj));
            crate::lean_mark_persistent(obj);
            assert!(lean_debug_is_persistent(obj));
            // Scalars are not persistent
            assert!(!lean_debug_is_persistent(crate::lean_box(42)));
            crate::lean_free_object(obj, crate::lean_ctor_object_size(0, 0));
        }
    }

    #[test]
    fn dump_ctor() {
        unsafe {
            let obj = crate::lean_alloc_ctor(3, 2, 0);
            crate::lean_ctor_set(obj, 0, crate::lean_box(10));
            crate::lean_ctor_set(obj, 1, crate::lean_box(20));
            lean_debug_dump(obj);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn dump_string() {
        unsafe {
            let s = crate::lean_mk_string("hello world");
            lean_debug_dump(s);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn dump_scalar() {
        unsafe {
            lean_debug_dump(crate::lean_box(42));
        }
    }

    #[test]
    fn dump_null() {
        unsafe {
            lean_debug_dump(std::ptr::null_mut());
        }
    }
}
