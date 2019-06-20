use crate::grammar::Grammar;
use crate::printer::IndentPrinter;
use crate::raw_grammar::RawFieldExt;
use crate::lalr1_common::ParseTable;
use std::collections::HashMap;
use crate::abstract_grammar::AbstractGrammar;

pub trait Codegen {
  fn gen(&self, g: &Grammar, table: &ParseTable) -> String;
}

#[allow(unused)]
pub struct RustCodegen;

impl Codegen for RustCodegen {
  fn gen(&self, g: &Grammar, table: &ParseTable) -> String {
    let mut p = IndentPrinter::new();
    p.ln(r#"#![allow(unused)]
#![allow(unused_mut)]

use regex::Regex;
use std::collections::HashMap;"#).ln("");

    if !g.raw.include.is_empty() {
      p.lns(&g.raw.include).ln("");
    }

    p.lns(r#"#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {"#).inc();
    for &(nt, _) in &g.nt {
      p.ln(format!("{},", nt));
    }
    for &(t, _) in &g.terminal {
      p.ln(format!("{},", t));
    }
    p.dec().ln("}\n");

//    p.lns(r#"#[derive(Debug, Clone, Copy)]
//pub enum LexerState {"#).inc();
//    for &state in &g.lex_state {
//      p.ln(format!("{},", state));
//    }
//    p.dec().ln("}\n");

//    p.lns(r#"macro_rules! map (
//  { $($key:expr => $value:expr),+ } => {{
//    let mut m = ::std::collections::HashMap::new();
//    $( m.insert($key, $value); )+
//    m
//  }};
//);"#).ln("");

//    p.ln("lazy_static! {").inc();
//    p.ln(format!("static ref LEX_RULES: [Vec<(Regex, fn(&mut Lexer) -> TokenType)>; {}] = [", g.lex.len())).inc();
//    {
//      let mut cnt = 0;
//      for lex_state_rules in &g.lex {
//        p.ln("vec![").inc();
//        for (re, _, _) in lex_state_rules {
//          // add enough # to prevent the re contains `#"`
//          let raw = "#".repeat(re.matches('#').count() + 1);
//          p.ln(format!(r#"(Regex::new(r{}"^{}"{}).unwrap(), lex_act{}),"#, raw, &re, raw, cnt));
//          cnt += 1;
//        }
//        p.dec().ln("],");
//      }
//    }
//    p.dec().ln("];\n"); // LEX_RULES

//    p.ln(format!("static ref TABLE: [HashMap<u32, Act>; {}] = [", table.action.len())).inc();
//    for act in &table.action {
//      let mut sorted = act.1.iter().collect::<Vec<_>>();
//      sorted.sort_unstable_by(|l, r| l.0.cmp(r.0));
//
//      let mut map = "map! { ".to_owned();
//      // manually join...
//      // rust's join seems still unstable now?
//      for (i, (&link, act)) in sorted.iter().enumerate() {
//        if i == 0 {
//          map += &format!("{} => Act::{:?}", link, act[0]);
//        } else {
//          map += &format!(", {} => Act::{:?}", link, act[0]);
//        }
//      }
//      map += " },";
//      p.ln(map);
//    }
//    p.dec().ln("];"); // TABLE
//    p.dec().ln("}\n"); // lazy_static

    p.ln(format!("static PARSER_ACT: [fn(&mut Parser); {}] = [", g.prod_extra.len())).inc();
    for i in 0..g.prod_extra.len() {
      p.ln(format!("parser_act{}, ", i));
    }
    p.dec().ln("];\n");

    p.ln(format!("static PRODUCTION_INFO: [(u32, u32); {}] = [", g.prod_extra.len())).inc();
    for &(_, (lhs, rhs), _) in &g.prod_extra {
      p.ln(format!("({}, {}),", lhs, g.prod[lhs as usize][rhs as usize].0.len()));
    }
    p.dec().ln("];\n");

    p.write(r#"#[derive(Debug, Clone, Copy)]
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
    loop {
      if self.string.is_empty() {
        return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
      }
      let (mut line, mut col) = (self.cur_line, self.cur_col);
      let mut last_acc = _Eof; // this is arbitrary, just a value that cannot be returned by user defined function
      let mut state = 0;
      let mut i = 0;
      while i < self.string.len() {
        // '\0' should not be in alphabet
        let ch = unsafe { *self.string.get_unchecked(i) };
        let &ec = unsafe { CH2EC.get_unchecked((ch & 0x7F) as usize) };
        let &nxt = unsafe { EDGE.get_unchecked(state as usize).get_unchecked(ec as usize) };
        let &acc = unsafe { ACC.get_unchecked(nxt as usize) };
        last_acc = if acc != _Eof { acc } else { last_acc };
        state = nxt;
        if nxt == 0 { // dead, should not eat this char
          if last_acc == _Eof { // completely dead
            return None;
          } else {
            // exec user defined function here
            let piece = unsafe { std::slice::from_raw_parts(self.string.as_ptr(), i) };
            self.string = unsafe { std::slice::from_raw_parts(self.string.as_ptr().add(i), self.string.len() - i) };
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
        let piece = unsafe { std::slice::from_raw_parts(self.string.as_ptr(), i) };
        self.string = unsafe { std::slice::from_raw_parts(self.string.as_ptr().add(i), 0) };
        if last_acc != _Eps {
          return Some(Token { ty: last_acc, piece, line, col });
        } else {
          return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
        }
      }
    }
  }
}
"#).ln("");

//    if let Some(ext) = &g.raw.lexer_field_ext {
//      for RawFieldExt { field, type_, init: _ } in ext {
//        p.ln(format!("pub {}: {},", field, type_));
//      }
//    }
//    p.dec().ln("}\n");

//    aw.lexer_field_ext {
//      for RawFieldExt { field, type_: _, init } in ext {
//        p.ln(format!("{}: {},", field, init));
//      }
//    }
//    p.dec().ln("}"); // Lexer
//    p.dec().ln("}\n"); // new

    p.lns("#[derive(Copy, Clone, Debug)]
enum Act {
  Acc,
  Shift(u32),
  Reduce(u32),
  Goto(u32),
}").ln("");

    let mut types = Vec::new();
    let mut types2id = HashMap::new();
    for &(_, ty) in &g.nt {
      types2id.entry(ty).or_insert_with(|| {
        let id = types.len() as u32;
        types.push(ty);
        id
      });
    }

    p.ln("enum StackItem<'a> {").inc();
    p.ln("_Token(Token<'a>),");
    for (i, ty) in types.iter().enumerate() {
      p.ln(format!("_{}({}),", i, ty));
    }
    p.dec().ln("}\n");

    // use these 2 forward declaration to make the huge code block below not need format!()


    p.write(r#"pub struct Parser<'a> {
  value_stk: Vec<StackItem<'a>>,
  state_stk: Vec<u32>,
  lexer: Lexer<'a>,
"#).inc();
    if let Some(ext) = &g.raw.parser_field_ext {
      for RawFieldExt { field, type_, init: _ } in ext {
        p.ln(format!("pub {}: {},", field, type_));
      }
    }
    p.dec().ln("}\n");

    p.write(r#"impl<'a> Parser<'a> {
  pub fn new(string: &'a str) -> Parser {
    Parser {
      value_stk: vec![],
      state_stk: vec![0],
      lexer: Lexer::new(string),
"#).inc().inc().inc();
    if let Some(ext) = &g.raw.parser_field_ext {
      for RawFieldExt { field, type_: _, init } in ext {
        p.ln(format!("{}: {},", field, init));
      }
    }
    p.dec().ln("}").dec().ln("}");

    let parse_res = g.nt[(g.prod_extra.last().unwrap().1).0 as usize].1;
    let res_id = types2id[parse_res];
    p.write(format!(r#"  pub fn parse(&mut self) -> Result<{}, Option<Token<'a>>> {{"#, parse_res));

    p.write(r#"    let mut token = match self.lexer.next() { Some(t) => t, None => return Err(None) };
    loop {
      let state = *self.state_stk.last().unwrap();
      let act = match TABLE[state as usize].get(&(token.ty as u32)) { Some(a) => *a, None => return Err(Some(token)) };

      match act {
        Act::Shift(s) => {
          self.value_stk.push(StackItem::_Token(token));
          self.state_stk.push(s);
          token = match self.lexer.next() { Some(t) => t, None => return Err(None) };
        }
        Act::Reduce(r) => {
          let info = PRODUCTION_INFO[r as usize];
          for _ in 0..info.1 { self.state_stk.pop().unwrap(); }
          PARSER_ACT[r as usize](self);
          let cur = *self.state_stk.last().unwrap();
          let nxt = match &TABLE[cur as usize][&info.0] { Act::Goto(n) => *n, _ => unreachable!() };
          self.state_stk.push(nxt);
        }
        Act::Acc => {
          self.state_stk.pop().unwrap();"#);
    p.ln(format!(r#"          let res = match self.value_stk.pop() {{ Some(StackItem::_{}(r)) => r, _ => unreachable!() }};"#, res_id));
    p.lns(r#"          return Ok(res);
        }
        _ => unreachable!(),
      }
    }
  }
}"#).ln("");

//    {
//      let mut cnt = 0;
//      for lex_state_rules in &g.lex {
//        for &(_, act, term) in lex_state_rules {
//          p.ln(format!("fn lex_act{}(_l: &mut Lexer) -> TokenType {{", cnt)).inc();
//          if !act.is_empty() { // just to make it prettier...
//            p.lns(act);
//          }
//          p.ln(format!("TokenType::{}", term));
//          p.dec().ln("}\n");
//          cnt += 1;
//        }
//      }
//    }

    for (i, &(act, (lhs, rhs), _)) in g.prod_extra.iter().enumerate() {
      p.ln(format!("fn parser_act{}(_p: &mut Parser) {{", i)).inc();
      let rhs = &g.prod[lhs as usize][rhs as usize];
      for (j, &x) in rhs.0.iter().enumerate().rev() {
        let j = j + 1;
        if x < AbstractGrammar::nt_num(g) {
          let id = types2id[g.nt[x as usize].1];
          p.ln(format!("let mut _{} = match _p.value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => unreachable!() }};", j, id));
        } else {
          p.ln(format!("let mut _{} = match _p.value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => unreachable!() }};", j));
        }
      }
      if !act.is_empty() { // just to make it prettier...
        p.lns(act);
      }
      let id = types2id[g.nt[lhs as usize].1];
      p.ln(format!("_p.value_stk.push(StackItem::_{}(_0));", id));
      p.dec().ln("}\n");
    }
    let mut s = p.finish();
    s.pop(); // // just to make it prettier...
    s.pop();
    s
  }
}