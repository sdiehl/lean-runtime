//! Lean 4 VM - Bytecode interpreter CLI

use clap::{Parser, Subcommand};
use lean4_vm::{Module, VM};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "lean4-vm")]
#[command(about = "Bytecode interpreter for Lean 4", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Include bytecode module (loaded before main file, can be repeated)
    #[arg(short = 'I', value_name = "FILE")]
    includes: Vec<PathBuf>,

    /// Load all .leanbc files from directory (can be repeated)
    #[arg(short = 'L', value_name = "DIR")]
    lib_dirs: Vec<PathBuf>,

    /// Bytecode file to run
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Show execution statistics (time, memory)
    #[arg(long)]
    stats: bool,

    /// Show wall-clock execution time
    #[arg(long)]
    time: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Disassemble bytecode file
    Disasm {
        /// Bytecode file to disassemble
        file: PathBuf,
    },
    /// Create standalone executable from bytecode
    Compile {
        /// Bytecode file to compile
        file: PathBuf,
        /// Output executable path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Link multiple bytecode files into one
    Link {
        /// Output bytecode file
        #[arg(short, long)]
        output: PathBuf,
        /// Input bytecode files (in dependency order, main module last)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

fn disassemble(module: &Module) {
    println!("Module");
    println!("  Strings: {}", module.strings.len());
    println!("  Functions: {}", module.functions.len());
    println!("  Externs: {}", module.externs.len());
    println!("  Entry: {}", module.entry);
    println!();

    println!("Strings");
    for (i, s) in module.strings.iter().enumerate() {
        println!("  [{}] {:?}", i, s);
    }
    println!();

    println!("Externs");
    for (i, ext) in module.externs.iter().enumerate() {
        println!("  [{}] {} (arity {})", i, ext.name, ext.arity);
    }
    println!();

    println!("Functions");
    for (i, func) in module.functions.iter().enumerate() {
        println!(
            "  [{}] {} (arity {}, locals {})",
            i, func.name, func.arity, func.num_locals
        );
        disasm_function(func);
        println!();
    }
}

fn disasm_function(func: &lean4_vm::Function) {
    use lean4_vm::Opcode;

    let code = &func.code;
    let mut pc = 0;

    while pc < code.len() {
        let start_pc = pc;
        let op_byte = code[pc];
        pc += 1;

        let op = match Opcode::from_u8(op_byte) {
            Some(op) => op,
            None => {
                println!("    {:04}: <invalid 0x{:02x}>", start_pc, op_byte);
                continue;
            }
        };

        let operands = match op {
            Opcode::LoadLocal | Opcode::StoreLocal => {
                let idx = read_u16(code, &mut pc);
                format!("{}", idx)
            }
            Opcode::AllocCtor => {
                let tag = code[pc];
                pc += 1;
                let fields = code[pc];
                pc += 1;
                format!("tag={}, fields={}", tag, fields)
            }
            Opcode::CtorGet | Opcode::CtorSet | Opcode::CtorSetTag => {
                let idx = code[pc];
                pc += 1;
                format!("{}", idx)
            }
            Opcode::Call | Opcode::TailCall => {
                let func_id = read_u32(code, &mut pc);
                let num_args = code[pc];
                pc += 1;
                format!("func={}, args={}", func_id, num_args)
            }
            Opcode::CallExtern => {
                let ext_id = read_u32(code, &mut pc);
                let num_args = code[pc];
                pc += 1;
                format!("extern={}, args={}", ext_id, num_args)
            }
            Opcode::Jump | Opcode::JumpIf | Opcode::JumpIfNot => {
                let offset = read_i32(code, &mut pc);
                let target = (pc as i32 + offset) as usize;
                format!("{} (-> {})", offset, target)
            }
            Opcode::Switch => {
                let num_cases = read_u16(code, &mut pc);
                let mut s = format!("cases={} [", num_cases);
                for i in 0..num_cases {
                    let offset = read_i32(code, &mut pc);
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("{}", offset));
                }
                let default = read_i32(code, &mut pc);
                s.push_str(&format!("] default={}", default));
                s
            }
            Opcode::NatLit => {
                let val = read_u64(code, &mut pc);
                format!("{}", val)
            }
            Opcode::StringLit => {
                let str_id = read_u32(code, &mut pc);
                format!("str[{}]", str_id)
            }
            Opcode::BoolLit => {
                let val = code[pc];
                pc += 1;
                format!("{}", val != 0)
            }
            Opcode::Apply | Opcode::TailApply => {
                let num_args = code[pc];
                pc += 1;
                format!("args={}", num_args)
            }
            Opcode::PartialApp => {
                let func_id = read_u32(code, &mut pc);
                let arity = code[pc];
                pc += 1;
                let num_args = code[pc];
                pc += 1;
                format!("func={}, arity={}, args={}", func_id, arity, num_args)
            }
            Opcode::AllocClosure => {
                let func_id = read_u32(code, &mut pc);
                let arity = code[pc];
                pc += 1;
                let num_captured = code[pc];
                pc += 1;
                format!(
                    "func={}, arity={}, captured={}",
                    func_id, arity, num_captured
                )
            }
            Opcode::ClosureGet | Opcode::ClosureSet => {
                let idx = code[pc];
                pc += 1;
                format!("{}", idx)
            }
            Opcode::LoadConst => {
                let const_id = read_u32(code, &mut pc);
                format!("const={}", const_id)
            }
            Opcode::ScalarProj => {
                let num_objs = code[pc];
                pc += 1;
                let offset = read_u16(code, &mut pc);
                format!("num_objs={}, offset={}", num_objs, offset)
            }
            Opcode::ScalarSet => {
                let num_objs = code[pc];
                pc += 1;
                let offset = read_u16(code, &mut pc);
                format!("num_objs={}, offset={}", num_objs, offset)
            }
            _ => String::new(),
        };

        if operands.is_empty() {
            println!("    {:04}: {:?}", start_pc, op);
        } else {
            println!("    {:04}: {:?} {}", start_pc, op, operands);
        }
    }
}

fn read_u16(code: &[u8], pc: &mut usize) -> u16 {
    let val = u16::from_le_bytes([code[*pc], code[*pc + 1]]);
    *pc += 2;
    val
}

fn read_u32(code: &[u8], pc: &mut usize) -> u32 {
    let val = u32::from_le_bytes([code[*pc], code[*pc + 1], code[*pc + 2], code[*pc + 3]]);
    *pc += 4;
    val
}

fn read_i32(code: &[u8], pc: &mut usize) -> i32 {
    read_u32(code, pc) as i32
}

fn read_u64(code: &[u8], pc: &mut usize) -> u64 {
    let lo = read_u32(code, pc) as u64;
    let hi = read_u32(code, pc) as u64;
    lo | (hi << 32)
}

fn load_module(path: &PathBuf) -> Module {
    let file = File::open(path).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {}", path.display(), e);
        process::exit(1);
    });
    Module::deserialize(&mut BufReader::new(file)).unwrap_or_else(|e| {
        eprintln!("Error loading {}: {}", path.display(), e);
        process::exit(1);
    })
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Disasm { file }) => {
            let module = load_module(&file);
            disassemble(&module);
        }
        Some(Commands::Compile { file, output }) => {
            let bytecode = std::fs::read(&file).unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", file.display(), e);
                process::exit(1);
            });

            let exe_path = std::env::current_exe().unwrap();
            let exe_dir = exe_path.parent().unwrap();

            let lean4_vm_path = exe_dir
                .join("../../lean4-vm")
                .canonicalize()
                .unwrap_or_else(|_| {
                    std::env::var("LEAN4_VM_PATH")
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| {
                            eprintln!("Error: cannot find lean4-vm. Set LEAN4_VM_PATH");
                            process::exit(1);
                        })
                });

            let lean_runtime_path = exe_dir
                .join("../../lean-runtime")
                .canonicalize()
                .unwrap_or_else(|_| {
                    std::env::var("LEAN_RUNTIME_PATH")
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| {
                            eprintln!("Error: cannot find lean-runtime. Set LEAN_RUNTIME_PATH");
                            process::exit(1);
                        })
                });

            lean4_vm::compile_executable(
                &bytecode,
                output.to_str().unwrap(),
                lean4_vm_path.to_str().unwrap(),
                lean_runtime_path.to_str().unwrap(),
            )
            .unwrap_or_else(|e| {
                eprintln!("Error compiling: {}", e);
                process::exit(1);
            });

            println!("Created executable: {}", output.display());
        }
        Some(Commands::Link { output, files }) => {
            // Load all input modules
            let modules: Vec<Module> = files.iter().map(load_module).collect();

            let total_funcs: usize = modules.iter().map(|m| m.functions.len()).sum();
            let total_strings: usize = modules.iter().map(|m| m.strings.len()).sum();
            let total_externs: usize = modules.iter().map(|m| m.externs.len()).sum();

            eprintln!(
                "Linking {} modules ({} functions, {} strings, {} externs)...",
                modules.len(),
                total_funcs,
                total_strings,
                total_externs
            );

            // Link them
            let linked = lean4_vm::link(modules).unwrap_or_else(|e| {
                eprintln!("Error linking: {}", e);
                process::exit(1);
            });

            eprintln!(
                "Linked: {} functions, {} strings, {} externs",
                linked.functions.len(),
                linked.strings.len(),
                linked.externs.len()
            );

            // Write output
            let mut out_file = std::fs::File::create(&output).unwrap_or_else(|e| {
                eprintln!("Error creating {}: {}", output.display(), e);
                process::exit(1);
            });
            linked.serialize(&mut out_file).unwrap_or_else(|e| {
                eprintln!("Error writing {}: {}", output.display(), e);
                process::exit(1);
            });

            println!("Created linked bytecode: {}", output.display());
        }
        None => {
            let main_file = cli.file.unwrap_or_else(|| {
                eprintln!("Error: no bytecode file specified");
                eprintln!("Usage: lean4-vm [OPTIONS] <FILE>");
                eprintln!("       lean4-vm disasm <FILE>");
                eprintln!("       lean4-vm compile <FILE> -o <OUTPUT>");
                eprintln!("       lean4-vm link -o <OUTPUT> <FILES>...");
                process::exit(1);
            });

            let load_start = Instant::now();
            let mut vm = VM::new();
            let mut total_functions = 0;
            let mut total_modules = 0;

            // Auto-detect stdlib relative to executable
            // Priority:
            // 1. Linked stdlib (vm/bc/stdlib/Init.leanbc) - fastest, production use
            // 2. Individual init modules (vm/init/*.leanbc) - fallback for development
            // 3. LEAN_VM env var
            let mut lib_dirs = cli.lib_dirs.clone();
            let mut include_files = cli.includes.clone();

            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    // Try linked stdlib first (workspace layout: target/release -> vm/bc/stdlib)
                    let workspace_stdlib = exe_dir.join("../../vm/bc/stdlib/Init.leanbc");
                    let installed_stdlib = exe_dir.join("bc/stdlib/Init.leanbc");

                    if workspace_stdlib.is_file() {
                        if let Ok(canonical) = workspace_stdlib.canonicalize() {
                            include_files.insert(0, canonical);
                        }
                    } else if installed_stdlib.is_file() {
                        include_files.insert(0, installed_stdlib);
                    } else {
                        // Fallback to individual init modules (flat layout)
                        let workspace_init = exe_dir.join("../../vm/init");
                        if workspace_init.is_dir() {
                            if let Ok(canonical) = workspace_init.canonicalize() {
                                lib_dirs.insert(0, canonical);
                            }
                        } else {
                            let installed_init = exe_dir.join("init");
                            if installed_init.is_dir() {
                                lib_dirs.insert(0, installed_init);
                            }
                        }
                    }
                }
            }
            // Also check LEAN_VM env var for init directory
            if let Ok(init_path) = std::env::var("LEAN_VM") {
                let path = PathBuf::from(init_path);
                if path.is_dir() && !lib_dirs.contains(&path) {
                    lib_dirs.insert(0, path);
                } else if path.is_file() && !include_files.contains(&path) {
                    include_files.insert(0, path);
                }
            }

            // Load modules from library directories
            for lib_dir in &lib_dirs {
                if !lib_dir.is_dir() {
                    eprintln!(
                        "Warning: {} is not a directory, skipping",
                        lib_dir.display()
                    );
                    continue;
                }
                let mut bc_files: Vec<_> = std::fs::read_dir(lib_dir)
                    .unwrap_or_else(|e| {
                        eprintln!("Error reading {}: {}", lib_dir.display(), e);
                        process::exit(1);
                    })
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|ext| ext == "leanbc"))
                    .collect();
                bc_files.sort();
                for bc_path in bc_files {
                    let module = load_module(&bc_path);
                    total_functions += module.functions.len();
                    total_modules += 1;
                    #[cfg(feature = "trace")]
                    eprintln!(
                        "Loaded {} ({} functions)",
                        bc_path.display(),
                        module.functions.len()
                    );
                    vm.load_module(module);
                }
            }

            // Load individual include files (including auto-detected stdlib)
            for inc_path in &include_files {
                let module = load_module(inc_path);
                total_functions += module.functions.len();
                total_modules += 1;
                #[cfg(feature = "trace")]
                eprintln!(
                    "Loaded include {} ({} functions)",
                    inc_path.display(),
                    module.functions.len()
                );
                vm.load_module(module);
            }

            let module = load_module(&main_file);
            total_functions += module.functions.len();
            total_modules += 1;
            vm.load_module(module);
            let load_time = load_start.elapsed();

            let exec_start = Instant::now();
            let result = vm.run();
            let exec_time = exec_start.elapsed();

            match result {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Runtime error: {}", e);
                    process::exit(1);
                }
            }

            // Print timing stats if requested
            if cli.time || cli.stats {
                eprintln!("Execution time: {:.3}s", exec_time.as_secs_f64());
            }

            if cli.stats {
                eprintln!("Modules loaded:    {}", total_modules);
                eprintln!("Functions loaded:  {}", total_functions);
                eprintln!("Load time:         {:.3}s", load_time.as_secs_f64());
                eprintln!("Execution time:    {:.3}s", exec_time.as_secs_f64());
                eprintln!(
                    "Total time:        {:.3}s",
                    (load_time + exec_time).as_secs_f64()
                );
            }
        }
    }
}
