use lalr1_core::{TableEntry, Table};
use common::{grammar::Grammar, re2dfa::Dfa, *};
use ll1_core::LLCtx;
use crate::{Config, fmt};

impl<W: std::io::Write> Config<'_, W> {
  fn rs_common(&mut self, g: &Grammar, dfa: &Dfa, types: &[&str], stack_need_fail: bool) {
    let verbose = self.verbose.is_some();
    let _ = write!(
      self.code_output, include_str!("template/common.rs.template"),
      include = g.raw.include,
      macros = if self.use_unsafe {
        "macro_rules! index { ($arr: expr, $idx: expr) => { unsafe { $arr.get_unchecked($idx) } }; } macro_rules! impossible { () => { unsafe { std::hint::unreachable_unchecked() } }; }"
      } else {
        "macro_rules! index { ($arr: expr, $idx: expr) => { &$arr[$idx] }; } macro_rules! impossible { () => { unreachable!() }; }"
      },
      token_kind = fmt::comma_sep(g.terms.iter().map(|x| x.name)),
      stack_item = fn2display(move |f| {
        if stack_need_fail { let _ = f.write_str("_Fail, "); }
        for (i, ty) in types.iter().enumerate() { let _ = write!(f, "_{}({}), ", i, ty); }
      }),
      dfa_size = dfa.nodes.len(),
      acc = fmt::acc(g, dfa, "TokenKind"),
      ec = fmt::comma_sep(dfa.ec.iter()),
      u_dfa_size = fmt::min_u(dfa.nodes.len()),
      ec_num = dfa.ec_num,
      dfa_edge = fmt::dfa_edge(dfa, ('[', ']')),
      show_token_prod = fn2display(move |f| {
        if verbose {
          let _ = f.write_str("fn show_token(id: u32) -> &'static str { [");
          for i in 0..g.token_num() { let _ = write!(f, "{:?}, ", g.show_token(i)); }
          let _ = f.write_str("][id as usize] }\nfn show_prod(id: u32) -> &'static str { [");
          for i in 0..g.prod.len() {
            // this cannot be simplified, must use the debug format of result String
            let _ = write!(f, "{:?}, ", format!("{}", g.show_prod(i, None)));
          }
          let _ = f.write_str("][id as usize] }");
        }
      }),
      parser_struct = fn2display(move |f| {
        if g.raw.parser_def.is_none() {
          let _ = write!(f, "pub struct Parser {{ {} }}", fmt::comma_sep(g.raw.parser_field.iter()));
        }
      })
    );
  }

  // log_reduce == self.log_reduce, but this functions cannot borrow self
  // is_pair == true: `stk` is Vec<(StackItem, integer)>; is_pair == false: `stk` is Vec<StackItem>
  fn gen_act<'a>(log_reduce: bool, g: &'a Grammar, types2id: HashMap<&'a str, u32>, is_pair: bool, handle_err: &'a str) -> impl std::fmt::Display + 'a {
    fn2display(move |f| {
      let pat = if is_pair { ", _" } else { "" };
      for (i, prod) in g.prod.iter().enumerate() {
        let _ = writeln!(f, "{} => {{", i);
        if log_reduce {
          let _ = writeln!(f, r#"println!("{}");"#, g.show_prod(i, None));
        }
        for (j, &x) in prod.rhs.iter().enumerate().rev() {
          let name = match prod.args { Some(args) => args[j].0.to_owned(), None => format!("_{}", j + 1) };
          if let Some(x) = g.as_nt(x) {
            let id = types2id[g.nt[x].ty];
            let _ = writeln!(f, "let {} = match stk.pop() {{ Some((StackItem::_{}(x){})) => x, _ => {} }};", name, id, pat, handle_err);
          } else {
            let _ = writeln!(f, "let {} = match stk.pop() {{ Some((StackItem::_Token(x){})) => x, _ => {} }};", name, pat, handle_err);
          }
        }
        let id = types2id[g.nt[prod.lhs as usize].ty];
        let _ = writeln!(f, "StackItem::_{}({{ {} }})\n}}", id, prod.act);
      }
    })
  }
}

impl<W: std::io::Write> Config<'_, W> {
  // return None if `rs_common` returns None, you can check the doc of `rs_common`
  pub fn rs_lalr1(&mut self, g: &Grammar, table: &Table, dfa: &Dfa) {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let res_id = types2id[parse_res];
    self.rs_common(g, dfa, &types, false);
    let _ = write!(
      self.code_output, include_str!("template/lalr1.rs.template"),
      u_lr_fsm_size = fmt::min_u(table.len()),
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      res_type = parse_res,
      res_id = res_id,
      prod_size = g.prod.len(),
      prod = fmt::comma_sep(g.prod.iter().map(|x| x.lhs)),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      lr_fsm_size = table.len(),
      u_act_size = fmt::min_u(table.len() * 4),
      action = fmt::action(g, table),
      goto = fmt::goto(g, &table, ('[', ']')),
      parser_act = Self::gen_act(self.log_reduce, g, types2id, true, "impossible!()"),
      log_token = if self.log_token { r#"println!("{:?}", token);"# } else { "" },
    );
  }

  pub fn rs_ll1(&mut self, g: &Grammar, ll: &LLCtx, dfa: &Dfa) {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let res_id = types2id[parse_res];
    self.rs_common(g, dfa, &types, true);
    let _ = write!(
      self.code_output, include_str!("template/ll1.rs.template"),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      follow = fn2display(move |f| {
        for follow in &ll.follow.0 {
          let _ = writeln!(f, "set!({}),", fmt::comma_sep((0..g.token_num()).filter(|&i| follow.test(i))));
        }
      }),
      table = fn2display(move |f| {
        for table in &ll.table {
          let _ = f.write_str("map!(");
          for (&predict, prod_ids) in table {
            let prod_id = prod_ids[0] as usize;
            let _ = write!(f, "{} => ({}, vec!{:?}), ", predict, prod_id, g.prod[prod_ids[0] as usize].rhs);
          }
          let _ = f.write_str("),\n");
        }
      }),
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      parser_act = Self::gen_act(self.log_reduce, g, types2id, false, "return StackItem::_Fail"),
      res_type = parse_res,
      res_nt_id = g.token_num() - 1,
      res_id = res_id
    );
  }
}