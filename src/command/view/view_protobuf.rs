use anyhow::Result;
use camino::Utf8PathBuf;
use prost::Message;
use serde_json;
use std::{fs};

use crate::command::view::utils::state::CallStack;
use crate::command::view::utils::UnifiedFormat;

pub fn parse_protobuf(path: &Utf8PathBuf) -> Result<Vec<UnifiedFormat>> {
    // Read the protobuf file
    let data = fs::read(&path)?;

    // Try CallStack first (most likely to be the top-level message)
    if let Ok(call_stack) = CallStack::decode(&data[..]) {
        return Ok(call_stack.entries.iter().map(|entry| {
            UnifiedFormat {
                pc: entry.pc.as_ref().map(|pc| (pc.fidx, pc.offset)),
                locals: None,
                value_stack: Some(
                    entry.locals.as_ref().and_then(|locals| locals.values.as_ref().map(|array| array.contents.iter().map(|&v| v as i64).collect()))
                        .unwrap_or_else(Vec::new)
                        .into_iter()
                        .chain(
                            entry.value_stack.as_ref().and_then(|stack| stack.values.as_ref().map(|array| array.contents.iter().map(|&v| v as i64).collect()))
                                .unwrap_or_else(Vec::new)
                        )
                        .collect()
                ),
                label_stack: entry.label_stack.as_ref().map(|stack| stack.begins.clone()),
                type_stack: None, // Protobuf v2 does not have type_stack
            }
        }).collect());
    }

    anyhow::bail!("Unable to decode protobuf file as any known message type");
}

pub fn view_protobuf(path: Utf8PathBuf) -> Result<()> {
    let unified_format = parse_protobuf(&path)?;
    let pretty_json = serde_json::to_string_pretty(&unified_format)?;
    println!("{}", pretty_json);
    Ok(())
}