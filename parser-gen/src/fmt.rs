use crate::*;

#[inline(always)]
pub fn comma_sep<'a, T: Display + 'a>(it: impl Iterator<Item=T> + Clone + 'a) -> impl Display + 'a {
  fmt_::sep(it, ",")
}

#[inline(always)]
pub fn min_u(x: usize) -> &'static str {
  // I don't think any number beyond `u32` is possible
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
  fmt_::fn2display(move |f| (for &(acc, _) in &dfa.nodes {
    match acc {
      Some(acc) => { let _ = write!(f, "{}::{}, ", namespace, g.raw.lexical.get_index(acc as usize).unwrap().1); }
      None => { let _ = write!(f, "{}::_Err, ", namespace); }
    }
  }, Ok(())).1)
}

pub fn dfa_edge<'a>(dfa: &'a Dfa, bracket: (char, char)) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    for (_, edges) in dfa.nodes.iter() {
      let mut outs = [0; 256];
      for (&k, &out) in edges {
        assert_ne!(out, 0);
        outs[k as usize] = out; }
      write!(f, "{}{}{},", bracket.0, comma_sep(outs.iter().take(dfa.ec_num)), bracket.1)?;
    }
    Ok(())
  })
}

pub fn goto<'a>(g: &'a Grammar, table: &'a Table, bracket: (char, char)) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    for t in table {
      // iterate over all non-terminals
      let goto = comma_sep((g.terms.len()..g.token_num()).map(|x| t.goto.get(&(x as u32)).unwrap_or(&0)));
      write!(f, "{}{}{},", bracket.0, goto, bracket.1)?;
    }
    Ok(())
  })
}

pub fn action<'a>(g: &'a Grammar, table: &'a Table, bracket: (char, char)) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    for TableEntry { act, .. } in table {
      f.write_char(bracket.0)?;
      for i in 0..g.terms.len() as u32 {
        let (tag, val) = act.get(&i).and_then(|x| x.get(0))
          .map(|&x| match x { Act::Acc => (2, 0), Act::Shift(x) => (0, x), Act::Reduce(x) => (1, x) })
          .unwrap_or((3, 0));
        write!(f, "{},", tag | (val << 2))?;
      };
      write!(f, "{},", bracket.1)?;
    }
    Ok(())
  })
}