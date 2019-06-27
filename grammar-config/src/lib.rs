#[macro_use]
extern crate smallvec;
extern crate serde;
extern crate serde_derive;
extern crate regex;

use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Assoc {
  Left,
  Right,
  NoAssoc,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawGrammar {
  pub include: String,
  pub terminal: Vec<RawTerminalRow>,
  // previously I support state/act just like lex/flex
  // later on I found they are not necessary in my application and removed them
  //               (re,     token )
  pub lexical: Vec<(String, String)>,
  pub parser_field_ext: Option<Vec<RawFieldExt>>,
  //                (nt    , type  )
  pub start: Option<(String, String)>,
  pub production: Vec<RawProduction>,
}

pub const EPS: &'static str = "_Eps";
pub const EOF: &'static str = "_Eof";
pub const ERR: &'static str = "_Err";


#[derive(Debug, Deserialize, Serialize)]
pub struct RawTerminalRow {
  pub assoc: Option<Assoc>,
  pub terms: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawFieldExt {
  pub field: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub init: String,
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