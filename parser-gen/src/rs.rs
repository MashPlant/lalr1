use re2dfa::dfa::Dfa;
use lalr1_core::{TableEntry, Table};
use common::{grammar::Grammar, HashMap};
use ll1_core::LLCtx;
use std::fmt::Write;
use crate::{Config, fmt};

impl<F> Config<'_, F> {
  // return None if this dfa is not suitable for a lexer
  // i.e., it doesn't accept anything, or it accept empty string
  // these 2 characteristics make lexer behaviour hard to define and make lex generator hard to write
  fn rs_common(&self, g: &Grammar, dfa_ec: &(Dfa, [u8; 256]), types: &[&str], stack_need_fail: bool) -> Option<String> {
    let (dfa, ec) = dfa_ec;
    if dfa.nodes.is_empty() || dfa.nodes[0].0.is_some() { return None; }
    Some(format!(
      include_str!("template/common.rs.template"),
      include = g.raw.include,
      macros = if self.use_unsafe {
        "macro_rules! index { ($arr: expr, $idx: expr) => { unsafe { *$arr.get_unchecked($idx as usize) } }; } macro_rules! impossible { () => { unsafe { std::hint::unreachable_unchecked() } }; }"
      } else {
        "macro_rules! index { ($arr: expr, $idx: expr) => { $arr[$idx as usize] }; } macro_rules! impossible { () => { unreachable!() }; }"
      },
      token_kind = fmt::token_kind(g),
      stack_item = {
        let mut s = String::new();
        if stack_need_fail { s += "_Fail, "; }
        for (i, ty) in types.iter().enumerate() { let _ = write!(s, "_{}({}), ", i, ty); }
        s
      },
      dfa_size = dfa.nodes.len(),
      acc = fmt::acc(g, dfa),
      ec = fmt::ec(ec),
      u_dfa_size = fmt::min_u(dfa.nodes.len()),
      ec_size = *ec.iter().max().unwrap() + 1,
      dfa_edge = fmt::dfa_edge(dfa, ec, ('[', ']')),
      show_token_prod = {
        if self.verbose.is_some() {
          format!("fn show_token(id: u32) -> &'static str {{ {:?}[id as usize] }} fn show_prod(id: u32) -> &'static str {{ {:?}[id as usize] }}",
                  (0..g.token_num() as u32).map(|i| g.show_token(i)).collect::<Vec<_>>(),
                  (0..g.prod.len() as u32).map(|i| g.show_prod(i, None)).collect::<Vec<_>>())
        } else { String::new() }
      },
      parser_struct = {
        let mut s = String::new();
        if g.raw.parser_def.is_none() {
          s += "pub struct Parser {\n";
          if let Some(ext) = &g.raw.parser_field {
            for field in ext { let _ = writeln!(s, "{},", field); }
          }
          s += "}\n";
        }
        s
      }
    ))
  }

  fn gen_act(&self, g: &Grammar, types2id: &HashMap<&str, u32>, handle_unexpect_stack: &str) -> String {
    let mut s = String::new();
    for (i, prod) in g.prod.iter().enumerate() {
      let _ = writeln!(s, "{} => {{", i);
      if self.log_reduce {
        let _ = writeln!(s, r#"println!("{}");"#, g.show_prod(i as u32, None));
      }
      for (j, &x) in prod.rhs.iter().enumerate().rev() {
        let name = match prod.args { Some(args) => args[j].0.to_owned(), None => format!("_{}", j + 1) };
        if let Some(x) = g.as_nt(x) {
          let id = types2id[g.nt[x].ty];
          let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => {} }};", name, id, handle_unexpect_stack);
        } else {
          let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => {} }};", name, handle_unexpect_stack);
        }
      }
      let id = types2id[g.nt[prod.lhs as usize].ty];
      let _ = writeln!(s, "StackItem::_{}({{ {} }})\n}}", id, prod.act);
    }
    s
  }
}

impl<F> Config<'_, F> {
  // return None if `rs_common` returns None, you can check the doc of `rs_common`
  pub fn rs_lalr1(&self, g: &Grammar, table: &Table, dfa_ec: &(Dfa, [u8; 256])) -> Option<String> {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let common = self.rs_common(g, dfa_ec, &types, false)?;
    let lalr1 = format!(
      include_str!("template/lalr1.rs.template"),
      u_lr_fsm_size = fmt::min_u(table.len()),
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      res_type = parse_res,
      res_id = types2id[parse_res],
      u_prod_len = fmt::min_u(g.prod.iter().map(|x| x.rhs.len()).max().unwrap()),
      prod_size = g.prod.len(),
      prod = {
        let mut s = String::new();
        for p in &g.prod { let _ = write!(s, "({}, {}), ", p.lhs, p.rhs.len()); }
        s
      },
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      lr_fsm_size = table.len(),
      action = {
        let mut s = String::new();
        for TableEntry { act, .. } in table {
          s.push('[');
          for i in 0..g.terms.len() as u32 {
            match act.get(&i) {
              Some(act) if !act.is_empty() => { let _ = write!(s, "Act::{:?}, ", act[0]); }
              _ => { s += "Act::Err, "; }
            }
          }
          s += "], ";
        }
        s
      },
      goto = fmt::goto(g, &table, ('[', ']')),
      parser_act = self.gen_act(g, &types2id, "impossible!()"),
      log_token = if self.log_token { r#"println!("{:?}", token);"# } else { "" },
    );
    Some(common + &lalr1)
  }

  pub fn rs_ll1(&self, g: &Grammar, ll: &LLCtx, dfa_ec: &(Dfa, [u8; 256])) -> Option<String> {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let common = self.rs_common(g, dfa_ec, &types, true)?;
    let ll1 = format!(
      include_str!("template/ll1.rs.template"),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      follow = {
        let mut s = String::new();
        for follow in &ll.follow.0 {
          s += "set!(";
          for i in 0..g.token_num() {
            if follow.test(i as usize) { let _ = write!(s, "{}, ", i); }
          }
          s += "),\n";
        }
        s
      },
      table = {
        let mut s = String::new();
        for table in &ll.table {
          s += "map!(";
          for (&predict, prod_ids) in table {
            let prod_id = prod_ids[0] as usize;
            let _ = write!(s, "{} => ({}, vec!{:?}), ", predict, prod_id, g.prod[prod_ids[0] as usize].rhs);
          }
          s += "),\n";
        }
        s
      },
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      parser_act = self.gen_act(g, &types2id, "return StackItem::_Fail"),
      res_type = parse_res,
      res_nt_id = g.token_num() - 1,
      res_id = types2id[parse_res]
    );
    Some(common + &ll1)
  }
}