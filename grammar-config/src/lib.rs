pub mod grammar;

pub use grammar::*;

use serde::{Serialize, Deserialize};
use indexmap::IndexMap;
use hashbrown::HashMap;

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
  // this is basically for the type checking for parser-macros
  // it would not be pleasing if you provide it from toml config file(but you can, any way)
  pub rhs_arg: Option<Vec<(Option<String>, String)>>,
  pub act: String,
  pub prec: Option<String>,
}

// about the distribution of non-terminal & terminal & eof & eps on u32:
// non-term: 0..nt_num(), term & eof & eps & err: nt_num()..token_num()
pub trait AbstractGrammar<'a> {
  // the right hand side of production
  type ProdRef: AsRef<[u32]> + 'a;
  // iter of (right hand side of production, production id)
  type ProdIter: IntoIterator<Item=&'a (Self::ProdRef, u32)>;

  // return (start lhs, (start prod rhs, start prod id))
  fn start(&'a self) -> (u32, &'a (Self::ProdRef, u32));

  // eps & eof & err are 3 special terms in the grammar
  // eps: indicate lexer produces a term which should be neglected by parser; also used in computing first & follow
  // eof: indicate lexer consumes all its input; also used in computing lookahead
  // err: indicate lexer meets a unrecognizable char; also used in lalr1_by_lr0 for the "special term"
  // lexer should not return eps; when lexer returns eof/err, parser should return Err(token)
  fn eps(&self) -> u32;

  fn eof(&self) -> u32;

  fn err(&self) -> u32;

  fn token_num(&self) -> u32;

  fn nt_num(&self) -> u32;

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter;
}

pub trait AbstractGrammarExt<'a>: AbstractGrammar<'a> {
  // id is returned from get_prod
  fn prod_pri(&self, id: u32) -> Option<u32>;

  fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)>;
}

pub struct ValidName;

impl ValidName {
  pub fn is_match(&self, s: &str) -> bool {
    let mut chs = s.chars();
    match chs.next() {
      Some(ch) if ch.is_ascii_alphabetic() => chs.all(|ch| ch.is_ascii_alphanumeric() || ch == '_'),
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
  let mut terms = vec![(EPS, None), (EOF, None), (ERR, None)];
  let mut term2id = HashMap::new();
  term2id.insert(EPS, 0);
  term2id.insert(EOF, 1);
  term2id.insert(ERR, 2);

  for (pri, pri_row) in priority.iter().enumerate() {
    let pri_assoc = (pri as u32, pri_row.assoc);
    for term in pri_row.terms.iter().map(String::as_str) {
      if term == EPS || term == EOF || term == ERR {
        return Err(format!("cannot assign priority to builtin term `{}`", term));
      } else if !VALID_NAME.is_match(term) {
        return Err(format!("term is not a valid variable name: `{}`", term));
      } else if term2id.contains_key(term) {
        return Err(format!("duplicate term when assigning priority: `{}`", term));
      } else {
        term2id.insert(term, terms.len() as u32);
        terms.push((term, Some(pri_assoc)));
      }
    }
  }

  for l in lexical {
    let (_, term) = (l.0.as_str(), l.1.as_str());
    if term != EOF && term != ERR && term != EPS && !VALID_NAME.is_match(term) {
      return Err(format!("term is not a valid variable name: `{}`", term));
    }
    term2id.entry(term).or_insert_with(|| {
      let id = terms.len() as u32;
      terms.push((term, None));
      id
    });
  }
  Ok((terms, term2id))
}