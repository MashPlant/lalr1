use re2dfa::dfa::Dfa;
use lalr1_core::{TableEntry, Table};
use common::{grammar::{Grammar, ERR}, HashMap};
use ll1_core::LLCtx;
use std::fmt::Write;

pub struct RustCodegen {
  pub log_token: bool,
  pub log_reduce: bool,
  pub use_unsafe: bool,
  pub show_token_prod: bool,
}

impl RustCodegen {
  fn gather_types<'a>(&self, g: &Grammar<'a>) -> (Vec<&'a str>, HashMap<&'a str, u32>) {
    let mut types = Vec::new();
    let mut types2id = HashMap::new();
    for nt in &g.nt {
      types2id.entry(nt.ty).or_insert_with(|| {
        let id = types.len() as u32;
        types.push(nt.ty);
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
    Some(format!(
      include_str!("template/common.rs.template"),
      include = g.raw.include,
      macros = if self.use_unsafe {
        "macro_rules! index { ($arr: expr, $idx: expr) => { unsafe { *$arr.get_unchecked($idx as usize) } }; } macro_rules! impossible { () => { unsafe { std::hint::unreachable_unchecked() } }; }"
      } else {
        "macro_rules! index { ($arr: expr, $idx: expr) => { $arr[$idx as usize] }; } macro_rules! impossible { () => { unreachable!() }; }"
      },
      token_kind = {
        let mut s = String::new();
        for t in &g.terms { let _ = write!(s, "{}, ", t.name); }
        s
      },
      stack_item = {
        let mut s = "_Token(Token<'p>), ".to_owned();
        if stack_need_fail { let _ = write!(s, "_Fail, "); }
        for (i, ty) in types.iter().enumerate() { let _ = write!(s, "_{}({}), ", i, ty); }
        s
      },
      dfa_size = dfa.nodes.len(),
      acc = {
        let mut s = String::new();
        for &(acc, _) in &dfa.nodes {
          match acc {
            Some(acc) => { let _ = write!(s, "TokenKind::{}, ", g.raw.lexical.get_index(acc as usize).unwrap().1); }
            None => { let _ = write!(s, "TokenKind::{}, ", ERR); }
          }
        }
        s
      },
      ec = {
        let mut s = String::new();
        for ch in 0..256 { let _ = write!(s, "{}, ", ec[ch]); }
        s
      },
      u_dfa_size = crate::min_u(dfa.nodes.len()),
      ec_size = *ec.iter().max().unwrap() + 1,
      dfa_edge = {
        let mut s = String::new();
        let mut outs = vec![0; (*ec.iter().max().unwrap() + 1) as usize];
        for (_, edges) in dfa.nodes.iter() {
          for x in &mut outs { *x = 0; }
          for (&k, &out) in edges { outs[ec[k as usize] as usize] = out; }
          let _ = write!(s, "{:?}, ", outs);
        }
        s
      },
      show_token_prod = {
        if self.show_token_prod {
          format!("fn show_token(id: u32) -> &'static str {{ {:?}[id as usize] }} fn show_prod(id: u32) -> &'static str {{ {:?}[id as usize] }}",
                  (0..g.token_num()).map(|i| g.show_token(i)).collect::<Vec<_>>(),
                  (0..g.prod.len()).map(|i| g.show_prod(i, None)).collect::<Vec<_>>())
        } else { String::new() }
      },
      parser_struct = {
        let mut s = String::new();
        if g.raw.parser_def.is_none() {
          let _ = writeln!(s, "struct Parser {{");
          if let Some(ext) = &g.raw.parser_field {
            for field in ext { let _ = writeln!(s, "{},", field); }
          }
          let _ = writeln!(s, "}}");
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
        let _ = writeln!(s, r#"println!("{}");"#, g.show_prod(i, None));
      }
      for (j, &x) in prod.rhs.iter().enumerate().rev() {
        let name = match prod.args {
          Some(args) => args[j].0.as_ref().map(|s| s.as_str()).unwrap_or("_").to_owned(),
          None => format!("_{}", j + 1),
        };
        if let Some(x) = g.as_nt(x) {
          let id = types2id[g.nt[x].ty];
          let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_{}(x)) => x, _ => {} }};", name, id, handle_unexpect_stack);
        } else {
          let _ = writeln!(s, "let {} = match value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => {} }};", name, handle_unexpect_stack);
        }
      }
      let id = types2id[g.nt[prod.lhs as usize].ty];
      let _ = writeln!(s, "StackItem::_{}({{ {} }})", id, prod.act);
      let _ = writeln!(s, "}}");
    }
    s
  }
}

impl RustCodegen {
  // return None if `gen_common` returns None, you can check the doc of `gen_common`
  pub fn gen_lalr1(&self, g: &Grammar, table: &Table, dfa: &Dfa, ec: &[u8; 256]) -> Option<String> {
    let (types, types2id) = self.gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let common = self.gen_common(g, dfa, ec, &types, false)?;
    let lalr1 = format!(
      include_str!("template/lalr1.rs.template"),
      u_lr_fsm_size = crate::min_u(table.len()),
      parser_type = g.raw.parser_def.as_deref().unwrap_or("Parser"),
      res_type = parse_res,
      res_id = types2id[parse_res],
      u_prod_len = crate::min_u(g.prod.iter().map(|x| x.rhs.len()).max().unwrap()),
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
          let _ = write!(s, "[");
          for i in 0..g.terms.len() as u32 {
            match act.get(&i) {
              Some(act) if !act.is_empty() => { let _ = write!(s, "Act::{:?}, ", act[0]); }
              _ => { let _ = write!(s, "Act::Err, "); }
            }
          }
          let _ = write!(s, "], ");
        }
        s
      },
      goto = {
        let mut s = String::new();
        for TableEntry { goto, .. } in table {
          let _ = write!(s, "[");
          for i in g.nt_range() { let _ = write!(s, "{}, ", goto.get(&(i as u32)).unwrap_or(&0)); }
          let _ = write!(s, "], ");
        }
        s
      },
      parser_act = self.gen_act(g, &types2id, "impossible!()"),
      log_token = if self.log_token { r#"println!("{:?}", token);"# } else { "" },
    );
    Some(common + &lalr1)
  }

  pub fn gen_ll1(&self, g: &Grammar, ll: &LLCtx, dfa: &Dfa, ec: &[u8; 256]) -> Option<String> {
    let (types, types2id) = self.gather_types(g);
    let parse_res = g.nt.last().unwrap().ty;
    let common = self.gen_common(g, dfa, ec, &types, true)?;
    let ll1 = format!(
      include_str!("template/ll1.rs.template"),
      term_num = g.terms.len(),
      nt_num = g.nt.len(),
      follow = {
        let mut s = String::new();
        for follow in &ll.follow.0 {
          let _ = write!(s, "set!(");
          for i in 0..g.token_num() {
            if follow.test(i as usize) { let _ = write!(s, "{}, ", i); }
          }
          let _ = writeln!(s, "),");
        }
        s
      },
      table = {
        let mut s = String::new();
        for table in &ll.table {
          let _ = write!(s, "map!(");
          for (&predict, prod_ids) in table {
            let prod_id = prod_ids[0] as usize;
            let _ = write!(s, "{} => ({}, vec!{:?}), ", predict, prod_id, g.prod[prod_ids[0] as usize].rhs);
          }
          let _ = writeln!(s, "),");
        }
        s
      },
      parser_type = g.raw.parser_def.as_deref().unwrap_or("Parser"),
      parser_act = self.gen_act(g, &types2id, "return StackItem::_Fail"),
      res_type = parse_res,
      res_nt_id = g.token_num() - 1,
      res_id = types2id[parse_res]
    );
    Some(common + &ll1)
  }
}