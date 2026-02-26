//! Simple test: hand-written bytecode for "Hello, World!"

use lean4_vm::*;

fn main() {
    // Build a simple module with one function that prints "Hello, World!"
    let mut module = Module::new();

    // Add string to pool
    module.strings.push("Hello, World!".to_string());

    // Add extern for println
    module.externs.push(ExternDecl {
        name: "lean_io_prim_println".to_string(),
        arity: 2,
    });

    // Build the main function bytecode
    let mut builder = BytecodeBuilder::new();

    // Push string literal
    builder.emit(Opcode::StringLit);
    builder.emit_u32(0); // string index 0

    // Push unit (for world arg)
    builder.emit(Opcode::UnitLit);

    // Call extern println with 2 args
    builder.emit(Opcode::CallExtern);
    builder.emit_u32(0); // extern index 0
    builder.emit_u8(2); // 2 args

    // Return result
    builder.emit(Opcode::Ret);

    let code = builder.finish();

    module.functions.push(Function {
        name: "main".to_string(),
        arity: 0,
        num_locals: 0,
        code,
    });

    module.entry = 0;

    // Run it
    let mut vm = VM::new();
    vm.load_module(module);

    match vm.run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
