use common::grammar::*;
use clap::{App, Arg};
use std::{io, fs};
use parser_gen::*;

fn main() -> io::Result<()> {
  let m = App::new("parser_gen")
    .author("MashPlant").about("Read config from a toml file, and generate a parser in various language")
    .arg(Arg::with_name("input").required(true))
    .arg(Arg::with_name("output").long("output").short("o").takes_value(true).required(true).value_name("path"))
    .arg(Arg::with_name("lang").long("lang").short("l").takes_value(true).possible_values(&["rs", "cpp"]).required(true))
    .arg(Arg::with_name("verbose").long("verbose").takes_value(true).value_name("path").help("Print some parser information (ll table or lr fsm) to the path"))
    .arg(Arg::with_name("show_fsm").long("show_fsm").takes_value(true).value_name("path").help("Print lr fsm in dot file format to the path"))
    .arg(Arg::with_name("show_dfa").long("show_dfa").takes_value(true).value_name("path").help("Print dfa in dot file format to the path"))
    .arg(Arg::with_name("log_token").long("log_token").help("Make parser print recognized token"))
    .arg(Arg::with_name("log_reduce").long("log_reduce").help("Make parser print the rule used when reducing"))
    .arg(Arg::with_name("use_unsafe").long("use_unsafe").help("Make parser use some unsafe operations to improve speed"))
    .get_matches();
  let mut cfg = Config {
    verbose: m.value_of("verbose"),
    show_fsm: m.value_of("show_fsm"),
    show_dfa: m.value_of("show_dfa"),
    log_token: m.is_present("log_token"),
    log_reduce: m.is_present("log_reduce"),
    use_unsafe: m.is_present("use_unsafe"),
    code: String::new(),
    lang: match m.value_of("lang") {
      Some("rs") => Lang::RS, Some("cpp") => Lang::CPP,
      _ => unreachable!()
    },
    on_conflict: |c| eprintln!("{}", c),
  };
  let input = fs::read_to_string(m.value_of("input").unwrap())?;
  let raw = toml::from_str::<RawGrammar>(&input).unwrap_or_else(|e| panic!("invalid grammar toml: {}", e));
  work(raw, PGAlgo::LALR1, &mut cfg);
  fs::write(m.value_of("output").unwrap(), &cfg.code)
}