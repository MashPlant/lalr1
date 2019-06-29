extern crate re2dfa;
extern crate lalr1_core;
extern crate grammar_config;

use re2dfa::dfa::Dfa;
use std::collections::HashMap;
use lalr1_core::ParseTable;
use grammar_config::Grammar;
use aho_corasick::AhoCorasick;
use std::fmt::Write;

pub struct RustCodegen {
  pub log_token: bool,
  pub log_reduce: bool,
}

pub fn min_u_of(x: u32) -> &'static str {
  match x {
    0..=255 => "u8",
    256..=65535 => "u16",
    _ => "u32",
  }
}

impl RustCodegen {
  fn gather_types<'a>(&self, g: &Grammar<'a>) -> (Vec<&'a str>, HashMap<&'a str, u32>) {
    let mut types = Vec::new();
    let mut types2id = HashMap::new();
    for &(_, ty) in &g.nt {
      types2id.entry(ty).or_insert_with(|| {
        let id = types.len() as u32;
        types.push(ty);
        id
      });
    }
    (types, types2id)
  }

  fn gen_common(&self, g: &Grammar, dfa: &Dfa, ec: &[u8; 128], types: &[&str]) -> String {
    let template = include_str!("template/common.rs.template");
    let pat = [
      "{include}",
      "{token_type}",
      "{stack_item}",
      "{dfa_size}",
      "{acc}",
      "{ec}",
      "{u_dfa_size}",
      "{ec_size}",
      "{dfa_edge}",
      "{parser_struct}",
    ];
    let rep = [
      // "{include}"
      g.raw.include.clone(),
      { // "{token_type}"
        let mut s = String::new();
        let _ = write!(s, "{} = {}, ", g.terms[0].0, g.nt.len());
        for &(t, _) in g.terms.iter().skip(1) {
          let _ = write!(s, "{}, ", t);
        }
        s
      },
      { // "{stack_item}"
        let mut s = "_Token(Token<'a>), ".to_owned();
        for (i, ty) in types.iter().enumerate() {
          let _ = write!(s, "_{}({}), ", i, ty);
        }
        s
      },
      // "{dfa_size}" ,
      dfa.nodes.len().to_string(),
      { // "{acc}"
        let mut s = String::new();
        for &(acc, _) in &dfa.nodes {
          match acc {
            Some(acc) => { let _ = write!(s, "{}, ", g.raw.lexical.get_index(acc as usize).unwrap().1); }
            None => s += "_Eof, ",
          }
        }
        s
      },
      { // "{ec}"
        let mut s = String::new();
        for ch in 0..128 {
          let _ = write!(s, "{}, ", ec[ch]);
        }
        s
      },
      // "{u_dfa_size}"
      min_u_of(dfa.nodes.len() as u32).to_owned(),
      // "{ec_size}"
      (*ec.iter().max().unwrap() + 1).to_string(),
      { // "{dfa_edge}"
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
      { // "{parser_struct}"
        let mut s = String::new();
        if g.raw.parser_def.is_none() {
          let _ = writeln!(s, "struct Parser {{");
          if let Some(ext) = &g.raw.parser_field {
            for field in ext {
              let _ = writeln!(s, "{},", field);
            }
          }
          let _ = writeln!(s, "}}");
        }
        s
      },
    ];
    AhoCorasick::new(&pat).replace_all(template, &rep)
  }
}

// I once tried to make the generated code perfectly indented by IndentPrinter, and I almost succeeded
// but such code is so unmaintainable, so I gave up, just use rustfmt or other tool to format the code...
impl RustCodegen {
  pub fn gen_lalr1(&self, g: &Grammar, table: &ParseTable, dfa: &Dfa, ec: &[u8; 128]) -> String {
    let (types, types2id) = self.gather_types(g);
    let common = self.gen_common(g, dfa, ec, &types);
    let template = include_str!("template/lalr1.rs.template");
    let pat = [
      "{u_lr_size}",
      "{parser_type}",
      "{res_type}",
      "{res_id}",
      "{u_lr_size}",
      "{u_prod_len}",
      "{prod_size}",
      "{prod}",
      "{token_size}",
      "{lr_size}",
      "{lr_edge}",
      "{parser_act}",
      "{log_token}",
    ];
    let parse_res = g.nt.last().unwrap().1;
    let res_id = types2id[parse_res];
    let rep = [
      // "{u_lr_size}"
      min_u_of(table.action.len() as u32).to_owned(),
      { // "{parser_type}"
        match &g.raw.parser_def {
          Some(def) => def.clone(),
          None => "Parser".to_owned(),
        }
      },
      // "{res_type}"
      parse_res.to_owned(),
      // "{res_id}"
      res_id.to_string(),
      // "{u_lr_size}"
      min_u_of(table.action.len() as u32).to_owned(),
      // "{u_prod_len}"
      min_u_of(g.prod_extra.iter().map(|&(_, (lhs, rhs), _)| g.prod[lhs as usize][rhs as usize].0.len()).max().unwrap() as u32).to_owned(),
      // "{prod_size}"
      g.prod_extra.len().to_string(),
      { // "{prod}"
        let mut s = String::new();
        for &(_, (lhs, rhs), _) in &g.prod_extra {
          let _ = write!(s, "({}, {}), ", lhs, g.prod[lhs as usize][rhs as usize].0.len());
        }
        s
      },
      // "{token_size}" ,
      (g.terms.len() + g.nt.len()).to_string(),
      // "{lr_size}"
      table.action.len().to_string(),
      { // "{lr_edge}"
        let mut s = String::new();
        for (_, edges) in &table.action {
          let _ = write!(s, "[");
          for i in 0..g.terms.len() + g.nt.len() {
            match edges.get(&(i as u32)) {
              Some(act) => { let _ = write!(s, "Act::{:?}, ", act[0]); }
              None => { let _ = write!(s, "Act::Err, "); }
            }
          }
          let _ = write!(s, "], ");
        }
        s
      },
      { // "{parser_act}"
        let mut s = String::new();
        for (i, &((act, args), (lhs, idx), _)) in g.prod_extra.iter().enumerate() {
          let _ = writeln!(s, "{} => {{", i);
          let rhs = &g.prod[lhs as usize][idx as usize];
          for (j, &x) in rhs.0.iter().enumerate().rev() {
            let name = match args {
              Some(args) => args[j].0.as_ref().map(|s| s.as_str()).unwrap_or("_"),
              None => "_",
            };
            if x < g.nt.len() as u32 {
              let id = types2id[g.nt[x as usize].1];
              let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => impossible!() }};", name, id);
            } else {
              let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => impossible!() }};", name);
            }
          }
          let _ = writeln!(s, "let _0 = {{ {} }};", act);
          let id = types2id[g.nt[lhs as usize].1];
          let _ = writeln!(s, "value_stk.push(StackItem::_{}(_0));", id);
          let _ = writeln!(s, "}}");
        }
        s
      },
      // "{log_token}"
      if self.log_token { r#"println("{:?}", token);"#.to_owned() } else { "".to_owned() },
    ];
    common + &AhoCorasick::new(&pat).replace_all(template, &rep)
  }
}