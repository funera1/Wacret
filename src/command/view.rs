use anyhow::Result;
use camino::Utf8PathBuf;
use prost::Message;
use serde_json;
use std::fs;

// Include the generated protobuf code
pub mod state {
    include!(concat!(env!("OUT_DIR"), "/state.rs"));
}

#[derive(Debug)]
struct Label {
    begin_addr: u32,
    target_addr: u32,
    sp: u32,
    tsp: u32,
    cell_num: u32,
    count: u32,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}, {}, {}, {}, {}, {}}}", 
               self.begin_addr, self.target_addr, self.sp, 
               self.tsp, self.cell_num, self.count)
    }
}

/// Convert bytes to integer value, mimicking Python's to_int function
fn bytes_to_int(bytes: &[u8]) -> Result<i64> {
    match bytes.len() {
        4 => {
            let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            // Handle negative values as per Python's to_int logic
            if raw > 0xffff0000 {
                Ok((raw as i64) - 0x100000000i64)
            } else {
                Ok(raw as i64)
            }
        },
        8 => {
            let low = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64;
            let high = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64;
            let raw = (high << 32) | low;
            Ok(raw as i64)
        },
        0 => Ok(0i64),
        _ => anyhow::bail!("Unsupported byte length: {}", bytes.len()),
    }
}

/// Parse and display a v1 format binary file
/// This implements the equivalent functionality of the Python parser script
pub fn view_v1_format(path: Utf8PathBuf) -> Result<()> {
    let data = fs::read(&path)?;
    let mut cursor = 0;

    // Helper functions for reading binary data in little-endian format
    let read_u32 = |cursor: &mut usize| -> Result<u32> {
        if *cursor + 4 > data.len() {
            anyhow::bail!("Unexpected end of file while reading u32");
        }
        let bytes = &data[*cursor..*cursor + 4];
        *cursor += 4;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };

    // Read u32 or return 0 if at end of file (handles truncated label data)
    let read_u32_or_zero = |cursor: &mut usize| -> u32 {
        if *cursor + 4 > data.len() {
            0
        } else {
            let bytes = &data[*cursor..*cursor + 4];
            *cursor += 4;
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }
    };

    let read_u8 = |cursor: &mut usize| -> Result<u8> {
        if *cursor >= data.len() {
            anyhow::bail!("Unexpected end of file while reading u8");
        }
        let byte = data[*cursor];
        *cursor += 1;
        Ok(byte)
    };

    // Read entry function index
    let entry_fidx = read_u32(&mut cursor)?;
    
    // Read return address
    let return_fidx = read_u32(&mut cursor)?;
    let return_offset = read_u32(&mut cursor)?;
    
    // Read type stack
    let type_stack_size = read_u32(&mut cursor)?;
    let mut type_stack = Vec::new();
    for _ in 0..type_stack_size {
        let typ = read_u8(&mut cursor)?;
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
        
        // Convert bytes to value (equivalent to Python's to_int function)
        let value = bytes_to_int(&value_bytes)?;
        value_stack.push(value);
    }

    // Read label stack
    let label_stack_size = read_u32(&mut cursor)?;
    let mut label_stack = Vec::new();
    
    for _ in 0..label_stack_size {
        let label = Label {
            begin_addr: read_u32_or_zero(&mut cursor),
            target_addr: read_u32_or_zero(&mut cursor),
            sp: read_u32_or_zero(&mut cursor),
            tsp: read_u32_or_zero(&mut cursor),
            cell_num: read_u32_or_zero(&mut cursor),
            count: read_u32_or_zero(&mut cursor),
        };
        label_stack.push(label);
    }

    // Display results
    println!("EntryFuncIdx: {}", entry_fidx);
    println!("ReturnAddress: {}, {}", return_fidx, return_offset);
    println!("StackSize: {}", type_stack_size);
    println!("TypeStack: {:?}", type_stack);
    println!("ValueStack: {:?}", value_stack);
    println!("LabelStackSize: {}", label_stack_size);
    println!("LabelStack: [");
    for label in &label_stack {
        println!("\t{}", label);
    }
    println!("]");

    Ok(())
}

pub fn view_protobuf(path: Utf8PathBuf) -> Result<()> {
    // Read the protobuf file
    let data = fs::read(&path)?;
    
    // Try to decode as different message types
    // Since we don't know the exact type, we'll try the most likely ones
    
    // Try CallStack first (most likely to be the top-level message)
    if let Ok(call_stack) = state::CallStack::decode(&data[..]) {
        let json = serde_json::to_string_pretty(&call_stack_to_json(&call_stack))?;
        println!("{}", json);
        return Ok(());
    }
    
    // Try CallStackEntry
    if let Ok(entry) = state::CallStackEntry::decode(&data[..]) {
        let json = serde_json::to_string_pretty(&call_stack_entry_to_json(&entry))?;
        println!("{}", json);
        return Ok(());
    }
    
    // Try CodePos
    if let Ok(code_pos) = state::CodePos::decode(&data[..]) {
        let json = serde_json::to_string_pretty(&code_pos_to_json(&code_pos))?;
        println!("{}", json);
        return Ok(());
    }
    
    anyhow::bail!("Unable to decode protobuf file as any known message type");
}

fn call_stack_to_json(call_stack: &state::CallStack) -> serde_json::Value {
    serde_json::json!({
        "entries": call_stack.entries.iter().map(call_stack_entry_to_json).collect::<Vec<_>>()
    })
}

fn call_stack_entry_to_json(entry: &state::CallStackEntry) -> serde_json::Value {
    serde_json::json!({
        "pc": entry.pc.as_ref().map(code_pos_to_json),
        "locals": entry.locals.as_ref().map(typed_array_to_json),
        "value_stack": entry.value_stack.as_ref().map(typed_array_to_json),
        "label_stack": entry.label_stack.as_ref().map(label_stack_to_json)
    })
}

fn code_pos_to_json(code_pos: &state::CodePos) -> serde_json::Value {
    serde_json::json!({
        "fidx": code_pos.fidx,
        "offset": code_pos.offset
    })
}

fn typed_array_to_json(typed_array: &state::TypedArray) -> serde_json::Value {
    serde_json::json!({
        "types": typed_array.types.as_ref().map(array8_to_json),
        "values": typed_array.values.as_ref().map(array32_to_json)
    })
}

fn array8_to_json(array8: &state::Array8) -> serde_json::Value {
    serde_json::json!({
        "contents": array8.contents
    })
}

fn array32_to_json(array32: &state::Array32) -> serde_json::Value {
    serde_json::json!({
        "contents": array32.contents
    })
}

#[allow(dead_code)]
fn array64_to_json(array64: &state::Array64) -> serde_json::Value {
    serde_json::json!({
        "contents": array64.contents
    })
}

fn label_stack_to_json(label_stack: &state::LabelStack) -> serde_json::Value {
    serde_json::json!({
        "begins": label_stack.begins,
        "targets": label_stack.targets,
        "stack_pointers": label_stack.stack_pointers,
        "cell_nums": label_stack.cell_nums
    })
}
