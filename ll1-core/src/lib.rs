extern crate grammar_config;
extern crate bitset;

use grammar_config::AbstractGrammar;
use bitset::BitSet;

pub struct First {
  pub token_num: u32,
  pub nt_num: u32,
  pub eps: u32,
  pub nt_first: Vec<BitSet>,
}

impl First {
  pub fn new<'a>(g: &'a impl AbstractGrammar<'a>) -> First {
    unimplemented!()
  }

  pub fn first(&self) {
    unimplemented!()
  }
}

pub struct Follow {}

pub struct LLCtx {
  first: First,
  follow: Follow,
}