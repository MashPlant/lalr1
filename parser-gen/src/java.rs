use crate::*;

impl<W: std::io::Write> Config<'_, W> {
  pub fn java_lalr1(&mut self, g: &Grammar, table: &Table, dfa: &Dfa) -> Result<()> {
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let res_id = types2id[parse_res];
    let terms2id = g.terms.iter().enumerate().map(|(idx, t)| (t.name, idx as u32)).collect::<HashMap<_, _>>();
    write!(
      self.code_output, include_str!("template/lalr1.java.template"),
      include = g.raw.include,
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      parser_field = g.raw.parser_field,
      acc = fmt::comma_sep(dfa.nodes.iter().map(move |&(acc, _)|
        acc.map(|x| terms2id[g.raw.lexical.get_index(x as _).unwrap().1]).unwrap_or(ERR_IDX as u32))),
      ec = fmt::comma_sep(dfa.ec.iter()),
      dfa_edge = fmt::dfa_edge(dfa, ('{', '}')),
      lexer_field = g.raw.lexer_field,
      lexer_action = g.raw.lexer_action,
      stack_item = fmt_::fn2display(move |f| (for (i, ty) in types.iter().enumerate() {
        let _ = writeln!(f, "public static final class StackItem{} extends StackItem {{ {} $; }}", i, ty);
      }, Ok(())).1),
      res_type = parse_res,
      res_id = res_id,
      prod = fmt::comma_sep(g.prod.iter().map(|x| x.lhs)),
      action = fmt::action(g, table, ('{', '}')),
      goto = fmt::goto(g, table, ('{', '}')),
      parser_act = fmt_::fn2display(move |f| (for (i, prod) in g.prod.iter().enumerate() {
        let _ = write!(f, "case {}:{{", i);
        for (j, &x) in prod.rhs.iter().enumerate().rev() {
          let name = fmt_::fn2display(move |f|
            match prod.args { Some(args) => f.write_str(args[j].0), None => write!(f, "${}", j + 1) });
          let (arg_ty, item_ty) = if let Some(x) = g.as_nt(x) {
            (g.nt[x].ty, format!("StackItem{}", types2id[g.nt[x].ty]))
          } else { ("Token", "StackItemToken".to_owned()) };
          let _ = writeln!(f, "{} {}=(({})stk.get(stk.size()-1)).$;stk.remove(stk.size()-1);", arg_ty, name, item_ty);
        }
        let _ = writeln!(f, "StackItem{0} $=new StackItem{0}();", types2id[g.nt[prod.lhs as usize].ty]);
        let _ = writeln!(f, "{}value=$;break;}}", if i == g.prod.len() - 1 { "$.$ = $1;" } else { prod.act });
      }, Ok(())).1)
    )
  }
}