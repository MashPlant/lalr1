A LALR(1) parser generator in & for rust.

Support some yacc/bison features, e.g., precedence and associativity(though the syntax is a little different).

Implement LALR(1) by naive `lalr1_by_lr1` and `lalr1_by_lr0`. To generate the parser of [decaf](https://github.com/MashPlant/decaf), it takes about 2s and 0.2s respectively.

Future extension: 

- [ ] multiple language codegen, e.g., cpp
- [x] using other parsing technique, e.g., LL(1)
- [x] use DFA to implement lexical analysis, instead of using many regular expressions now

---

## Codegen tool: toml to rust

Quite a traditional approach. Use a toml to describe the grammar, and generate it to source code of rust. The generated code has no extra dependency.

## #[lalr1]

Use rust's proc macro to describe the grammar.

```rust
#![feature(proc_macro_hygiene)]
extern crate parser_macros;

use parser_macros::lalr1;

struct Parser;

// why am I using toml here for lexer?
// because rust attributes have no advantage in describing the lexer over toml

#[lalr1(Expr)]
#[lex(r#"
priority = [
  { assoc = 'left', terms = ['Add', 'Sub'] },
  { assoc = 'left', terms = ['Mul', 'Div', 'Mod'] },
  { assoc = 'no_assoc', terms = ['UMinus'] },
  { assoc = 'no_assoc', terms = ['RParen'] },
]

[lexical]
'\(' = 'LParen'
'\)' = 'RParen'
'\d+' = 'IntConst'
'\+' = 'Add'
'-' = 'Sub'
'\*' = 'Mul'
'/' = 'Div'
'%' = 'Mod'
'\d+' = 'IntConst'
'\s+' = '_Eps'
"#)]
impl Parser {
  #[rule(Expr -> Expr Add Expr)]
  fn expr_add(l: i32, _op: Token<'_>, r: i32) -> i32 {
    println!("Add {} {}", l, r);
    l + r
  }

  #[rule(Expr -> Expr Sub Expr)]
  fn expr_sub(l: i32, _op: Token<'_>, r: i32) -> i32 {
    println!("Sub {} {}", l, r);
    l - r
  }

  #[rule(Expr -> Expr Mul Expr)]
  fn expr_mul(l: i32, _op: Token<'_>, r: i32) -> i32 {
    println!("Mul {} {}", l, r);
    l * r
  }

  #[rule(Expr -> Expr Div Expr)]
  fn expr_div(l: i32, _op: Token<'_>, r: i32) -> i32 {
    println!("Div {} {}", l, r);
    l / r
  }

  #[rule(Expr -> Expr Mod Expr)]
  fn expr_mod(l: i32, _op: Token<'_>, r: i32) -> i32 {
    println!("Mod {} {}", l, r);
    l % r
  }

  #[rule(Expr -> Sub Expr)]
  #[prec(UMinus)]
  fn expr_neg(_op: Token<'_>, r: i32) -> i32 {
    println!("Neg {}", r);
    -r
  }

  #[rule(Expr -> LParen Expr RParen)]
  fn expr_paren(_l: Token<'_>, i: i32, _r: Token<'_>) -> i32 {
    println!("Paren {}", i);
    i
  }

  #[rule(Expr -> IntConst)]
  fn expr_int(i: Token<'_>) -> i32 {
    let int = std::str::from_utf8(i.piece).unwrap().parse::<i32>().unwrap();
    println!("Int {}", int);
    int
  }
}

let mut p = Parser;
assert_eq!(p.parse(Lexer::new(b"1 - 2 * (3 + 4 * 5 / 6) + -7 * -9 % 10")), Ok(-8));
```

A pity is that I don't know how to expand macro in proc macro. If so, we can simply write

```rust
bin!(expr_add, Add, +);
bin!(expr_sub, Sub, -);
...
```

which is terser.

## #[ll1]
An similar example, notice that the parser generator won't try to solve the problem of left recursion or left common factor, also priority will be ignored. All have to be done manually. 

```rust
#![feature(proc_macro_hygiene)]
extern crate parser_macros;

use parser_macros::ll1;

struct Parser;

pub enum Op {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
}

#[ll1(Expr)]
#[lex(r#"
priority = []

[lexical]
'\(' = 'LParen'
'\)' = 'RParen'
'\d+' = 'IntConst'
'\+' = 'Add'
'-' = 'Sub'
'\*' = 'Mul'
'/' = 'Div'
'%' = 'Mod'
'\d+' = 'IntConst'
'\s+' = '_Eps'
"#)]
impl Parser {
  #[rule(Expr -> Term1 Expr1)]
  fn r0(mut t: i32, remain: Vec<(Op, i32)>) -> i32 {
    for (op, i) in remain.into_iter().rev() {
      match op {
        Op::Add => t += i,
        Op::Sub => t -= i,
        _ => unreachable!(),
      }
    }
    t
  }

  #[rule(Expr1 -> Add Term1 Expr1)]
  fn r1(_op: Token<'_>, t: i32, mut remain: Vec<(Op, i32)>) -> Vec<(Op, i32)> {
    remain.push((Op::Add, t));
    remain
  }

  #[rule(Expr1 -> Sub Term1 Expr1)]
  fn r2(_op: Token<'_>, t: i32, mut remain: Vec<(Op, i32)>) -> Vec<(Op, i32)> {
    remain.push((Op::Sub, t));
    remain
  }

  #[rule(Expr1 ->)]
  fn r3() -> Vec<(Op, i32)> {
    vec![]
  }

  #[rule(Term1 -> Term2 Expr2)]
  fn r4(mut t: i32, remain: Vec<(Op, i32)>) -> i32 {
    for (op, i) in remain.into_iter().rev() {
      match op {
        Op::Mul => t *= i,
        Op::Div => t /= i,
        Op::Mod => t %= i,
        _ => unreachable!(),
      }
    }
    t
  }

  #[rule(Expr2 -> Mul Term2 Expr2)]
  fn r5(_op: Token<'_>, t: i32, mut remain: Vec<(Op, i32)>) -> Vec<(Op, i32)> {
    remain.push((Op::Mul, t));
    remain
  }

  #[rule(Expr2 -> Div Term2 Expr2)]
  fn r6(_op: Token<'_>, t: i32, mut remain: Vec<(Op, i32)>) -> Vec<(Op, i32)> {
    remain.push((Op::Div, t));
    remain
  }

  #[rule(Expr2 -> Mod Term2 Expr2)]
  fn r7(_op: Token<'_>, t: i32, mut remain: Vec<(Op, i32)>) -> Vec<(Op, i32)> {
    remain.push((Op::Mod, t));
    remain
  }

  #[rule(Expr2 ->)]
  fn r8() -> Vec<(Op, i32)> {
    vec![]
  }

  #[rule(Term2 -> IntConst)]
  fn r9(i: Token<'_>) -> i32 {
    let int = std::str::from_utf8(i.piece).unwrap().parse::<i32>().unwrap();
    int
  }

  #[rule(Term2 -> Sub Term2)]
  fn r10(i: Token<'_>, r: i32) -> i32 {
    -r
  }

  #[rule(Term2 -> LParen Expr RParen)]
  fn r11(_l: Token<'_>, x: i32, _r: Token<'_>) -> i32 {
    x
  }
}
```

This will generate a `parse(lexer)` function for `Parser`, and it will call `Parser::_parse`, which is supposed to be implemented by the user. When carefully implemented, this can provide some error recovering. An incomplete example can be found at test/src/ll1.rs.

## What does lr fsm looks like

example:

grammar:

```
E -> E + F
E -> E - F
E -> F
F -> F * num
F -> F / num
F -> F % num
F -> num
F -> ( E )
```

picture:

lr0:

![lr0](./pic/lr0.png)

lr1:

![lr1](./pic/lr1.png)

lalr1:

![lalr1](./pic/lalr1.png)
