use parser_macros::lalr1;

// Test lexing Lua literal string

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|window| window == needle)
}

#[allow(unused)]
struct Parser;

#[lalr1(Expr)]
#[lex = r#"
priority = []

lexer_action = '''
let mut piece = piece;
if last_acc == TokenKind::Str {
  let mut needle = vec![b'='; piece.len()];
  needle[0] = b']';
  needle[piece.len() - 1] = b']';
  let end = find_subsequence(self.string, &needle).map(|x| x + piece.len()).unwrap_or(self.string.len());
  piece = idx!(self.string, ..end);
  self.string = idx!(self.string, end..);
  for &ch in piece {
    if ch == b'\n' {
      self.line += 1;
      self.col = 1;
    } else { self.col += 1; }
  }
}
'''

[lexical]
'\[' = 'LBrk'
'\]' = 'RBrk'
'\[=*\[' = 'Str'
'\s+' = '_Eps'
"#]
impl Parser {
  #[rule = "Expr -> Str"]
  fn expr(_: Token) -> () {} // not using Parser in this test
}

fn assert(t: Token, kind: TokenKind, piece: &[u8]) {
  assert_eq!(t.kind, kind);
  assert_eq!(t.piece, piece);
}

#[test]
fn literal_string() {
  use TokenKind::*;
  let mut l = Lexer::new(b"[[=]]] [=[]]]=] [=[]==]]=]");
  assert(l.next(), Str, b"=]]");
  assert(l.next(), RBrk, b"]");
  assert(l.next(), Str, b"]]]=]");
  assert(l.next(), Str, b"]==]]=]");
  assert(l.next(), _Eof, b"");

  // copied from Programming in Lua 4th
  let s = r#"[[
<html>
<head>
<title>An HTML Page</title>
</head>
<body>
<a href="http://www.lua.org">Lua</a>
</body>
</html>
]]"#.as_bytes();
  let mut l = Lexer::new(s);
  assert(l.next(), Str, &s[2..]);
  assert(l.next(), _Eof, b"");
}