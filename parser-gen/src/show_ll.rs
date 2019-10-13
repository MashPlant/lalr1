// show ll1 ps table

use grammar_config::AbstractGrammarExt;
use std::fmt::Write;
use ll1_core::{LLTable, LLCtx};
use bitset::BitSet;

pub fn show_prod_token<'a>(g: &impl AbstractGrammarExt<'a>) -> String {
  let mut text = String::new();
  let _ = writeln!(text, "Productions:");
  for i in 0..g.prod_num() {
    let _ = writeln!(text, "  {}: {}", i, g.show_prod(i, None));
  }
  text.push('\n');

  let _ = writeln!(text, "Tokens:");
  for i in 0..g.token_num() {
    let _ = writeln!(text, "  {}: {}", i, g.show_token(i));
  }
  text.push('\n');
  text
}

pub fn table<'a>(ll: &LLCtx, g: &impl AbstractGrammarExt<'a>) -> String {
  let mut s = show_prod_token(g);
  for (idx, t) in ll.table.iter().enumerate() {
    let _ = writeln!(s, "{}:", g.show_token(idx as u32));
    let mut show_set = |name: &str, set: &BitSet| {
      let _ = write!(s, "{}:", name);
      for i in 0..set.inner_len() * 64 { // this is quite ugly, may be I will encapsulate it later on
        if set.test(i) {
          let _ = write!(s, " {}", g.show_token(i as u32));
        }
      }
      let _ = writeln!(s);
    };
    show_set("first", &ll.first.first[idx]);
    show_set("follow", &ll.follow.follow[idx]);
    // this is not necessary, but sorting it will provide better readability
    let mut t = t.iter().map(|(ch, prod)| (*ch, prod)).collect::<Vec<_>>();
    t.sort_unstable_by_key(|x| x.0);
    for (ch, prod) in t {
      let _ = write!(s, "  {} => ", g.show_token(ch as u32));
      for (idx, &prod) in prod.iter().enumerate() {
        if idx == 0 {
          let _ = write!(s, "{}", g.show_prod(prod, None));
        } else {
          let _ = write!(s, "; {}(✗)", g.show_prod(prod, None));
        }
      }
      let _ = writeln!(s);
    }
    let _ = writeln!(s);
  }
  s
}

pub fn conflict<'a>(table: &LLTable, g: &impl AbstractGrammarExt<'a>) -> Vec<String> {
  let mut ret = Vec::new();
  for entry in table {
    for (&predict, prod_ids) in entry {
      if prod_ids.len() > 1 {
        let first_prod = g.show_prod(prod_ids[0], None);
        for &other in prod_ids.iter().skip(1) {
          ret.push(format!("Conflict at prod `{}` and `{}`, both's PS contains term `{}`.", first_prod,
                           g.show_prod(other, None), g.show_token(predict)));
        }
      }
    }
  }
  ret
}