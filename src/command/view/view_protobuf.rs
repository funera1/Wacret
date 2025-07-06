use anyhow::Result;
use camino::Utf8PathBuf;
use prost::Message;
use serde_json;
use std::{fs};

use crate::command::view::utils::state::CallStack;
use crate::command::view::utils::UnifiedFormat;

pub fn parse_protobuf(path: &Utf8PathBuf, merged_stack: bool) -> Result<Vec<UnifiedFormat>> {
    // Read the protobuf file
    let data = fs::read(&path)?;

    // Try CallStack first (most likely to be the top-level message)
    if let Ok(call_stack) = CallStack::decode(&data[..]) {
        return Ok(call_stack.entries.iter().map(|entry| {
            let locals = entry.locals.as_ref().and_then(|locals| 
                locals.values.as_ref().map(|array| array.contents.iter().map(|&v| v as i64).collect())
            ).unwrap_or_else(Vec::new);
            
            let value_stack = entry.value_stack.as_ref().and_then(|stack| 
                stack.values.as_ref().map(|array| array.contents.iter().map(|&v| v as i64).collect())
            ).unwrap_or_else(Vec::new);

            if merged_stack {
                let merged_values = locals.into_iter()
                    .chain(value_stack)
                    .collect::<Vec<_>>();
                UnifiedFormat {
                    pc: entry.pc.as_ref().map(|pc| (pc.fidx, pc.offset)),
                    return_address: None, // Protobuf v2 does not have return_address
                    locals: None,
                    value_stack: if merged_values.is_empty() { None } else { Some(merged_values) },
                    label_stack: entry.label_stack.as_ref().map(|stack| stack.begins.clone()),
                    type_stack: None, // Protobuf v2 does not have type_stack
                }
            } else {
                UnifiedFormat {
                    pc: entry.pc.as_ref().map(|pc| (pc.fidx, pc.offset)),
                    return_address: None, // Protobuf v2 does not have return_address
                    locals: if locals.is_empty() { None } else { Some(locals) },
                    value_stack: if value_stack.is_empty() { None } else { Some(value_stack) },
                    label_stack: entry.label_stack.as_ref().and_then(|stack| 
                        if stack.begins.is_empty() { None } else { Some(stack.begins.clone()) }
                    ),
                    type_stack: None, // Protobuf v2 does not have type_stack
                }
            }
        }).collect());
    }

    anyhow::bail!("Unable to decode protobuf file as any known message type");
}

pub fn view_protobuf(path: Utf8PathBuf, merged_stack: bool) -> Result<()> {
    let unified_format = parse_protobuf(&path, merged_stack)?;
    let pretty_json = serde_json::to_string_pretty(&unified_format)?;
    println!("{}", pretty_json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::view::utils::state::{CallStack, CallStackEntry, CodePos, TypedArray, Array32, LabelStack};
    use prost::Message;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_call_stack() -> CallStack {
        CallStack {
            entries: vec![
                CallStackEntry {
                    pc: Some(CodePos {
                        fidx: 1,
                        offset: 100,
                    }),
                    locals: Some(TypedArray {
                        types: None,
                        values: Some(Array32 {
                            contents: vec![10, 20, 30],
                        }),
                    }),
                    value_stack: Some(TypedArray {
                        types: None,
                        values: Some(Array32 {
                            contents: vec![40, 50],
                        }),
                    }),
                    label_stack: Some(LabelStack {
                        begins: vec![1000, 2000],
                        targets: vec![],
                        stack_pointers: vec![],
                        cell_nums: vec![],
                    }),
                },
                CallStackEntry {
                    pc: Some(CodePos {
                        fidx: 2,
                        offset: 200,
                    }),
                    locals: Some(TypedArray {
                        types: None,
                        values: Some(Array32 {
                            contents: vec![60],
                        }),
                    }),
                    value_stack: Some(TypedArray {
                        types: None,
                        values: Some(Array32 {
                            contents: vec![70, 80, 90],
                        }),
                    }),
                    label_stack: Some(LabelStack {
                        begins: vec![3000],
                        targets: vec![],
                        stack_pointers: vec![],
                        cell_nums: vec![],
                    }),
                },
            ],
        }
    }

    fn create_test_protobuf_file() -> Result<NamedTempFile> {
        let call_stack = create_test_call_stack();
        let encoded = call_stack.encode_to_vec();
        
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&encoded)?;
        temp_file.flush()?;
        
        Ok(temp_file)
    }

    #[test]
    fn test_parse_protobuf_separated_stacks() -> Result<()> {
        let temp_file = create_test_protobuf_file()?;
        let path = Utf8PathBuf::from_path_buf(temp_file.path().to_path_buf()).unwrap();
        
        let result = parse_protobuf(&path, false)?;
        
        assert_eq!(result.len(), 2);
        
        // First entry
        assert_eq!(result[0].pc, Some((1, 100)));
        assert_eq!(result[0].locals, Some(vec![10, 20, 30]));
        assert_eq!(result[0].value_stack, Some(vec![40, 50]));
        assert_eq!(result[0].label_stack, Some(vec![1000, 2000]));
        assert_eq!(result[0].return_address, None);
        assert_eq!(result[0].type_stack, None);
        
        // Second entry
        assert_eq!(result[1].pc, Some((2, 200)));
        assert_eq!(result[1].locals, Some(vec![60]));
        assert_eq!(result[1].value_stack, Some(vec![70, 80, 90]));
        assert_eq!(result[1].label_stack, Some(vec![3000]));
        assert_eq!(result[1].return_address, None);
        assert_eq!(result[1].type_stack, None);
        
        Ok(())
    }

    #[test]
    fn test_parse_protobuf_merged_stacks() -> Result<()> {
        let temp_file = create_test_protobuf_file()?;
        let path = Utf8PathBuf::from_path_buf(temp_file.path().to_path_buf()).unwrap();
        
        let result = parse_protobuf(&path, true)?;
        
        assert_eq!(result.len(), 2);
        
        // First entry - locals and value_stack should be merged
        assert_eq!(result[0].pc, Some((1, 100)));
        assert_eq!(result[0].locals, None);
        assert_eq!(result[0].value_stack, Some(vec![10, 20, 30, 40, 50])); // locals + value_stack
        assert_eq!(result[0].label_stack, Some(vec![1000, 2000]));
        
        // Second entry - locals and value_stack should be merged
        assert_eq!(result[1].pc, Some((2, 200)));
        assert_eq!(result[1].locals, None);
        assert_eq!(result[1].value_stack, Some(vec![60, 70, 80, 90])); // locals + value_stack
        assert_eq!(result[1].label_stack, Some(vec![3000]));
        
        Ok(())
    }

    #[test]
    fn test_parse_protobuf_empty_stacks() -> Result<()> {
        let call_stack = CallStack {
            entries: vec![
                CallStackEntry {
                    pc: Some(CodePos {
                        fidx: 1,
                        offset: 100,
                    }),
                    locals: None,
                    value_stack: None,
                    label_stack: None,
                },
            ],
        };
        
        let encoded = call_stack.encode_to_vec();
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(&encoded)?;
        temp_file.flush()?;
        
        let path = Utf8PathBuf::from_path_buf(temp_file.path().to_path_buf()).unwrap();
        
        // Test separated mode
        let result = parse_protobuf(&path, false)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].locals, None);
        assert_eq!(result[0].value_stack, None);
        assert_eq!(result[0].label_stack, None);
        
        // Test merged mode
        let result = parse_protobuf(&path, true)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].locals, None);
        assert_eq!(result[0].value_stack, None);
        assert_eq!(result[0].label_stack, None);
        
        Ok(())
    }

    #[test]
    fn test_parse_protobuf_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid protobuf data").unwrap();
        temp_file.flush().unwrap();
        
        let path = Utf8PathBuf::from_path_buf(temp_file.path().to_path_buf()).unwrap();
        
        let result = parse_protobuf(&path, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unable to decode protobuf file"));
    }
}