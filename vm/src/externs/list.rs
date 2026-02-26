//! List operations
//!
//! Note: All List.* functions (l_List_*) are implemented in Init bytecode.
//! Load them with -L init flag. There are no lean_* list primitives in the runtime.
//! Lists are represented as constructor chains (nil=tag 0, cons=tag 1).

use super::ExternFn;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // List operations come from Init bytecode, not VM primitives
        // Use: cargo run --release -- -L init program.leanbc
    ]
}
