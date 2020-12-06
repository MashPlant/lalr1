use lalr1_core::*;
use common::{grammar::Grammar, *};
use std::fmt::{Display, Formatter, Write};
use crate::show_ll::show_prod_token;

pub fn table<'a>(orig_table: &'a Table, table: &'a Table, g: &'a Grammar) -> impl Display + 'a {
  assert_eq!(orig_table.len(), table.len());
  fn2display(move |f| {
    let _ = write!(f, "{}", show_prod_token(g));
    for (idx, (o, n)) in orig_table.iter().zip(table.iter()).enumerate() {
      let _ = writeln!(f, "State {}:", idx);
      for item in o.closure { // o and n have the same items
        let _ = writeln!(f, "  {}", g.show_prod(item.prod_id as usize, Some(item.dot)));
      }
      let _ = f.write_str("\n");
      // can't use o.iter().zip(n.iter()) here
      // because can't assume the 2 iterators go in the same order(though they actually do)
      for (ch, ao) in &o.act {
        let an = &n.act[ch];
        for o in ao {
          let keep = match an.iter().enumerate().find(|(_, n)| *n == o) {
            // selected => ✓, eliminated by prec and assoc => -, eliminated "forcefully" => ✗
            Some((0, _)) => "✓", None => "-", Some(_) => "✗"
          };
          let _ = writeln!(f, "  {} => {:?} ({})", g.show_token(*ch as usize), o, keep);
        }
      }
      let _ = f.write_str("\n");
    }
  })
}

pub fn conflict(g: &Grammar, c: &[Conflict]) -> Vec<String> {
  let mut ret = Vec::new();
  for c in c {
    let ch = g.show_token(c.ch as usize);
    match c.kind {
      ConflictKind::SR { s, r } =>
        ret.push(format!("shift-reduce conflict at state {} when faced with token \"{}\", it can either shift {}, or reduce {}(\"{}\")",
                         c.state, ch, s, r, g.show_prod(r as usize, None))),
      ConflictKind::RR { r1, r2 } =>
        ret.push(format!("reduce-reduce conflict at state {} when faced with token \"{}\", it can either reduce {}(\"{}\"), or reduce {}(\"{}\")",
                         c.state, ch, r1, g.show_prod(r1 as usize, None), r2, g.show_prod(r2 as usize, None))),
      ConflictKind::Many(ref acts) => {
        let mut msg = format!("Too many conflicts at state {} when faced with token \"{}\":\n", c.state, ch);
        for a in acts {
          match *a {
            Act::Shift(s) => { let _ = write!(msg, "  - shift {}\n", s); }
            Act::Reduce(r) => { let _ = write!(msg, "  - reduce {}('{}')\n", r, g.show_prod(r as usize, None)); }
            _ => unreachable!("there should be a bug in lr"),
          }
        }
        ret.push(msg);
      }
    }
  }
  ret
}

fn show_link(g: &Grammar, link: &HashMap<u32, u32>, idx: usize, f: &mut Formatter) {
  let mut link = link.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>();
  link.sort_unstable_by_key(|kv| kv.1);
  for (k, v) in link { let _ = writeln!(f, r#"{} -> {} [label="{}"];"#, idx, v, g.show_token(k as usize)); }
}

pub fn lr0_dot<'a>(g: &'a Grammar, lr0: &'a Lr0Fsm) -> impl Display + 'a {
  fn2display(move |f| {
    let _ = f.write_str("digraph {\n");
    for (idx, Lr0Node { closure, link }) in lr0.iter().enumerate() {
      show_link(g, link, idx, f);
      let _ = write!(f, "{}[shape=box, label=\"", idx);
      for (idx, lr0) in closure.iter().enumerate() {
        if idx != 0 { let _ = f.write_str(r#"\n"#); }
        let _ = write!(f, "{}", g.show_prod(lr0.prod_id as usize, Some(lr0.dot)));
      }
      let _ = f.write_str("\"]\n");
    }
    let _ = f.write_str("}");
  })
}

pub fn lr1_dot<'a>(g: &'a Grammar, lr1: &'a Lr1Fsm) -> impl Display + 'a {
  fn2display(move |f| {
    let _ = f.write_str("digraph {\n");
    for (idx, Lr1Node { closure, link }) in lr1.iter().enumerate() {
      show_link(g, link, idx, f);
      let _ = write!(f, "{}[shape=box, label=\"", idx);
      for (idx, Lr1Item { lr0, lookahead }) in closure.iter().enumerate() {
        if idx != 0 { let _ = f.write_str(r#"\n"#); }
        let _ = write!(f, "{},", g.show_prod(lr0.prod_id as usize, Some(lr0.dot)));
        let mut first = true;
        for i in 0..g.terms.len() {
          if lookahead.test(i) {
            let sep = if first { "" } else { "/" };
            first = false;
            let _ = write!(f, "{}{}", sep, g.show_token(i));
          }
        }
      }
      let _ = f.write_str("\"]\n");
    }
    let _ = f.write_str("}");
  })
}