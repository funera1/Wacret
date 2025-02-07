# Wacret
Wasm Checkpoint/Restore Tool

## Feature
- Create: 型スタックテーブルの作成

### Create: 型スタックテーブルの生成
Wasmコードは各命令・ブロックがスタックの操作の型（i32.addなら[i32, i32] -> [i32]）を持つため、すべての実行地点で型スタックが一意に定まる。
このコマンドは、Wasmコードのすべての実行地点の型スタックを求めるためのコマンドである。

本コマンドが生成する型スタックテーブルは、tablemap_func, tablemap_offset, type_tableの3つのファイルから構成される
type_tableは、Wasmコードのすべての実行地点での型スタックをバイナリ形式で格納されている。 
tablemap_funcとtablemap_offsetは、実行地点情報（関数idと命令オフセット）から、対象の型スタックをtype_tableから取得するための情報が保持されている。

```
"tablemap_func format"
関数fについて
 - 関数fの関数id(u32)
 - tablemap_offsetにおける関数fセクションの先頭アドレス(u64)
```
```
"tablemap_offset format"
関数fについて
 - fのローカルの長さ(u32)
 - fのローカル (local.len * u8)
 - 各コード位置について
     - offset  (u32)
     - type_tableにおける関数f,命令オフセットoの型スタックがあるアドレス (u64)
```
```
"type_table format"
各コード位置について
 - 型スタックの長さ (u32)
 - 型スタックの中身 (stack.len * u8)
但し、OpcodeがCallの場合は、「呼び出し途中」と「呼び出し後」の2パターン書く
```

## Build
```
git clone git@github.com:funera1/Wacret.git
cd Wacret
cargo build
```

## Usage
```
Usage: wacret <COMMAND>

Commands:
  create  Create type stack tables for checkpointing a wasm app
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
