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

pub fn view_v1_format(path: Utf8PathBuf) -> Result<()> {
    let data = fs::read(&path)?;
    let mut cursor = 0;

    println!("File size: {} bytes", data.len());

    // Helper function to read bytes as little-endian integer
    let read_u32 = |cursor: &mut usize| -> Result<u32> {
        if *cursor + 4 > data.len() {
            anyhow::bail!("Unexpected end of file while reading u32 at position {}, file size: {}", *cursor, data.len());
        }
        let bytes = &data[*cursor..*cursor + 4];
        *cursor += 4;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };

    let read_u32_or_zero = |cursor: &mut usize| -> u32 {
        if *cursor + 4 > data.len() {
            0 // ファイル終端に達した場合は0を返す
        } else {
            let bytes = &data[*cursor..*cursor + 4];
            *cursor += 4;
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }
    };

    let read_u8 = |cursor: &mut usize| -> Result<u8> {
        if *cursor >= data.len() {
            anyhow::bail!("Unexpected end of file while reading u8 at position {}, file size: {}", *cursor, data.len());
        }
        let byte = data[*cursor];
        *cursor += 1;
        Ok(byte)
    };

    // エントリー関数
    let entry_fidx = read_u32(&mut cursor)?;
    println!("Read entry_fidx: {} at position {}", entry_fidx, cursor);
    
    // リターンアドレス
    let return_fidx = read_u32(&mut cursor)?;
    println!("Read return_fidx: {} at position {}", return_fidx, cursor);
    let return_offset = read_u32(&mut cursor)?;
    println!("Read return_offset: {} at position {}", return_offset, cursor);
    
    // 型スタック
    let type_stack_size = read_u32(&mut cursor)?;
    println!("Read type_stack_size: {} at position {}", type_stack_size, cursor);
    let mut type_stack = Vec::new();
    for i in 0..type_stack_size {
        let typ = read_u8(&mut cursor)?;
        type_stack.push(typ);
        if i < 10 || i == type_stack_size - 1 {
            println!("Read type[{}]: {} at position {}", i, typ, cursor);
        } else if i == 10 {
            println!("... (skipping type output) ...");
        }
    }
    println!("Finished reading type stack at position {}", cursor);

    // 値スタック
    println!("Starting value stack reading at position {}", cursor);
    let mut value_stack = Vec::new();
    for i in 0..type_stack_size {
        let typ = type_stack[i as usize];
        // Pythonコードでは 4 * type_stack[i] バイト読み取っている
        let byte_count = (typ as usize) * 4;
        println!("Reading value[{}]: type={}, byte_count={} at position {}", i, typ, byte_count, cursor);
        let mut value_bytes = Vec::new();
        
        for j in 0..byte_count {
            if cursor >= data.len() {
                anyhow::bail!("Unexpected end of file while reading value stack at position {}, byte {} of {}", cursor, j, byte_count);
            }
            value_bytes.push(data[cursor]);
            cursor += 1;
        }
        
        // バイト配列を適切な値に変換 - Pythonのto_int相当の処理
        let value: i64 = match byte_count {
            4 => {
                if value_bytes.len() >= 4 {
                    let raw = u32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]]);
                    // Pythonのto_int関数のロジック: 0xffff0000より大きい場合は負の値として扱う
                    if raw > 0xffff0000 {
                        (raw as i64) - 0x100000000i64
                    } else {
                        raw as i64
                    }
                } else {
                    0i64
                }
            },
            8 => {
                if value_bytes.len() >= 8 {
                    let low = u32::from_le_bytes([value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]]) as u64;
                    let high = u32::from_le_bytes([value_bytes[4], value_bytes[5], value_bytes[6], value_bytes[7]]) as u64;
                    let raw = (high << 32) | low;
                    raw as i64
                } else {
                    0i64
                }
            },
            _ => 0i64,
        };
        value_stack.push(value);
        
        if i < 5 || i == type_stack_size - 1 {
            println!("Read value[{}]: {} (0x{:x}) at position {}", i, value, value as u64, cursor);
        } else if i == 5 {
            println!("... (skipping value output) ...");
        }
    }
    println!("Finished reading value stack at position {}", cursor);

    // ラベルスタック
    println!("Starting label stack reading at position {}", cursor);
    let label_stack_size = read_u32(&mut cursor)?;
    println!("Read label_stack_size: {} at position {}", label_stack_size, cursor);
    
    let mut label_stack = Vec::new();
    for i in 0..label_stack_size {
        println!("Reading label[{}] at position {}", i, cursor);
        let begin_addr = read_u32_or_zero(&mut cursor);
        let target_addr = read_u32_or_zero(&mut cursor);
        let sp = read_u32_or_zero(&mut cursor);
        let tsp = read_u32_or_zero(&mut cursor);
        let cell_num = read_u32_or_zero(&mut cursor);
        let count = read_u32_or_zero(&mut cursor);

        let label = Label {
            begin_addr,
            target_addr,
            sp,
            tsp,
            cell_num,
            count,
        };
        println!("Read label[{}]: {} at position {}", i, label, cursor);
        label_stack.push(label);
    }
    
    println!("Finished reading label stack at position {}", cursor);

    // print
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
