#![feature(proc_macro_hygiene)]
extern crate parser_macros;

use parser_macros::lalr1;

pub struct Parser;

#[lalr1(StmtList)]
#[log_reduce]
#[lex(r##"
priority = [
]

[lexical]
';' = "S"
'\.' = "Dot"
'\s+' = "_Eps"
"[A-Za-z][_0-9A-Za-z]*" = "Identifier"
"##)]
impl Parser {
  #[rule(StmtList -> StmtList Expr S)]
  fn stmt_list(_1: (), _2: (), _3: Token) -> () {}

  #[rule(StmtList ->)]
  fn stmt_list0() -> () {}

  #[rule(Expr -> MaybeOwner Identifier)]
  fn expr_lvalue(owner: (), name: Token) -> () {}

  #[rule(MaybeOwner -> Expr Dot)]
  fn maybe_owner1(e: (), _d: Token) -> () {}

  #[rule(MaybeOwner ->)]
  fn maybe_owner0() -> () {}
}

#[test]
fn test() {
  match Parser.parse(Lexer::new(br###"a;a;a;a;a;"###)) {
    Ok(_) => println!("Ok"),
    Err(token) => println!("Err at {:?}", token),
  }
}

