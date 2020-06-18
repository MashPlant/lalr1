use common::{grammar::{Grammar, ERR}, HashMap};
use std::fmt::Write;
use re2dfa::Dfa;
use lalr1_core::Table;

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

pub fn token_kind(g: &Grammar) -> String {
  let mut s = String::new();
  for t in &g.terms { let _ = write!(s, "{}, ", t.name); }
  s
}

pub fn acc(g: &Grammar, dfa: &Dfa) -> String {
  let mut s = String::new();
  for &(acc, _) in &dfa.nodes {
    match acc {
      Some(acc) => { let _ = write!(s, "TokenKind::{}, ", g.raw.lexical.get_index(acc as usize).unwrap().1); }
      None => { let _ = write!(s, "TokenKind::{}, ", ERR); }
    }
  }
  s
}

pub fn ec(ec: &[u8]) -> String {
  let mut s = String::new();
  for &ch in ec { let _ = write!(s, "{}, ", ch); }
  s
}

pub fn dfa_edge(dfa: &Dfa, ec: &[u8], bracket: (char, char)) -> String {
  let mut s = String::new();
  let mut outs = vec![0; (*ec.iter().max().unwrap() + 1) as usize];
  for (_, edges) in dfa.nodes.iter() {
    for x in &mut outs { *x = 0; }
    for (&k, &out) in edges { outs[ec[k as usize] as usize] = out; }
    let _ = write!(s, "{}", bracket.0);
    for &x in &outs { let _ = write!(s, "{}, ", x); }
    let _ = write!(s, "{}, ", bracket.1);
  }
  s
}

pub fn goto(g: &Grammar, table: &Table, bracket: (char, char)) -> String {
  let mut s = String::new();
  for t in table {
    let _ = write!(s, "{}", bracket.0);
    // iterate over all non-terminals
    for i in g.terms.len()..g.token_num() { let _ = write!(s, "{}, ", t.goto.get(&(i as u32)).unwrap_or(&0)); }
    let _ = write!(s, "{}, ", bracket.1);
  }
  s
}