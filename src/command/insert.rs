use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use std::fs;
use walrus::{Module, ir::{Instr, InstrLocId}};

/// Insert a NOP instruction at a specific offset within a specific function
pub fn insert_nop(
    input_path: Utf8PathBuf,
    output_path: Utf8PathBuf,
    function_index: u32,
    offset: u32,
) -> Result<()> {
    // Read the original WASM file
    let wasm_bytes = fs::read(&input_path)
        .map_err(|e| anyhow!("Failed to read input file {}: {}", input_path, e))?;

    // Inject NOP into the WASM binary
    let modified_bytes = inject_nop(&wasm_bytes, function_index, offset)?;

    // Write the modified WASM binary
    fs::write(&output_path, &modified_bytes)
        .map_err(|e| anyhow!("Failed to write output file {}: {}", output_path, e))?;

    log::info!(
        "Successfully inserted NOP at offset {} in function {} and wrote to {}",
        offset,
        function_index,
        output_path
    );

    Ok(())
}

/// Inject a NOP instruction at the specified offset within the specified function
fn inject_nop(wasm_bytes: &[u8], func_index: u32, instr_offset: u32) -> Result<Vec<u8>> {
    // Configure Walrus to generate the name section
    let mut config = walrus::ModuleConfig::new();
    config.generate_name_section(true);

    // Parse the WASM module using Walrus
    let mut module = Module::from_buffer_with_config(wasm_bytes, &config)
        .map_err(|e| anyhow!("Failed to parse WASM module: {}", e))?;

    // Get the function ID from the index
    let func_id = module.funcs.iter().nth(func_index as usize)
        .ok_or_else(|| anyhow!("Function with index {} not found", func_index))?.id();

    // Get the mutable function body
    let func = module.funcs.get_mut(func_id);

    // Ensure the function is local
    let body = match &mut func.kind {
        walrus::FunctionKind::Local(local) => local,
        _ => return Err(anyhow!("Function with index {} is not a local function", func_index)),
    };

    // Get the entry block ID
    let entry_block_id = body.entry_block();

    // Get a mutable reference to the instruction sequence
    let instr_seq = body.block_mut(entry_block_id);

    // Insert a NOP-equivalent instruction (e.g., Drop) at the specified offset
    let nop_instr = Instr::Nop(walrus::ir::Nop {});
    let nop_loc_id = InstrLocId::new(instr_seq.instrs.len() as u32);
    instr_seq.instrs.insert(instr_offset as usize, (nop_instr, nop_loc_id));

    // Log the instruction sequence after insertion
    log::debug!("Instruction sequence after insertion: {:?}", instr_seq.instrs);

    // Serialize the modified module back to bytes
    let modified_bytes = module.emit_wasm();

    Ok(modified_bytes)
}

/// Inject a NOP instruction at the specified offset within the specified function,
/// and then restore the original function order after rewriting.
fn inject_nop_preserve_order(wasm_bytes: &[u8], func_index: u32, instr_offset: u32) -> Result<Vec<u8>> {
    use walrus::{FunctionId, Module, ModuleConfig, ir::*};
    use wasmparser::{Parser, Payload};
    use wasm_encoder::*;

    // === Step 1: Record original function order ===
    let mut original_func_order = Vec::new();
    let mut imported_funcs = 0u32;

    for payload in Parser::new(0).parse_all(wasm_bytes) {
        match payload? {
            Payload::ImportSection(s) => {
                for import in s {
                    let import = import?;
                    if let wasmparser::TypeRef::Func(_) = import.ty {
                        imported_funcs += 1;
                    }
                }
            }
            Payload::FunctionSection(s) => {
                for (i, _) in s.into_iter().enumerate() {
                    // Store original local function index (not including imports)
                    original_func_order.push(imported_funcs + i as u32);
                }
            }
            _ => {}
        }
    }

    // === Step 2: Modify the code using Walrus ===
    let mut config = ModuleConfig::new();
    config.generate_name_section(true);
    let mut module = Module::from_buffer_with_config(wasm_bytes, &config)?;

    let func_id = module.funcs.iter().nth(func_index as usize)
        .ok_or_else(|| anyhow!("Function with index {} not found", func_index))?.id();

    let func = module.funcs.get_mut(func_id);
    let body = match &mut func.kind {
        walrus::FunctionKind::Local(local) => local,
        _ => return Err(anyhow!("Function at index {} is not a local function", func_index)),
    };

    let entry = body.entry_block();
    let instrs = body.block_mut(entry);
    let nop = Instr::Nop(Nop {});
    let loc_id = InstrLocId::new(instrs.instrs.len() as u32);
    instrs.instrs.insert(instr_offset as usize, (nop, loc_id));

    let new_bytes = module.emit_wasm();

    // === Step 3: Restore original function order ===
    let walrus_module = walrus::ModuleConfig::new().parse(&new_bytes)?;
    let mut output = ModuleEncoder::new();

    // Reuse sections from walrus output, but reorder the function + code section manually
    let mut encoder = output.section(FunctionSection::new());
    let mut code_sec = CodeSection::new();

    let code_bodies: Vec<_> = walrus_module.funcs.iter_local().collect();

    for &old_idx in &original_func_order {
        let func = walrus_module.funcs.iter().nth(old_idx as usize).ok_or(anyhow!("Missing function index {} after rewriting", old_idx))?;
        if let walrus::FunctionKind::Local(local) = &func.kind {
            let ty = func.ty();
            encoder.function(ty);
            let mut func_writer = Function::new(vec![]);
            for instr in &local.entry_block().instrs {
                func_writer.instruction(instr.0.clone());
            }
            func_writer.instruction(&Instruction::End);
            code_sec.function(&func_writer);
        }
    }

    output.section(&encoder);
    output.section(&code_sec);

    Ok(output.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_nop_file_not_found() {
        let result = insert_nop(
            "nonexistent.wasm".into(),
            "output.wasm".into(),
            0,
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_nop_invalid_function_index() {
        // Test with empty WASM bytes that would have no functions
        let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]; // WASM magic + version
        let result = inject_nop(&empty_wasm, 999, 0);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("Function with index 999 not found"));
        }
    }
}
