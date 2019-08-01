use lalr1_core::RawTable;
use grammar_config::Grammar;
use std::fmt::Write;

pub fn text(original_table: &RawTable, table: &RawTable, g: &Grammar) -> String {
  assert_eq!(original_table.len(), table.len());
  let mut text = String::new();
  for i in 0..g.prod_extra.len() as u32 {
    let _ = writeln!(text, "Production {}: {}", i, g.show_prod(i));
  }
  text.push_str("\n\n");

  for (idx, (o, n)) in original_table.iter().zip(table.iter()).enumerate() {
    let _ = writeln!(text, "State {}:", idx);
    for item in o.items { // o and n have the same items
      let _ = writeln!(text, "  {}", g.show_prod_dotted(item.prod_id, item.dot));
    }
    text.push('\n');
    // can't use o.iter().zip(n.iter()) here
    // because can't assume the 2 iterators go in the same order(though they actually do)
    for (ch, ao) in &o.act {
      let an = &n.act[ch];
      for o in ao {
        let keep = match an.iter().enumerate().find(|(_, n)| *n == o) {
          // selected => ✓, eliminated by prec and assoc => -, eliminated "forcefully" => ✗
          Some((0, _)) => "✓", None => "-", Some(_) => "✗"
        };
        let _ = writeln!(text, "  {} => {:?} ({})", g.show_token(*ch), o, keep);
      }
    }
    text.push_str("\n\n");
  }
  text
}