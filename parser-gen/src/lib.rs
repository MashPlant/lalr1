mod fmt;
pub mod rs;
pub mod cpp;
pub mod show_lr;
pub mod show_ll;

pub use rs::*;

use common::grammar::{RawGrammar, Grammar};
use lalr1_core::*;
use ll1_core::LLCtx;
use re2dfa::Dfa;
use std::fs;

pub trait Codegen {
  fn grammar_error(&mut self, reason: String) { panic!("invalid grammar, reason: {}", reason) }

  fn re2dfa_error(&mut self, re: &str, reason: String) { panic!("invalid regex {}, reason: {}", re, reason) }

  fn dfa_ec(&mut self, dfa_ec: &(Dfa, [u8; 256]));

  fn ll(&mut self, g: &Grammar, ll: LLCtx, dfa_ec: &(Dfa, [u8; 256]));

  fn lr0(&mut self, g: &Grammar, lr0: Lr0Fsm); // it is not meant for a parser generator, so `dfa_ec` is not passed

  fn lr1(&mut self, g: &Grammar, lr1: &Lr1Fsm, dfa_ec: &(Dfa, [u8; 256]), orig_table: Table, table: Table, conflict: Vec<Conflict>);
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PGAlgo { LL1, LR0, LR1, LALR1 }

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Lang { RS, CPP }

pub struct Config<'a, F> {
  pub verbose: Option<&'a str>,
  pub show_fsm: Option<&'a str>,
  pub show_dfa: Option<&'a str>,
  pub log_token: bool,
  pub log_reduce: bool,
  pub use_unsafe: bool,
  pub code: String,
  pub lang: Lang,
  pub on_conflict: F,
}

const INVALID_DFA: &str = "final dfa is not suitable for a lexer, i.e., it doesn't accept anything, or it accepts empty string";

impl<F: Fn(String)> Codegen for Config<'_, F> {
  fn dfa_ec(&mut self, dfa_ec: &(Dfa, [u8; 256])) {
    if let Some(path) = self.show_dfa.as_ref() {
      fs::write(path, dfa_ec.0.print_dot()).unwrap_or_else(|e| panic!("failed to write dfa into \"{}\": {}", path, e));
    }
  }

  fn ll(&mut self, g: &Grammar, ll: LLCtx, dfa_ec: &(Dfa, [u8; 256])) {
    if let Some(path) = self.verbose.as_ref() {
      fs::write(path, show_ll::table(&ll, g)).unwrap_or_else(|e| panic!("failed to write ll1 table into \"{}\": {}", path, e));
    }
    for c in show_ll::conflict(&ll.table, g) { (self.on_conflict)(c); }
    self.code = match self.lang {
      Lang::RS => self.rs_ll1(&g, &ll, dfa_ec),
      _ => unimplemented!("ll1 codegen is currently only implemented for rust"),
    }.unwrap_or_else(|| panic!(INVALID_DFA));
  }

  // won't be called
  fn lr0(&mut self, _g: &Grammar, _lr0: Lr0Fsm) {}

  fn lr1(&mut self, g: &Grammar, lr1: &Lr1Fsm, dfa_ec: &(Dfa, [u8; 256]), orig_table: Table, table: Table, conflict: Vec<Conflict>) {
    if let Some(path) = self.verbose.as_ref() {
      fs::write(path, show_lr::table(&orig_table, &table, g)).unwrap_or_else(|e| panic!("failed to write lr1 information into \"{}\": {}", path, e));
    }
    if let Some(path) = self.show_fsm.as_ref() {
      fs::write(path, show_lr::lr1_dot(g, &lr1)).unwrap_or_else(|e| panic!("failed to write lr1 fsm into \"{}\": {}", path, e));
    }
    for c in show_lr::conflict(g, &conflict) { (self.on_conflict)(c); }
    if conflict.iter().any(Conflict::is_many) { panic!(">= 3 conflicts on one token, give up solving conflicts"); }
    self.code = match self.lang {
      Lang::RS => self.rs_lalr1(&g, &table, dfa_ec),
      Lang::CPP => self.cpp_lalr1(&g, &table, dfa_ec),
    }.unwrap_or_else(|| panic!(INVALID_DFA));
  }
}

pub fn work(mut raw: RawGrammar, algo: PGAlgo, gen: &mut impl Codegen) {
  use PGAlgo::*;
  let dfa_ec = match re2dfa::re2dfa(raw.lexical.iter().map(|(s, _)| s)) {
    Ok(x) => x, Err((idx, reason)) => return gen.re2dfa_error(raw.lexical.get_index(idx).unwrap().0, reason)
  };
  gen.dfa_ec(&dfa_ec);
  let ref g = match raw.extend(true) { Ok(x) => x, Err(reason) => return gen.grammar_error(reason) };
  match algo {
    LL1 => gen.ll(g, LLCtx::new(g), &dfa_ec),
    LR0 => gen.lr0(g, lr0::work(g)),
    LALR1 | LR1 => {
      let lr1 = if algo == LALR1 { lalr1_by_lr0::work(lr0::work(g), g) } else { lr1::work(g) };
      let orig_table = mk_table::mk_table(&lr1, g);
      let mut table = orig_table.clone();
      let conflict = lalr1_core::mk_table::solve(&mut table, g);
      gen.lr1(g, &lr1, &dfa_ec, orig_table, table, conflict);
    }
  }
}