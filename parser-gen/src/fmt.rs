use common::{grammar::{Grammar, ERR}, HashMap};
use std::fmt::{self, Write, Display};
use re2dfa::Dfa;
use lalr1_core::Table;

pub struct CommaSep<I>(pub I);

// `Clone` is required, because we can't use `.next(&mut self)` on `self.0`
impl<T: Display, I: Iterator<Item=T> + Clone> Display for CommaSep<I> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // error is impossible in out context
    for t in self.0.clone() { let _ = write!(f, "{}, ", t); }
    Ok(())
  }
}

pub fn min_u(x: usize) -> &'static str {
  // I don't think any number beyond `u32` is really possible
  match x { 0..=255 => "u8", 256..=65535 => "u16", _ => "u32" }
}

pub fn gather_types<'a>(g: &Grammar<'a>) -> (Vec<&'a str>, HashMap<&'a str, u32>) {
  let mut types = Vec::new();
  let mut types2id = HashMap::new();
  for nt in &g.nt {
    types2id.entry(nt.ty).or_insert_with(|| {
      let id = types.len() as u32;
      types.push(nt.ty);
      id
    });
  }
  (types, types2id)
}

pub fn acc(g: &Grammar, dfa: &Dfa, namespace: &str) -> String {
  let mut s = String::new();
  for &(acc, _) in &dfa.nodes {
    match acc {
      Some(acc) => { let _ = write!(s, "{}::{}, ", namespace, g.raw.lexical.get_index(acc as usize).unwrap().1); }
      None => { let _ = write!(s, "{}::{}, ", namespace, ERR); }
    }
  }
  s
}

pub fn dfa_edge(dfa: &Dfa, ec: &[u8], bracket: (char, char)) -> String {
  let mut s = String::new();
  let mut outs = vec![0; (*ec.iter().max().unwrap() + 1) as usize];
  for (_, edges) in dfa.nodes.iter() {
    for x in &mut outs { *x = 0; }
    for (&k, &out) in edges { outs[ec[k as usize] as usize] = out; }
    let _ = write!(s, "{}{}{}, ", bracket.0, CommaSep(outs.iter()), bracket.1);
  }
  s
}

pub fn goto(g: &Grammar, table: &Table, bracket: (char, char)) -> String {
  let mut s = String::new();
  for t in table {
    // iterate over all non-terminals
    let goto = CommaSep((g.terms.len()..g.token_num()).map(|x| t.goto.get(&(x as u32)).unwrap_or(&0)));
    let _ = write!(s, "{}{}{}, ", bracket.0, goto, bracket.1);
  }
  s
}