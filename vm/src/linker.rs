//! Bytecode linker - merges multiple .leanbc modules into one
//!
//! Handles:
//! - String deduplication
//! - Extern deduplication (true VM primitives only)
//! - Function concatenation (must be unique by name)
//! - Bytecode rewriting with new IDs
//! - Converting cross-module CallExtern to direct Call
//! - Pre-resolving module indices (sets high bit to mark absolute func IDs)

/// Marker bit for pre-resolved function IDs.
/// When set, the lower 31 bits are an absolute function index (no module resolution needed).
pub const FUNC_ID_RESOLVED_BIT: u32 = 0x80000000;

use crate::bytecode::{ConstantDecl, ExternDecl, Function, Module, Opcode};
use std::collections::HashMap;
use std::io;

/// Remapping tables for a single module during linking
struct ModuleRemap {
    strings: HashMap<u32, u32>,         // old string id -> new string id
    functions: HashMap<u32, u32>,       // old func id -> new func id
    externs: HashMap<u32, ExternRemap>, // old extern id -> remap info
    constants: HashMap<u32, u32>,       // old const id -> new const id
}

/// What to do with an extern reference
enum ExternRemap {
    /// Keep as extern with new ID
    Extern(u32),
    /// Convert to direct function call
    Function(u32),
}

/// Link multiple modules into a single module
pub fn link(modules: Vec<Module>) -> io::Result<Module> {
    if modules.is_empty() {
        return Ok(Module::new());
    }

    // Collect all unique strings
    let mut string_pool: Vec<String> = Vec::new();
    let mut string_map: HashMap<String, u32> = HashMap::new();

    // Collect all functions (must be unique by name)
    let mut all_functions: Vec<Function> = Vec::new();
    let mut func_name_to_id: HashMap<String, u32> = HashMap::new();

    // Collect true externs (VM primitives not matching any function)
    let mut extern_pool: Vec<ExternDecl> = Vec::new();
    let mut extern_map: HashMap<(String, u8), u32> = HashMap::new(); // (name, arity) -> id

    // Collect constants
    let mut const_pool: Vec<ConstantDecl> = Vec::new();
    let mut const_name_to_id: HashMap<String, u32> = HashMap::new();

    // Per-module remapping tables
    let mut remaps: Vec<ModuleRemap> = Vec::new();

    // First pass: collect all items, build initial mappings
    for module in &modules {
        let mut remap = ModuleRemap {
            strings: HashMap::new(),
            functions: HashMap::new(),
            externs: HashMap::new(),
            constants: HashMap::new(),
        };

        // Dedupe strings
        for (old_id, s) in module.strings.iter().enumerate() {
            let new_id = *string_map.entry(s.clone()).or_insert_with(|| {
                let id = string_pool.len() as u32;
                string_pool.push(s.clone());
                id
            });
            remap.strings.insert(old_id as u32, new_id);
        }

        // Collect functions (error on collision)
        for (old_id, func) in module.functions.iter().enumerate() {
            if let Some(&existing_id) = func_name_to_id.get(&func.name) {
                // Check if it's actually the same function (same arity, same code)
                let existing = &all_functions[existing_id as usize];
                if existing.arity == func.arity && existing.code == func.code {
                    // Same function, just map to existing
                    remap.functions.insert(old_id as u32, existing_id);
                    continue;
                }
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate function: {}", func.name),
                ));
            }
            let new_id = all_functions.len() as u32;
            func_name_to_id.insert(func.name.clone(), new_id);
            remap.functions.insert(old_id as u32, new_id);
            all_functions.push(func.clone());
        }

        // Collect constants (dedupe by name)
        for (old_id, c) in module.const_decls.iter().enumerate() {
            let new_id = *const_name_to_id.entry(c.name.clone()).or_insert_with(|| {
                let id = const_pool.len() as u32;
                // Note: init_func will be remapped later
                const_pool.push(c.clone());
                id
            });
            remap.constants.insert(old_id as u32, new_id);
        }

        remaps.push(remap);
    }

    // Second pass: resolve externs
    // An extern that matches a function name becomes a direct call
    for (module_idx, module) in modules.iter().enumerate() {
        for (old_id, ext) in module.externs.iter().enumerate() {
            if let Some(&func_id) = func_name_to_id.get(&ext.name) {
                // This extern is actually a function we're linking in
                remaps[module_idx]
                    .externs
                    .insert(old_id as u32, ExternRemap::Function(func_id));
            } else {
                // True extern (VM primitive)
                let key = (ext.name.clone(), ext.arity);
                let new_id = *extern_map.entry(key).or_insert_with(|| {
                    let id = extern_pool.len() as u32;
                    extern_pool.push(ext.clone());
                    id
                });
                remaps[module_idx]
                    .externs
                    .insert(old_id as u32, ExternRemap::Extern(new_id));
            }
        }
    }

    // Third pass: rewrite bytecode with new IDs
    let mut linked_functions: Vec<Function> = Vec::new();
    for (module_idx, module) in modules.iter().enumerate() {
        let remap = &remaps[module_idx];
        for (old_func_id, func) in module.functions.iter().enumerate() {
            // Skip if this function was deduped to an earlier one
            let new_func_id = remap.functions[&(old_func_id as u32)];
            if new_func_id as usize != linked_functions.len() {
                continue; // Already added by earlier module
            }

            let new_code = rewrite_bytecode(&func.code, remap, Some(&func.name))?;
            linked_functions.push(Function {
                name: func.name.clone(),
                arity: func.arity,
                num_locals: func.num_locals,
                code: new_code,
            });
        }
    }

    // Remap constant init_func references
    let mut linked_constants: Vec<ConstantDecl> = Vec::new();
    let mut seen_constants: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (module_idx, module) in modules.iter().enumerate() {
        let remap = &remaps[module_idx];
        for c in &module.const_decls {
            if seen_constants.contains(&c.name) {
                continue;
            }
            seen_constants.insert(c.name.clone());
            let new_init = remap.functions[&c.init_func];
            linked_constants.push(ConstantDecl {
                name: c.name.clone(),
                init_func: new_init,
            });
        }
    }

    // Create combined init function that calls all module inits in order
    let init_func_id = if let Some(combined_init) = create_combined_init(&modules, &remaps) {
        let id = linked_functions.len() as u32;
        linked_functions.push(combined_init);
        Some(id)
    } else {
        None
    };

    // Find the entry point from the last module (typically the main module)
    let last_module = modules.last().unwrap();
    let last_remap = remaps.last().unwrap();
    let entry = last_remap.functions[&last_module.entry];

    Ok(Module {
        strings: string_pool,
        constants: Vec::new(),
        externs: extern_pool,
        const_decls: linked_constants,
        functions: linked_functions,
        entry,
        init_func: init_func_id,
    })
}

/// Rewrite bytecode with remapped IDs
fn rewrite_bytecode(
    code: &[u8],
    remap: &ModuleRemap,
    func_name: Option<&str>,
) -> io::Result<Vec<u8>> {
    let mut result = Vec::with_capacity(code.len());
    let mut pc = 0;

    while pc < code.len() {
        let start_pc = pc;
        let op_byte = code[pc];
        result.push(op_byte);
        pc += 1;

        let op = match Opcode::from_u8(op_byte) {
            Some(op) => op,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid opcode: 0x{:02x} at pc={} in {:?}",
                        op_byte, start_pc, func_name
                    ),
                ));
            }
        };

        match op {
            // Instructions with string ID
            Opcode::StringLit => {
                let old_id = read_u32(code, &mut pc);
                let new_id = remap.strings.get(&old_id).copied().unwrap_or(old_id);
                result.extend_from_slice(&new_id.to_le_bytes());
            }

            // Instructions with function ID - mark as pre-resolved
            Opcode::Call | Opcode::TailCall => {
                let old_func_id = read_u32(code, &mut pc);
                let num_args = code[pc];
                pc += 1;
                // Extract just the func_idx (ignore mod_idx since we're linking)
                let old_func_idx = old_func_id & 0xFFFF;
                let new_func_id = remap
                    .functions
                    .get(&old_func_idx)
                    .copied()
                    .unwrap_or(old_func_idx);
                // Set resolved bit to skip mod_idx resolution at runtime
                let resolved_func_id = FUNC_ID_RESOLVED_BIT | new_func_id;
                result.extend_from_slice(&resolved_func_id.to_le_bytes());
                result.push(num_args);
            }

            Opcode::PartialApp => {
                let old_func_id = read_u32(code, &mut pc);
                let arity = code[pc];
                pc += 1;
                let num_args = code[pc];
                pc += 1;
                // Extract just the func_idx (ignore mod_idx since we're linking)
                let old_func_idx = old_func_id & 0xFFFF;
                let new_func_id = remap
                    .functions
                    .get(&old_func_idx)
                    .copied()
                    .unwrap_or(old_func_idx);
                // Set resolved bit to skip mod_idx resolution at runtime
                let resolved_func_id = FUNC_ID_RESOLVED_BIT | new_func_id;
                result.extend_from_slice(&resolved_func_id.to_le_bytes());
                result.push(arity);
                result.push(num_args);
            }

            Opcode::AllocClosure => {
                let old_func_id = read_u32(code, &mut pc);
                let arity = code[pc];
                pc += 1;
                let num_captured = code[pc];
                pc += 1;
                // Extract just the func_idx (ignore mod_idx since we're linking)
                let old_func_idx = old_func_id & 0xFFFF;
                let new_func_id = remap
                    .functions
                    .get(&old_func_idx)
                    .copied()
                    .unwrap_or(old_func_idx);
                // Set resolved bit to skip mod_idx resolution at runtime
                let resolved_func_id = FUNC_ID_RESOLVED_BIT | new_func_id;
                result.extend_from_slice(&resolved_func_id.to_le_bytes());
                result.push(arity);
                result.push(num_captured);
            }

            // Extern calls - may convert to direct calls
            Opcode::CallExtern => {
                let old_ext_id = read_u32(code, &mut pc);
                let num_args = code[pc];
                pc += 1;

                match remap.externs.get(&old_ext_id) {
                    Some(ExternRemap::Function(func_id)) => {
                        // Convert to direct call with resolved bit set
                        let len = result.len();
                        result[len - 1] = Opcode::Call as u8;
                        let resolved_func_id = FUNC_ID_RESOLVED_BIT | func_id;
                        result.extend_from_slice(&resolved_func_id.to_le_bytes());
                        result.push(num_args);
                    }
                    Some(ExternRemap::Extern(new_ext_id)) => {
                        result.extend_from_slice(&new_ext_id.to_le_bytes());
                        result.push(num_args);
                    }
                    None => {
                        // Keep original (shouldn't happen)
                        result.extend_from_slice(&old_ext_id.to_le_bytes());
                        result.push(num_args);
                    }
                }
            }

            // CallImport uses string ID for function name
            Opcode::CallImport => {
                let old_str_id = read_u32(code, &mut pc);
                let num_args = code[pc];
                pc += 1;
                let new_str_id = remap
                    .strings
                    .get(&old_str_id)
                    .copied()
                    .unwrap_or(old_str_id);
                result.extend_from_slice(&new_str_id.to_le_bytes());
                result.push(num_args);
            }

            // LoadModuleConst uses constant ID
            Opcode::LoadModuleConst => {
                let old_const_id = read_u32(code, &mut pc);
                let new_const_id = remap
                    .constants
                    .get(&old_const_id)
                    .copied()
                    .unwrap_or(old_const_id);
                result.extend_from_slice(&new_const_id.to_le_bytes());
            }

            // Instructions with u16 operand
            Opcode::LoadLocal | Opcode::StoreLocal => {
                let val = read_u16(code, &mut pc);
                result.extend_from_slice(&val.to_le_bytes());
            }

            // Instructions with two u8 operands
            Opcode::AllocCtor => {
                result.push(code[pc]); // tag
                pc += 1;
                result.push(code[pc]); // fields
                pc += 1;
            }

            // Instructions with one u8 operand
            Opcode::CtorGet
            | Opcode::CtorSet
            | Opcode::CtorSetTag
            | Opcode::ClosureGet
            | Opcode::ClosureSet
            | Opcode::Apply
            | Opcode::TailApply
            | Opcode::BoolLit => {
                result.push(code[pc]);
                pc += 1;
            }

            // Instructions with i32 operand (jumps)
            Opcode::Jump | Opcode::JumpIf | Opcode::JumpIfNot => {
                let offset = read_i32(code, &mut pc);
                result.extend_from_slice(&offset.to_le_bytes());
            }

            // Switch: num_cases (u16), offsets (i32 each), default (i32)
            Opcode::Switch => {
                let num_cases = read_u16(code, &mut pc);
                result.extend_from_slice(&num_cases.to_le_bytes());
                for _ in 0..num_cases {
                    let offset = read_i32(code, &mut pc);
                    result.extend_from_slice(&offset.to_le_bytes());
                }
                let default = read_i32(code, &mut pc);
                result.extend_from_slice(&default.to_le_bytes());
            }

            // NatLit has u64 operand
            Opcode::NatLit => {
                let val = read_u64(code, &mut pc);
                result.extend_from_slice(&val.to_le_bytes());
            }

            // ScalarProj/ScalarSet: num_objs (u8), offset (u16)
            Opcode::ScalarProj | Opcode::ScalarSet => {
                result.push(code[pc]); // num_objs
                pc += 1;
                let offset = read_u16(code, &mut pc);
                result.extend_from_slice(&offset.to_le_bytes());
            }

            // LoadConst has u32 operand
            Opcode::LoadConst => {
                let val = read_u32(code, &mut pc);
                result.extend_from_slice(&val.to_le_bytes());
            }

            // No operands
            Opcode::Pop
            | Opcode::Dup
            | Opcode::GetTag
            | Opcode::Inc
            | Opcode::Dec
            | Opcode::IsShared
            | Opcode::IsExclusive
            | Opcode::Ret
            | Opcode::Unreachable
            | Opcode::Box
            | Opcode::Unbox
            | Opcode::IsScalar
            | Opcode::NatAdd
            | Opcode::NatSub
            | Opcode::NatMul
            | Opcode::NatDiv
            | Opcode::NatMod
            | Opcode::NatLt
            | Opcode::NatLe
            | Opcode::NatEq
            | Opcode::NatSucc
            | Opcode::StringAppend
            | Opcode::StringLength
            | Opcode::StringEq
            | Opcode::UnitLit
            | Opcode::Trace => {}
        }
    }

    Ok(result)
}

/// Create a combined init function that calls all module inits
fn create_combined_init(modules: &[Module], remaps: &[ModuleRemap]) -> Option<Function> {
    let init_calls: Vec<u32> = modules
        .iter()
        .zip(remaps.iter())
        .filter_map(|(m, r)| m.init_func.map(|old_id| r.functions[&old_id]))
        .collect();

    if init_calls.is_empty() {
        return None;
    }

    // Build bytecode that calls each init in order and returns unit
    let mut code = Vec::new();

    for func_id in init_calls {
        // Call init function with 0 args
        code.push(Opcode::Call as u8);
        code.extend_from_slice(&func_id.to_le_bytes());
        code.push(0); // num_args
                      // Pop the result (init functions return IO Unit)
        code.push(Opcode::Pop as u8);
    }

    // Return unit
    code.push(Opcode::UnitLit as u8);
    code.push(Opcode::Ret as u8);

    Some(Function {
        name: "_linked_init".to_string(),
        arity: 0,
        num_locals: 0,
        code,
    })
}

fn read_u16(code: &[u8], pc: &mut usize) -> u16 {
    if *pc + 2 > code.len() {
        panic!("read_u16: pc={} but code len={}", *pc, code.len());
    }
    let val = u16::from_le_bytes([code[*pc], code[*pc + 1]]);
    *pc += 2;
    val
}

fn read_u32(code: &[u8], pc: &mut usize) -> u32 {
    if *pc + 4 > code.len() {
        panic!("read_u32: pc={} but code len={}", *pc, code.len());
    }
    let val = u32::from_le_bytes([code[*pc], code[*pc + 1], code[*pc + 2], code[*pc + 3]]);
    *pc += 4;
    val
}

fn read_i32(code: &[u8], pc: &mut usize) -> i32 {
    if *pc + 4 > code.len() {
        panic!("read_i32: pc={} but code len={}", *pc, code.len());
    }
    let val = i32::from_le_bytes([code[*pc], code[*pc + 1], code[*pc + 2], code[*pc + 3]]);
    *pc += 4;
    val
}

fn read_u64(code: &[u8], pc: &mut usize) -> u64 {
    if *pc + 8 > code.len() {
        panic!("read_u64: pc={} but code len={}", *pc, code.len());
    }
    let val = u64::from_le_bytes([
        code[*pc],
        code[*pc + 1],
        code[*pc + 2],
        code[*pc + 3],
        code[*pc + 4],
        code[*pc + 5],
        code[*pc + 6],
        code[*pc + 7],
    ]);
    *pc += 8;
    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_empty() {
        let result = link(vec![]).unwrap();
        assert!(result.functions.is_empty());
    }

    #[test]
    fn test_link_single_module() {
        let mut m = Module::new();
        m.strings.push("hello".to_string());
        m.functions.push(Function {
            name: "main".to_string(),
            arity: 0,
            num_locals: 0,
            code: vec![Opcode::UnitLit as u8, Opcode::Ret as u8],
        });
        m.entry = 0;

        let result = link(vec![m]).unwrap();
        assert_eq!(result.strings.len(), 1);
        assert_eq!(result.functions.len(), 1);
    }

    #[test]
    fn test_string_dedup() {
        let mut m1 = Module::new();
        m1.strings.push("shared".to_string());
        m1.strings.push("unique1".to_string());
        m1.functions.push(Function {
            name: "f1".to_string(),
            arity: 0,
            num_locals: 0,
            code: vec![Opcode::UnitLit as u8, Opcode::Ret as u8],
        });
        m1.entry = 0;

        let mut m2 = Module::new();
        m2.strings.push("shared".to_string()); // should dedupe
        m2.strings.push("unique2".to_string());
        m2.functions.push(Function {
            name: "f2".to_string(),
            arity: 0,
            num_locals: 0,
            code: vec![Opcode::UnitLit as u8, Opcode::Ret as u8],
        });
        m2.entry = 0;

        let result = link(vec![m1, m2]).unwrap();
        assert_eq!(result.strings.len(), 3); // shared, unique1, unique2
    }
}
