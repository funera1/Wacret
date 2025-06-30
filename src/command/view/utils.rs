use anyhow::Result;
use serde::Serialize;

// Include the generated protobuf code
pub mod state {
    include!(concat!(env!("OUT_DIR"), "/state.rs"));
}

#[derive(Debug, Clone, Serialize)]
pub struct Label {
    pub begin_addr: u32,
    pub target_addr: u32,
    pub sp: u32,
    pub tsp: u32,
    pub cell_num: u32,
    pub count: u32,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}, {}, {}, {}, {}, {}}}", 
               self.begin_addr, self.target_addr, self.sp, 
               self.tsp, self.cell_num, self.count)
    }
}

#[derive(Debug, Serialize)]
pub struct UnifiedFormat {
    pub pc: Option<(u32, u64)>,
    pub return_address: Option<(u32, u64)>,
    pub locals: Option<Vec<i64>>,
    pub value_stack: Option<Vec<i64>>,
    pub label_stack: Option<Vec<u32>>,
    pub type_stack: Option<Vec<u8>>,
}

/// Convert bytes to integer value, mimicking Python's to_int function
pub fn bytes_to_int(bytes: &[u8]) -> Result<i64> {
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
pub fn read_u32(cursor: &mut usize, data: &[u8]) -> Result<u32> {
    if *cursor + 4 > data.len() {
        anyhow::bail!("Unexpected end of file while reading u32");
    }
    let bytes = &data[*cursor..*cursor + 4];
    *cursor += 4;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Helper function to read a u32 value or return 0 if at the end of the file
pub fn read_u32_or_zero(cursor: &mut usize, data: &[u8]) -> u32 {
    if *cursor + 4 > data.len() {
        0
    } else {
        let bytes = &data[*cursor..*cursor + 4];
        *cursor += 4;
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

/// Helper function to read a u8 value from binary data
pub fn read_u8(cursor: &mut usize, data: &[u8]) -> Result<u8> {
    if *cursor >= data.len() {
        anyhow::bail!("Unexpected end of file while reading u8");
    }
    let byte = data[*cursor];
    *cursor += 1;
    Ok(byte)
}
