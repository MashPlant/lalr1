#![allow(unused)]
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod printer;
mod parser;
mod raw_grammar;
mod abstract_grammar;
mod grammar;
mod lr1;
mod lalr1;
mod bitset;

use crate::abstract_grammar::AbstractGrammar;

use std::fs::read_to_string;

struct GrammarStub {
  prod: Vec<Vec<Vec<u32>>>
}

impl<'a> AbstractGrammar<'a> for GrammarStub {
  type ProdRef = Vec<u32>;
  type ProdIter = &'a Vec<Vec<u32>>;

  fn eps(&self) -> u32 {
    3
  }

  fn eof(&self) -> u32 {
    4
  }

  fn token_num(&self) -> u32 {
    7
  }

  fn nt_num(&self) -> u32 {
    3
  }

  fn get_prod(&'a self, lhs: u32) -> Self::ProdIter {
    &self.prod[lhs as usize]
  }
}

fn main() {
  let stub = GrammarStub {
    prod: vec![
      vec![
        vec![1]
      ],
      vec![
        vec![1, 5, 2],
        vec![2],
      ],
      vec![
        vec![6]
      ]
    ]
  };
  lr1::work(&stub, &stub.prod[0][0]);
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