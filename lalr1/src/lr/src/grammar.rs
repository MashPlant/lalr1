use crate::raw_grammar::*;
use crate::abstract_grammar::{AbstractGrammar, AbstractGrammarExt};
use smallvec::SmallVec;

pub type ProdVec = SmallVec<[u32; 6]>;

#[derive(Debug)]
pub struct Grammar<'a> {
  pub raw: &'a RawGrammar,
  //                 name
  pub terminal: Vec<(&'a str, Option<(u32, Assoc)>)>,
  //          (name   , type_  )>
  pub nt: Vec<(&'a str, &'a str)>,
  pub prod: Vec<Vec<(ProdVec, u32)>>,
  //                   act      (lhs, index of this prod in self.prod[lhs])
  pub prod_extra: Vec<(&'a str, (u32, u32), Option<(u32, Assoc)>)>,
}

impl<'a> AbstractGrammar<'a> for Grammar<'a> {
  type ProdRef = ProdVec;
  type ProdIter = &'a Vec<(ProdVec, u32)>;

  fn start(&'a self) -> &'a (Self::ProdRef, u32) {
    &self.prod.last().unwrap()[0]
  }

  // first terminal
  fn eps(&self) -> u32 {
    self.prod.len() as u32
  }

  // second terminal
  fn eof(&self) -> u32 {
    self.prod.len() as u32 + 1
  }

  fn token_num(&self) -> u32 {
    self.terminal.len() as u32 + self.prod.len() as u32
  }

  fn nt_num(&self) -> u32 {
    self.prod.len() as u32
  }

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter {
    &self.prod[lhs as usize]
  }
}

impl<'a> AbstractGrammarExt<'a> for Grammar<'a> {
  fn prod_pri_assoc(&self, id: u32) -> Option<(u32, Assoc)> {
    self.prod_extra[id as usize].2
  }

  fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)> {
    self.terminal[ch as usize - self.nt.len()].1
  }
}