use common::{grammar::Grammar, re2dfa::Dfa, *};
use std::fmt::{Write, Display};
use lalr1_core::{Table, TableEntry, Act};

#[inline(always)]
pub fn comma_sep<T: Display>(it: impl Iterator<Item=T> + Clone) -> impl Display {
  fn2display(move |f| { for t in it.clone() { let _ = write!(f, "{}, ", t); } })
}

#[inline(always)]
pub fn min_u(x: usize) -> &'static str {
  // I don't think any number beyond `u32` is really possible
  match x { 0..=255 => "u8", 256..=65535 => "u16", _ => "u32" }
}

pub fn gather_types<'a>(g: &Grammar<'a>) -> (Vec<&'a str>, HashMap<&'a str, u32>) {
  let mut types = Vec::new();
  let mut types2id = HashMap::default();
  for nt in &g.nt {
    types2id.entry(nt.ty).or_insert_with(|| {
      let id = types.len() as u32;
      types.push(nt.ty);
      id
    });
  }
  (types, types2id)
}

pub fn acc<'a>(g: &'a Grammar, dfa: &'a Dfa, namespace: &'a str) -> impl Display + 'a {
  fn2display(move |f| {
    for &(acc, _) in &dfa.nodes {
      match acc {
        Some(acc) => { let _ = write!(f, "{}::{}, ", namespace, g.raw.lexical.get_index(acc as usize).unwrap().1); }
        None => { let _ = write!(f, "{}::_Err, ", namespace); }
      }
    }
  })
}

pub fn dfa_edge<'a>(dfa: &'a Dfa, bracket: (char, char)) -> impl Display + 'a {
  fn2display(move |f| {
    let mut outs = [0; 256];
    for (_, edges) in dfa.nodes.iter() {
      for x in outs.iter_mut() { *x = 0; }
      for (&k, &out) in edges { outs[k as usize] = out; }
      let _ = write!(f, "{}{}{}, ", bracket.0, comma_sep(outs.iter().take(dfa.ec_num)), bracket.1);
    }
  })
}

pub fn goto<'a>(g: &'a Grammar, table: &'a Table, bracket: (char, char)) -> impl Display + 'a {
  fn2display(move |f| {
    for t in table {
      // iterate over all non-terminals
      let goto = comma_sep((g.terms.len()..g.token_num()).map(|x| t.goto.get(&(x as u32)).unwrap_or(&0)));
      let _ = write!(f, "{}{}{}, ", bracket.0, goto, bracket.1);
    }
  })
}

pub fn action<'a>(g: &'a Grammar, table: &'a Table) -> impl Display + 'a {
  fn2display(move |f| {
    for TableEntry { act, .. } in table {
      let _ = f.write_char('{');
      for i in 0..g.terms.len() as u32 {
        let (tag, val) = act.get(&i).and_then(|x| x.get(0))
          .map(|&x| match x { Act::Acc => (2, 0), Act::Shift(x) => (0, x), Act::Reduce(x) => (1, x) })
          .unwrap_or((3, 0));
        let _ = write!(f, "{}, ", tag | (val << 2));
      };
      let _ = f.write_str("},");
    }
  })
}