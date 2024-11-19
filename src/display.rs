use camino::Utf8PathBuf;
use anyhow::Result;

pub fn main(path: Utf8PathBuf) -> Result<()> {
    // wasmコードを配列に詰める
    // wasmコードの型スタックを計算する
    // wasmコードから高速バイトコードを生成する
    // 高速バイトコードを配列に詰める。このとき、高速バイトコードの各命令は、wasmコードの命令と等価な位置(index)へ格納する
    // 高速バイトコードの型スタックを計算する
    // それぞれのバイトコードと型スタックをhuman-friendryな形式で表示する
    return Ok(());
}