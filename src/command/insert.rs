use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use std::fs;
use wasmparser::{Parser, Payload};
use wasm_encoder::{Module, CodeSection, Function, Instruction};

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
    let parser = Parser::new(0);
    let mut module = Module::new();
    let mut code_section = CodeSection::new();
    let mut current_func_index = 0u32;

    for payload in parser.parse_all(wasm_bytes) {
        match payload? {
            Payload::CodeSectionEntry(function_body) => {
                let mut func = Function::new(vec![]);
                
                if current_func_index == func_index {
                    log::info!("Processing target function {} for NOP insertion", func_index);
                    
                    let mut operators = function_body.get_operators_reader()?;
                    let mut current_offset = 0u32;
                    let mut nop_inserted = false;

                    while !operators.eof() {
                        // Insert NOP at the target offset
                        if !nop_inserted && current_offset == instr_offset {
                            log::info!("Inserting NOP at offset {}", current_offset);
                            func.instruction(&Instruction::Nop);
                            nop_inserted = true;
                        }

                        let _op = operators.read()?;
                        // TODO: Convert and add the original instruction
                        // For now, just skip the original instructions
                        current_offset += 1;
                    }

                    // If we haven't inserted the NOP yet and the target offset is at the end
                    if !nop_inserted && current_offset <= instr_offset {
                        log::info!("Inserting NOP at end of function at offset {}", current_offset);
                        func.instruction(&Instruction::Nop);
                    }
                } else {
                    // For non-target functions, just add a NOP as placeholder
                    func.instruction(&Instruction::Nop);
                }

                code_section.function(&func);
                current_func_index += 1;
            }
            _ => {
                // TODO: Copy other sections to the module
                // For now, skip all other sections
            }
        }
    }

    module.section(&code_section);
    Ok(module.finish())
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
}
