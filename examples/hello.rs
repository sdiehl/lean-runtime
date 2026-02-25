//! Hand-written example of what EmitRust.lean should generate
//! for: def main := IO.println "Hello"

use lean_runtime::*;

unsafe fn _lean_main(_rw: *mut LeanObject) -> *mut LeanObject {
    let s = lean_mk_string("Hello, World!");
    lean_io_prim_println(s, lean_box(0))
}

fn main() {
    unsafe {
        let rw = lean_box(0); // RealWorld token
        let result = _lean_main(rw);
        if !lean_io_result_is_ok(result) {
            std::process::exit(1);
        }
        lean_dec(result);
    }
}
