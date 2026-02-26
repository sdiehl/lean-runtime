//! Format/pretty-printing operations
//!
//! Note: Std.Format functions (l_Std_Format_*) come from Std bytecode.
//! Load them with -L flag if needed.

use super::ExternFn;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Format operations are implemented in Std bytecode
        // No lean_* format primitives in runtime
    ]
}
