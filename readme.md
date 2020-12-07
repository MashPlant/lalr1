## Introduction

An LALR1(1)/LL(1) parser generator in Rust, for multiple languages.

Support some yacc/bison features, such as precedence and associativity.

There was a naive `lalr1_by_lr1` implementation, which is removed now. Its efficiency is not too bad, but still significantly slower than yacc/bison. Now a more efficient method `lalr1_by_lr0` is applied. It has about the same speed as yacc/bison. You can refer to the dragon book for the theory about this method.

Currently this repository provided 4 tools that can be used directly, including 2 executable programs and 2 proc macros. They are listed as follow.

## `simple_grammar`: display parsing table

Run `simple_grammar` on a specific example:

```bash
$ cd parser-gen
$ cargo run --example simple_grammar --features="clap" -- examples/expr.cfg -g lalr1 -o expr.dot
# use your favorite dot file viewer to check output "expr.dot"
```

Then use your favorite dot file viewer to view this file, you may get:

<img src="parser-gen/examples/expr.png" width=600 alt="">

Note that you can also use LL(1) grammar in `simple_grammar`, but since I don't know any proper way to show LL(1) table in graphics, it will just show some text information, including first/follow/predict set.

## `parser_gen`: toml to code

Run `parser_gen` on a specific example:

```bash
$ cd parser-gen
# we now support cpp & rust & java, this is a rust example
$ cargo run --bin parser_gen --features="clap toml" -- examples/calc.toml -o calc.rs -l rs
# this is a cpp example
$ cargo run --bin parser_gen --features="clap toml" -- examples/calc_cpp.toml -o calc.cpp -l cpp
# this is a java example
$ cargo run --bin parser_gen --features="clap toml" -- examples/calc_java.toml -o Parser.java -l java
```

Generated file will contain a `struct Parser` and a `struct Lexer`. Their apis are easy to understand. Note that the generated C++ code requires C++17 to compile.

## `#[lalr1]`

Use rust's proc macro to describe the grammar.

The specific api of proc macro is described in [another documentation](https://mashplant.online/2020/08/17/lalr1-introduction/) (in Chinese), which is part of the experiment guide of THU compiling principle course. It will take me too much time if I am to also maintain an English version of this documentation.

See `tests/src/lalr1.rs` to have a glance at the usage.

## `#[ll1]`

Like `#[lalr1]`, but use LL(1) grammar. The parser generator won't try to solve the problem of left recursion or left common factor, nor it will consider precedence and associativity. All have to be done manually. 

`#[ll1]` will generate a `parse(lexer)` function for `Parser`, and it will call `Parser::_parse`, which is supposed to be implemented by the user. When carefully implemented, this can provide some error recovering.

See `tests/src/ll.rs` to have a glance at the usage, note that error recovering is not implemented in this file.
