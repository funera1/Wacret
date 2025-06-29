use anyhow::Result;
use camino::Utf8PathBuf;
use prost::Message;
use serde::Serialize;
use serde_json;
use std::fs;

// Include the generated protobuf code
pub mod state {
    include!(concat!(env!("OUT_DIR"), "/state.rs"));
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Serialize)]
struct V1FormatData {
    #[serde(rename = "EntryFuncIdx")]
    entry_func_idx: u32,
    #[serde(rename = "ReturnAddress")]
    return_address: (u32, u32),
    #[serde(rename = "StackSize")]
    stack_size: u32,
    #[serde(rename = "TypeStack")]
    type_stack: Vec<u8>,
    #[serde(rename = "ValueStack")]
    value_stack: Vec<i64>,
    #[serde(rename = "LabelStackSize")]
    label_stack_size: u32,
    #[serde(rename = "LabelStack")]
    label_stack: Vec<Label>,
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

/// Helper function to read a u32 value from binary data
fn read_u32(cursor: &mut usize, data: &[u8]) -> Result<u32> {
    if *cursor + 4 > data.len() {
        anyhow::bail!("Unexpected end of file while reading u32");
    }
    let bytes = &data[*cursor..*cursor + 4];
    *cursor += 4;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Helper function to read a u32 value or return 0 if at the end of the file
fn read_u32_or_zero(cursor: &mut usize, data: &[u8]) -> u32 {
    if *cursor + 4 > data.len() {
        0
    } else {
        let bytes = &data[*cursor..*cursor + 4];
        *cursor += 4;
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

/// Helper function to read a u8 value from binary data
fn read_u8(cursor: &mut usize, data: &[u8]) -> Result<u8> {
    if *cursor >= data.len() {
        anyhow::bail!("Unexpected end of file while reading u8");
    }
    let byte = data[*cursor];
    *cursor += 1;
    Ok(byte)
}

/// Parse a v1 format binary file and return the parsed data
fn parse_v1_format(path: &Utf8PathBuf) -> Result<V1FormatData> {
    let data = fs::read(path)?;
    let mut cursor = 0;

    // Read entry function index
    let entry_fidx = read_u32(&mut cursor, &data)?;

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

        // Convert bytes to value (equivalent to Python's to_int function)
        let value = bytes_to_int(&value_bytes)?;
        value_stack.push(value);
    }

    // Read label stack
    let label_stack_size = read_u32(&mut cursor, &data)?;
    let mut label_stack = Vec::new();

    for _ in 0..label_stack_size {
        let label = Label {
            begin_addr: read_u32_or_zero(&mut cursor, &data),
            target_addr: read_u32_or_zero(&mut cursor, &data),
            sp: read_u32_or_zero(&mut cursor, &data),
            tsp: read_u32_or_zero(&mut cursor, &data),
            cell_num: read_u32_or_zero(&mut cursor, &data),
            count: read_u32_or_zero(&mut cursor, &data),
        };
        label_stack.push(label);
    }

    Ok(V1FormatData {
        entry_func_idx: entry_fidx,
        return_address: (return_fidx, return_offset),
        stack_size: type_stack_size,
        type_stack,
        value_stack,
        label_stack_size,
        label_stack,
    })
}

/// Parse and display a v1 format binary file
/// This implements the equivalent functionality of the Python parser script
pub fn view_v1_format(path: Utf8PathBuf, json_output: bool) -> Result<()> {
    let parsed_data = parse_v1_format(&path)?;

    // Output results in requested format
    if json_output {
        // JSON output
        let json = serde_json::to_string_pretty(&parsed_data)?;
        println!("{}", json);
    } else {
        // Original format output
        println!("EntryFuncIdx: {}", parsed_data.entry_func_idx);
        println!("ReturnAddress: {}, {}", parsed_data.return_address.0, parsed_data.return_address.1);
        println!("StackSize: {}", parsed_data.stack_size);
        println!("TypeStack: {:?}", parsed_data.type_stack);
        println!("ValueStack: {:?}", parsed_data.value_stack);
        println!("LabelStackSize: {}", parsed_data.label_stack_size);
        println!("LabelStack: [");
        for label in &parsed_data.label_stack {
            println!("\t{}", label);
        }
        println!("]");
    }

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

fn parse_typed_array(cursor: &mut usize, data: &[u8]) -> Result<serde_json::Value> {
    let type_stack_size = read_u32(cursor, data)?;
    let mut types = Vec::new();
    for _ in 0..type_stack_size {
        types.push(read_u8(cursor, data)?);
    }

    let mut values = Vec::new();
    for typ in &types {
        let byte_count = (*typ as usize) * 4;
        let mut value_bytes = vec![0; byte_count];
        value_bytes.copy_from_slice(&data[*cursor..*cursor + byte_count]);
        *cursor += byte_count;
        values.push(bytes_to_int(&value_bytes)?);
    }

    Ok(serde_json::json!({ "types": types, "values": values }))
}

fn parse_label_stack(cursor: &mut usize, data: &[u8]) -> Result<serde_json::Value> {
    let label_stack_size = read_u32(cursor, data)?;
    let mut begins = Vec::new();
    let mut targets = Vec::new();
    let mut stack_pointers = Vec::new();
    let mut cell_nums = Vec::new();

    for _ in 0..label_stack_size {
        begins.push(read_u32_or_zero(cursor, data));
        targets.push(read_u32_or_zero(cursor, data));
        stack_pointers.push(read_u32_or_zero(cursor, data));
        cell_nums.push(read_u32_or_zero(cursor, data));
    }

    Ok(serde_json::json!({
        "begins": begins,
        "targets": targets,
        "stack_pointers": stack_pointers,
        "cell_nums": cell_nums
    }))
}

/// Parse and display multiple v1 format binary files
/// This function aggregates multiple v1 stack files into a unified JSON format
pub fn view_v1_format_multiple(paths: Vec<Utf8PathBuf>, json_output: bool) -> Result<()> {
    let mut call_stack = Vec::new();

    for (frame_index, path) in paths.iter().enumerate() {
        let data = fs::read(path)?;
        let frame = parse_v1_frame(&data, frame_index)?;
        call_stack.push(frame);
    }

    if json_output {
        let output = serde_json::json!({ "call_stack": call_stack });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for frame in call_stack {
            println!("{:?}", frame);
        }
    }

    Ok(())
}

/// Parse a single v1 frame from binary data
fn parse_v1_frame(data: &[u8], frame_index: usize) -> Result<serde_json::Value> {
    let mut cursor = 0;

    let pc = {
        let fidx = read_u32(&mut cursor, data)?;
        let offset = read_u32(&mut cursor, data)?;
        serde_json::json!({ "fidx": fidx, "offset": offset })
    };

    let locals = parse_typed_array(&mut cursor, data)?;
    let value_stack = parse_typed_array(&mut cursor, data)?;
    let label_stack = parse_label_stack(&mut cursor, data)?;

    Ok(serde_json::json!({
        "frame_index": frame_index,
        "pc": pc,
        "locals": locals,
        "value_stack": value_stack,
        "label_stack": label_stack
    }))
}
