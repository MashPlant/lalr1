use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use regex::Regex;
use crate::grammar::Grammar;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
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
  pub start: Option<String>,
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
          token2id.insert(token, id as u32);
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

    if self.production.is_empty() {
      return Err("Grammar must have at least one production rule.".into());
    }

    for production in &self.production {
      let split = production.rule.split("->").collect::<Box<[_]>>();
      if split.len() != 2 {
        return Err(format!("Production is not in the form LHS -> RHS: `{}`.", production.rule));
      }
      let (lhs, rhs) = (split[0].trim(), split[1].trim());
      let mut this_prod = Vec::new();
      let lhs_token = match token2id.get(lhs) {
        None => return Err(format!("Production lhs contains undefined token: `{}` in `{}`.", lhs, production.rule)),
        Some(id) => *id,
      };
      this_prod.push(lhs_token);
      for rhs in rhs.split_whitespace() {
        match token2id.get(rhs) {
          None => return Err(format!("Production rhs contains undefined token: `{}` in `{}`.", rhs, production.rule)),
          Some(id) => this_prod.push(*id),
        }
      }
      prod.push((this_prod, production.act.as_str()));
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