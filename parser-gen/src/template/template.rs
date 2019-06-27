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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenType { {token_type} }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Act { Shift({u_lr_size}), Reduce({u_lr_size}), Goto({u_lr_size}), Acc, Err }

pub enum StackItem<'a> { {stack_item} }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    Lexer { string, cur_line: 1, cur_col: 1 }
  }

  pub fn next(&mut self) -> Option<Token<'a>> {
    use TokenType::*;
    static ACC: [TokenType; {dfa_size}] = [{acc}];
    static EC: [u8; 128] = [{ec}];
    static DFA_EDGE: [[{u_dfa_size}; {ec_size}]; {dfa_size}] = [{dfa_edge}];
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
        let nxt = index!(index!(DFA_EDGE, state), ec);
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

impl<'a> Iterator for Lexer<'a> {
  type Item = Token<'a>;
  fn next(&mut self) -> Option<Self::Item> {
    Lexer::next(self)
  }
}

{parser_struct}

impl {parser_type} {
  #[allow(unused)]
  #[allow(unused_mut)]
  pub fn parse<'a, L: IntoIterator<Item=Token<'a>>>(&mut self, lexer: L) -> Result<{res_type}, Option<Token<'a>>> {
    static PROD: [({u_lr_size}, {u_prod_len}); {prod_size}] = [{prod}];
    static LR_EDGE: [[Act; {token_size}]; {lr_size}] = [{lr_edge}];
    let mut value_stk: Vec<StackItem<'a>> = vec![];
    let mut state_stk: Vec<{u_lr_size}> = vec![0];
    let mut lexer = lexer.into_iter();
    let mut token = match lexer.next() { Some(t) => t, None => return Err(None) };
    {log_token}
    loop {
      let state = index!(state_stk, state_stk.len() - 1);
      let act = index!(index!(LR_EDGE, state), token.ty);
      match act {
        Act::Shift(s) => {
          value_stk.push(StackItem::_Token(token));
          state_stk.push(s);
          token = match lexer.next() { Some(t) => t, None => return Err(None) };
          {log_token}
        }
        Act::Reduce(r) => {
          let prod = index!(PROD, r);
          for _ in 0..prod.1 { match state_stk.pop() { None => impossible!(), Some(_) => {} }; }
          match r {
            {parser_act}
            _ => impossible!(),
          }
          let cur = index!(state_stk, state_stk.len() - 1);
          let nxt = match index!(index!(LR_EDGE, cur), prod.0) { Act::Goto(n) => n, _ => impossible!() };
          state_stk.push(nxt);
        }
        Act::Acc => {
          match state_stk.pop() { None => impossible!(), Some(_) => {} };
          let res = match value_stk.pop() { Some(StackItem::_{res_id}(r)) => r, _ => impossible!() };
          return Ok(res);
        }
        Act::Err => return Err(Some(token)),
        _ => impossible!(),
      }
    }
  }
}