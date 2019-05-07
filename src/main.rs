#![allow(unused)]
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate smallvec;

mod printer;
mod parser;
mod raw_grammar;
mod abstract_grammar;
mod grammar;
mod lr1;
mod lalr1;
mod bitset;

use crate::abstract_grammar::{AbstractGrammar, AbstractGrammarExt};
use crate::raw_grammar::Assoc;
use std::fs::read_to_string;

struct GrammarStub {
  prod: Vec<Vec<(Vec<u32>, u32)>>
}

impl<'a> AbstractGrammar<'a> for GrammarStub {
  type ProdRef = Vec<u32>;
  type ProdIter = &'a Vec<(Vec<u32>, u32)>;

  fn start(&'a self) -> &'a (Self::ProdRef, u32) {
    &self.prod[0][0]
  }

  fn eps(&self) -> u32 {
    3
  }

  fn eof(&self) -> u32 {
    4
  }

  fn token_num(&self) -> u32 {
    8
  }

  fn nt_num(&self) -> u32 {
    3
  }

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter {
    &self.prod[lhs as usize]
  }
}

impl<'a> AbstractGrammarExt<'a> for GrammarStub {
  fn cmp_priority(&self, a: u32, b: u32) -> std::cmp::Ordering {
    unimplemented!()
  }

  fn get_assoc(&self, ch: u32) -> Assoc {
    unimplemented!()
  }
}

fn main() {
  let stub = GrammarStub {
    prod: vec![
      vec![
        (vec![1], 0) // E' -> E
      ],
      vec![
        (vec![1, 5, 1], 1), // E -> E + E
        (vec![1, 6, 1], 2), // E -> E * E
        (vec![2], 3) // E -> T
      ],
      vec![
        (vec![7], 4) // T -> num
      ]
    ]
  };
  let a = lr1::work(&stub);
  for (i, a) in a.iter().enumerate() {
    println!("{}: {:?}", i, a);
  }
  let a = lalr1::work(&a, &stub);
//  println!("{:?}", a);
  for (i, a) in a.action.iter().enumerate() {
    println!("{}: {:?}", i, a);
  }

//  let prog = read_to_string("test.decaf").unwrap();
//  let mut lex = parser::Lexer::new(&prog);
//  while let Some(tk) = lex.next() {
//    println!("{:?}", tk);
//  }
//  let s = read_to_string("decaf.toml").unwrap();
//  let g: RawGrammar = toml::from_str(&s).unwrap();
//  let g = g.to_grammar().unwrap();
//  println!("{}", g.gen());
}