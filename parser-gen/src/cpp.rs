use re2dfa::dfa::Dfa;
use lalr1_core::{TableEntry, Table, Act};
use common::{grammar::Grammar};
use std::fmt::Write;
use crate::{Config, fmt};

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
      token_kind = fmt::token_kind(g),
      stack_item = types.join(", "),
      acc = fmt::acc(g, dfa),
      ec = fmt::ec(ec),
      u_dfa_size = fmt::min_u(dfa.nodes.len()),
      ec_size = *ec.iter().max().unwrap() + 1,
      dfa_edge = fmt::dfa_edge(dfa, ec, ('{', '}')),
      parser_struct = {
        let mut s = String::new();
        if g.raw.parser_def.is_none() {
          let _ = writeln!(s, "struct Parser {{");
          if let Some(ext) = &g.raw.parser_field {
            for field in ext { let _ = writeln!(s, "{};", field); }
          }
          let _ = writeln!(s, "}};");
        }
        s
      },
      u_lr_fsm_size = fmt::min_u(table.len()),
      parser_type = g.raw.parser_def.as_deref().unwrap_or("Parser"),
      res_type = parse_res,
      u_prod = fmt::min_u(table.len().max(g.prod.iter().map(|x| x.rhs.len()).max().unwrap())),
      prod = {
        let mut s = String::new();
        for p in &g.prod { let _ = write!(s, "{{{}, {}}}, ", p.lhs, p.rhs.len()); }
        s
      },
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      action = {
        let mut s = String::new();
        for TableEntry { act, .. } in table {
          let _ = write!(s, "{{");
          for i in 0..g.terms.len() as u32 {
            match act.get(&i) {
              Some(act) if !act.is_empty() => match act[0] {
                Act::Acc => { let _ = write!(s, "Act{{.kind = Act::Acc, .val = 0}}, "); }
                Act::Shift(x) => { let _ = write!(s, "Act{{.kind = Act::Shift, .val = {}}}, ", x); }
                Act::Reduce(x) => { let _ = write!(s, "Act{{.kind = Act::Reduce, .val = {}}}, ", x); }
              }
              _ => { let _ = write!(s, "Act{{.kind = Act::Err, .val = 0}}, "); }
            }
          }
          let _ = write!(s, "}}, ");
        }
        s
      },
      goto = fmt::goto(g, table, ('{', '}')),
      parser_act = {
        let mut s = String::new();
        for (i, prod) in g.prod.iter().enumerate() {
          let _ = writeln!(s, "case {}: {{", i);
          for (j, &x) in prod.rhs.iter().enumerate().rev() {
            let name = match prod.args {
              Some(args) => args[j].0.as_deref().unwrap_or("_").to_owned(),
              None => format!("_{}", j + 1),
            };
            if let Some(x) = g.as_nt(x) {
              let _ = writeln!(s, "auto {}(std::move(*std::get_if<{}>(&value_stk.back()))); value_stk.pop_back();", name, g.nt[x].ty);
            } else {
              let _ = writeln!(s, "auto {}(std::move(*std::get_if<Token>(&value_stk.back()))); value_stk.pop_back();", name);
            }
          }
          if i == g.prod.len() - 1 {
            let _ = writeln!(s, "value = _1;\nbreak;\n}}");
          } else {
            let _ = writeln!(s, "{}\nbreak;\n}}", prod.act.replace("__", "value"));
          }
        }
        s
      }
    ))
  }
}