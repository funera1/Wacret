use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use std::fs;
use walrus::{Module, ir::{Instr, InstrLocId, Drop}};

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
    // Parse the WASM module using Walrus
    let mut module = Module::from_buffer(wasm_bytes)
        .map_err(|e| anyhow!("Failed to parse WASM module: {}", e))?;

    // Get the function ID from the index
    let func_id = module.funcs.iter().nth(func_index as usize)
        .ok_or_else(|| anyhow!("Function with index {} not found", func_index))?.id();

    // Get the mutable function body
    let func = module.funcs.get_mut(func_id);
    let body = func.kind.unwrap_local_mut();

    // Get the entry block ID
    let entry_block_id = body.entry_block();

    // Get a mutable reference to the instruction sequence
    let instr_seq = body.block_mut(entry_block_id);

    // Insert a NOP-equivalent instruction (e.g., Drop) at the specified offset
    let nop_instr = Instr::Drop(Drop {});
    let nop_loc_id = InstrLocId::new(instr_seq.instrs.len() as u32);
    instr_seq.instrs.insert(instr_offset as usize, (nop_instr, nop_loc_id));

    // Serialize the modified module back to bytes
    let modified_bytes = module.emit_wasm();

    Ok(modified_bytes)
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
