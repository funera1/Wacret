# Wacret
Wasm Checkpoint/Restore Tool

## Feature
- Create: 型スタックテーブルの作成
- Display: Wasmの標準インタプリタ、高速インタプリタにおけるコードと各実行地点の型スタックの表示

### Create: 型スタックテーブルの生成
Wasmコードは各命令・ブロックがスタックの操作の型（i32.addなら[i32, i32] -> [i32]）を持つため、すべての実行地点で型スタックが一意に定まる。
このコマンドは、Wasmコードのすべての実行地点の型スタックを求めるためのコマンドである。

本コマンドが生成する型スタックテーブルは、tablemap_func, tablemap_offset, type_tableの3つのファイルから構成される
type_tableは、Wasmコードのすべての実行地点での型スタックをバイナリ形式で格納されている。 
tablemap_funcとtablemap_offsetは、実行地点情報（関数idと命令オフセット）から、対象の型スタックをtype_tableから取得するための情報が保持されている。

### Display: Wasmの標準インタプリタ、高速インタプリタにおけるコードと各実行地点の型スタックの表示
WasmにはWasmコードをそのまま解釈・実行する標準インタプリタとWasmコードからlocal.get/const命令を省略し最適化バイトコードを生成し、その最適化バイトコードを解釈・実行する高速インタプリタが存在する。
このコマンドは、Wasmコードと最適化バイトコードそれぞれの命令と対応関係、型スタックをcsv形式で出力する。
生成されるファイル名はwasm.csvである。csvは以下のように表示される。
![wasm.csv](/docs/images/wasm-csv.png)



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
  display
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
