use crate::core::function_v2::Function;
use crate::core::stack_table::{StackTable, StEntry};
use crate::core::module;

use camino::Utf8PathBuf;

use std::fs::File;
use std::io::Write;
use anyhow::Result;
use rmp_serde::encode::to_vec_named;

pub fn create_table_v2(path: Utf8PathBuf) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&path).unwrap();

    // コードから各セクションの情報を抽出
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // 関数クラスを初期化
    let funcs = m.new_function_v2()?;
    
    // 型スタック・命令スタックテーブルを生成
    let stack_tables= funcs.iter().map(|f| {
        match f {
            Function::ImportFunction(_) => {
                return Vec::new();
            }
            Function::BytecodeFunction(f) => {
                return f.create_stack_table().expect("failed to create stack table");
            }
        }
    }).collect::<Vec<_>>();
    
    // Vec<Vec<CodePos>>からStackTableに変換
    let stack_tables: Vec<StackTable> = stack_tables
        .into_iter()
        .map(|st| {
            let inner = st.into_iter().map(|entry| StEntry::from_codepos(entry)).collect();
            StackTable::new(inner)
        })
        .collect();
    
    
    // stack_tableをserialize
    let buf = to_vec_named(&stack_tables).unwrap();
    
    // bufをファイルに書き込む
    let mut f: File = File::create("stack-table")?;
    f.write(buf.as_slice())?;
    log::debug!("write type_table");


    Ok(())
}
