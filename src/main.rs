use std::fs::File;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
enum OpType {
    Add,
    Sub,
    Next,
    Prev,
    LoopStart,
    LoopEnd,
    Print,
    Read,
    Clear,
}

#[derive(Debug)]
struct Op {
    op_type: OpType,
    operand: usize,
}

fn preoptimise(src: &str) -> String {
    // Here we introduce a new operation, 0, which sets the current cell to 0.
    // Since [-] does this, we can replace it with 0
    src.replace("[-]", "0")
}

fn parse(src: &str) -> Vec<Op> {
    let mut iter = src.chars().peekable();
    let mut ops: Vec<Op> = vec![];
    let mut loops = 0usize;
    let mut loop_stack: Vec<usize> = vec![];

    while let Some(c) = iter.next() {
        match c {
            '+' | '-' | '>' | '<' | '.' | ',' | '0' => {
                let op_type = match c {
                    '+' => OpType::Add,
                    '-' => OpType::Sub,
                    '>' => OpType::Next,
                    '<' => OpType::Prev,
                    '.' => OpType::Print,
                    ',' => OpType::Read,
                    '0' => OpType::Clear,
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

                ops.push(Op {
                    op_type,
                    operand: count,
                });
            }
            '[' => {
                ops.push(Op {
                    op_type: OpType::LoopStart,
                    operand: loops,
                });
                loop_stack.push(loops);
                loops += 1;
            }
            ']' => {
                ops.push(Op {
                    op_type: OpType::LoopEnd,
                    operand: loop_stack.pop().unwrap(),
                });
            }
            _ => continue,
        }
    }

    ops
}

fn print(ops: &[Op]) {
    for op in ops {
        println!("{:?} {}", op.op_type, op.operand);
    }
}

fn interpret(ops: &[Op]) {
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
                    let dest = ops[pc].operand;
                    let index = ops.iter().position(|op| match op.op_type {
                        OpType::LoopEnd => op.operand == dest,
                        _ => false,
                    });

                    pc = index.unwrap();
                }
            }
            OpType::LoopEnd => {
                if tape[ptr] != 0 {
                    let dest = ops[pc].operand;
                    let index = ops.iter().position(|op| match op.op_type {
                        OpType::LoopStart => op.operand == dest,
                        _ => false,
                    });

                    pc = index.unwrap();
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
            OpType::Clear => {
                tape[ptr] = 0;
            }
        }

        pc += 1;
    }
}

fn compile(ops: &[Op]) -> String {
    // x1 = ptr
    // x0 = current cell = [x1]

    let mut code = String::new();

    let load = |c: &mut String| c.push_str("\tldr x0, [x1]\n");
    let store = |c: &mut String| c.push_str("\tstr x0, [x1]\n");

    let add = |c: &mut String, operand: usize| c.push_str(&format!("\tadd x0, x0, #{}\n", operand));
    let sub = |c: &mut String, operand: usize| c.push_str(&format!("\tsub x0, x0, #{}\n", operand));

    let add_ptr = |c: &mut String, operand: usize| {
        c.push_str(&format!("\tadd x1, x1, #{}\n", operand as i32 * 8))
    };
    let sub_ptr = |c: &mut String, operand: usize| {
        c.push_str(&format!("\tsub x1, x1, #{}\n", operand as i32 * 8))
    };

    let push_x1 = |c: &mut String| c.push_str("\tstr x1, [sp, #-16]!\n");
    let pop_x1 = |c: &mut String| c.push_str("\tldr x1, [sp], #16\n");

    code.push_str(".globl _main\n");
    code.push_str(".align 4\n");
    code.push('\n');
    code.push_str("_main:\n");

    code.push_str("\t// Allocate tape memory initialized to zero\n");
    code.push_str("\tmov x0, #1024\n");
    code.push_str("\tbl _calloc\n"); // TODO: check success
    code.push_str("\tmov x1, x0\n");
    code.push_str("\tmov x0, #0\n");
    code.push('\n');

    for op in ops {
        code.push_str(&format!("\t// {:?} {}\n", op.op_type, op.operand));

        match op.op_type {
            OpType::Add => {
                add(&mut code, op.operand);
            }
            OpType::Sub => {
                sub(&mut code, op.operand);
            }
            OpType::Next => {
                store(&mut code);
                add_ptr(&mut code, op.operand);
                load(&mut code);
            }
            OpType::Prev => {
                store(&mut code);
                sub_ptr(&mut code, op.operand);
                load(&mut code);
            }
            OpType::LoopStart => {
                // If the byte at the data pointer is zero jump forward to the
                // command after the matching ']'
                code.push_str("\tcmp x0, #0\n");
                code.push_str(&format!("\tbeq l{}_end\n", op.operand));
                code.push_str(&format!("l{}_start:\n", op.operand));
            }
            OpType::LoopEnd => {
                // If the byte at the data pointer is nonzero jump back to the
                // command after the matching '['
                code.push_str("\tcmp x0, #0\n");
                code.push_str(&format!("\tbne l{}_start\n", op.operand));
                code.push_str(&format!("l{}_end:\n", op.operand));
            }
            OpType::Print => {
                for _ in 0..op.operand {
                    push_x1(&mut code);
                    // Print tape[ptr] as char
                    code.push_str("\tbl _putchar\n");
                    pop_x1(&mut code);
                }
            }
            OpType::Read => {
                push_x1(&mut code);
                // Read char to x0
                code.push_str("\tbl _getchar\n");
                pop_x1(&mut code);
            }
            OpType::Clear => {
                code.push_str("\tmov x0, #0\n");
            }
        }

        code.push('\n');
    }

    // TODO: free tape memory

    code.push_str("\t// Exit\n");
    code.push_str("\tmov x0, 0\n");
    code.push_str("\tbl _exit\n");

    code
}

fn main() {
    let _hello_world = "+++++++++++[>++++++>+++++++++>++++++++>++++>+++>+<<<<<<-]>++++++.>++.+++++++..+++.>>.>-.<<-.<.+++.------.--------.>>>+.>-.";
    let _cell_size = "++++++++[>++++++++<-]>[<++++>-]+<[>-<[>++++<-]>[<++++++++>-]<[>++++++++<-]+>[>++++++++++[>+++++<-]>+.-.[-]<<[-]<->]<[>>+++++++[>+++++++<-]>.+++++.[-]<<<-]]>[>++++++++[>+++++++<-]>.[-]<<-]<+++++++++++[>+++>+++++++++>+++++++++>+<<<<-]>-.>-.+++++++.+++++++++++.<.>>.++.+++++++..<-.>>-[[-]<]";
    let _echo = ",[.,]";

    let src = _hello_world.to_owned() + _cell_size + _echo;
    let opt = preoptimise(&src);

    let ops = parse(&opt);

    // println!("Parsing code");
    // println!("------------");
    // print(&ops);
    // println!();

    // println!("Interpreting code");
    // println!("-----------------");
    // interpret(&ops);
    // println!();
    // println!();

    println!("Compiling code");
    println!("--------------");

    let res = compile(&ops);

    let lines = res
        .split('\n')
        .filter(|x| x.starts_with('\t') && !x.starts_with("\t//"))
        .count();

    println!("Generated {} instructions", lines);
    // write res to file
    let mut file = File::create("out.s").unwrap();
    file.write_all(res.as_bytes()).unwrap();
}
