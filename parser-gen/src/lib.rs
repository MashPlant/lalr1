mod fmt;
pub mod rs;
pub mod cpp;
pub mod java;
pub mod show_lr;
pub mod show_ll;

use common::*;
use lalr1_core::*;
use ll1_core::*;
use re2dfa::*;
use std::{fs::File, io::{Result, Write, BufWriter}, fmt::Write as _};

pub trait Codegen {
  fn grammar_error(&mut self, reason: String) -> ! { panic!("invalid grammar, reason: {}", reason) }

  fn re2dfa_error(&mut self, re: &str, reason: String) -> ! { panic!("invalid regex {}, reason: {}", re, reason) }

  fn dfa(&mut self, dfa: &Dfa);

  fn ll(&mut self, g: &Grammar, ll: LLCtx, dfa: &Dfa) -> Result<()>;

  fn lr1(&mut self, g: &Grammar, lr1: &Lr1Fsm, dfa: &Dfa, orig_table: Table, table: Table, conflict: Vec<Conflict>) -> Result<()>;
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PGAlgo { LL1, LR1, LALR1 }

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Lang { Rs, Cpp, Java }

pub struct Config<'a, W> {
  pub verbose: Option<&'a str>,
  pub show_fsm: Option<&'a str>,
  pub show_dfa: Option<&'a str>,
  pub log_token: bool,
  pub log_reduce: bool,
  pub use_unsafe: bool,
  pub lang: Lang,
  pub on_conflict: fn(String),
  pub code_output: W,
}

fn write(path: &str, s: impl Display) -> Result<()> {
  write!(BufWriter::new(File::create(path)?), "{}", s)
}

impl<W: Write> Codegen for Config<'_, W> {
  fn dfa(&mut self, dfa: &Dfa) {
    // these 2 characteristics make lexer behaviour hard to define and make lex generator hard to write
    if dfa.nodes.is_empty() || dfa.nodes[0].0.is_some() { panic!("final dfa is not suitable for a lexer, i.e., it doesn't accept anything, or it accepts empty string"); }
    if let Some(path) = self.show_dfa {
      write(path, dfa.print_dot()).expect("failed to write dfa");
    }
  }

  fn ll(&mut self, g: &Grammar, ll: LLCtx, dfa: &Dfa) -> Result<()> {
    if let Some(path) = self.verbose {
      write(path, show_ll::table(&ll, g)).expect("failed to write ll1 table");
    }
    for c in show_ll::conflict(&ll.table, g) { (self.on_conflict)(c); }
    match self.lang {
      Lang::Rs => self.rs_ll1(&g, &ll, dfa),
      _ => unimplemented!("ll1 codegen is currently only implemented for rust"),
    }
  }

  fn lr1(&mut self, g: &Grammar, lr1: &Lr1Fsm, dfa: &Dfa, orig_table: Table, table: Table, conflict: Vec<Conflict>) -> Result<()> {
    if let Some(path) = self.verbose {
      write(path, show_lr::table(&orig_table, &table, g)).expect("failed to write lr1 table");
    }
    if let Some(path) = self.show_fsm {
      write(path, show_lr::lr1_dot(g, &lr1)).expect("failed to write lr1 fsm");
    }
    for c in show_lr::conflict(g, &conflict) { (self.on_conflict)(c); }
    if conflict.iter().any(Conflict::is_many) { panic!(">= 3 conflicts on one token, give up solving conflicts"); }
    match self.lang {
      Lang::Rs => self.rs_lalr1(&g, &table, dfa),
      Lang::Cpp => self.cpp_lalr1(&g, &table, dfa),
      Lang::Java => self.java_lalr1(&g, &table, dfa),
    }
  }
}

pub fn work(mut raw: RawGrammar, algo: PGAlgo, gen: &mut impl Codegen) -> Result<()> {
  use PGAlgo::*;
  let dfa = match re2dfa(raw.lexical.iter().map(|(s, _)| s.as_bytes())) {
    Ok(x) => x, Err((idx, reason)) => gen.re2dfa_error(raw.lexical.get_index(idx).unwrap().0, reason)
  };
  gen.dfa(&dfa);
  let ref g = match raw.extend(true) { Ok(x) => x, Err(reason) => gen.grammar_error(reason) };
  match algo {
    LL1 => gen.ll(g, LLCtx::new(g), &dfa),
    LALR1 | LR1 => {
      let lr1 = if algo == LALR1 { lalr1_by_lr0::work(lr0::work(g), g) } else { lr1::work(g) };
      let orig_table = mk_table::mk_table(&lr1, g);
      let mut table = orig_table.clone();
      let conflict = lalr1_core::mk_table::solve(&mut table, g);
      gen.lr1(g, &lr1, &dfa, orig_table, table, conflict)
    }
  }
}