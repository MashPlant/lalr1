use crate::*;

impl<W: std::io::Write> Config<'_, W> {
  fn rs_common(&mut self, g: &Grammar, dfa: &Dfa, types: &[&str], stack_need_fail: bool) -> Result<()> {
    let verbose = self.verbose.is_some();
    write!(
      self.code_output, include_str!("template/common.rs.template"),
      include = g.raw.include,
      macros = if self.use_unsafe {
        "macro_rules!idx{($arr:expr,$idx:expr)=>{unsafe{$arr.get_unchecked($idx)}};}macro_rules!err{()=>{unsafe{std::hint::unreachable_unchecked()}};}"
      } else {
        "macro_rules!idx{($arr:expr,$idx:expr)=>{&$arr[$idx]};}macro_rules!err{()=>{unreachable!()};}"
      },
      token_kind = fmt::comma_sep(g.terms.iter().map(|x| x.name)),
      stack_item = fmt_::fn2display(move |f| {
        if stack_need_fail { f.write_str("_Fail,")?; }
        for (i, ty) in types.iter().enumerate() { write!(f, "_S{}({}),", i, ty)?; }
        Ok(())
      }),
      dfa_size = dfa.nodes.len(),
      acc = fmt::acc(g, dfa),
      ec = fmt::comma_sep(dfa.ec.iter()),
      u_dfa_size = fmt::min_u(dfa.nodes.len()),
      ec_num = dfa.ec_num,
      dfa_edge = fmt::dfa_edge(dfa, ('[', ']')),
      show_token_prod = fmt_::fn2display(move |f| if verbose {
        f.write_str("fn show_token(id:u32)->&'static str{[")?;
        for i in 0..g.token_num() { write!(f, "{:?}, ", g.show_token(i))?; }
        f.write_str("][id as usize]}fn show_prod(id:u32)->&'static str{[")?;
        for i in 0..g.prod.len() {
          // this cannot be simplified, must use the debug format of result String
          write!(f, "{:?},", format!("{}", g.show_prod(i, None)))?;
        }
        f.write_str("][id as usize]}")
      } else { Ok(()) }),
      parser_struct = fmt_::fn2display(move |f| if g.raw.parser_def.is_none() {
        write!(f, "pub struct Parser{{{}}}", fmt::comma_sep(g.raw.parser_field.iter()))
      } else { Ok(()) })
    )
  }

  // log_reduce == self.log_reduce, but this functions cannot borrow self
  // is_pair == true: `stk` is Vec<(StackItem, integer)>; is_pair == false: `stk` is Vec<StackItem>
  fn gen_act<'a>(log_reduce: bool, g: &'a Grammar, types2id: HashMap<&'a str, u32>, is_pair: bool, handle_err: &'a str) -> impl std::fmt::Display + 'a {
    fmt_::fn2display(move |f| {
      let pat = if is_pair { ",_" } else { "" };
      for (i, prod) in g.prod.iter().enumerate() {
        writeln!(f, "{}=>{{", i)?;
        if log_reduce {
          writeln!(f, r#"println!("{}");"#, g.show_prod(i, None))?;
        }
        for (j, &x) in prod.rhs.iter().enumerate().rev() {
          let name = fmt_::fn2display(move |f|
            match prod.args { Some(args) => f.write_str(args[j].0), None => write!(f, "_{}", j + 1) });
          if let Some(x) = g.as_nt(x) {
            let id = types2id[g.nt[x].ty];
            writeln!(f, "let {}=match stk.pop(){{Some((_S{}(x){}))=>x,_=>{}}};", name, id, pat, handle_err)?;
          } else {
            writeln!(f, "let {}=match stk.pop(){{Some((_Token(x){}))=>x,_=>{}}};", name, pat, handle_err)?;
          }
        }
        let id = types2id[g.nt[prod.lhs as usize].ty];
        writeln!(f, "_S{}({{{}}})\n}}", id, prod.act)?;
      }
      Ok(())
    })
  }
}

impl<W: std::io::Write> Config<'_, W> {
  // return None if `rs_common` returns None, you can check the doc of `rs_common`
  pub fn rs_lalr1(&mut self, g: &Grammar, table: &Table, dfa: &Dfa) -> Result<()> {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let res_id = types2id[parse_res];
    self.rs_common(g, dfa, &types, false)?;
    write!(
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
      action = fmt::action(g, table, ('[', ']')),
      goto = fmt::goto(g, &table, ('[', ']')),
      parser_act = Self::gen_act(self.log_reduce, g, types2id, true, "err!()"),
      log_token = if self.log_token { r#"println!("{:?}",token);"# } else { "" },
    )
  }

  pub fn rs_ll1(&mut self, g: &Grammar, ll: &LLCtx, dfa: &Dfa) -> Result<()> {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let res_id = types2id[parse_res];
    self.rs_common(g, dfa, &types, true)?;
    write!(
      self.code_output, include_str!("template/ll1.rs.template"),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      follow = fmt_::fn2display(move |f| (for i in 0..g.nt.len() {
        f.write_str("set!(")?;
        bitset::ibs(ll.follow.get(i)).ones(|i| { let _ = write!(f, "{},", i); });
        f.write_str("),\n")?;
      }, Ok(())).1),
      table = fmt_::fn2display(move |f| (for table in &ll.table {
        f.write_str("map!(")?;
        for (&predict, prod_ids) in table {
          let prod_id = prod_ids[0] as usize;
          write!(f, "{}=>({},vec!{:?}),", predict, prod_id, g.prod[prod_id].rhs)?;
        }
        f.write_str("),\n")?;
      }, Ok(())).1),
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      parser_act = Self::gen_act(self.log_reduce, g, types2id, false, "return _Fail"),
      res_type = parse_res,
      res_nt_id = g.token_num() - 1,
      res_id = res_id
    )
  }
}