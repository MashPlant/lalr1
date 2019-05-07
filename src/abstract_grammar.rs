use std::cmp::Ordering;
use crate::raw_grammar::Assoc;

// about the distribution of non-terminal & terminal & eof & eps on u32:
// non-terminal: 0..nt_num(), terminal & eof & eps: nt_num()..token_num()
pub trait AbstractGrammar<'a> {
  type ProdRef: AsRef<[u32]> + 'a;
  type ProdIter: IntoIterator<Item=&'a Self::ProdRef>;

  fn eps(&self) -> u32;

  fn eof(&self) -> u32;

  fn token_num(&self) -> u32;

  fn nt_num(&self) -> u32;

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter;
}

pub trait AbstractGrammarExt<'a>: AbstractGrammar<'a> {
  fn cmp_priority(&self, a: u32, b: u32) -> Ordering;

  fn get_assoc(&self, ch: u32) -> Assoc;
}