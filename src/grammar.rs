use std::collections::{HashMap, HashSet};
use crate::raw_grammar::*;
use crate::printer::*;
use crate::abstract_grammar::{AbstractGrammar, AbstractGrammarExt};
use std::iter::Map;
use std::slice::Iter;
use smallvec::SmallVec;
use crate::codegen::Codegen;
use crate::lalr1::ParseTable;

pub type ProdVec = SmallVec<[u32; 6]>;

#[derive(Debug)]
pub struct Grammar<'a> {
  pub raw: &'a RawGrammar,
  pub terminal: Vec<(&'a str, Option<(u32, Assoc)>)>,
  //          (name   , type_  )>
  pub nt: Vec<(&'a str, &'a str)>,
  pub lex_state: Vec<&'a str>,
  //               (re    , act    , term   )
  pub lex: Vec<Vec<(String, &'a str, &'a str)>>,
  pub prod: Vec<Vec<(ProdVec, u32)>>,
  pub prod_pri_assoc: Vec<Option<(u32, Assoc)>>,
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
    self.prod_pri_assoc[id as usize]
  }

  fn term_pri_assoc(&self, ch: u32) -> Option<(u32, Assoc)> {
    self.terminal[ch as usize].1
  }
}

impl Grammar<'_> {
  pub fn gen<CG: Codegen>(&self, cg: &CG, table: &ParseTable) -> String {
    cg.gen(self, table)
  }
}