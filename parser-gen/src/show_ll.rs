// show ll1 ps table
use common::{grammar::Grammar, BitSet};
use std::fmt::Write;
use ll1_core::{LLTable, LLCtx};

pub fn show_prod_token(g: &Grammar) -> String {
  let mut s = "Productions:\n".to_owned();
  for i in 0..g.prod.len() as u32 { let _ = writeln!(s, "  {}: {}", i, g.show_prod(i, None)); }
  s += "\nTokens:\n";
  for i in 0..g.token_num() as u32 { let _ = writeln!(s, "  {}: {}", i, g.show_token(i)); }
  s.push('\n');
  s
}

pub fn table(ll: &LLCtx, g: &Grammar) -> String {
  let mut s = show_prod_token(g);
  for (idx, t) in ll.table.iter().enumerate() {
    let _ = writeln!(s, "{}:", g.show_token(idx as u32));
    let mut show_set = |name: &str, set: &BitSet| {
      let _ = write!(s, "{}:", name);
      for i in 0..set.inner_len() * 64 { // this is quite ugly, may be I will encapsulate it later on
        if set.test(i) { let _ = write!(s, " {}", g.show_token(i as u32)); }
      }
      s.push('\n');
    };
    show_set("first", &ll.first.0[idx]);
    show_set("follow", &ll.follow.0[idx]);
    // this is not necessary, but sorting it will provide better readability
    let mut t = t.iter().map(|(ch, prod)| (*ch, prod)).collect::<Vec<_>>();
    t.sort_unstable_by_key(|x| x.0);
    for (ch, prod) in t {
      let _ = write!(s, "  {} => ", g.show_token(ch));
      for (idx, &prod) in prod.iter().enumerate() {
        let prod = g.show_prod(prod, None);
        if idx == 0 { s += &prod; } else { let _ = write!(s, "; {}(✗)", prod); }
      }
      s.push('\n');
    }
    s.push('\n');
  }
  s
}

pub fn conflict(table: &LLTable, g: &Grammar) -> Vec<String> {
  let mut ret = Vec::new();
  for entry in table {
    for (&predict, prod_ids) in entry {
      if prod_ids.len() > 1 {
        let first_prod = g.show_prod(prod_ids[0], None);
        for &other in prod_ids.iter().skip(1) {
          ret.push(format!("conflict at prod \"{}\" and \"{}\", both's PS contains \"{}\"",
                           first_prod, g.show_prod(other, None), g.show_token(predict)));
        }
      }
    }
  }
  ret
}