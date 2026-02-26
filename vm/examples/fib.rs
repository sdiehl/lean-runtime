//! Fibonacci example - tests function calls, recursion, and nat operations

use lean4_vm::*;

fn main() {
    let mut module = Module::new();

    // Strings
    module.strings.push("fib 10 = ".to_string());
    module.strings.push("fib 20 = ".to_string());

    // Externs
    module.externs.push(ExternDecl {
        name: "lean_io_prim_println".to_string(),
        arity: 2,
    });
    module.externs.push(ExternDecl {
        name: "lean_nat_repr".to_string(), // Would need this for real, but we'll work around
        arity: 1,
    });

    // Function 0: fib(n)
    // if n == 0 then 0
    // else if n == 1 then 1
    // else fib(n-1) + fib(n-2)
    let fib_code = {
        let mut b = BytecodeBuilder::new();

        let else1 = b.new_label();
        let else2 = b.new_label();
        let end = b.new_label();

        // Load n (local 0)
        b.emit(Opcode::LoadLocal);
        b.emit_u16(0);

        // Push 0
        b.emit(Opcode::NatLit);
        b.emit_u64(0);

        // n == 0?
        b.emit(Opcode::NatEq);
        b.emit_jump(Opcode::JumpIfNot, else1);

        // return 0
        b.emit(Opcode::NatLit);
        b.emit_u64(0);
        b.emit_jump(Opcode::Jump, end);

        // else1: check if n == 1
        b.mark_label(else1);
        b.emit(Opcode::LoadLocal);
        b.emit_u16(0);
        b.emit(Opcode::NatLit);
        b.emit_u64(1);
        b.emit(Opcode::NatEq);
        b.emit_jump(Opcode::JumpIfNot, else2);

        // return 1
        b.emit(Opcode::NatLit);
        b.emit_u64(1);
        b.emit_jump(Opcode::Jump, end);

        // else2: fib(n-1) + fib(n-2)
        b.mark_label(else2);

        // fib(n-1)
        b.emit(Opcode::LoadLocal);
        b.emit_u16(0);
        b.emit(Opcode::NatLit);
        b.emit_u64(1);
        b.emit(Opcode::NatSub);
        b.emit(Opcode::Call);
        b.emit_u32(0); // fib is function 0
        b.emit_u8(1); // 1 arg

        // fib(n-2)
        b.emit(Opcode::LoadLocal);
        b.emit_u16(0);
        b.emit(Opcode::NatLit);
        b.emit_u64(2);
        b.emit(Opcode::NatSub);
        b.emit(Opcode::Call);
        b.emit_u32(0); // fib is function 0
        b.emit_u8(1); // 1 arg

        // add them
        b.emit(Opcode::NatAdd);

        // end:
        b.mark_label(end);
        b.emit(Opcode::Ret);

        b.finish()
    };

    module.functions.push(Function {
        name: "fib".to_string(),
        arity: 1,
        num_locals: 1,
        code: fib_code,
    });

    // Function 1: main
    // Compute fib(10) and fib(20), print results
    let main_code = {
        let mut b = BytecodeBuilder::new();

        // Compute fib(10)
        b.emit(Opcode::NatLit);
        b.emit_u64(10);
        b.emit(Opcode::Call);
        b.emit_u32(0); // fib
        b.emit_u8(1);

        // Store in local 0
        b.emit(Opcode::StoreLocal);
        b.emit_u16(0);

        // Compute fib(20)
        b.emit(Opcode::NatLit);
        b.emit_u64(20);
        b.emit(Opcode::Call);
        b.emit_u32(0); // fib
        b.emit_u8(1);

        // Store in local 1
        b.emit(Opcode::StoreLocal);
        b.emit_u16(1);

        // For now, just return fib(10) since we can't easily print nats
        // We'll print the result as a scalar value
        b.emit(Opcode::LoadLocal);
        b.emit_u16(0);

        b.emit(Opcode::Ret);

        b.finish()
    };

    module.functions.push(Function {
        name: "main".to_string(),
        arity: 0,
        num_locals: 2,
        code: main_code,
    });

    module.entry = 1; // main is function 1

    // Run it
    let mut vm = VM::new();
    vm.load_module(module.clone());

    match vm.run() {
        Ok(result) => {
            if result.is_scalar() {
                println!("fib(10) = {}", result.unbox());
            } else {
                println!("Result: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Also test fib(20)
    let mut vm2 = VM::new();
    let mut module2 = module.clone();
    module2.functions[1].code = {
        let mut b = BytecodeBuilder::new();
        b.emit(Opcode::NatLit);
        b.emit_u64(20);
        b.emit(Opcode::Call);
        b.emit_u32(0);
        b.emit_u8(1);
        b.emit(Opcode::Ret);
        b.finish()
    };
    vm2.load_module(module2);

    match vm2.run() {
        Ok(result) => {
            if result.is_scalar() {
                println!("fib(20) = {}", result.unbox());
            } else {
                println!("Result: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
