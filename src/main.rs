extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod printer;
mod parser;

use crate::printer::IdentPrinter;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use regex::Regex;
use std::fs::read_to_string;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Assoc {
  Left,
  Right,
  NoAssoc,
  Token,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawGrammar {
  pub include: String,
  pub lexer_state_ext: Option<Vec<String>>,
  pub lexer_field_ext: Option<Vec<RawLexerFieldExt>>,
  pub token: Vec<RawTokenRow>,
  pub lexical: Vec<RawLexicalRule>,
  pub production: Vec<RawProduction>,
}

impl RawGrammar {
  pub fn to_grammar(&self) -> Result<Grammar, String> {
    let mut token2id = HashMap::new();
    let mut id2token = Vec::new();
    let mut lex = Vec::new();
    let mut prod = Vec::new();
    let mut lexer_state2id = HashMap::new();
    let mut id2lexer_state = Vec::new();

    token2id.insert("_Skip", 0);
    id2token.push(("_Skip", Assoc::Token));
    lexer_state2id.insert("_Initial", 0);
    id2lexer_state.push("_Initial");

    let valid_token_name = regex::Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*$").unwrap();
    for token_row in &self.token {
      for token in token_row.tokens.iter().map(String::as_str) {
        if token == "_Skip" {
          return Err("Token cannot have the builtin name `_Skip`.".into());
        } else if token2id.contains_key(token) {
          return Err(format!("Find duplicate token: `{}`.", token));
        } else if !valid_token_name.is_match(token) {
          return Err(format!("Token is not a valid variable name: `{}`.", token));
        } else {
          let id = id2token.len();
          token2id.insert(token, id);
          id2token.push((token, token_row.assoc));
        }
      }
    }

    if let Some(ext) = &self.lexer_state_ext {
      for state in ext.iter().map(String::as_str) {
        if state == "_Initial" {
          return Err("Lexer state cannot have the builtin name `_Initial`.".into());
        } else if lexer_state2id.contains_key(state) {
          return Err(format!("Find duplicate lexer state: `{}`.", state));
        } else {
          let len = id2lexer_state.len();
          lexer_state2id.insert(state, len);
          id2lexer_state.push(state);
        }
      }
    }

    for lexical in &self.lexical {
      if let Err(err) = Regex::new(&if lexical.escape { regex::escape(&lexical.re) } else { lexical.re.to_owned() }) {
        return Err(format!("Error regex: `{}`, reason: {}.", lexical.re, err));
      } else {
        match lexer_state2id.get(lexical.state.as_str()) {
          None => return Err(format!("Lexer rule contains undefined lexer states: `{}`.", lexical.state)),
          Some(&id) => {
            if lex.len() < id + 1 {
              lex.resize_with(id + 1, || Vec::new());
            }
            lex[id].push((lexical.re.as_str(), lexical.act.as_str(), lexical.escape));
          }
        }
      }

      // maybe also validate act's validity
    }

    for production in &self.production {
      let split = production.rule.split("->").collect::<Box<[_]>>();
      if split.len() != 2 {
        return Err(format!("Production is not in the form LHS -> RHS: `{}`.", production.rule));
      }
      let (lhs, rhs) = (split[0].trim(), split[1].trim());
      let lhs_token = match token2id.get(lhs) {
        None => return Err(format!("Production lhs contains undefined token: `{}` in `{}`.", lhs, production.rule)),
        Some(id) => *id,
      };
      let mut rhs_token = Vec::new();
      for rhs in rhs.split_whitespace() {
        match token2id.get(rhs) {
          None => return Err(format!("Production rhs contains undefined token: `{}` in `{}`.", rhs, production.rule)),
          Some(id) => rhs_token.push(*id),
        }
      }
      prod.push(((lhs_token, rhs_token), production.act.as_str()));
    }

    Ok(Grammar {
      raw: self,
      token2id,
      id2token,
      lexer_state: id2lexer_state,
      lex,
      prod,
    })
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawTokenRow {
  pub assoc: Assoc,
  pub tokens: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawLexerFieldExt {
  pub field: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub init: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawLexicalRule {
  #[serde(default = "default_state")]
  pub state: String,
  pub re: String,
  pub act: String,
  // whether use regex::escape to modify the pattern string
  // in most case, yes(like "+"); if it is "real" regex, no(like "[0-9]")
  #[serde(default = "default_escape")]
  pub escape: bool,
}

fn default_state() -> String {
  "_Initial".into()
}

fn default_escape() -> bool {
  true
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawProduction {
  pub rule: String,
  pub act: String,
}

#[derive(Debug)]
pub struct Grammar<'a> {
  pub raw: &'a RawGrammar,
  pub token2id: HashMap<&'a str, usize>,
  pub id2token: Vec<(&'a str, Assoc)>,
  pub lexer_state: Vec<&'a str>,
  pub lex: Vec<Vec<(&'a str, &'a str, bool)>>,
  pub prod: Vec<((usize, Vec<usize>), &'a str)>,
}


impl Grammar<'_> {
  pub fn gen(&self) -> String {
    let mut p = IdentPrinter::new();
    p.ln(r#"#![allow(dead_code)]
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
  ty: TokenType,
  piece: &'a str,
  line: u32,
  col: u32,
}
"#).ln("");

    p.lns(r#"pub struct Lexer<'a> {
  string: &'a str,
  states: Vec<LexerState>,
  cur_line: u32,
  cur_col: u32,
  piece: &'a str,"#).inc();
    if let Some(ext) = &self.raw.lexer_field_ext {
      for RawLexerFieldExt { field, type_, init: _ } in ext {
        p.ln(format!("{}: {},", field, type_));
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
    for (re, act) in &LEX_RULES[*self.states.last()? as usize] {
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
    let ty = act(self);
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
          p.ln(format!("fn lex_act{}(l: &mut Lexer) -> TokenType {{", cnt)).inc();
          p.lns(act);
          p.dec().ln("}\n");
          cnt += 1;
        }
      }
    }
    p.finish()
  }
}

fn main() {
  let prog = read_to_string("test.decaf").unwrap();
  let mut lex = parser::Lexer::new(&prog);
  while let Some(tk) = lex.next() {
    println!("{:?}", tk);
  }
//  let s = read_to_string("decaf.toml").unwrap();
//  let g: RawGrammar = toml::from_str(&s).unwrap();
//  let g = g.to_grammar().unwrap();
//  println!("{}", g.gen());
//  println!("{:#?}", g);
//  let g = RawGrammar {
//    include: r##"#include <iostream>
//using namespace std;
//"##.into(),
//    lexer_state_ext: Some(vec!["S".into()]),
//    lexer_field_ext: Some(vec![RawLexerFieldExt {
//      field: "string_builder".into(),
//      type_: "String".into(),
//      init: "String::new()".into(),
//    }]),
//    token: vec![
//      RawTokenRow { assoc: Assoc::Token, tokens: vec!["Integer".into(), "Identifier".into()] },
//      RawTokenRow { assoc: Assoc::Left, tokens: vec!["Add".into(), "Sub".into(), "Le".into(), "Ge".into(), "Lt".into(), "Gt".into()] },
//      RawTokenRow { assoc: Assoc::Left, tokens: vec!["Mul".into(), "Div".into()] },
//      RawTokenRow { assoc: Assoc::NoAssoc, tokens: vec!["Else".into()] },
//    ],
//    lexical: vec![
//      RawLexicalRule { state: "S".into(), re: r#"[0-9]+"#.into(), act: r#"return TokenType::Integer;"#.into() },
//      RawLexicalRule { state: "Initial".into(), re: r#"[A-Za-z][_0-9A-Za-z]*\n"#.into(), act: r#"return TokenType::Identifier;"#.into() },
//    ],
//    production: vec![
//      RawProduction {
//        rule: "Identifier -> Identifier Integer".into(),
//        act: r##"{
//  println!("hello world!");
//}"##.into(),
//      }
//    ],
//  };
//  println!("{}", toml::to_string(&g).unwrap());
}