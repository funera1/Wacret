use anyhow::Result;
use camino::Utf8PathBuf;
use prost::Message;
use serde_json;
use std::{any, fs};

use crate::command::view::utils::{code_pos_to_json, label_stack_to_json, typed_array_to_json};
use crate::command::view::utils::state::{CallStack, CallStackEntry, CodePos};

pub fn parse_protobuf(path: &Utf8PathBuf) -> Result<String> {
    // Read the protobuf file
    let data = fs::read(&path)?;

    // Try CallStack first (most likely to be the top-level message)
    if let Ok(call_stack) = CallStack::decode(&data[..]) {
        let json = serde_json::to_string_pretty(&call_stack_to_json(&call_stack))?;
        return Ok(json);
    }
    
    anyhow::bail!("Unable to decode protobuf file as any known message type");
}

pub fn view_protobuf(path: Utf8PathBuf) -> Result<()> {
    let json = parse_protobuf(&path)?;
    println!("{}", json);
    Ok(())
}

fn call_stack_to_json(call_stack: &CallStack) -> serde_json::Value {
    serde_json::json!({
        "entries": call_stack.entries.iter().map(call_stack_entry_to_json).collect::<Vec<_>>()
    })
}

fn call_stack_entry_to_json(entry: &CallStackEntry) -> serde_json::Value {
    serde_json::json!({
        "pc": entry.pc.as_ref().map(code_pos_to_json),
        "locals": entry.locals.as_ref().map(typed_array_to_json),
        "value_stack": entry.value_stack.as_ref().map(typed_array_to_json),
        "label_stack": entry.label_stack.as_ref().map(label_stack_to_json)
    })
}