// show ll1 ps table
use common::{grammar::Grammar, *};
use std::fmt::Display;
use ll1_core::{LLTable, LLCtx};

pub fn show_prod_token<'a>(g: &'a Grammar) -> impl Display + 'a {
  fn2display(move |f| {
    let _ = f.write_str("Productions:\n");
    for i in 0..g.prod.len() { let _ = writeln!(f, "  {}: {}", i, g.show_prod(i, None)); }
    let _ = f.write_str("\nTokens:\n");
    for i in 0..g.token_num() { let _ = writeln!(f, "  {}: {}", i, g.show_token(i)); }
    let _ = f.write_str("\n");
  })
}

pub fn table<'a>(ll: &'a LLCtx, g: &'a Grammar) -> impl Display + 'a {
  fn2display(move |f| {
    let _ = write!(f, "{}", show_prod_token(g));
    for (idx, t) in ll.table.iter().enumerate() {
      let _ = writeln!(f, "{}:", g.show_token(idx));
      let mut show_set = |name: &str, set: &BitSet| {
        let _ = write!(f, "{}:", name);
        for i in 0..set.inner_len() * 64 { // this is quite ugly, may be I will encapsulate it later on
          if set.test(i) { let _ = write!(f, " {}", g.show_token(i)); }
        }
        let _ = f.write_str("\n");
      };
      show_set("first", &ll.first.0[idx]);
      show_set("follow", &ll.follow.0[idx]);
      // this is not necessary, but sorting it will provide better readability
      let mut t = t.iter().map(|(ch, prod)| (*ch, prod)).collect::<Vec<_>>();
      t.sort_unstable_by_key(|x| x.0);
      for (ch, prod) in t {
        let _ = write!(f, "  {} => ", g.show_token(ch as usize));
        for (idx, &prod) in prod.iter().enumerate() {
          let prod = g.show_prod(prod as usize, None);
          if idx == 0 { let _ = write!(f, "{}", prod); } else { let _ = write!(f, "; {}(✗)", prod); }
        }
        let _ = f.write_str("\n");
      }
      let _ = f.write_str("\n");
    }
  })
}

pub fn conflict(table: &LLTable, g: &Grammar) -> Vec<String> {
  let mut ret = Vec::new();
  for entry in table {
    for (&predict, prod_ids) in entry {
      if prod_ids.len() > 1 {
        let first_prod = g.show_prod(prod_ids[0] as usize, None);
        for &other in prod_ids.iter().skip(1) {
          ret.push(format!("conflict at prod \"{}\" and \"{}\", both's PS contains \"{}\"",
                           first_prod, g.show_prod(other as usize, None), g.show_token(predict as usize)));
        }
      }
    }
  }
  ret
}