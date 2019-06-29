extern crate serde;
extern crate serde_derive;
extern crate indexmap;
extern crate smallvec;

pub mod grammar;
pub use grammar::*;

use serde::{Serialize, Deserialize};
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Assoc {
  Left,
  Right,
  NoAssoc,
}

// previously I support state/act just like lex/flex
// later I found they are not necessary in my application and removed them
#[derive(Debug, Deserialize, Serialize)]
pub struct RawGrammar {
  pub include: String,
  pub priority: Vec<RawPriorityRow>,
  /// map re to term
  pub lexical: IndexMap<String, String>,
  /// this string should contain name & type
  /// e.g.: "a: u32" for rust, "int a" for c++
  pub parser_field: Option<Vec<String>>,
  pub start: Option<String>,
  pub production: Vec<RawProduction>,
  /// None -> will define a struct Parser<'a> { _p: std::marker::PhantomData<&'a ()>,  parser_field_ext }
  /// Some -> will not define a struct
  #[serde(skip_serializing)]
  pub parser_def: Option<String>,
}

pub const EPS: &'static str = "_Eps";
pub const EOF: &'static str = "_Eof";
pub const ERR: &'static str = "_Err";

#[derive(Debug, Deserialize, Serialize)]
pub struct RawPriorityRow {
  pub assoc: Assoc,
  pub terms: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawProduction {
  pub lhs: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub rhs: Vec<RawProductionRhs>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawProductionRhs {
  pub rhs: String,
  /// this is basically for the type checking for parser-macros
  /// it would not be pleasing if you provide it from toml config file(but you can, any way)
  pub rhs_arg: Option<Vec<(Option<String>, String)>>,
  pub act: String,
  pub prec: Option<String>,
}

// about the distribution of non-terminal & terminal & eof & eps on u32:
// non-terminal: 0..nt_num(), terminal & eof & eps: nt_num()..token_num()
pub trait AbstractGrammar<'a> {
  // the right hand side of production
  type ProdRef: AsRef<[u32]> + 'a;
  // iter of (right hand side of production, production id)
  type ProdIter: IntoIterator<Item=&'a (Self::ProdRef, u32)>;

  fn start(&'a self) -> &'a (Self::ProdRef, u32);

  fn eps(&self) -> u32;

  fn eof(&self) -> u32;

  fn token_num(&self) -> u32;

  fn nt_num(&self) -> u32;

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter;
}

pub trait AbstractGrammarExt<'a>: AbstractGrammar<'a> {
  // id is returned from get_prod
  fn prod_pri_assoc(&self, id: u32) -> Option<(u32, Assoc)>;

  fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)>;
}

pub struct ValidName;

impl ValidName {
  pub fn is_match(&self, s: &str) -> bool {
    let mut chs = s.chars();
    match chs.next() {
      Some(ch) if ch.is_ascii_alphabetic() => chs.all(|ch| ch.is_ascii_alphabetic() || ch == '_'),
      _ => false,
    }
  }
}

// use this instead of a function for compatibility, I used to use a regex here
pub const VALID_NAME: ValidName = ValidName;

/// input: the two field in RawGrammar(or constructed in other ways)
/// return: (Vec<(term, pri_assoc)>, term2id)
pub fn parse_term<'a>(
  priority: &'a [RawPriorityRow],
  lexical: &'a IndexMap<String, String>,
) -> Result<(Vec<(&'a str, Option<(u32, Assoc)>)>, HashMap<&'a str, u32>), String> {
  let mut terms = vec![(EPS, None), (EOF, None)];
  let mut term2id = HashMap::new();
  term2id.insert(EPS, 0);
  term2id.insert(EOF, 1);

  for (pri, pri_row) in priority.iter().enumerate() {
    let pri_assoc = (pri as u32, pri_row.assoc);
    for term in pri_row.terms.iter().map(String::as_str) {
      if term == EPS {
        return Err(format!("Cannot assign priority to builtin term `{}`.", EPS));
      } else if term == EOF {
        return Err(format!("Cannot assign priority to builtin term `{}`.", EOF));
      } else if !VALID_NAME.is_match(term) {
        return Err(format!("Term is not a valid variable name: `{}`.", term));
      } else if term2id.contains_key(term) {
        return Err(format!("Find duplicate term in assigning priority: `{}`.", term));
      } else {
        term2id.insert(term, terms.len() as u32);
        terms.push((term, Some(pri_assoc)));
      }
    }
  }

  for l in lexical {
    let (_, term) = (l.0.as_str(), l.1.as_str());
    if term == EOF {
      return Err(format!("User define lex rule cannot return token `{}`.", EOF));
    } else if term != EPS && !VALID_NAME.is_match(term) {
      return Err(format!("Term is not a valid variable name: `{}`.", term));
    }
    term2id.entry(term).or_insert_with(|| {
      let id = terms.len() as u32;
      terms.push((term, None));
      id
    });
  }
  Ok((terms, term2id))
}

/*
      if rhs_ty.len() != rhs_tk.len() {
        panic!("Production `{}`'s rhs and method `{}`'s arguments have different length.", rule, method.sig.ident);
      }
      for (&rhs_tk, rhs_ty) in rhs_tk.iter().zip(rhs_ty.iter()) {
        match rhs_ty {
          ArgInfo::Self_ => panic!("Method `{}` takes self argument in illegal position.", method.sig.ident),
          ArgInfo::Arg { name, ty } => {
            name_rhs.push(name.clone());
            match (nt2id.get(rhs_tk), term2id.get(rhs_tk)) {
              (Some(&nt_id), _) => {
                let nt_ty = &nt[nt_id as usize].1;
                if nt_ty != ty {
                  panic!("Production `{}`'s rhs and method `{}`'s arguments have conflict signature: `{}` requires `{}`, while method takes `{}`.",
                         rule, method.sig.ident, rhs_tk, nt_ty, ty);
                }
                prod_rhs.push(nt_id);
              }
              (_, Some(&t)) => {
                if !ty.starts_with("Token") { // maybe user will use some lifetime specifier
                  panic!("Production `{}`'s rhs and method `{}`'s arguments have conflict signature: `{}` requires Token, while method takes `{}`.",
                         rule, method.sig.ident, rhs_tk, ty);
                }
                prod_rhs.push(t + nt.len() as u32 + 1); // +1 for push a _start to nt later
                pri_assoc = terms[t as usize].1;
              }
              (None, None) => panic!("Production rhs contains undefined item: `{}`", rhs_tk),
            }
          }
        }
      }
*/