#![feature(proc_macro_hygiene)]
extern crate parser_macros;
extern crate lazy_static;

use parser_macros::*;

struct Parser;

#[ll1(Expr)]
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

#[test]
fn foo() {
  let mut p = Parser;
  assert_eq!(p.parse(Lexer::new(b"1 - 2 * (3 + 4 * 5 / 6) + -7 * -9 % 10")), Ok(-8));
}
