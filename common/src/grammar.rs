use serde::Deserialize;
use crate::{IndexMap, HashMap, SmallVec};

pub type ProdVec = SmallVec<[u32; 6]>;

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Assoc { Left, Right, NoAssoc }

// previously I support state/act just like lex/flex
// later I found they are not necessary in my application and removed them
#[derive(Debug, Deserialize)]
pub struct RawGrammar {
  pub include: String,
  pub priority: Vec<RawPriorityRow>,
  // map re to term
  pub lexical: IndexMap<String, String>,
  // this string should contain name & type
  // e.g.: "a: u32" for rust, "int a" for c++
  pub parser_field: Option<Vec<String>>,
  pub start: String,
  pub production: Vec<RawProduction>,
  // None -> will define a struct Parser<'a> { _p: std::marker::PhantomData<&'a ()>,  parser_field_ext }
  // Some -> will not define a struct
  #[serde(skip_serializing)]
  pub parser_def: Option<String>,
}

pub const EPS: &str = "_Eps";
pub const EOF: &str = "_Eof";
pub const ERR: &str = "_Err";

#[derive(Debug, Deserialize)]
pub struct RawPriorityRow {
  pub assoc: Assoc,
  pub terms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawProduction {
  pub lhs: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub rhs: Vec<RawProductionRhs>,
}

#[derive(Debug, Deserialize)]
pub struct RawProductionRhs {
  pub rhs: Vec<String>,
  // this is basically for the type checking for parser-macros
  // it would not be pleasing if you provide it from toml config file(but you can, any way)
  pub rhs_arg: Option<Vec<(Option<String>, String)>>,
  pub act: String,
  pub prec: Option<String>,
}

pub fn validate_variable_name(s: &str) -> bool {
  let mut chs = s.chars();
  match chs.next() {
    Some(ch) if ch.is_ascii_alphabetic() => chs.all(|ch| ch.is_ascii_alphanumeric() || ch == '_'),
    _ => false,
  }
}

// input: the two field in RawGrammar(or constructed in other ways)
// return: (Vec<(term, pri_assoc)>, term2id)
fn parse_term<'a>(priority: &'a [RawPriorityRow], lexical: &'a IndexMap<String, String>)
                  -> Result<(Vec<(&'a str, Option<(u32, Assoc)>)>, HashMap<&'a str, u32>), String> {
  let mut terms = vec![(EPS, None), (EOF, None), (ERR, None)];
  let mut term2id = HashMap::new();
  term2id.insert(EPS, 0);
  term2id.insert(EOF, 1);
  term2id.insert(ERR, 2);

  for (pri, pri_row) in priority.iter().enumerate() {
    let pri_assoc = (pri as u32, pri_row.assoc);
    for term in pri_row.terms.iter().map(String::as_str) {
      if term == EPS || term == EOF || term == ERR {
        return Err(format!("cannot assign priority to builtin term \"{}\"", term));
      } else if !validate_variable_name(term) {
        return Err(format!("term is not a valid variable name: \"{}\"", term));
      } else if term2id.contains_key(term) {
        return Err(format!("duplicate term when assigning priority: \"{}\"", term));
      } else {
        term2id.insert(term, terms.len() as u32);
        terms.push((term, Some(pri_assoc)));
      }
    }
  }

  for (_, term) in lexical {
    if term != EOF && term != ERR && term != EPS && !validate_variable_name(term) {
      return Err(format!("term is not a valid variable name: \"{}\"", term));
    }
    term2id.entry(term).or_insert_with(|| {
      let id = terms.len() as u32;
      terms.push((term, None));
      id
    });
  }
  Ok((terms, term2id))
}

#[derive(Debug)]
pub struct Grammar<'a> {
  pub raw: &'a RawGrammar,
  //                 name
  pub terms: Vec<(&'a str, Option<(u32, Assoc)>)>,
  //          (name   , type_  )>
  // nt.len() == prod.len()
  pub nt: Vec<(&'a str, &'a str)>,
  pub prod: Vec<Vec<(ProdVec, u32)>>,
  //                   (act, arg)                     (lhs, index of this prod in raw.prod[lhs]) pri
  pub prod_extra: Vec<((&'a str, Option<&'a Vec<(Option<String>, String)>>), (u32, u32), Option<u32>)>,
}

impl RawGrammar {
  // will add a production _Start -> Start, so need mut
  pub fn extend(&mut self) -> Result<Grammar, String> {
    let (terms, term2id) = parse_term(&self.priority, &self.lexical)?;
    let mut nt = Vec::new();
    let mut nt2id = HashMap::new();

    if self.production.is_empty() {
      return Err("grammar must have at least one production rule".into());
    }

    // 2 pass scan, so a non-term can be used before declared

    // getting production must be after this mut operation
    // this may seem stupid...
    let start = self.start.clone();
    self.production.push(RawProduction {
      lhs: format!("_{}", start),
      // determine later
      type_: String::new(),
      rhs: vec![RawProductionRhs {
        rhs: vec![start.clone()],
        act: "_1".to_owned(),
        // the type "" is invalid, but will not be checked
        rhs_arg: Some(vec![(Some("_1".to_owned()), String::new())]),
        prec: None,
      }],
    });

    for prod in &self.production {
      let lhs = prod.lhs.as_str();
      // again this may seem stupid...
      // self.production.last().unwrap().lhs is generated by the code above
      if !validate_variable_name(lhs) && lhs != &self.production.last().unwrap().lhs {
        return Err(format!("non-term is not a valid variable name: \"{}\"", lhs));
      } else if term2id.contains_key(lhs) {
        return Err(format!("non-term has a duplicate name with term: \"{}\"", lhs));
      } else {
        match nt2id.get(lhs) {
          None => {
            let id = nt.len() as u32;
            nt.push((lhs, prod.type_.as_str()));
            nt2id.insert(lhs, id);
          }
          Some(&old) => if prod.type_.as_str() != nt[old as usize].1 {
            return Err(format!("non-term \"{}\" is assigned to different types: \"{}\" and \"{}\"", lhs, nt[old as usize].1, prod.type_));
          }
        };
      }
    }
    // set the type of _Start the same as Start
    nt.last_mut().unwrap().1 = nt[nt2id[start.as_str()] as usize].1;

    let mut prod = vec![Vec::new(); nt.len()];
    let mut prod_extra = Vec::new();
    let mut prod_id = 0u32;

    for (idx, raw_prod) in self.production.iter().enumerate() {
      let lhs = nt2id.get(raw_prod.lhs.as_str()).unwrap();
      let lhs_prod = &mut prod[*lhs as usize];
      for rhs in &raw_prod.rhs {
        let mut prod_rhs = ProdVec::new();
        let mut prod_pri = None;
        for rhs in &rhs.rhs {
          let rhs = rhs.as_str();
          // impossible to have a (Some(), Some()) here
          match (nt2id.get(rhs), term2id.get(rhs)) {
            (Some(&nt), _) => prod_rhs.push(nt),
            (_, Some(&t)) => {
              prod_rhs.push(t + nt.len() as u32);
              prod_pri = terms[t as usize].1.map(|(pri, _)| pri);
            }
            _ => return Err(format!("production rhs contains undefined token: \"{}\"", rhs)),
          }
        }
        if let Some(prec) = rhs.prec.as_ref() {
          match term2id.get(prec.as_str()) {
            None => return Err(format!("prec uses undefined term: \"{}\"", prec)),
            Some(&t) => {
              prod_pri = terms[t as usize].1.map(|(pri, _)| pri);
            }
          }
        }
        let id = lhs_prod.len() as u32;
        lhs_prod.push((prod_rhs, prod_id));
        prod_extra.push(((rhs.act.as_str(), rhs.rhs_arg.as_ref()), (*lhs, id), prod_pri));
        prod_id += 1;

        // no type checking for _Start
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
                let nt_ty = &nt[nt_id as usize].1;
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
    Ok(Grammar { raw: self, nt, terms, prod, prod_extra })
  }
}

impl Grammar<'_> {
  pub fn start(&self) -> (u32, &(ProdVec, u32)) {
    let last = self.prod.len() - 1;
    (last as u32, &self.prod[last][0])
  }

  pub fn eps(&self) -> u32 { self.nt.len() as u32 }
  pub fn eof(&self) -> u32 { self.nt.len() as u32 + 1 }
  pub fn err(&self) -> u32 { self.nt.len() as u32 + 2 }

  pub fn token_num(&self) -> u32 { self.terms.len() as u32 + self.nt.len() as u32 }
  pub fn nt_num(&self) -> u32 { self.nt.len() as u32 }
  pub fn prod_num(&self) -> u32 { self.prod_extra.len() as u32 }

  pub fn get_prod(&self, lhs: u32) -> &[(ProdVec, u32)] { &self.prod[lhs as usize] }

  pub fn prod_pri(&self, id: u32) -> Option<u32> { self.prod_extra[id as usize].2 }

  pub fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)> { self.terms[ch as usize - self.nt.len()].1 }

  pub fn show_token(&self, id: u32) -> &str {
    let id = id as usize;
    self.nt.get(id).map(|x| x.0).unwrap_or_else(|| self.terms[id - self.nt.len()].0)
  }

  pub fn show_prod(&self, id: u32, dot: Option<u32>) -> String {
    let (_, (lhs, idx), _) = self.prod_extra[id as usize];
    let (prod, _) = &self.prod[lhs as usize][idx as usize];
    let mut s = format!("{} ->", self.nt[lhs as usize].0);
    for (idx, &rhs) in prod.iter().enumerate() {
      s.push(if Some(idx as u32) == dot { '.' } else { ' ' });
      s += self.show_token(rhs);
    }
    if Some(prod.len() as u32) == dot { s.push('.'); }
    s
  }
}