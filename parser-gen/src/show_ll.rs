// show ll1 ps table
use crate::*;

pub fn show_prod_token<'a>(g: &'a Grammar) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    f.write_str("Productions:\n")?;
    for i in 0..g.prod.len() { writeln!(f, "  {}: {}", i, g.show_prod(i, None))?; }
    f.write_str("\nTokens:\n")?;
    for i in 0..g.token_num() { writeln!(f, "  {}: {}", i, g.show_token(i))?; }
    f.write_str("\n")
  })
}

pub fn table<'a>(ll: &'a LLCtx, g: &'a Grammar) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    write!(f, "{}", show_prod_token(g))?;
    for (idx, t) in ll.table.iter().enumerate() {
      writeln!(f, "{}:", g.show_token(idx))?;
      let mut show_set = |name: &str, set: &[u32]| {
        write!(f, "{}:", name)?;
        bitset::ibs(set).ones(|i| { let _ = write!(f, " {}", g.show_token(i)); });
        f.write_str("\n")?;
        Ok(())
      };
      show_set("first", &ll.first.get(idx))?;
      show_set("follow", &ll.follow.get(idx))?;
      // this is not necessary, but sorting it will provide better readability
      let mut t = t.iter().map(|(ch, prod)| (*ch, prod)).collect::<Vec<_>>();
      t.sort_unstable_by_key(|x| x.0);
      for (ch, prod) in t {
        write!(f, "  {} => ", g.show_token(ch as _))?;
        for (idx, &prod) in prod.iter().enumerate() {
          let prod = g.show_prod(prod as _, None);
          if idx == 0 { write!(f, "{}", prod)?; } else { write!(f, "; {}(✗)", prod)?; }
        }
        f.write_str("\n")?;
      }
      f.write_str("\n")?;
    }
    Ok(())
  })
}

pub fn conflict(table: &LLTable, g: &Grammar) -> Vec<String> {
  let mut ret = Vec::new();
  for entry in table {
    for (&predict, prod_ids) in entry {
      if prod_ids.len() > 1 {
        let first_prod = g.show_prod(prod_ids[0] as _, None);
        for &other in prod_ids.iter().skip(1) {
          ret.push(format!("conflict at prod \"{}\" and \"{}\", both's PS contains \"{}\"",
            first_prod, g.show_prod(other as _, None), g.show_token(predict as _)));
        }
      }
    }
  }
  ret
}