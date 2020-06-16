use serde::Deserialize;
use crate::{IndexMap, HashMap, SmallVec, ToUsize};
use std::ops::Range;

pub type ProdVec = SmallVec<[u32; 4]>;

#[derive(Copy, Clone, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Assoc { Left, Right, NoAssoc }

// previously I support state/act just like lex/flex, later I found they are not necessary in my application and removed them
#[derive(Deserialize)]
pub struct RawGrammar {
  pub include: String,
  pub priority: Vec<RawPriorityRow>,
  // map re to term
  pub lexical: IndexMap<String, String>,
  // this string should contain name & type, e.g.: "a: u32" for rust, "int a" for c++
  pub parser_field: Option<Vec<String>>,
  pub start: String,
  pub production: Vec<RawProduction>,
  // None -> will define a struct Parser<'a> { _p: std::marker::PhantomData<&'a ()>, parser_field_ext }
  // Some -> will not define a struct (the original code has already defined it)
  pub parser_def: Option<String>,
}

pub const EPS: &str = "_Eps";
pub const EOF: &str = "_Eof";
pub const ERR: &str = "_Err";
pub const EPS_IDX: usize = 0;
pub const EOF_IDX: usize = 1;
pub const ERR_IDX: usize = 2;


#[derive(Deserialize)]
pub struct RawPriorityRow {
  pub assoc: Assoc,
  pub terms: Vec<String>,
}

#[derive(Deserialize)]
pub struct RawProduction {
  pub lhs: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub rhs: Vec<RawProductionRhs>,
}

#[derive(Deserialize)]
pub struct RawProductionRhs {
  pub rhs: Vec<String>,
  // this is basically for the type checking for parser-macros
  // it would not be pleasing if you provide it from toml config file(but you can, any way)
  pub rhs_arg: Option<Vec<(Option<String>, String)>>,
  pub act: String,
  pub prec: Option<String>,
}

// note: EPS/EOF/ERR's contents are not valid variable names
pub fn validate_variable_name(s: &str) -> bool {
  let mut chs = s.chars();
  match chs.next() {
    Some(ch) if ch.is_ascii_alphabetic() => chs.all(|ch| ch.is_ascii_alphanumeric() || ch == '_'),
    _ => false,
  }
}

// input: the two field in RawGrammar(or constructed in other ways)
// return: (Vec<(term, pri_assoc)>, term2id)
fn parse_term<'a>(priority: &'a [RawPriorityRow], lexical: &'a IndexMap<String, String>) -> Result<(Vec<Term<'a>>, HashMap<&'a str, u32>), String> {
  let mut terms = vec![Term { name: EPS, pri_assoc: None }, Term { name: EOF, pri_assoc: None }, Term { name: ERR, pri_assoc: None }];
  let mut term2id = HashMap::new();
  term2id.insert(EPS, 0);
  term2id.insert(EOF, 1);
  term2id.insert(ERR, 2);

  for (pri, pri_row) in priority.iter().enumerate() {
    let pri_assoc = (pri as u32, pri_row.assoc);
    for name in pri_row.terms.iter().map(String::as_str) {
      if !validate_variable_name(name) {
        return Err(format!("term is not a valid variable name: \"{}\"", name));
      } else if term2id.contains_key(name) {
        return Err(format!("duplicate term when assigning priority: \"{}\"", name));
      } else {
        term2id.insert(name, terms.len() as u32);
        terms.push(Term { name, pri_assoc: Some(pri_assoc) });
      }
    }
  }

  for (_, name) in lexical {
    if name != EOF && name != ERR && name != EPS && !validate_variable_name(name) {
      return Err(format!("term is not a valid variable name: \"{}\"", name));
    }
    term2id.entry(name).or_insert_with(|| {
      let id = terms.len() as u32;
      terms.push(Term { name, pri_assoc: None });
      id
    });
  }
  Ok((terms, term2id))
}

// terminal id is distributed in [0, terms.len())
// non-terminal id is distributed in [terms.len(), terms.len() + nt.len())
// there are 3 fixed terminal id: EPS_IDX, EOF_IDX, ERR_IDX (of course they are in [0, terms.len()))
pub struct Grammar<'a> {
  pub raw: &'a RawGrammar,
  pub terms: Vec<Term<'a>>,
  pub nt: Vec<NonTerm<'a>>,
  pub prod: Vec<Prod<'a>>,
}

pub struct Term<'a> {
  pub name: &'a str,
  pub pri_assoc: Option<(u32, Assoc)>,
}

pub struct NonTerm<'a> {
  pub name: &'a str,
  pub ty: &'a str,
  // starting index in `prod`, all prods until next nt's `start_idx` (or end) belong to this nt
  pub start_idx: usize,
}

#[derive(Clone)]
pub struct Prod<'a> {
  pub rhs: ProdVec,
  pub act: &'a str,
  pub args: Option<&'a Vec<(Option<String>, String)>>,
  // start counting from 0, instead of `terms.len()`
  pub lhs: u32,
  // index in prod
  pub id: u32,
  pub pri: Option<u32>,
}

impl RawGrammar {
  // will add a production _Start -> Start, so need mut
  pub fn extend(&mut self) -> Result<Grammar, String> {
    let (terms, term2id) = parse_term(&self.priority, &self.lexical)?;
    let mut nt = Vec::new();
    let mut nt2id = HashMap::new();

    if self.production.is_empty() { return Err("grammar must have at least one production rule".to_owned()); }

    // 2 pass scan, so a non-term can be used before declared

    // add non-term _Start and related rule (_Start -> Start) to productions
    // this name will not conflict with any user-input name, because they are not allowed to start with '_'
    // it must be done before any borrow operation, otherwise the compiler will complain
    let start = self.start.clone();
    self.production.push(RawProduction {
      lhs: format!("_{}", start),
      type_: String::new(), // won't be used
      rhs: vec![RawProductionRhs {
        rhs: vec![start.clone()],
        act: "_1".to_owned(),
        // the type "" is invalid, but it will not be checked
        rhs_arg: Some(vec![(Some("_1".to_owned()), String::new())]),
        prec: None,
      }],
    });

    for (idx, prod) in self.production.iter().enumerate() {
      let lhs = prod.lhs.as_str();
      // _Start is at `self.production.len() - 1`, this name is invalid, but won't cause error
      if !validate_variable_name(lhs) && idx != self.production.len() - 1 {
        return Err(format!("non-term is not a valid variable name: \"{}\"", lhs));
      } else if term2id.contains_key(lhs) {
        return Err(format!("non-term has a duplicate name with term: \"{}\"", lhs));
      } else {
        match nt2id.get(lhs) {
          None => {
            let id = nt.len() as u32;
            nt.push(NonTerm { name: lhs, ty: &prod.type_, start_idx: 0 }); // fill `start_idx` later
            nt2id.insert(lhs, id);
          }
          Some(&old) => if prod.type_.as_str() != nt[old as usize].ty {
            return Err(format!("non-term \"{}\" is assigned to different types: \"{}\" and \"{}\"", lhs, nt[old as usize].ty, prod.type_));
          }
        };
      }
    }
    // set the type of _Start the same as Start
    nt.last_mut().unwrap().ty = nt[nt2id[start.as_str()] as usize].ty;

    let mut prod = vec![Vec::new(); nt.len()];
    for (idx, raw_prod) in self.production.iter().enumerate() {
      let lhs = nt2id[raw_prod.lhs.as_str()];
      let lhs_prod = &mut prod[lhs as usize];
      for rhs in &raw_prod.rhs {
        let mut prod_rhs = ProdVec::new();
        let mut prod_pri = None;
        for rhs in &rhs.rhs {
          // impossible to have a (Some(), Some()) here, because we have checked that term & non-term don't have any duplicate name
          match (nt2id.get(rhs.as_str()), term2id.get(rhs.as_str())) {
            (Some(&nt), _) => prod_rhs.push(nt + terms.len() as u32),
            (_, Some(&t)) => {
              prod_rhs.push(t);
              prod_pri = terms[t as usize].pri_assoc.map(|(pri, _)| pri);
            }
            _ => return Err(format!("production rhs contains undefined token: \"{}\"", rhs)),
          }
        }
        if let Some(prec) = rhs.prec.as_ref() {
          match term2id.get(prec.as_str()) {
            None => return Err(format!("prec uses undefined term: \"{}\"", prec)),
            Some(&t) => prod_pri = terms[t as usize].pri_assoc.map(|(pri, _)| pri),
          }
        }
        lhs_prod.push(Prod { rhs: prod_rhs, act: &rhs.act, args: rhs.rhs_arg.as_ref(), lhs, id: 0, pri: prod_pri });

        // no type checking for _Start, it must be valid
        if idx == self.production.len() - 1 { break; }
        // type checking
        if let Some(rhs_arg) = &rhs.rhs_arg {
          if rhs_arg.len() != rhs.rhs.len() {
            return Err(format!("production \"{} -> {}\" rhs and method arguments have different length: {} vs {}",
                               raw_prod.lhs, rhs.rhs.join(" "), rhs.rhs.len(), rhs_arg.len()));
          }
          for (rhs_tk, (_, rhs_ty)) in rhs.rhs.iter().zip(rhs_arg.iter()) {
            let rhs_tk = rhs_tk.as_str();
            match (nt2id.get(rhs_tk), term2id.get(rhs_tk)) {
              (Some(&nt_id), _) => {
                let nt_ty = nt[nt_id as usize].ty;
                if nt_ty != rhs_ty {
                  return Err(format!("production \"{} -> {}\" rhs and method arguments have conflict signature: `{}` requires `{}`, while method takes `{}`",
                                     raw_prod.lhs, rhs.rhs.join(" "), rhs_tk, nt_ty, rhs_ty));
                }
              }
              (_, Some(_)) => if !rhs_ty.starts_with("Token") { // maybe user will use some lifetime specifier
                return Err(format!("production \"{} -> {}\" rhs and method arguments have conflict signature: `{}` requires Token, while method takes `{}`",
                                   raw_prod.lhs, rhs.rhs.join(" "), rhs_tk, rhs_ty));
              }
              _ => {} // unreachable, because checked above
            }
          }
        }
      }
    }
    let mut start_idx = 0;
    for (nt, prods) in nt.iter_mut().zip(prod.iter_mut()) {
      nt.start_idx = start_idx;
      start_idx += prods.len();
    }
    let mut prod = prod.into_iter().flat_map(|x| x.into_iter()).collect::<Vec<_>>();
    for (idx, prod) in prod.iter_mut().enumerate() { prod.id = idx as u32; }
    Ok(Grammar { raw: self, nt, terms, prod })
  }
}

impl Grammar<'_> {
  pub fn start(&self) -> (u32, &Prod) {
    (self.nt.len() as u32 - 1, self.prod.last().unwrap())
  }

  pub fn token_num(&self) -> usize { self.terms.len() + self.nt.len() }
  // try to convert a general id (in [0, terms.len() + nt.len())) to a index in `nt` (result is in [0, nt.len()))
  pub fn as_nt(&self, ch: impl ToUsize) -> Option<usize> { ch.usize().checked_sub(self.terms.len()) }
  pub fn nt_range(&self) -> Range<usize> { self.terms.len()..self.token_num() }

  pub fn get_prod(&self, ch: impl ToUsize) -> &[Prod] {
    let lhs = ch.usize();
    let start = self.nt[lhs].start_idx;
    let end = self.nt.get(lhs + 1).map(|x| x.start_idx).unwrap_or(self.prod.len());
    &self.prod[start..end]
  }

  pub fn show_token(&self, id: usize) -> &str {
    self.terms.get(id).map(|x| x.name).unwrap_or_else(|| self.nt[id - self.terms.len()].name)
  }

  pub fn show_prod(&self, id: usize, dot: Option<u32>) -> String {
    let prod = &self.prod[id];
    let mut s = format!("{} ->", self.nt[prod.lhs as usize].name);
    for (idx, &rhs) in prod.rhs.iter().enumerate() {
      s.push(if Some(idx as u32) == dot { '.' } else { ' ' });
      s += self.show_token(rhs as usize);
    }
    if Some(prod.rhs.len() as u32) == dot { s.push('.'); }
    s
  }
}