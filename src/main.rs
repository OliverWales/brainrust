use std::io::Read;

enum OpType {
    Add,
    Sub,
    Next,
    Prev,
    LoopStart,
    LoopEnd,
    Print,
    Read,
}

struct Op {
    op_type: OpType,
    operand: usize,
}

fn main() {
    // let src = "+++++++++++[>++++++>+++++++++>++++++++>++++>+++>+<<<<<<-]>++++++.>++.+++++++..+++.>>.>-.<<-.<.+++.------.--------.>>>+.>-.";
    // let src = "++++++++[>++++++++<-]>[<++++>-]+<[>-<[>++++<-]>[<++++++++>-]<[>++++++++<-]+>[>++++++++++[>+++++<-]>+.-.[-]<<[-]<->] <[>>+++++++[>+++++++<-]>.+++++.[-]<<<-]] >[>++++++++[>+++++++<-]>.[-]<<-]<+++++++++++[>+++>+++++++++>+++++++++>+<<<<-]>-.>-.+++++++.+++++++++++.<.>>.++.+++++++..<-.>>-[[-]<]";
    let src = ",[.,]";

    let mut iter = src.chars().peekable();
    let mut ops: Vec<Op> = vec![];
    let mut stack = vec![];

    while let Some(c) = iter.next() {
        match c {
            '+'|'-'|'>'|'<'|'.'|',' => {
                let op_type = match c {
                    '+' => OpType::Add,
                    '-' => OpType::Sub,
                    '>' => OpType::Next,
                    '<' => OpType::Prev,
                    '.' => OpType::Print,
                    ',' => OpType::Read,
                    _ => continue,
                };

                let mut count = 1;

                while let Some(&c_next) = iter.peek() {
                    if c_next == c {
                        count += 1;
                        iter.next();
                    } else {
                        break;
                    }
                }

                ops.push(Op { op_type, operand: count });
            }
            '[' => {
                stack.push(ops.len());
                ops.push(Op { op_type: OpType::LoopStart, operand: 0 });
            }
            ']' => {
                let start = stack.pop().unwrap();
                ops[start] = Op { op_type: OpType::LoopStart, operand: ops.len() };

                ops.push(Op { op_type: OpType::LoopEnd, operand: start });
            }
            _ => continue,
        }
    }

    let mut tape: Vec<i32> = vec![0i32; 1024];
    let mut ptr = 0usize;
    let mut pc = 0usize;

    while pc < ops.len() {
        match ops[pc].op_type {
            OpType::Add => tape[ptr] += ops[pc].operand as i32,
            OpType::Sub => tape[ptr] -= ops[pc].operand as i32,
            OpType::Next => ptr += ops[pc].operand,
            OpType::Prev => ptr -= ops[pc].operand,
            OpType::LoopStart => {
                if tape[ptr] == 0 {
                    pc = ops[pc].operand;
                }
            }
            OpType::LoopEnd => {
                if tape[ptr] != 0 {
                    pc = ops[pc].operand;
                }
            }
            OpType::Print => {
                for _ in 0..ops[pc].operand {
                    print!("{}", tape[ptr] as u8 as char);
                }
            }
            OpType::Read => {
                for _ in 0..ops[pc].operand {
                    let mut buf = [0u8; 1];
                    std::io::stdin().read_exact(&mut buf).unwrap();
                    tape[ptr] = buf[0] as i32;
                }
            }
        }

        pc += 1;
    }
}
