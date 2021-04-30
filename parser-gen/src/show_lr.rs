use crate::*;

pub fn table<'a>(orig_table: &'a Table, table: &'a Table, g: &'a Grammar) -> impl Display + 'a {
  assert_eq!(orig_table.len(), table.len());
  fmt_::fn2display(move |f| {
    write!(f, "{}", show_ll::show_prod_token(g))?;
    for (idx, (o, n)) in orig_table.iter().zip(table.iter()).enumerate() {
      writeln!(f, "State {}:", idx)?;
      for item in o.closure { // o and n have the same items
        writeln!(f, "  {}", g.show_prod(item.prod_id as _, Some(item.dot)))?;
      }
      f.write_str("\n")?;
      for ((ch, ao), (ch1, an)) in o.act.iter().zip(n.act.iter()) {
        debug_assert_eq!(ch, ch1);
        for o in ao {
          let keep = match an.iter().enumerate().find(|(_, n)| *n == o) {
            // selected => ✓, eliminated by prec and assoc => -, eliminated "forcefully" => ✗
            Some((0, _)) => "✓", None => "-", Some(_) => "✗"
          };
          writeln!(f, "  {} => {:?} ({})", g.show_token(*ch as _), o, keep)?;
        }
      }
      f.write_str("\n")?;
    }
    Ok(())
  })
}

pub fn conflict(g: &Grammar, c: &[Conflict]) -> Vec<String> {
  let mut ret = Vec::new();
  for c in c {
    let ch = g.show_token(c.ch as _);
    match c.kind {
      ConflictKind::SR { s, r } =>
        ret.push(format!("shift-reduce conflict at state {} when faced with token \"{}\", it can either shift {}, or reduce {}(\"{}\")",
          c.state, ch, s, r, g.show_prod(r as _, None))),
      ConflictKind::RR { r1, r2 } =>
        ret.push(format!("reduce-reduce conflict at state {} when faced with token \"{}\", it can either reduce {}(\"{}\"), or reduce {}(\"{}\")",
          c.state, ch, r1, g.show_prod(r1 as _, None), r2, g.show_prod(r2 as _, None))),
      ConflictKind::Many(ref acts) => {
        let mut msg = format!("Too many conflicts at state {} when faced with token \"{}\":\n", c.state, ch);
        for a in acts {
          match *a {
            Act::Shift(s) => { let _ = write!(msg, "  - shift {}\n", s); }
            Act::Reduce(r) => { let _ = write!(msg, "  - reduce {}('{}')\n", r, g.show_prod(r as _, None)); }
            _ => unreachable!("there should be a bug in lr"),
          }
        }
        ret.push(msg);
      }
    }
  }
  ret
}

fn show_link(g: &Grammar, link: &HashMap<u32, u32>, idx: usize, f: &mut Formatter) -> FmtResult {
  let mut link = link.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>();
  link.sort_unstable_by_key(|kv| kv.1);
  for (k, v) in link { writeln!(f, r#"{} -> {} [label="{}"];"#, idx, v, g.show_token(k as _))?; }
  Ok(())
}

pub fn lr0_dot<'a>(g: &'a Grammar, lr0: &'a Lr0Fsm) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    f.write_str("digraph {\n")?;
    for (idx, Lr0Node { closure, link }) in lr0.iter().enumerate() {
      show_link(g, link, idx, f)?;
      write!(f, "{}[shape=box, label=\"", idx)?;
      for (idx, lr0) in closure.iter().enumerate() {
        if idx != 0 { f.write_str(r#"\n"#)?; }
        write!(f, "{}", g.show_prod(lr0.prod_id as _, Some(lr0.dot)))?;
      }
      f.write_str("\"]\n")?;
    }
    f.write_str("}")
  })
}

pub fn lr1_dot<'a>(g: &'a Grammar, lr1: &'a Lr1Fsm) -> impl Display + 'a {
  fmt_::fn2display(move |f| {
    f.write_str("digraph {\n")?;
    for (idx, Lr1Node { closure, link }) in lr1.iter().enumerate() {
      show_link(g, link, idx, f)?;
      write!(f, "{}[shape=box, label=\"", idx)?;
      for (idx, Lr1Item { lr0, lookahead }) in closure.iter().enumerate() {
        if idx != 0 { f.write_str(r#"\n"#)?; }
        write!(f, "{},", g.show_prod(lr0.prod_id as _, Some(lr0.dot)))?;
        let mut first = true;
        bitset::ibs(lookahead).ones(|i| {
          let sep = if first { "" } else { "/" };
          first = false;
          let _ = write!(f, "{}{}", sep, g.show_token(i));
        });
      }
      f.write_str("\"]\n")?;
    }
    f.write_str("}")
  })
}