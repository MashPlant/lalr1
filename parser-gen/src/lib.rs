extern crate re2dfa;
extern crate lalr1_core;
extern crate grammar_config;

use re2dfa::dfa::Dfa;
use std::collections::HashMap;
use grammar_config::{RawFieldExt, AbstractGrammar};
use lalr1_core::{ParseTable, Grammar};
use aho_corasick::AhoCorasick;
use std::fmt::Write;

pub trait Codegen {
  fn gen(&self, g: &Grammar, table: &ParseTable, dfa: &Dfa, ec: &[u8; 128]) -> String;
}

pub struct RustCodegen {
  pub log_token: bool,
  pub log_reduce: bool,
}

pub fn min_u_of(x: u32) -> String {
  match x {
    0..=255 => "u8".into(),
    256..=65535 => "u16".into(),
    _ => "u32".into(),
  }
}

// I once tried to make the generated code perfectly indented by IndentPrinter, and I almost succeeded
// but such code is so unmaintainable, so I gave up, just use rustfmt or other tool to format the code...
impl Codegen for RustCodegen {
  fn gen(&self, g: &Grammar, table: &ParseTable, dfa: &Dfa, ec: &[u8; 128]) -> String {
    let template = include_str!("template/template.rs");
    let pat = [
      "{{INCLUDE}}",
      "{{TOKEN_TYPE}}",
      "{{U_LR_SIZE}}",
      "{{STACK_ITEM}}",
      "{{DFA_SIZE}}",
      "{{ACC}}",
      "{{EC}}",
      "{{U_DFA_SIZE}}",
      "{{EC_SIZE}}",
      "{{DFA_EDGE}}",
      "{{PARSER_FIELD}}",
      "{{PARSER_INIT}}",
      "{{RESULT_TYPE}}",
      "{{RESULT_ID}}",
      "{{U_LR_SIZE}}",
      "{{U_PROD_LEN}}",
      "{{PROD_SIZE}}",
      "{{PROD}}",
      "{{TOKEN_SIZE}}",
      "{{LR_SIZE}}",
      "{{LR_EDGE}}",
      "{{PARSER_ACT}}",
      "{{LOG_TOKEN}}",
    ];
    let mut types = Vec::new();
    let mut types2id = HashMap::new();
    for &(_, ty) in &g.nt {
      types2id.entry(ty).or_insert_with(|| {
        let id = types.len() as u32;
        types.push(ty);
        id
      });
    }
    let parse_res = g.nt[(g.prod_extra.last().unwrap().1).0 as usize].1;
    let res_id = types2id[parse_res];
    let rep = [
      // "{{INCLUDE}}"
      g.raw.include.clone(),
      { // "{{TOKEN_TYPE}}"
        let mut s = String::new();
        for &(nt, _) in &g.nt {
          let _ = write!(s, "{}, ", nt);
        }
        for &(t, _) in &g.terminal {
          let _ = write!(s, "{}, ", t);
        }
        s
      },
      // {{U_LR_SIZE}}
      min_u_of(table.action.len() as u32),
      { // "{{STACK_ITEM}}"
        let mut s = "_Token(Token<'a>), ".to_owned();
        for (i, ty) in types.iter().enumerate() {
          let _ = write!(s, "_{}({}), ", i, ty);
        }
        s
      },
      // "{{DFA_SIZE}}" ,
      dfa.nodes.len().to_string(),
      { // "{{ACC}}"
        let mut s = String::new();
        for &(acc, _) in &dfa.nodes {
          match acc {
            Some(acc) => { let _ = write!(s, "{}, ", g.raw.lexical[acc as usize].1); }
            None => s += "_Eof, ",
          }
        }
        s
      },
      { // "{{EC}}"
        let mut s = String::new();
        for ch in 0..128 {
          let _ = write!(s, "{}, ", ec[ch]);
        }
        s
      },
      // "{{U_DFA_SIZE}}"
      min_u_of(dfa.nodes.len() as u32),
      // "{{EC_SIZE}}"
      (*ec.iter().max().unwrap() + 1).to_string(),
      { // "{{DFA_EDGE}}"
        let mut s = String::new();
        let mut outs = vec![0; (*ec.iter().max().unwrap() + 1) as usize];
        for (_, edges) in dfa.nodes.iter() {
          for x in &mut outs { *x = 0; }
          for (&k, &out) in edges {
            outs[ec[k as usize] as usize] = out;
          }
          let _ = write!(s, "{:?}, ", outs);
        }
        s
      },
      { // "{{PARSER_FIELD}}"
        let mut s = String::new();
        if let Some(ext) = &g.raw.parser_field_ext {
          for RawFieldExt { field, type_, init: _ } in ext {
            let _ = writeln!(s, "pub {}: {},", field, type_);
          }
        }
        s
      },
      { // "{{PARSER_INIT}}"
        let mut s = String::new();
        if let Some(ext) = &g.raw.parser_field_ext {
          for RawFieldExt { field, type_: _, init } in ext {
            let _ = writeln!(s, "{}: {},", field, init);
          }
        }
        s
      },
      // "{{RESULT_TYPE}}"
      parse_res.to_owned(),
      // "{{RESULT_ID}}"
      res_id.to_string(),
      // "{{U_LR_SIZE}}"
      min_u_of(table.action.len() as u32),
      // "{{U_PROD_LEN}}"
      min_u_of(g.prod_extra.iter().map(|&(_, (lhs, rhs), _)| g.prod[lhs as usize][rhs as usize].0.len()).max().unwrap() as u32),
      // "{{PROD_SIZE}}"
      g.prod_extra.len().to_string(),
      { // "{{PROD}}"
        let mut s = String::new();
        for &(_, (lhs, rhs), _) in &g.prod_extra {
          let _ = write!(s, "({}, {}), ", lhs, g.prod[lhs as usize][rhs as usize].0.len());
        }
        s
      },
      // "{{TOKEN_SIZE}}" ,
      (g.terminal.len() + g.nt.len()).to_string(),
      // "{{LR_SIZE}}"
      table.action.len().to_string(),
      { // "{{LR_EDGE}}"
        let mut s = String::new();
        for (_, edges) in &table.action {
          let _ = write!(s, "[");
          for i in 0..g.terminal.len() + g.nt.len() {
            match edges.get(&(i as u32)) {
              Some(act) => { let _ = write!(s, "Act::{:?}, ", act[0]); }
              None => { let _ = write!(s, "Act::Err, "); }
            }
          }
          let _ = write!(s, "], ");
        }
        s
      },
      { // "{{PARSER_ACT}}"
        let mut s = String::new();
        for (i, &(act, (lhs, rhs), _)) in g.prod_extra.iter().enumerate() {
          let _ = writeln!(s, "{} => {{", i);
          let rhs = &g.prod[lhs as usize][rhs as usize];
          for (j, &x) in rhs.0.iter().enumerate().rev() {
            let j = j + 1;
            if x < AbstractGrammar::nt_num(g) {
              let id = types2id[g.nt[x as usize].1];
              let _ = writeln!(s, "let mut _{} = match self.value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => impossible!() }};", j, id);
            } else {
              let _ = writeln!(s, "let mut _{} = match self.value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => impossible!() }};", j);
            }
          }
          if !act.is_empty() {
            let _ = writeln!(s, "{}", act);
          }
          if self.log_reduce {
            let _ = writeln!(s, r#"println!("{{:?}}", _0);"#);
          }
          let id = types2id[g.nt[lhs as usize].1];
          let _ = writeln!(s, "self.value_stk.push(StackItem::_{}(_0));", id);
          let _ = writeln!(s, "}}");
        }
        s
      },
      // "{{LOG_TOKEN}}"
      if self.log_token { r#"println("{:?}", token);"#.to_owned() } else { "".to_owned() },
    ];
    let ac = AhoCorasick::new(&pat);
    ac.replace_all(template, &rep)
  }
}