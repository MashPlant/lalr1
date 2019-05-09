use crate::grammar::Grammar;
use crate::printer::IndentPrinter;
use crate::raw_grammar::RawLexerFieldExt;

pub trait Codegen {
  fn gen(&self, g: &Grammar) -> String;
}

struct RustCodegen;

impl Codegen for RustCodegen {
  fn gen(&self, g: &Grammar) -> String {
    let mut p = IndentPrinter::new();
    p.ln(r#"#![allow(unused)]
#![allow(unused_mut)]

use regex::Regex;
use std::collections::HashMap;"#).ln("");

    p.lns(r#"#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {"#).inc();
    for &(token, _) in &g.terminal {
      p.ln(format!("{},", token));
    }
    p.dec().ln("}\n");

    p.lns(r#"#[derive(Debug, Clone, Copy)]
pub enum LexerState {"#).inc();
    for (i, &state) in g.lex_state.iter().enumerate() {
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
    if let Some(ext) = &g.raw.lexer_field_ext {
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
    if let Some(ext) = &g.raw.lexer_field_ext {
      for RawLexerFieldExt { field, type_: _, init } in ext {
        p.ln(format!("{}: {},", field, init));
      }
    }
    p.dec().ln("}"); // Lexer
    p.dec().ln("}\n"); // new

    p.lns(r#"pub fn next(&mut self) -> Option<Token> {
  loop {
    if g.string.is_empty() {
      return Some(Token { ty: TokenType::_Eof, piece: "", line: g.cur_line, col: g.cur_col });
    }
    let mut max: Option<(&str, fn(&mut Lexer) -> TokenType)> = None;
    for (re, act) in &LEX_RULES[*g.states.last()? as usize] {
      match re.find(g.string) {
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
    g.piece = piece;
    let ty = act(self);
    g.string = &g.string[piece.len()..];
    let (line, col) = (g.cur_line, g.cur_col);
    for (i, l) in piece.split('\n').enumerate() {
      g.cur_line += 1;
      if i == 0 {
        g.cur_col += l.len() as u32;
      } else {
        g.cur_col = l.len() as u32;
      }
    }
    if ty != TokenType::_Eps {
      break Some(Token { ty, piece, line, col });
    }
  }
}"#);
    p.dec().ln("}\n"); // impl

    p.ln("lazy_static! {").inc();
    p.ln(format!("static ref LEX_RULES: [Vec<(Regex, fn(&mut Lexer) -> TokenType)>; {}] = [", g.lex.len())).inc();
    {
      let mut cnt = 0;
      for lex_state_rules in &g.lex {
        p.ln("vec![").inc();
        for (re, _, _) in lex_state_rules {
          // add enough # to prevent the re contains `#"`
          let raw = "#".repeat(re.matches('#').count() + 1);
          p.ln(format!(r#"(Regex::new(r{}"^{}"{}).unwrap(), lex_act{}),"#, raw, &re, raw, cnt));
          cnt += 1;
        }
        p.dec().ln("],");
      }
    }
    p.dec().ln("];"); // LEX_RULES
    p.dec().ln("}\n"); // lazy_static

    {
      let mut cnt = 0;
      for lex_state_rules in &g.lex {
        for &(_, act, term) in lex_state_rules {
          p.ln(format!("fn lex_act{}(_l: &mut Lexer) -> TokenType {{", cnt)).inc();
          if !act.is_empty() { // just to make it prettier...
            p.lns(act);
          }
          p.ln(format!("TokenType::{}", term));
          p.dec().ln("}\n");
          cnt += 1;
        }
      }
    }
    p.finish()
  }
}