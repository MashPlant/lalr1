use common::grammar::*;
use lalr1_core::*;
use parser_gen::*;
use clap::{App, Arg};
use std::{io, fs, process};

fn main() -> io::Result<()> {
  let m = App::new("parser_gen")
    .arg(Arg::with_name("input").required(true))
    .arg(Arg::with_name("output").long("output").short("o").takes_value(true).required(true))
    .arg(Arg::with_name("verbose").long("verbose").short("v").takes_value(true))
    .arg(Arg::with_name("lang").long("lang").short("l").takes_value(true).possible_values(&["rs"]).required(true))
    .arg(Arg::with_name("log_token").long("log_token"))
    .arg(Arg::with_name("log_reduce").long("log_reduce"))
    .arg(Arg::with_name("use_unsafe").long("use_unsafe"))
    .get_matches();
  let input = fs::read_to_string(m.value_of("input").unwrap())?;
  let mut raw = toml::from_str::<RawGrammar>(&input).unwrap_or_else(|reason| {
    eprintln!("Invalid toml, reason: {}.", reason);
    process::exit(1);
  });
  let (dfa, ec) = re2dfa::re2dfa(raw.lexical.iter().map(|(k, _)| k)).unwrap_or_else(|(idx, reason)| {
    eprintln!("Invalid regex {}, reason: {}.", raw.lexical.get_index(idx).unwrap().0, reason);
    process::exit(1);
  });
  let ref g = raw.extend().unwrap_or_else(|reason| {
    eprintln!("Invalid grammar, reason: {}.", reason);
    process::exit(1);
  });
  let lr0 = lr0::work(g);
  let lr1 = lalr1_by_lr0::work(lr0, g);
  let orig_table = mk_table::mk_table(&lr1, g);
  let mut table = orig_table.clone();
  let conflict = lalr1_core::mk_table::solve(&mut table, g);
  if let Some(verbose) = m.value_of("verbose") {
    fs::write(&verbose, show_lr::table(&orig_table, &table, g))?;
  }
  if let Some(show_fsm) = m.value_of("show_fsm") {
    fs::write(&show_fsm, show_lr::lr1_dot(g, &lr1))?;
  }
  for c in show_lr::conflict(g, &conflict) {
    eprintln!("{}", c);
  }
  if conflict.iter().any(|c| if let ConflictKind::Many(_) = c.kind { true } else { false }) {
    eprintln!(">= 3 conflicts on one token detected, failed to solve conflicts.");
    process::exit(1);
  }
  let code = match m.value_of("lang") {
    Some("rs") => RustCodegen { log_token: m.is_present("log_token"), log_reduce: m.is_present("log_reduce"), use_unsafe: m.is_present("use_unsafe"), show_token_prod: m.is_present("verbose") }
      .gen_lalr1(&g, &table, &dfa, &ec).unwrap_or_else(|| {
      eprintln!("{}", INVALID_DFA);
      process::exit(1)
    }),
    _ => unreachable!(),
  };
  fs::write(m.value_of("output").unwrap(), &code)
}