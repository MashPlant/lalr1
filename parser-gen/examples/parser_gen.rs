use common::grammar::*;
use clap::{App, Arg};
use std::{io, fs};
use parser_gen::workflow::*;

fn main() -> io::Result<()> {
  let m = App::new("parser_gen")
    .arg(Arg::with_name("input").required(true))
    .arg(Arg::with_name("output").long("output").short("o").takes_value(true).required(true))
    .arg(Arg::with_name("verbose").long("verbose").short("v").takes_value(true))
    .arg(Arg::with_name("show_fsm").long("show_fsm").takes_value(true))
    .arg(Arg::with_name("show_dfa").long("show_dfa").takes_value(true))
    .arg(Arg::with_name("log_token").long("log_token"))
    .arg(Arg::with_name("log_reduce").long("log_reduce"))
    .arg(Arg::with_name("use_unsafe").long("use_unsafe"))
    .arg(Arg::with_name("lang").long("lang").short("l").takes_value(true).possible_values(&["rs"]).required(true))
  .get_matches();
  let mut cfg = Config {
    verbose: m.value_of("verbose"),
    show_fsm: m.value_of("show_fsm"),
    show_dfa: m.value_of("show_dfa"),
    log_token: m.is_present("log_token"),
    log_reduce: m.is_present("log_reduce"),
    use_unsafe: m.is_present("use_unsafe"),
    code: String::new(),
    lang: match m.value_of("lang") { Some("rs") => Lang::RS, _ => unreachable!() },
    on_conflict: |c| eprintln!("{}", c),
  };
  let input = fs::read_to_string(m.value_of("input").unwrap())?;
  let raw = toml::from_str::<RawGrammar>(&input).unwrap_or_else(|e| panic!("invalid grammar toml: {}", e));
  work(raw, PGAlgo::LALR1, &mut cfg);
  fs::write(m.value_of("output").unwrap(), &cfg.code)
}