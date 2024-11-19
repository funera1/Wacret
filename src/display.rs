use crate::core::function::{BytecodeFunction, Function};
use crate::core::module;
use crate::compile;

use camino::Utf8PathBuf;
use anyhow::Result;

pub fn main(path: Utf8PathBuf) -> Result<()> {
    // pathからwasmコードを取得
    let buf: Vec<u8> = std::fs::read(&path).unwrap();
    let m = module::new_module(&buf)?;
    log::debug!("function size is {}", m.funcs.len());

    // wasmコードを配列に詰める

    // wasmコードの型スタックを計算する
    let funcs = m.parse()?;


    // wasmコードから高速バイトコードを生成する
    let compiled_funcs = compile::compile_fast_bytecode();


    // 高速バイトコードを配列に詰める。このとき、高速バイトコードの各命令は、wasmコードの命令と等価な位置(index)へ格納する
    // 高速バイトコードの型スタックを計算する
    // それぞれのバイトコードと型スタックをhuman-friendryな形式で表示する
    return Ok(());
}