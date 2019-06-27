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

{{INCLUDE}}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType { {{TOKEN_TYPE}} }

#[derive(Copy, Clone, Debug)]
pub enum Act { Shift({{U_LR_SIZE}}), Reduce({{U_LR_SIZE}}), Goto({{U_LR_SIZE}}), Acc, Err }

pub enum StackItem<'a> { {{STACK_ITEM}} }

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  pub ty: TokenType,
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
    use TokenType::*;
    static ACC: [TokenType; {{DFA_SIZE}}] = [{{ACC}}];
    static EC: [u8; 128] = [{{EC}}];
    static EDGE: [[{{U_DFA_SIZE}}; {{EC_SIZE}}]; {{DFA_SIZE}}] = [{{DFA_EDGE}}];
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
        let ec = index!(EC, ch & 0x7F);
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
  pub state_stk: Vec<{{U_LR_SIZE}}>,
  pub lexer: Lexer<'a>,
  {{PARSER_FIELD}}
}

impl<'a> Parser<'a> {
  #[allow(unused)]
  #[allow(unused_mut)]
  pub fn parse(&mut self) -> Result<{{RESULT_TYPE}}, Option<Token<'a>>> {
    static PROD: [({{U_LR_SIZE}}, {{U_PROD_LEN}}); {{PROD_SIZE}}] = [{{PROD}}];
    static EDGE: [[Act; {{TOKEN_SIZE}}]; {{LR_SIZE}}] = [{{LR_EDGE}}];
    let mut token = match self.lexer.next() { Some(t) => t, None => return Err(None) };{{LOG_TOKEN}}
    loop {
      let state = index!(self.state_stk, self.state_stk.len() - 1);
      let act = index!(index!(EDGE, state), token.ty);
      match act {
        Act::Shift(s) => {
          self.value_stk.push(StackItem::_Token(token));
          self.state_stk.push(s);
          token = match self.lexer.next() { Some(t) => t, None => return Err(None) };{{LOG_TOKEN}}
        }
        Act::Reduce(r) => {
          let prod = index!(PROD, r);
          for _ in 0..prod.1 { match self.state_stk.pop() { None => impossible!(), Some(_) => {} }; }
          match r {
            {{PARSER_ACT}}
            _ => impossible!(),
          }
          let cur = index!(self.state_stk, self.state_stk.len() - 1);
          let nxt = match index!(index!(EDGE, cur), prod.0) { Act::Goto(n) => n, _ => impossible!() };
          self.state_stk.push(nxt);
        }
        Act::Acc => {
          match self.state_stk.pop() { None => impossible!(), Some(_) => {} };
          let res = match self.value_stk.pop() { Some(StackItem::_{{RESULT_ID}}(r)) => r, _ => impossible!() };
          return Ok(res);
        }
        Act::Err => return Err(Some(token)),
        _ => impossible!(),
      }
    }
  }
}