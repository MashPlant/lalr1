use parser_macros::lalr1;

#[allow(dead_code)]
struct Parser;

#[lalr1(Expr)]
#[log_reduce]
#[log_token]
#[show_fsm("a.dot")]
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
  fn expr_add(l: i32, _op: Token<'_>, r: i32) -> i32 { l + r }
  #[rule(Expr -> Expr Sub Expr)]
  fn expr_sub(l: i32, _op: Token<'_>, r: i32) -> i32 { l - r }
  #[rule(Expr -> Expr Mul Expr)]
  fn expr_mul(l: i32, _op: Token<'_>, r: i32) -> i32 { l * r }
  #[rule(Expr -> Expr Div Expr)]
  fn expr_div(l: i32, _op: Token<'_>, r: i32) -> i32 { l / r }
  #[rule(Expr -> Expr Mod Expr)]
  fn expr_mod(l: i32, _op: Token<'_>, r: i32) -> i32 { l % r }
  #[rule(Expr -> Sub Expr)]
  #[prec(UMinus)]
  fn expr_neg(_op: Token<'_>, r: i32) -> i32 { -r }
  #[rule(Expr -> LParen Expr RParen)]
  fn expr_paren(_l: Token<'_>, i: i32, _r: Token<'_>) -> i32 { i }
  #[rule(Expr -> IntConst)]
  fn expr_int(i: Token<'_>) -> i32 { std::str::from_utf8(i.piece).unwrap().parse().unwrap() }
}

#[test]
fn lalr1() {
  assert_eq!(Parser.parse(&mut Lexer::new(b"1 - 2 * (3 + 4 * 5 / 6) + -7 * -9 % 10")), Ok(-8));
}
