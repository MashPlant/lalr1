use re2dfa::dfa::Dfa;
use std::collections::HashMap;
use lr::{Grammar, RawFieldExt, ParseTable, AbstractGrammar};
use aho_corasick::AhoCorasick;
use std::fmt::Write;

pub trait Codegen {
  fn gen(&self, g: &Grammar, table: &ParseTable, dfa: &Dfa, ec: &[u8; 128]) -> String;
}

pub struct RustCodegen {
  pub log_token: bool,
  pub log_reduce: bool,
}

fn min_u_of(x: u32) -> String {
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
      "{{U_LR_SIZE}}",
      "{{U_PROD_LEN}}",
      "{{U_PROD_SIZE}}",
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
    let rep = [
      // "{{INCLUDE}}"
      g.raw.include.clone(),
      { // "{{TOKEN_TYPE}}"
        let mut s = String::new();
        for &(nt, _) in &g.nt {
          write!(s, "{}, ", nt).unwrap();
        }
        for &(t, _) in &g.terminal {
          write!(s, "{}, ", t).unwrap();
        }
        s
      },
      // {{U_LR_SIZE}}
      min_u_of(table.action.len() as u32),
      { // "{{STACK_ITEM}}"
        let mut s = "_token(token<'a>), ".to_owned();
        for (i, ty) in types.iter().enumerate() {
          write!(s, "_{}({}), ", i, ty).unwrap();
        }
        s
      },
      // "{{DFA_SIZE}}" ,
      dfa.nodes.len().to_string(),
      { // "{{ACC}}"
        let mut s = String::new();
        for &(acc, _) in &dfa.nodes {
          match acc {
            Some(acc) => write!(s, "{}, ", g.terminal[acc as usize].0).unwrap(),
            None => s += "_Eps, ",
          }
        }
        s
      },
      { // "{{EC}}"
        let mut s = String::new();
        for ch in 0..128 {
          write!(s, "{}, ", ec[ch]).unwrap();
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
          write!(s, "{:?}, ", outs).unwrap();
        }
        s
      },
      { // "{{PARSER_FIELD}}"
        let mut s = String::new();
        if let Some(ext) = &g.raw.parser_field_ext {
          for RawFieldExt { field, type_, init: _ } in ext {
            writeln!(s, "pub {}: {},", field, type_).unwrap();
          }
        }
        s
      },
      { // "{{PARSER_INIT}}"
        let mut s = String::new();
        if let Some(ext) = &g.raw.parser_field_ext {
          for RawFieldExt { field, type_: _, init } in ext {
            writeln!(s, "{}: {},", field, init).unwrap();
          }
        }
        s
      },
      //  "{{U_LR_SIZE}}"
      min_u_of(table.action.len() as u32),
      // "{{U_PROD_LEN}}"
      g.prod_extra.iter().map(|&(_, (lhs, rhs), _)| g.prod[lhs as usize][rhs as usize].0.len()).max().unwrap().to_string(),
      // "{{U_PROD_SIZE}}"
      g.prod_extra.len().to_string(),
      { // "{{PROD}}"
        let mut s = String::new();
        for &(_, (lhs, rhs), _) in &g.prod_extra {
          write!(s, "({}, {}), ", lhs, g.prod[lhs as usize][rhs as usize].0.len()).unwrap();
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
          write!(s, "[").unwrap();
          for i in 0..g.terminal.len() + g.nt.len() {
            match edges.get(&(i as u32)) {
              Some(act) => write!(s, "Act::{:?}, ", act[0]).unwrap(),
              None => write!(s, "Act::Err, ").unwrap(),
            }
          }
          write!(s, "], ").unwrap();
        }
        s
      },
      { // "{{PARSER_ACT}}"
        let mut s = String::new();
        for (i, &(act, (lhs, rhs), _)) in g.prod_extra.iter().enumerate() {
          writeln!(s, "{} => {{", i).unwrap();
          let rhs = &g.prod[lhs as usize][rhs as usize];
          for (j, &x) in rhs.0.iter().enumerate().rev() {
            let j = j + 1;
            if x < AbstractGrammar::nt_num(g) {
              let id = types2id[g.nt[x as usize].1];
              writeln!(s, "let mut _{} = match self.value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => impossible!() }};", j, id).unwrap();
            } else {
              writeln!(s, "let mut _{} = match self.value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => impossible!() }};", j).unwrap();
            }
          }
          if !act.is_empty() {
            writeln!(s, "{}", act).unwrap();
          }
          if self.log_reduce {
            writeln!(s, r#"println!("{{:?}}", _0);"#).unwrap();
          }
          let id = types2id[g.nt[lhs as usize].1];
          writeln!(s, "self.value_stk.push(StackItem::_{}(_0));", id).unwrap();
          writeln!(s, "}}").unwrap();
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