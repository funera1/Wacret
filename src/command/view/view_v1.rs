use anyhow::Result;
use camino::Utf8PathBuf;
use serde_json;
use std::fs;

use super::utils::{read_u32, read_u32_or_zero, read_u8, bytes_to_int, Label, UnifiedFormat};

/// Parse a v1 format binary file and return the parsed data
fn parse_v1_format(path: &Utf8PathBuf) -> Result<UnifiedFormat> {
    let data = fs::read(path)?;
    let mut cursor = 0;

    // Read entry function index
    let _entry_fidx = read_u32(&mut cursor, &data)?;

    // Read return address
    let return_fidx = read_u32(&mut cursor, &data)?;
    let return_offset = read_u32(&mut cursor, &data)?;

    // Read type stack
    let type_stack_size = read_u32(&mut cursor, &data)?;
    let mut type_stack = Vec::new();
    for _ in 0..type_stack_size {
        let typ = read_u8(&mut cursor, &data)?;
        type_stack.push(typ);
    }

    // Read value stack
    let mut value_stack = Vec::new();
    for i in 0..type_stack_size {
        let typ = type_stack[i as usize];
        let byte_count = (typ as usize) * 4;

        // Read bytes for this value
        let mut value_bytes = Vec::new();
        for _ in 0..byte_count {
            if cursor >= data.len() {
                anyhow::bail!("Unexpected end of file while reading value stack");
            }
            value_bytes.push(data[cursor]);
            cursor += 1;
        }
        value_stack.push(bytes_to_int(&value_bytes)?);
    }

    // Read label stack
    let label_stack_size = read_u32(&mut cursor, &data)?;
    let mut label_stack = Vec::new();
    for _ in 0..label_stack_size {
        let begin_addr = read_u32_or_zero(&mut cursor, &data);
        let target_addr = read_u32_or_zero(&mut cursor, &data);
        let sp = read_u32_or_zero(&mut cursor, &data);
        let tsp = read_u32_or_zero(&mut cursor, &data);
        let cell_num = read_u32_or_zero(&mut cursor, &data);
        let count = read_u32_or_zero(&mut cursor, &data);
        label_stack.push(Label {
            begin_addr,
            target_addr,
            sp,
            tsp,
            cell_num,
            count,
        });
    }

    Ok(UnifiedFormat {
        pc: Some((_entry_fidx, 10000000000 as u64)),
        return_address: Some((return_fidx, return_offset as u64)),
        locals: None, // V1 format does not have locals
        value_stack: Some(value_stack),
        label_stack: Some(label_stack.iter().map(|label| label.begin_addr).collect()),
        type_stack: Some(type_stack),
    })
}

/// Parse and display a v1 format binary file
/// This implements the equivalent functionality of the Python parser script
pub fn view_v1_format(path: Utf8PathBuf, json_output: bool) -> Result<()> {
    let parsed_data = parse_v1_format(&path)?;

    // Output results in requested format
    if json_output {
        // JSON output
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "pc": parsed_data.pc,
            "type_stack": parsed_data.type_stack,
            "value_stack": parsed_data.value_stack,
            "label_stack": parsed_data.label_stack
        }))?;
        println!("{}", json);
    } else {
        // Original format output
        println!("EntryFuncIdx: {}", parsed_data.pc.map_or(0, |pc| pc.0));
        println!("ReturnAddress: {:?}", parsed_data.pc);
        println!("StackSize: {}", parsed_data.value_stack.as_ref().map_or(0, |stack| stack.len()));
        println!("TypeStack: {:?}", parsed_data.type_stack);
        println!("ValueStack: {:?}", parsed_data.value_stack);
        println!("LabelStackSize: {}", parsed_data.label_stack.as_ref().map_or(0, |stack| stack.len()));
        println!("LabelStack: {:?}", parsed_data.label_stack);
    }

    Ok(())
}

/// Parse and display multiple v1 format binary files
/// This function aggregates multiple v1 stack files into a unified JSON format
pub fn view_v1_format_multiple(paths: Vec<Utf8PathBuf>, json_output: bool) -> Result<()> {
    let mut call_stack = Vec::new();

    for path in paths.iter() {
        match parse_v1_format(path) {
            Ok(frame) => {
                call_stack.push(UnifiedFormat {
                    pc: frame.pc,
                    return_address: frame.return_address,
                    type_stack: frame.type_stack,
                    value_stack: frame.value_stack,
                    label_stack: frame.label_stack,
                    locals: frame.locals,
                });
            }
            Err(e) => {
                eprintln!("Failed to parse file {}: {}", path, e);
            }
        }
    }

    if json_output {
        let output = serde_json::to_string_pretty(&call_stack)?;
        println!("{}", output);
    } else {
        for frame in call_stack {
            println!("{:?}", frame);
        }
    }

    Ok(())
}
