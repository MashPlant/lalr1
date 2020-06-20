use common::{grammar::*, parse_arrow_prod};
use parser_gen::{show_lr, show_ll};
use lalr1_core::*;
use common::{IndexMap, HashSet};
use typed_arena::Arena;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn parse_lines<'a>(s: &'a str, arena: &'a Arena<u8>) -> Result<RawGrammar<'a>, String> {
  let mut production = Vec::new();
  let mut all_lhs = HashSet::new();
  for s in s.lines().map(|s| s.trim()).filter(|s| !s.is_empty()) {
    let (lhs, rhs) = parse_arrow_prod(s).ok_or_else(|| format!("invalid input \"{}\", expect form of \"lhs -> rhs1 rhs2 ...\"", s))?;
    if lhs == START_NT_NAME || rhs.iter().any(|&x| x == START_NT_NAME) {
      // we are not going to validate user input names (see `raw.extend(false)`)
      // so we should check it here manually that the manually added START_NT_NAME isn't used
      return Err(format!("invalid token name: \"{}\"", START_NT_NAME));
    }
    all_lhs.insert(lhs);
    production.push(RawProduction { lhs, ty: "", rhs: vec![RawProductionRhs { rhs, rhs_arg: None, act: "", prec: None }] });
  }
  let start = production.get(0).ok_or_else(|| "grammar must have at least one production rule".to_owned())?.lhs;
  let mut lexical = IndexMap::default();
  for p in &production {
    for r in &p.rhs {
      for &r in &r.rhs {
        if !all_lhs.contains(r) {
          // use current len as a unique id (key will be used regex)
          let k = &*arena.alloc_str(&lexical.len().to_string());
          lexical.insert(k, r);
        }
      }
    }
  }
  Ok(RawGrammar { include: "", priority: vec![], lexical, parser_field: Vec::new(), start, production, parser_def: None })
}

#[wasm_bindgen]
pub fn parser(s: &str, algo: &str, table: bool) -> Result<String, JsValue> {
  let arena = Arena::<u8>::new();
  let mut raw = parse_lines(s, &arena).map_err(|e| JsValue::from(&format!("input is invalid: {}", e)))?;
  let ref g = raw.extend(false).unwrap(); // it should not fail
  let result = match algo {
    "lr(0)" => show_lr::lr0_dot(g, &lr0::work(g)),
    "lr(1)" | "lalr(1)" => {
      let lr1 = if algo == "lr(1)" { lr1::work(g) } else { lalr1_by_lr0::work(lr0::work(g), g) };
      if table {
        let orig_table = mk_table::mk_table(&lr1, g);
        let mut table = orig_table.clone();
        let _ = lalr1_core::mk_table::solve(&mut table, g);
        show_lr::table(&orig_table, &table, g)
      } else { show_lr::lr1_dot(g, &lr1) }
    }
    "ll(1)" => show_ll::table(&ll1_core::LLCtx::new(g), g),
    _ => return Err(JsValue::from(&format!(r#"grammar "{}" is invalid, expect one of "lr0", "lr1", "lalr1", "ll1"#, algo)))
  };
  Ok(result.replace(EOF, "#"))
}

#[wasm_bindgen]
pub fn lexer(s: &str) -> Result<String, JsValue> {
  let re = s.lines().collect::<Vec<_>>();
  let (dfa, _) = re2dfa::re2dfa(&re).map_err(|(idx, e)|
    JsValue::from(&format!("invalid regex {}, reason: {}", re[idx], e)))?;
  Ok(dfa.print_dot())
}
