// show ll1 ps table

use grammar_config::Grammar;
use std::fmt::Write;
use ll1_core::LLTable;

pub fn text(table: &LLTable, g: &Grammar) -> String {
  let mut s = String::new();
  for (idx, t) in table.iter().enumerate() {
    let _ = writeln!(s, "{}:", g.show_token(idx as u32));
    // this is not necessary, but sorting it will provide better readability
    let mut t = t.iter().map(|(ch, prod)| (*ch, prod)).collect::<Vec<_>>();
    t.sort_unstable_by_key(|x| x.0);
    for (ch, prod) in t {
      let _ = write!(s, "  {} => ", g.show_token(ch as u32));
      for (idx, &prod) in prod.iter().enumerate() {
        if idx == 0 {
          let _ = write!(s, "{}", g.show_prod(prod));
        } else {
          let _ = write!(s, "; {}(✗)", g.show_prod(prod));
        }
      }
      let _ = writeln!(s);
    }
    let _ = writeln!(s);
  }
  s
}