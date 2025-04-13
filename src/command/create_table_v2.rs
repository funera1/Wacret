use crate::core::stack_table::StackTables;
use crate::core::module;

use camino::Utf8PathBuf;

use std::fs::File;
use std::io::Write;
use anyhow::Result;

pub fn create_table_v2(path: Utf8PathBuf) -> Result<()> {
    let buf: Vec<u8> = std::fs::read(&path).unwrap();

    // コードから各セクションの情報を抽出
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // 関数クラスを初期化
    let funcs = m.new_function_v2()?;
    
    // 型スタック・命令スタックテーブルを生成
    let stack_tables = StackTables::from_func(funcs)?;
    
    // stack_tableをserialize
    let buf = stack_tables.serialize();
    
    // bufをファイルに書き込む
    let mut f: File = File::create("stack-table.msgpack")?;
    f.write(buf.as_slice())?;
    log::debug!("write type_table");

    println!("write stack table to stack-table.msgpack");

    Ok(())
}
