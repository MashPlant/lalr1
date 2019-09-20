pub mod show_lr;
pub mod show_ll;

use re2dfa::dfa::Dfa;
use hashbrown::HashMap;
use lalr1_core::{TableEntry, Table};
use grammar_config::{Grammar, AbstractGrammar, AbstractGrammarExt};
use aho_corasick::AhoCorasick;
use ll1_core::LLCtx;
use std::fmt::Write;

pub struct RustCodegen {
  pub log_token: bool,
  pub log_reduce: bool,
  pub use_unsafe: bool,
}

pub fn min_u(x: u32) -> &'static str {
  match x { 0..=255 => "u8", 256..=65535 => "u16", _ => "u32", }
}

pub const INVALID_DFA: &str = "The merged dfa is not suitable for a lexer, i.e., it doesn't accept anything, or it accept empty string.";

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

  // return None if this dfa is not suitable for a lexer
  // i.e., it doesn't accept anything, or it accept empty string
  // these 2 characteristics make lexer behaviour hard to define and make lex generator hard to write
  fn gen_common(&self, g: &Grammar, dfa: &Dfa, ec: &[u8; 256], types: &[&str], stack_need_fail: bool) -> Option<String> {
    if dfa.nodes.is_empty() || dfa.nodes[0].0.is_some() { return None; }
    let template = include_str!("template/common.rs.template");
    let pat = [
      "{include}",
      "{macros}",
      "{token_kind}",
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
      { // "{macros}"
        if self.use_unsafe {
          r#"macro_rules! index { ($arr: expr, $idx: expr) => { unsafe { *$arr.get_unchecked($idx as usize) } }; }
macro_rules! impossible { () => { unsafe { std::hint::unreachable_unchecked() } }; }"#.to_owned()
        } else {
          r#"macro_rules! index { ($arr: expr, $idx: expr) => { $arr[$idx as usize] }; }
macro_rules! impossible { () => { unreachable!() }; }"#.to_owned()
        }
      },
      { // "{token_kind}"
        let mut s = String::new();
        let _ = write!(s, "{} = {}, ", g.terms[0].0, g.nt.len());
        for &(t, _) in g.terms.iter().skip(1) {
          let _ = write!(s, "{}, ", t);
        }
        s
      },
      { // "{stack_item}"
        let mut s = "_Token(Token<'p>), ".to_owned();
        if stack_need_fail {
          let _ = write!(s, "_Fail, ");
        }
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
            Some(acc) => { let _ = write!(s, "TokenKind::{}, ", g.raw.lexical.get_index(acc as usize).unwrap().1); }
            None => { let _ = write!(s, "TokenKind::{}, ", grammar_config::ERR); }
          }
        }
        s
      },
      { // "{ec}"
        let mut s = String::new();
        for ch in 0..256 {
          let _ = write!(s, "{}, ", ec[ch]);
        }
        s
      },
      // "{u_dfa_size}"
      min_u(dfa.nodes.len() as u32).to_owned(),
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
    Some(AhoCorasick::new(&pat).replace_all(template, &rep))
  }

  fn gen_act(&self, g: &Grammar, types2id: &HashMap<&str, u32>, handle_unexpect_stack: &str) -> String {
    let mut s = String::new();
    for (i, &((act, args), (lhs, idx), _)) in g.prod_extra.iter().enumerate() {
      let _ = writeln!(s, "{} => {{", i);
      if self.log_reduce {
        let _ = writeln!(s, r#"println!("{}");"#, g.show_prod(i as u32, None));
      }
      let rhs = &g.prod[lhs as usize][idx as usize];
      for (j, &x) in rhs.0.iter().enumerate().rev() {
        let name = match args {
          Some(args) => args[j].0.as_ref().map(|s| s.as_str()).unwrap_or("_").to_owned(),
          None => format!("_{}", j + 1),
        };
        if x < g.nt.len() as u32 {
          let id = types2id[g.nt[x as usize].1];
          let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => {} }};", name, id, handle_unexpect_stack);
        } else {
          let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => {} }};", name, handle_unexpect_stack);
        }
      }
      let id = types2id[g.nt[lhs as usize].1];
      let _ = writeln!(s, "StackItem::_{}({{ {} }})", id, act);
      let _ = writeln!(s, "}}");
    }
    s
  }
}

impl RustCodegen {
  // return None if `gen_common` returns None, you can check the doc of `gen_common`
  pub fn gen_lalr1(&self, g: &Grammar, table: &Table, dfa: &Dfa, ec: &[u8; 256]) -> Option<String> {
    let (types, types2id) = self.gather_types(g);
    let common = self.gen_common(g, dfa, ec, &types, false)?;
    let template = include_str!("template/lalr1.rs.template");
    let pat = [
      "{u_lr_fsm_size}",
      "{parser_type}",
      "{res_type}",
      "{res_id}",
      "{u_lr_fsm_size}",
      "{u_prod_len}",
      "{prod_size}",
      "{prod}",
      "{term_num}",
      "{nt_num}",
      "{lr_fsm_size}",
      "{action}",
      "{goto}",
      "{parser_act}",
      "{log_token}",
    ];
    let parse_res = g.nt.last().unwrap().1;
    let res_id = types2id[parse_res];
    let rep = [
      // "{u_lr_fsm_size}"
      min_u(table.len() as u32).to_owned(),
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
      // "{u_lr_fsm_size}"
      min_u(table.len() as u32).to_owned(),
      // "{u_prod_len}"
      min_u(g.prod_extra.iter().map(|&(_, (lhs, rhs), _)| g.prod[lhs as usize][rhs as usize].0.len()).max().unwrap() as u32).to_owned(),
      // "{prod_size}"
      g.prod_extra.len().to_string(),
      { // "{prod}"
        let mut s = String::new();
        for &(_, (lhs, rhs), _) in &g.prod_extra {
          let _ = write!(s, "({}, {}), ", lhs, g.prod[lhs as usize][rhs as usize].0.len());
        }
        s
      },
      // "{term_num}"
      g.terms.len().to_string(),
      // "{nt_num}"
      g.nt.len().to_string(),
      // "{lr_fsm_size}"
      table.len().to_string(),
      { // "{action}"
        let mut s = String::new();
        for TableEntry { act, .. } in table {
          let _ = write!(s, "[");
          for i in g.nt.len() as u32..(g.terms.len() + g.nt.len()) as u32 {
            match act.get(&i) {
              Some(act) if !act.is_empty() => { let _ = write!(s, "Act::{:?}, ", act[0]); }
              _ => { let _ = write!(s, "Act::Err, "); }
            }
          }
          let _ = write!(s, "], ");
        }
        s
      },
      { // "{goto}"
        let mut s = String::new();
        for TableEntry { goto, .. } in table {
          let _ = write!(s, "[");
          for i in 0..g.nt.len() as u32 {
            let _ = write!(s, "{:?}, ", goto.get(&i));
          }
          let _ = write!(s, "], ");
        }
        s
      },
      // "{parser_act}"
      self.gen_act(g, &types2id, "impossible!()"),
      // "{log_token}"
      if self.log_token { r#"println!("{:?}", token);"#.to_owned() } else { "".to_owned() },
    ];
    Some(common + &AhoCorasick::new(&pat).replace_all(template, &rep))
  }

  pub fn gen_ll1(&self, g: &Grammar, ll: &LLCtx, dfa: &Dfa, ec: &[u8; 256]) -> Option<String> {
    let (types, types2id) = self.gather_types(g);
    let common = self.gen_common(g, dfa, ec, &types, true)?;
    let template = include_str!("template/ll1.rs.template");
    let pat = [
      "{nt_num}",
      "{follow}",
      "{table}",
      "{parser_type}",
      "{parser_act}",
      "{res_type}",
      "{res_nt_id}",
      "{res_id}",
    ];
    let parse_res = g.nt.last().unwrap().1;
    let res_id = types2id[parse_res];
    let rep = [
      // "{nt_num}",
      g.nt_num().to_string(),
      { // "{follow}",
        let mut s = String::new();
        for follow in &ll.follow.nt_follow {
          let _ = write!(s, "set!(");
          for i in 0..g.token_num() {
            if follow.test(i as usize) {
              let _ = write!(s, "{}, ", i);
            }
          }
          let _ = writeln!(s, "),");
        }
        s
      },
      { // "{table}",
        let mut s = String::new();
        for table in &ll.table {
          let _ = write!(s, "map!(");
          for (&predict, prod_ids) in table {
            let prod_id = prod_ids[0] as usize;
            let (_, (lhs, idx), _) = g.prod_extra[prod_id];
            let (prod, _) = &g.prod[lhs as usize][idx as usize];
            let _ = write!(s, "{} => ({}, vec!{:?}), ", predict, prod_id, prod);
          }
          let _ = writeln!(s, "),");
        }
        s
      },
      { // "{parser_type}"
        match &g.raw.parser_def {
          Some(def) => def.clone(),
          None => "Parser".to_owned(),
        }
      },
      // "{parser_act}",
      self.gen_act(g, &types2id, "return StackItem::_Fail"),
      // "{res_type}"
      parse_res.to_owned(),
      // "{res_nt_id}"
      (g.nt.len() - 1).to_string(),
      // "{res_id}"
      res_id.to_string(),
    ];
    Some(common + &AhoCorasick::new(&pat).replace_all(template, &rep))
  }
}