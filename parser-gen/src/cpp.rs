use crate::*;

impl<W: std::io::Write> Config<'_, W> {
  pub fn cpp_lalr1(&mut self, g: &Grammar, table: &Table, dfa: &Dfa) -> Result<()> {
    let (types, _) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    write!(
      self.code_output, include_str!("template/lalr1.cpp.template"),
      include = g.raw.include,
      token_kind = fmt::comma_sep(g.terms.iter().map(|x| x.name)),
      stack_item = types.join(","),
      lexer_field = g.raw.lexer_field,
      acc = fmt::acc(g, dfa, "Token"),
      ec = fmt::comma_sep(dfa.ec.iter()),
      u_dfa_size = fmt::min_u(dfa.nodes.len()),
      ec_num = dfa.ec_num,
      dfa_edge = fmt::dfa_edge(dfa, ('{', '}')),
      lexer_action = g.raw.lexer_action,
      parser_struct = fmt_::fn2display(move |f| if g.raw.parser_def.is_none() {
        writeln!(f, r"struct Parser{{std::variant<{},Token>parse(Lexer&lexer);{}}};", parse_res, g.raw.parser_field)
      } else { Ok(()) }),
      u_lr_fsm_size = fmt::min_u(table.len()),
      u_act_size = fmt::min_u(table.len() * 4),
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      res_type = parse_res,
      prod = fmt::comma_sep(g.prod.iter().map(|x| x.lhs)),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      action = fmt::action(g, table, ('{', '}')),
      goto = fmt::goto(g, table, ('{', '}')),
      parser_act = fmt_::fn2display(|f| (for (i, prod) in g.prod.iter().enumerate() {
        write!(f, "case {}:{{", i)?;
        for (j, &x) in prod.rhs.iter().enumerate().rev() {
          let name = fmt_::fn2display(move |f|
            match prod.args { Some(args) => f.write_str(args[j].0), None => write!(f, "_{}", j + 1) });
          let ty = if let Some(x) = g.as_nt(x) { g.nt[x].ty } else { "Token" };
          writeln!(f, "[[maybe_unused]]{1} {}(std::move(*std::get_if<{1}>(&stk.back().first)));stk.pop_back();", name, ty)?;
        }
        writeln!(f, "{}break;}}", if i == g.prod.len() - 1 { "__=_1;" } else { prod.act })?;
      }, Ok(())).1),
      epilogue = g.raw.epilogue.unwrap_or("")
    )
  }
}