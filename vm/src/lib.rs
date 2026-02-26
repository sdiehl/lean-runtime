//! Lean 4 Bytecode Virtual Machine
//!
//! This crate provides a bytecode interpreter for Lean 4 programs.
//! It can be used in two modes:
//!
//! 1. **Standalone**: Run `.leanbc` bytecode files directly
//! 2. **Embedded**: Include bytecode in a Rust executable
//!
//! # Example: Embedding bytecode
//!
//! ```rust,ignore
//! use lean4_vm::{VM, Module};
//!
//! // Bytecode is embedded at compile time
//! const BYTECODE: &[u8] = include_bytes!("program.leanbc");
//!
//! fn main() {
//!     let module = Module::deserialize(&mut &BYTECODE[..]).unwrap();
//!     let mut vm = VM::new();
//!     vm.load_module(module);
//!     let result = vm.run().unwrap();
//! }
//! ```

pub mod bytecode;
pub mod externs;
pub mod linker;
pub mod value;
pub mod vm;

pub use bytecode::{BytecodeBuilder, ExternDecl, Function, Module, Opcode};
pub use linker::{link, FUNC_ID_RESOLVED_BIT};
pub use value::LeanValue;
pub use vm::{VMError, VM};

/// Generate a standalone executable that embeds the given bytecode.
///
/// This creates a small Rust source file that includes the bytecode
/// and links against lean4-vm. The result can be compiled with rustc
/// to produce a native executable.
pub fn generate_embedded_executable(bytecode: &[u8], output_path: &str) -> std::io::Result<String> {
    // Write bytecode to a temporary file
    let bc_path = format!("{}.leanbc", output_path);
    std::fs::write(&bc_path, bytecode)?;

    // Generate Rust source
    let rust_src = format!(
        r#"
//! Auto-generated Lean executable
//!
//! This file embeds Lean bytecode and runs it via lean4-vm.

use lean4_vm::{{VM, Module}};

const BYTECODE: &[u8] = include_bytes!("{}");

fn main() {{
    let module = match Module::deserialize(&mut &BYTECODE[..]) {{
        Ok(m) => m,
        Err(e) => {{
            eprintln!("Error loading bytecode: {{}}", e);
            std::process::exit(1);
        }}
    }};

    let mut vm = VM::new();
    vm.load_module(module);

    match vm.run() {{
        Ok(_) => {{}},
        Err(e) => {{
            eprintln!("Runtime error: {{}}", e);
            std::process::exit(1);
        }}
    }}
}}
"#,
        bc_path
    );

    Ok(rust_src)
}

/// Compile bytecode to a standalone executable using rustc.
///
/// This is the main entry point for creating executables from bytecode.
/// It:
/// 1. Writes the bytecode to a .leanbc file
/// 2. Generates a Rust wrapper that embeds the bytecode
/// 3. Invokes rustc to compile the executable
pub fn compile_executable(
    bytecode: &[u8],
    output_path: &str,
    lean4_vm_path: &str,
    lean_runtime_path: &str,
) -> std::io::Result<()> {
    use std::process::Command;

    // Write bytecode
    let bc_path = format!("{}.leanbc", output_path);
    std::fs::write(&bc_path, bytecode)?;

    // Generate and write Rust source
    let rs_path = format!("{}.rs", output_path);
    let rust_src = format!(
        r#"
use lean4_vm::{{VM, Module}};

const BYTECODE: &[u8] = include_bytes!("{}.leanbc");

fn main() {{
    let module = match Module::deserialize(&mut &BYTECODE[..]) {{
        Ok(m) => m,
        Err(e) => {{
            eprintln!("Error loading bytecode: {{}}", e);
            std::process::exit(1);
        }}
    }};

    let mut vm = VM::new();
    vm.load_module(module);

    match vm.run() {{
        Ok(_) => {{}},
        Err(e) => {{
            eprintln!("Runtime error: {{}}", e);
            std::process::exit(1);
        }}
    }}
}}
"#,
        output_path
    );

    std::fs::write(&rs_path, rust_src)?;

    // Build lean4-vm as rlib if not already built
    let vm_lib = format!("{}/target/release/liblean4_vm.rlib", lean4_vm_path);
    if !std::path::Path::new(&vm_lib).exists() {
        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(lean4_vm_path)
            .status()?;
        if !status.success() {
            return Err(std::io::Error::other("Failed to build lean4-vm"));
        }
    }

    // Compile with rustc
    let runtime_lib = format!("{}/target/release/liblean_runtime.rlib", lean_runtime_path);
    let status = Command::new("rustc")
        .args([
            &rs_path,
            "--edition",
            "2021",
            "--extern",
            &format!("lean4_vm={}", vm_lib),
            "--extern",
            &format!("lean_runtime={}", runtime_lib),
            "-L",
            &format!("{}/target/release/deps", lean4_vm_path),
            "-L",
            &format!("{}/target/release/deps", lean_runtime_path),
            "-o",
            output_path,
        ])
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("Failed to compile executable"));
    }

    // Clean up temporary files
    std::fs::remove_file(&rs_path)?;
    // Keep .leanbc for debugging, or remove:
    // std::fs::remove_file(&bc_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::BufReader;
    use std::path::Path;

    /// Load the linked Init stdlib module
    fn load_init_modules(vm: &mut VM) {
        // Try linked stdlib first (faster)
        let stdlib_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("bc/stdlib/Init.leanbc");
        if stdlib_path.exists() {
            let file = fs::File::open(&stdlib_path).unwrap();
            let module = Module::deserialize(&mut BufReader::new(file)).unwrap();
            vm.load_module(module);
            return;
        }

        // Fallback to individual init modules (legacy)
        let init_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("init");
        if !init_dir.exists() {
            return;
        }
        let mut bc_files: Vec<_> = fs::read_dir(&init_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "leanbc"))
            .collect();
        bc_files.sort();
        for path in bc_files {
            let file = fs::File::open(&path).unwrap();
            let module = Module::deserialize(&mut BufReader::new(file)).unwrap();
            vm.load_module(module);
        }
    }

    /// Run a bytecode test file and return success/failure
    fn run_bytecode_test(path: &Path) -> Result<(), String> {
        let file = fs::File::open(path).map_err(|e| format!("Failed to open {:?}: {}", path, e))?;
        let module = Module::deserialize(&mut BufReader::new(file))
            .map_err(|e| format!("Failed to load {:?}: {}", path, e))?;

        let mut vm = VM::new();
        load_init_modules(&mut vm);
        vm.load_module(module);

        vm.run()
            .map(|_| ())
            .map_err(|e| format!("Runtime error in {:?}: {}", path, e))
    }

    /// Discover and test all .leanbc files in the tests directory
    #[test]
    fn run_all_bytecode_tests() {
        let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        if !tests_dir.exists() {
            eprintln!("Tests directory not found: {:?} (skipping)", tests_dir);
            return;
        }

        let mut bc_files: Vec<_> = fs::read_dir(&tests_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "leanbc"))
            .collect();
        bc_files.sort();

        let mut passed = 0;
        let mut failures: Vec<(String, String)> = Vec::new();

        for path in &bc_files {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            match run_bytecode_test(path) {
                Ok(_) => passed += 1,
                Err(e) => failures.push((name, e)),
            }
        }

        if !failures.is_empty() {
            eprintln!("\nFailed tests:");
            for (name, err) in &failures {
                eprintln!("  {}: {}", name, err);
            }
        }

        eprintln!(
            "\nBytecode tests: {} passed, {} failed, {} total",
            passed,
            failures.len(),
            bc_files.len()
        );

        assert!(
            failures.is_empty(),
            "{} bytecode test(s) failed",
            failures.len()
        );
    }

    /// Run stress tests (slower, run with `cargo test -- --ignored`)
    #[test]
    #[ignore]
    fn run_stress_tests() {
        let stress_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/stress_tests");
        if !stress_dir.exists() {
            eprintln!("Stress tests directory not found: {:?}", stress_dir);
            return;
        }

        let mut bc_files: Vec<_> = fs::read_dir(&stress_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "leanbc"))
            .collect();
        bc_files.sort();

        if bc_files.is_empty() {
            eprintln!("No stress test files found");
            return;
        }

        let mut passed = 0;
        let mut failures: Vec<(String, String)> = Vec::new();

        for path in &bc_files {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            eprintln!("Running stress test: {}", name);
            match run_bytecode_test(path) {
                Ok(_) => {
                    eprintln!("  {} passed", name);
                    passed += 1;
                }
                Err(e) => failures.push((name, e)),
            }
        }

        eprintln!(
            "\nStress tests: {} passed, {} failed, {} total",
            passed,
            failures.len(),
            bc_files.len()
        );

        assert!(
            failures.is_empty(),
            "{} stress test(s) failed",
            failures.len()
        );
    }
}
