//! Panic and error handling

use crate::object::LeanObject;
use crate::string::lean_string_to_str;

pub unsafe fn lean_panic_fn(msg: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    let msg_str = lean_string_to_str(msg);
    panic!("Lean panic: {}", msg_str);
}

pub unsafe fn lean_internal_panic(msg: *const i8) -> ! {
    let c_str = std::ffi::CStr::from_ptr(msg);
    panic!("Lean internal panic: {}", c_str.to_string_lossy());
}

pub unsafe fn lean_internal_panic_unreachable() -> ! {
    panic!("Lean: unreachable code");
}

pub unsafe fn lean_internal_panic_out_of_memory() -> ! {
    panic!("Lean: out of memory");
}
