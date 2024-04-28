# brainrust

A simple brainfuck interpreter + compiler in rust. Compiles to Apple Silicon (arm64) assembly.

To run:

- Set the variable src equal to the brainfuck program you want to interpret + compile
- Run `cargo run && gcc out.s -o out.o && ./out.o`
