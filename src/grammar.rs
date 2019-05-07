use std::collections::{HashMap, HashSet};
use crate::raw_grammar::*;
use crate::printer::*;

#[derive(Debug)]
pub struct Grammar<'a> {
  pub raw: &'a RawGrammar,
  pub token2id: HashMap<&'a str, u32>,
  pub id2token: Vec<(&'a str, Assoc)>,
  pub lexer_state: Vec<&'a str>,
  pub lex: Vec<Vec<(&'a str, &'a str, bool)>>,
  pub prod: Vec<(Vec<u32>, &'a str)>,
}


impl Grammar<'_> {
  pub fn eof(&self) -> u32 {
    unimplemented!()
  }

  pub fn token_num(&self) -> u32 {
    unimplemented!()
  }

  pub fn nt_num(&self) -> u32 {
    unimplemented!()
  }
//  pub fn is_non_terminal(&self, ch: u32) -> bool {
//    unimplemented!()
//  }

  pub fn get_prod(&self, ch: u32) -> &[Vec<u32>] {
    unimplemented!()
  }

//  pub fn add_first(&self, ch: u32, first: &mut BitVec<BigEndian, u64>) {
//    unimplemented!()
//  }

  pub fn gen(&self) -> String {
    let mut p = IdentPrinter::new();
    p.ln(r#"#![allow(unused)]
#![allow(unused_mut)]

use regex::Regex;
use std::collections::HashMap;"#).ln("");

    p.lns(r#"#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {"#).inc();
    for &(token, _) in &self.id2token {
      p.ln(format!("{},", token));
    }
    p.dec().ln("}\n");

    p.lns(r#"#[derive(Debug, Clone, Copy)]
pub enum LexerState {"#).inc();
    for (i, &state) in self.lexer_state.iter().enumerate() {
      p.ln(format!("{} = {},", state, i));
    }
    p.dec().ln("}\n");

    p.lns(r#"macro_rules! map (
  { $($key:expr => $value:expr),+ } => {{
    let mut m = ::std::collections::HashMap::new();
    $( m.insert($key, $value); )+
    m
  }};
);"#).ln("");

    p.lns(r#"#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  pub ty: TokenType,
  pub piece: &'a str,
  pub line: u32,
  pub col: u32,
}
"#).ln("");

    p.lns(r#"pub struct Lexer<'a> {
  pub string: &'a str,
  pub states: Vec<LexerState>,
  pub cur_line: u32,
  pub cur_col: u32,
  pub piece: &'a str,"#).inc();
    if let Some(ext) = &self.raw.lexer_field_ext {
      for RawLexerFieldExt { field, type_, init: _ } in ext {
        p.ln(format!("pub {}: {},", field, type_));
      }
    }
    p.dec().ln("}\n");

    p.ln("impl Lexer<'_> {").inc();
    p.lns(r#"pub fn new(string: &str) -> Lexer {"#).inc();
    p.lns(r#"Lexer {
  string,
  states: vec![LexerState::_Initial],
  cur_line: 1,
  cur_col: 0,
  piece: "","#).inc();
    if let Some(ext) = &self.raw.lexer_field_ext {
      for RawLexerFieldExt { field, type_: _, init } in ext {
        p.ln(format!("{}: {},", field, init));
      }
    }
    p.dec().ln("}"); // Lexer
    p.dec().ln("}\n"); // new

    p.lns(r#"pub fn next(&mut self) -> Option<Token> {
  loop {
    if self.string.is_empty() {
      return None;
    }
    let mut max: Option<(&str, fn(&mut Lexer) -> TokenType)> = None;
    for (re, act) in &LEX_RULES[*self.states.last()? as u32] {
      match re.find(self.string) {
        None => {}
        Some(n) => {
          let n = n.as_str();
          if match max {
            None => true,
            Some((o, _)) => o.len() < n.len(),
          } {
            max = Some((n, *act));
          }
        }
      }
    }
    let (piece, act) = max?;
    self.piece = piece;
    let ty = act(self);
    self.string = &self.string[piece.len()..];
    let (line, col) = (self.cur_line, self.cur_col);
    for (i, l) in piece.split('\n').enumerate() {
      self.cur_line += 1;
      if i == 0 {
        self.cur_col += l.len() as u32;
      } else {
        self.cur_col = l.len() as u32;
      }
    }
    if ty != TokenType::_Skip {
      break Some(Token { ty, piece, line, col });
    }
  }
}"#);
    p.dec().ln("}\n"); // impl

    p.ln("lazy_static! {").inc();
    p.ln(format!("static ref LEX_RULES: [Vec<(Regex, fn(&mut Lexer) -> TokenType)>; {}] = [", self.lex.len())).inc();
    {
      let mut cnt = 0;
      for lex_state_rules in &self.lex {
        p.ln("vec![").inc();
        for &(re, _, escape) in lex_state_rules {
          // add enough # to prevent the re contains `#"`
          let raw = "#".repeat(re.matches('#').count() + 1);
          p.ln(format!(r#"(Regex::new(r{}"^{}"{}).unwrap(), lex_act{}),"#, raw, &if escape { regex::escape(re) } else { re.to_owned() }, raw, cnt));
          cnt += 1;
        }
        p.dec().ln("],");
      }
    }
    p.dec().ln("];"); // LEX_RULES
    p.dec().ln("}\n"); // lazy_static

    {
      let mut cnt = 0;
      for lex_state_rules in &self.lex {
        for &(_, act, _) in lex_state_rules {
          p.ln(format!("fn lex_act{}(_l: &mut Lexer) -> TokenType {{", cnt)).inc();
          p.lns(act);
          p.dec().ln("}\n");
          cnt += 1;
        }
      }
    }
    p.finish()
  }
}