use re2dfa::dfa::Dfa;
use lalr1_core::{TableEntry, Table, Act::*};
use common::{grammar::*, HashMap};
use std::fmt::Write;
use crate::{Config, fmt};

impl<F> Config<'_, F> {
  pub fn java_lalr1(&self, g: &Grammar, table: &Table, dfa_ec: &(Dfa, [u8; 256])) -> Option<String> {
    let (dfa, ec) = dfa_ec;
    if dfa.nodes.is_empty() || dfa.nodes[0].0.is_some() { return None; }
    let (types, types2id) = fmt::gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    Some(format!(
      include_str!("template/lalr1.java.template"),
      include = g.raw.include,
      parser_type = g.raw.parser_def.unwrap_or("Parser"),
      acc = {
        let mut s = String::new();
        let terms2id = g.terms.iter().enumerate().map(|(idx, t)| (t.name, idx as u32)).collect::<HashMap<_, _>>();
        for &(acc, _) in &dfa.nodes {
          let _ = write!(s, "{}, ", acc.map(|x| terms2id[g.raw.lexical.get_index(x as usize).unwrap().1])
            .unwrap_or(ERR_IDX as u32));
        }
        s
      },
      ec = fmt::ec(ec),
      dfa_edge = fmt::dfa_edge(dfa, ec, ('{', '}')),
      stack_item = {
        let mut s = String::new();
        for (i, ty) in types.iter().enumerate() {
          let _ = writeln!(s, "public static final class StackItem{} extends StackItem {{ {} $; }}", i, ty);
        }
        s
      },
      res_type = parse_res,
      res_id = types2id[parse_res],
      prod = {
        let mut s = String::new();
        for p in &g.prod { let _ = write!(s, "{}, ", p.lhs); }
        s
      },
      action = {
        let mut s = String::new();
        for TableEntry { act, .. } in table {
          s.push('{');
          for i in 0..g.terms.len() as u32 {
            let (tag, val) = match act.get(&i) {
              Some(act) if !act.is_empty() => match act[0] { Acc => (2, 0), Shift(x) => (0, x), Reduce(x) => (1, x) }
              _ => (3, 0)
            };
            let _ = write!(s, "{} | ({} << 2), ", tag, val);
          };
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
            let name = match prod.args { Some(args) => args[j].0.to_owned(), None => format!("${}", j + 1) };
            let (arg_ty, item_ty) = if let Some(x) = g.as_nt(x) {
              (g.nt[x].ty, format!("StackItem{}", types2id[g.nt[x].ty]))
            } else { ("Token", "StackItemToken".to_owned()) };
            let _ = writeln!(s, "{} {} = (({}) stk.get(stk.size() - 1)).$; stk.remove(stk.size() - 1);", arg_ty, name, item_ty);
          }
          let _ = writeln!(s, "StackItem{0} $ = new StackItem{0}();", types2id[g.nt[prod.lhs as usize].ty]);
          let _ = writeln!(s, "{}\nvalue = $;\nbreak;\n}}", if i == g.prod.len() - 1 { "$.$ = $1;" } else { prod.act });
        }
        s
      }
    ))
  }
}