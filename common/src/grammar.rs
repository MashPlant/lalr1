use serde::Deserialize;
use std::borrow::Cow;
use crate::*;

pub type ProdVec = SmallVec<[u32; 4]>;

#[derive(Copy, Clone, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Assoc { Left, Right, NoAssoc }

// previously I support state/act just like lex/flex, later I found they are not necessary in my application and removed them
//
// we are using str, not String here, because in most of my application we work with borrowed string
// if you need to dynamically generate strings and add them to RawGrammar, you can use a typed_arena::Arena to store them
#[derive(Deserialize)]
pub struct RawGrammar<'a> {
  pub include: &'a str,
  pub epilogue: Option<&'a str>,
  pub priority: Vec<RawPriorityRow<'a>>,
  // map re to term
  // K must be Cow<str>, because sometimes we have to write escape chars in the key string
  // so the key may not be a borrow from the input string
  // but we can always avoid escape chars in the value string
  pub lexical: IndexMap<Cow<'a, str>, &'a str>,
  // this string should contain full field definition, e.g.: "a: u32, b: u32,"
  #[serde(default)] pub lexer_field: &'a str,
  // run before Lexer::next() returns
  #[serde(default)] pub lexer_action: &'a str,
  #[serde(default)] pub parser_field: &'a str,
  pub start: &'a str,
  pub production: Vec<RawProduction<'a>>,
  // None -> will define a struct Parser { parser_field }
  // Some -> will not define a struct (the original code has already defined it)
  pub parser_def: Option<&'a str>,
}

// start nt is the non-terminal that we manually add to the grammar with production "_ -> UserStart"
pub const START_NT_NAME: &str = "_";
pub const EPS: &str = "_Eps";
pub const EOF: &str = "_Eof";
pub const ERR: &str = "_Err";
pub const EPS_IDX: usize = 0;
pub const EOF_IDX: usize = 1;
pub const ERR_IDX: usize = 2;

#[derive(Deserialize)]
pub struct RawPriorityRow<'a> {
  pub assoc: Assoc,
  #[serde(borrow)]
  pub terms: Vec<&'a str>,
}

#[derive(Deserialize)]
pub struct RawProduction<'a> {
  pub lhs: &'a str,
  pub ty: &'a str,
  pub rhs: Vec<RawProductionRhs<'a>>,
}

#[derive(Deserialize)]
pub struct RawProductionRhs<'a> {
  pub rhs: Vec<&'a str>,
  // this is basically for the type checking for parser-macros
  // it would not be pleasing if you provide it from toml config file(but you can, any way)
  // when it exists, it must have the same size as `rhs`, and each element is a (name, type) pair
  pub rhs_arg: Option<Vec<(&'a str, &'a str)>>,
  pub act: &'a str,
  pub prec: Option<&'a str>,
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
fn parse_term<'a>(priority: &'a [RawPriorityRow], lexical: &'a IndexMap<Cow<'a, str>, &'a str>, validate_name: bool) -> Result<(Vec<Term<'a>>, HashMap<&'a str, u32>), String> {
  let mut terms = vec![Term { name: EPS, pri_assoc: None }, Term { name: EOF, pri_assoc: None }, Term { name: ERR, pri_assoc: None }];
  let mut term2id = HashMap::default();
  term2id.insert(EPS, 0);
  term2id.insert(EOF, 1);
  term2id.insert(ERR, 2);

  for (pri, pri_row) in priority.iter().enumerate() {
    let pri_assoc = (pri as u32, pri_row.assoc);
    for &name in &pri_row.terms {
      if validate_name && !validate_variable_name(name) {
        return Err(format!("term is not a valid variable name: \"{}\"", name));
      } else if term2id.contains_key(name) {
        return Err(format!("duplicate term when assigning priority: \"{}\"", name));
      } else {
        term2id.insert(name, terms.len() as u32);
        terms.push(Term { name, pri_assoc: Some(pri_assoc) });
      }
    }
  }

  for (_, &name) in lexical {
    if name != EOF && name != ERR && name != EPS && validate_name && !validate_variable_name(name) {
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
  pub raw: &'a RawGrammar<'a>,
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
  pub args: Option<&'a Vec<(&'a str, &'a str)>>,
  // start counting from 0, instead of `terms.len()`
  pub lhs: u32,
  // index in prod
  pub id: u32,
  pub pri: Option<u32>,
}

impl RawGrammar<'_> {
  // will add a production _Start -> Start, so need mut
  // if `validate_name == true`, will call `validate_variable_name` to check every token's name
  // otherwise those names will not be checked
  pub fn extend(&mut self, validate_name: bool) -> Result<Grammar, String> {
    let (terms, term2id) = parse_term(&self.priority, &self.lexical, validate_name)?;
    let mut nt = Vec::new();
    let mut nt2id = HashMap::default();

    if self.production.is_empty() { return Err("grammar must have at least one production rule".to_owned()); }

    // 2 pass scan, so a non-term can be used before declared

    // add non-term START_NT_NAME ("_") and related rule (_ -> UserStart) to productions
    // this name will not conflict with any user-input name, because they are not allowed to start with '_'
    // it must be done before any borrow operation, otherwise the compiler will complain
    self.production.push(RawProduction {
      lhs: START_NT_NAME,
      ty: "", // won't be used
      rhs: vec![RawProductionRhs { rhs: vec![self.start], act: "_1", rhs_arg: None, prec: None }],
    });

    for (idx, prod) in self.production.iter().enumerate() {
      let lhs = prod.lhs;
      // _Start is at `self.production.len() - 1`, this name is invalid, but won't cause error
      if validate_name && !validate_variable_name(lhs) && idx != self.production.len() - 1 {
        return Err(format!("non-term is not a valid variable name: \"{}\"", lhs));
      } else if term2id.contains_key(lhs) {
        return Err(format!("non-term has a duplicate name with term: \"{}\"", lhs));
      } else {
        match nt2id.get(lhs) {
          None => {
            let id = nt.len() as u32;
            nt.push(NonTerm { name: lhs, ty: &prod.ty, start_idx: 0 }); // fill `start_idx` later
            nt2id.insert(lhs, id);
          }
          Some(&old) => if prod.ty != nt[old as usize].ty {
            return Err(format!("non-term \"{}\" is assigned to different types: \"{}\" and \"{}\"", lhs, nt[old as usize].ty, prod.ty));
          }
        };
      }
    }
    // set the type of _Start the same as Start
    nt.last_mut().unwrap().ty = nt[*nt2id.get(self.start).ok_or_else(||
      format!("start non-term \"{}\" undefined", self.start))? as usize].ty;

    let mut prod = vec![Vec::new(); nt.len()];
    for raw_prod in &self.production {
      let lhs = nt2id[raw_prod.lhs];
      let lhs_prod = &mut prod[lhs as usize];
      for rhs in &raw_prod.rhs {
        let mut prod_rhs = ProdVec::new();
        let mut prod_pri = None;
        for rhs in &rhs.rhs {
          // impossible to have a (Some(), Some()) here, because we have checked that term & non-term don't have any duplicate name
          match (nt2id.get(rhs), term2id.get(rhs)) {
            (Some(&nt), _) => prod_rhs.push(nt + terms.len() as u32),
            (_, Some(&t)) => {
              prod_rhs.push(t);
              prod_pri = terms[t as usize].pri_assoc.map(|(pri, _)| pri);
            }
            _ => return Err(format!("production rhs contains undefined token: \"{}\"", rhs)),
          }
        }
        if let Some(prec) = rhs.prec.as_ref() {
          match term2id.get(prec) {
            None => return Err(format!("prec uses undefined term: \"{}\"", prec)),
            Some(&t) => prod_pri = terms[t as usize].pri_assoc.map(|(pri, _)| pri),
          }
        }
        lhs_prod.push(Prod { rhs: prod_rhs, act: &rhs.act, args: rhs.rhs_arg.as_ref(), lhs, id: 0, pri: prod_pri });

        // type checking
        if let Some(rhs_arg) = &rhs.rhs_arg {
          if rhs_arg.len() != rhs.rhs.len() {
            return Err(format!("production \"{} -> {}\" rhs and method arguments have different length: {} vs {}",
              raw_prod.lhs, rhs.rhs.join(" "), rhs.rhs.len(), rhs_arg.len()));
          }
          for (&rhs_tk, &(_, rhs_ty)) in rhs.rhs.iter().zip(rhs_arg.iter()) {
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
  pub fn as_nt(&self, ch: u32) -> Option<usize> { (ch as usize).checked_sub(self.terms.len()) }

  // get all productions with lhs = parameter `lhs`
  // please note that parameter `lhs` should be within [0, nt.len()), not [terms.len(), terms.len() + nt.len())
  pub fn get_prod(&self, lhs: usize) -> &[Prod] {
    let start = self.nt[lhs].start_idx;
    let end = self.nt.get(lhs + 1).map(|x| x.start_idx).unwrap_or(self.prod.len());
    &self.prod[start..end]
  }

  // parameter `id` is a general id (in [0, terms.len() + nt.len()))
  pub fn show_token(&self, id: usize) -> &str {
    self.terms.get(id).map(|x| x.name).unwrap_or_else(|| self.nt[id - self.terms.len()].name)
  }

  // parameter `id` is a production id (in [0, prod.len()))
  pub fn show_prod<'a>(&'a self, id: usize, dot: Option<u32>) -> impl Display + 'a {
    fmt_::fn2display(move |f| {
      let prod = &self.prod[id];
      let _ = write!(f, "{} ->", self.nt[prod.lhs as usize].name);
      for (idx, &rhs) in prod.rhs.iter().enumerate() {
        let sep = if Some(idx as u32) == dot { '.' } else { ' ' };
        let _ = write!(f, "{}{}", sep, self.show_token(rhs as _));
      }
      if Some(prod.rhs.len() as u32) == dot { let _ = f.write_str("."); }
      Ok(())
    })
  }
}
