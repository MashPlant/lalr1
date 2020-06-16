pub mod rs;
pub mod show_lr;
pub mod show_ll;

pub use rs::*;

use common::grammar::{RawGrammar, Grammar};
use lalr1_core::*;
use ll1_core::LLCtx;
use re2dfa::Dfa;

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

pub fn work(mut raw: RawGrammar, algo: PGAlgo, gen: &mut impl Codegen) {
  use PGAlgo::*;
  let dfa_ec = match re2dfa::re2dfa(raw.lexical.iter().map(|(s, _)| s)) {
    Ok(x) => x, Err((idx, reason)) => return gen.re2dfa_error(raw.lexical.get_index(idx).unwrap().0, reason)
  };
  gen.dfa_ec(&dfa_ec);
  let ref g = match raw.extend() { Ok(x) => x, Err(reason) => return gen.grammar_error(reason) };
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

pub(crate) fn min_u(x: u32) -> &'static str {
  match x { 0..=255 => "u8", 256..=65535 => "u16", _ => "u32" }
}

pub const INVALID_DFA: &str = "final dfa is not suitable for a lexer, i.e., it doesn't accept anything, or it accepts empty string";