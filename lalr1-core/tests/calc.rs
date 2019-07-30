#![allow(unused)]
#![allow(unused_mut)]

#[cfg(not(feature = "unsafe_parser"))]
macro_rules! index {
  ($arr: expr, $idx: expr) => { $arr[$idx as usize] };
}

#[cfg(feature = "unsafe_parser")]
macro_rules! index {
  ($arr: expr, $idx: expr) => { unsafe { *$arr.get_unchecked($idx as usize) } };
}

// just another name for unreachable
#[cfg(not(feature = "unsafe_parser"))]
macro_rules! impossible {
  () => { unreachable!() };
}

#[cfg(feature = "unsafe_parser")]
macro_rules! impossible {
  () => { unsafe { std::hint::unreachable_unchecked() } };
}



#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenKind { Expr, _Expr, _Eps, _Eof, Or, And, BOr, BXor, BAnd, Eq, Ne, Le, Ge, Lt, Gt, Shl, Shr, Add, Sub, Mul, Div, Mod, UMinus, Not, LBracket, RParenthesis, Repeat, IntConst }

#[derive(Copy, Clone, Debug)]
pub enum Act { Shift(u8), Reduce(u8), Goto(u8), Acc, Err }

pub enum StackItem<'a> { _Token(Token<'a>), _0(i64) }

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  pub ty: TokenKind,
  pub piece: &'a [u8],
  pub line: u32,
  pub col: u32,
}

pub struct Lexer<'a> {
  pub string: &'a [u8],
  pub cur_line: u32,
  pub cur_col: u32,
}

impl<'a> Lexer<'a> {
  pub fn new(string: &[u8]) -> Lexer {
    Lexer {
      string,
      cur_line: 1,
      cur_col: 1,
    }
  }

  pub fn next(&mut self) -> Option<Token<'a>> {
    use TokenKind::*;
    static ACC: [TokenKind; 24] = [_Eof, _Eof, IntConst, _Eps, Add, _Eof, Div, Mod, BXor, Gt, Mul, BOr, Lt, BAnd, Sub, Eq, Ne, Repeat, Ge, Shr, Or, Le, Shl, And, ];
    static EC: [u8; 128] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 3, 4, 0, 0, 0, 5, 6, 0, 7, 0, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 0, 0, 10, 11, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, ];
    static EDGE: [[u8; 15]; 24] = [[0, 3, 5, 7, 13, 10, 4, 14, 6, 2, 12, 1, 9, 8, 11], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0], [0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 19, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 21, 0, 0, 0], [0, 0, 0, 0, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], ];
    loop {
      if self.string.is_empty() {
        return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
      }
      let (mut line, mut col) = (self.cur_line, self.cur_col);
      let mut last_acc = _Eof; // this is arbitrary, just a value that cannot be returned by user defined function
      let mut state = 0;
      let mut i = 0;
      while i < self.string.len() {
        let ch = index!(self.string, i);
        let ec = index!(EC, ch);
        let nxt = index!(index!(EDGE, state), ec);
        let acc = index!(ACC, nxt);
        last_acc = if acc != _Eof { acc } else { last_acc };
        state = nxt;
        if nxt == 0 { // dead, should not eat this char
          if last_acc == _Eof { // completely dead
            return None;
          } else {
            let piece = &self.string[..i];
            self.string = &self.string[i..];
            if last_acc != _Eps {
              return Some(Token { ty: last_acc, piece, line, col });
            } else {
              line = self.cur_line;
              col = self.cur_col;
              last_acc = _Eof;
              state = 0;
              i = 0;
            }
          }
        } else { // continue, eat this char
          if ch == b'\n' {
            self.cur_line += 1;
            self.cur_col = 1;
          } else {
            self.cur_col += 1;
          }
          i += 1;
        }
      }
      // end of file
      if last_acc == _Eof { // completely dead
        return None;
      } else {
        // exec user defined function here
        let piece = &self.string[..i];
        self.string = &self.string[i..];
        if last_acc != _Eps {
          return Some(Token { ty: last_acc, piece, line, col });
        } else {
          return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
        }
      }
    }
  }
}

pub struct Parser<'a> {
  pub value_stk: Vec<StackItem<'a>>,
  pub state_stk: Vec<u8>,
  pub lexer: Lexer<'a>,

}

impl<'a> Parser<'a> {
  pub fn new(string: &'a [u8]) -> Parser {
    Parser {
      value_stk: vec![],
      state_stk: vec![0],
      lexer: Lexer::new(string),

    }
  }

  pub fn parse(&mut self) -> Result<i64, Option<Token<'a>>> {
    static PROD: [(u8, u8); 22] = [(0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 3), (0, 2), (0, 2), (0, 1), (1, 1), ];
    static EDGE: [[Act; 28]; 43] = [[Act::Goto(1), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Err, Act::Err, Act::Err, Act::Acc, Act::Shift(5), Act::Shift(6), Act::Shift(7), Act::Shift(8), Act::Shift(9), Act::Shift(10), Act::Shift(11), Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Goto(23), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(24), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Err, Act::Err, Act::Err, Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Reduce(20), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Goto(25), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(26), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(27), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(28), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(29), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(30), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(31), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(32), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(33), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(34), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(35), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(36), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(37), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(38), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(39), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(40), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(41), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Goto(42), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(3), Act::Err, Act::Err, Act::Err, Act::Shift(4), ], [Act::Err, Act::Err, Act::Err, Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Reduce(18), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Reduce(19), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(12), Act::Reduce(12), Act::Shift(6), Act::Shift(7), Act::Shift(8), Act::Shift(9), Act::Shift(10), Act::Shift(11), Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(11), Act::Reduce(11), Act::Reduce(11), Act::Shift(7), Act::Shift(8), Act::Shift(9), Act::Shift(10), Act::Shift(11), Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(14), Act::Reduce(14), Act::Reduce(14), Act::Reduce(14), Act::Shift(8), Act::Shift(9), Act::Shift(10), Act::Shift(11), Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(15), Act::Reduce(15), Act::Reduce(15), Act::Reduce(15), Act::Reduce(15), Act::Shift(9), Act::Shift(10), Act::Shift(11), Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(13), Act::Reduce(13), Act::Reduce(13), Act::Reduce(13), Act::Reduce(13), Act::Reduce(13), Act::Shift(10), Act::Shift(11), Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(5), Act::Reduce(5), Act::Reduce(5), Act::Reduce(5), Act::Reduce(5), Act::Reduce(5), Act::Err, Act::Err, Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(6), Act::Reduce(6), Act::Reduce(6), Act::Reduce(6), Act::Reduce(6), Act::Reduce(6), Act::Err, Act::Err, Act::Shift(12), Act::Shift(13), Act::Shift(14), Act::Shift(15), Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(9), Act::Reduce(9), Act::Reduce(9), Act::Reduce(9), Act::Reduce(9), Act::Reduce(9), Act::Reduce(9), Act::Reduce(9), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(10), Act::Reduce(10), Act::Reduce(10), Act::Reduce(10), Act::Reduce(10), Act::Reduce(10), Act::Reduce(10), Act::Reduce(10), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(7), Act::Reduce(7), Act::Reduce(7), Act::Reduce(7), Act::Reduce(7), Act::Reduce(7), Act::Reduce(7), Act::Reduce(7), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(8), Act::Reduce(8), Act::Reduce(8), Act::Reduce(8), Act::Reduce(8), Act::Reduce(8), Act::Reduce(8), Act::Reduce(8), Act::Err, Act::Err, Act::Err, Act::Err, Act::Shift(16), Act::Shift(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Reduce(16), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Reduce(17), Act::Shift(18), Act::Shift(19), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Reduce(0), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Reduce(1), Act::Shift(20), Act::Shift(21), Act::Shift(22), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Reduce(2), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Reduce(3), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], [Act::Err, Act::Err, Act::Err, Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Reduce(4), Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, Act::Err, ], ];
    let mut token = match self.lexer.next() { Some(t) => t, None => return Err(None) };
    loop {
      let state = index!(self.state_stk, self.state_stk.len() - 1);
      let act = index!(index!(EDGE, state), token.ty);
      match act {
        Act::Shift(s) => {
          self.value_stk.push(StackItem::_Token(token));
          self.state_stk.push(s);
          token = match self.lexer.next() { Some(t) => t, None => return Err(None) };
        }
        Act::Reduce(r) => {
          let prod = index!(PROD, r);
          for _ in 0..prod.1 { match self.state_stk.pop() { None => impossible!(), Some(_) => {} }; }
          match r {
            0 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 + _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            1 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 - _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            2 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 * _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            3 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 / _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            4 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 % _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            5 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = (_1 == _3) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            6 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = (_1 != _3) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            7 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = (_1 < _3) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            8 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = (_1 > _3) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            9 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = (_1 <= _3) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            10 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = (_1 >= _3) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            11 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = ((_1 != 0) && (_3 != 0)) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            12 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = ((_1 != 0) || (_3 != 0)) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            13 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 & _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            14 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 | _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            15 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 ^ _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            16 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 << _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            17 => {
              let mut _3 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1 >> _3;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            18 => {
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let _0 = -_2;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            19 => {
              let mut _2 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let _0 = (_2 == 0) as i64;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            20 => {
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_Token(x)) => x, _ => impossible!() };
              let _0 = std::str::from_utf8(_1.piece).unwrap().parse().unwrap();
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }
            21 => {
              let mut _1 = match self.value_stk.pop() { Some(StackItem::_0(x)) => x, _ => impossible!() };
              let _0 = _1;
              println!("{:?}", _0);
              self.value_stk.push(StackItem::_0(_0));
            }

            _ => impossible!(),
          }
          let cur = index!(self.state_stk, self.state_stk.len() - 1);
          let nxt = match index!(index!(EDGE, cur), prod.0) { Act::Goto(n) => n, _ => impossible!() };
          self.state_stk.push(nxt);
        }
        Act::Acc => {
          match self.state_stk.pop() { None => impossible!(), Some(_) => {} };
          let res = match self.value_stk.pop() { Some(StackItem::_0(r)) => r, _ => impossible!() };
          return Ok(res);
        }
        Act::Err => return Err(Some(token)),
        _ => impossible!(),
      }
    }
  }
}

#[test]
fn calc() {
  let mut parser = Parser::new(b"1 + 2 * 3");
  assert_eq!(parser.parse().unwrap(), 7);
}