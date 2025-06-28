#!/usr/bin/python3
import sys

class Label:
    def __init__(self, begin_addr, target_addr, sp, tsp, cell_num, count):
        self.begin_addr = begin_addr
        self.target_addr = target_addr
        self.sp = sp
        self.tsp = tsp
        self.cell_num = cell_num
        self.count = count
    def __str__(self):
        return '{%d, %d, %d, %d, %d, %d}' % (self.begin_addr, self.target_addr, self.sp, self.tsp, self.cell_num, self.count)

def to_int(b: bytes):
    a = int.from_bytes(b, 'little')
    if a > 0xffff0000:
        return a - 0xffffffff
    else:
        return a

def parse(file: str):
    with open(file, 'rb') as f:
        # エントリー関数
        entry_fidx = to_int(f.read(4))
        # リターンアドレス
        return_fidx = to_int(f.read(4))
        return_offset = to_int(f.read(4))
        # 型スタック
        type_stack_size = to_int(f.read(4))
        type_stack = []
        for i in range(type_stack_size):
            type = to_int(f.read(1))
            type_stack.append(type)

        # 値スタック
        value_stack = []
        for i in range(type_stack_size):
            value = to_int(f.read(4 * type_stack[i]))
            value_stack.append(value)

        # TODO: ラベルスタック
        label_stack_size = to_int(f.read(4))
        label_stack = []
        for i in range(label_stack_size):
            begin_addr = to_int(f.read(4))
            target_addr = to_int(f.read(4))
            sp = to_int(f.read(4))
            tsp = to_int(f.read(4))
            cell_num = to_int(f.read(4))
            count = to_int(f.read(4))

            label = Label(begin_addr, target_addr, sp, tsp, cell_num, count)
            label_stack.append(label)


        # print
        print(f"EntryFuncIdx: {entry_fidx}")
        print(f"ReturnAddress: {return_fidx}, {return_offset}")
        print(f"StackSize: {type_stack_size}")
        print(f"TypeStack: {type_stack}")
        print(f"ValueStack: {value_stack}")
        print(f"LabelStackSize: {label_stack_size}")
        print("LabelStack: [")
        for label in label_stack:
            print(f"\t{label}")
        print("]")

if __name__ == '__main__':
    file = sys.argv[1]
    parse(file)
