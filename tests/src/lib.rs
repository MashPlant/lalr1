#![feature(proc_macro_hygiene)]
extern crate lalr1_macro;

use lalr1_macro::lalr1;

#[test]
fn foo() {
  struct Parser;

  #[lalr1(Expr)]
  #[term(IntConst(r"\d+"))]
  #[term(_Eps(r"\s+"))]
  #[term(Add(r"\+" left) Sub("-" left))]
  #[term(Div("/" left) Mul(r"\*" left))]
  #[term(RParen(r"\)" no_assoc))]
  #[term(LParen(r"\("))]
  #[term(UMinus(no_assoc))]
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
  assert_eq!(p.parse(Lexer::new(b"1 - 2 * (3 + 4 * 5 / 6) + 7")), Ok(-4));
}
