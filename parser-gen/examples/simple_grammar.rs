use clap::{App, Arg};
use std::{io, fs};
use common::*;
use parser_gen::*;
use lalr1_core::*;

fn parse_lines(s: &str) -> Result<RawGrammar, String> {
  let mut production = Vec::new();
  let mut all_lhs = HashSet::default();
  for s in s.lines() {
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
        // use current len as terminal regex
        if !all_lhs.contains(r) { lexical.insert(lexical.len().to_string().into(), r); }
      }
    }
  }
  Ok(RawGrammar { include: "", priority: vec![], lexical, parser_field: Vec::new(), start, production, parser_def: None })
}

fn main() -> io::Result<()> {
  let m = App::new("simple_grammar")
    .arg(Arg::new("input").required(true))
    .arg(Arg::new("output").long("output").short('o').takes_value(true).required(true))
    .arg(Arg::new("grammar").long("grammar").short('g').takes_value(true).possible_values(&["lr0", "lr1", "lalr1", "ll1"]).required(true))
    .get_matches();
  let input = fs::read_to_string(m.value_of("input").unwrap())?;
  let mut raw = parse_lines(&input).expect("invalid input grammar");
  let ref g = raw.extend(false).unwrap(); // it should not fail
  let result = match m.value_of("grammar") {
    Some("lr0") => format!("{}", show_lr::lr0_dot(g, &lr0::work(g))),
    Some("lr1") => format!("{}", show_lr::lr1_dot(g, &lr1::work(g))),
    Some("lalr1") => format!("{}", show_lr::lr1_dot(g, &lalr1_by_lr0::work(lr0::work(g), g))),
    Some("ll1") => format!("{}", show_ll::table(&ll1_core::LLCtx::new(g), g)),
    _ => unreachable!(),
  };
  fs::write(m.value_of("output").unwrap(), result.replace("_Eof", "#"))
}