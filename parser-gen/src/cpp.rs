use re2dfa::dfa::Dfa;
use lalr1_core::{TableEntry, Table, Act::*};
use common::{grammar::Grammar};
use std::fmt::Write;
use crate::{Config, fmt::{self, CommaSep}};

impl<F> Config<'_, F> {
  pub fn cpp_lalr1(&self, g: &Grammar, table: &Table, dfa_ec: &(Dfa, [u8; 256])) -> Option<String> {
    let (dfa, ec) = dfa_ec;
    if dfa.nodes.is_empty() || dfa.nodes[0].0.is_some() { return None; }
    let (types, _) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    Some(format!(
      include_str!("template/lalr1.cpp.template"),
      include = g.raw.include,
      u_term_num = fmt::min_u(g.terms.len()),
      token_kind = CommaSep(g.terms.iter().map(|x| x.name)),
      stack_item = types.join(", "),
      acc = fmt::acc(g, dfa, "Token"),
      ec = CommaSep(ec.iter()),
      u_dfa_size = fmt::min_u(dfa.nodes.len()),
      ec_size = *ec.iter().max().unwrap() + 1,
      dfa_edge = fmt::dfa_edge(dfa, ec, ('{', '}')),
      parser_struct = {
        let mut s = String::new();
        if g.raw.parser_def.is_none() {
          let _ = writeln!(s, r"struct Parser {{
  std::variant<{}, Token> parse(Lexer &lexer);", parse_res);
          for &field in &g.raw.parser_field { let _ = writeln!(s, "{};", field); }
          s += "};\n"
        }
        s
      },
      u_lr_fsm_size = fmt::min_u(table.len()),
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      res_type = parse_res,
      prod = CommaSep(g.prod.iter().map(|x| x.lhs)),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      action = {
        let mut s = String::new();
        for TableEntry { act, .. } in table {
          s.push('{');
          for i in 0..g.terms.len() as u32 {
            let (kind, val) = match act.get(&i) {
              Some(act) if !act.is_empty() => match act[0] { Acc => ("Acc", 0), Shift(x) => ("Shift", x), Reduce(x) => ("Reduce", x) }
              _ => ("Err", 0)
            };
            let _ = write!(s, "{{Act::{}, {}}}, ", kind, val);
          }
          s += "},";
        }
        s
      },
      goto = fmt::goto(g, table, ('{', '}')),
      parser_act = {
        let mut s = String::new();
        for (i, prod) in g.prod.iter().enumerate() {
          let _ = writeln!(s, "case {}: {{", i);
          for (j, &x) in prod.rhs.iter().enumerate().rev() {
            let name = match prod.args { Some(args) => args[j].0.to_owned(), None => format!("_{}", j + 1) };
            let ty = if let Some(x) = g.as_nt(x) { g.nt[x].ty } else { "Token" };
            let _ = writeln!(s, "[[maybe_unused]] {1} {}(std::move(*std::get_if<{1}>(&stk.back().first))); stk.pop_back();", name, ty);
          }
          let _ = writeln!(s, "{}\nbreak;\n}}", if i == g.prod.len() - 1 { "__ = _1;" } else { prod.act });
        }
        s
      }
    ))
  }
}