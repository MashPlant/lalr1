use lalr1_core::*;
use common::grammar::Grammar;
use std::{fmt::Write, borrow::Borrow};
use crate::show_ll::show_prod_token;

pub fn table(orig_table: &Table, table: &Table, g: &Grammar) -> String {
  assert_eq!(orig_table.len(), table.len());
  let mut text = show_prod_token(g);
  for (idx, (o, n)) in orig_table.iter().zip(table.iter()).enumerate() {
    let _ = writeln!(text, "State {}:", idx);
    for item in o.closure { // o and n have the same items
      let _ = writeln!(text, "  {}", g.show_prod(item.prod_id, Some(item.dot)));
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
    text.push('\n');
  }
  text
}

pub fn conflict(g: &Grammar, c: &[Conflict]) -> Vec<String> {
  let mut ret = Vec::new();
  for c in c {
    let ch = g.show_token(c.ch);
    match c.kind {
      ConflictKind::SR { s, r } =>
        ret.push(format!("shift-reduce conflict at state {} when faced with token \"{}\", it can either shift {}, or reduce {}(\"{}\")",
                         c.state, ch, s, r, g.show_prod(r, None))),
      ConflictKind::RR { r1, r2 } =>
        ret.push(format!("reduce-reduce conflict at state {} when faced with token \"{}\", it can either reduce {}(\"{}\"), or reduce {}(\"{}\")",
                         c.state, ch, r1, g.show_prod(r1, None), r2, g.show_prod(r2, None))),
      ConflictKind::Many(ref acts) => {
        let mut msg = format!("Too many conflicts at state {} when faced with token \"{}\":\n", c.state, ch);
        for a in acts {
          match a {
            Act::Shift(s) => { msg.push_str(&format!("  - shift {}\n", s)); }
            Act::Reduce(r) => { msg.push_str(&format!("  - reduce {}('{}')\n", r, g.show_prod(*r, None))); }
            _ => unreachable!("there should be a bug in lr"),
          }
        }
        ret.push(msg);
      }
    }
  }
  ret
}

fn show_link(g: &Grammar, link: &Link, idx: usize, s: &mut String) {
  let mut link = link.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>();
  link.sort_unstable_by_key(|kv| kv.1);
  for (k, v) in link {
    let _ = writeln!(s, r#"{} -> {} [label="{}"];"#, idx, v, g.show_token(k));
  }
}

pub fn lr0_dot(g: &Grammar, lr0: &Lr0Fsm) -> String {
  let mut s = "digraph {\n".to_owned();
  for (idx, Lr0Node { closure, link }) in lr0.iter().enumerate() {
    show_link(g, link, idx, &mut s);
    let mut text = String::new();
    for lr0 in closure {
      text += &g.show_prod(lr0.prod_id, Some(lr0.dot));
      text.push(',');
      text.pop();
      text += r#"\n"#;
    }
    text.pop();
    text.pop();
    let _ = writeln!(s, r#"{}[shape=box, label="{}"]"#, idx, text);
  }
  s.push('}');
  s
}

pub fn lr1_dot(g: &Grammar, lr1: &Lr1Fsm) -> String {
  let mut s = "digraph {\n".to_owned();
  for (idx, Lr1Node { closure, link }) in lr1.iter().enumerate() {
    show_link(g, link.borrow(), idx, &mut s);
    let mut text = String::new();
    for Lr1Item { lr0, lookahead } in closure {
      text += &g.show_prod(lr0.prod_id, Some(lr0.dot));
      text.push(',');
      for i in g.nt_num()..g.token_num() {
        if lookahead.test(i as usize) {
          text += g.show_token(i);
          text.push('/');
        }
      }
      text.pop();
      text += r#"\n"#;
    }
    text.pop();
    text.pop();
    let _ = writeln!(s, r#"{}[shape=box, label="{}"]"#, idx, text);
  }
  s.push('}');
  s
}