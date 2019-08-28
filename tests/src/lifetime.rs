use parser_macros::lalr1;

#[allow(dead_code)]
struct Parser<'p> {
  tokens: Vec<Token<'p>>,
}

#[lalr1(Tokens)]
#[lex(r#"
priority = []

[lexical]
'\s+' = '_Eps'
'[A-Za-z][_0-9A-Za-z]*' = 'Token'
"#)]
impl<'p> Parser<'p> {
  #[rule(Tokens -> Tokens Token)]
  fn r1(&mut self, l: (), r: Token<'_>) -> () {
    self.tokens.push(r);
  }

  #[rule(Tokens ->)]
  fn r2(&mut self) -> () {}
}

#[test]
fn lifetime() {
  let mut p = Parser { tokens: vec![] };

  assert_eq!(p.parse(&mut Lexer::new(b"abc abcd abcde")), Ok(()));
  println!("{:?}", p.tokens);
}